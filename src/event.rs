use std::cell::RefCell;
use std::rc::Rc;

use blitz_dom::{Document as _, Node};
use blitz_traits::events::{
    BlitzKeyEvent, BlitzMouseButtonEvent, KeyState, MouseEventButton, MouseEventButtons, UiEvent,
};
use dioxus_core::{ElementId, RuntimeGuard};
use dioxus_html::input_data::keyboard_types::Location;
use dioxus_native_dom::DioxusDocument;
use smol_str::SmolStr;
use termwiz::input::{InputEvent as TzInputEvent, KeyCode, Modifiers as TzModifiers};

use crate::config::Config;
use crate::geometry::Rect;
use crate::hooks::{
    map_code, map_modifiers, raw_input_from_termwiz, CursorMode, CursorState, CursorUnit,
    RawMouseState,
};
use crate::render::DioxusRenderer;
use crate::scene::CellMetrics;

pub struct EventContext<'a> {
    pub renderer: &'a mut DioxusRenderer,
    pub cfg: Config,
    pub viewport: Rect,
    pub pixel_viewport: Option<Rect>,
    pub pixel_scale: f32,
    pub cell_metrics: CellMetrics,
}

pub struct EventDispatcher {
    input_state: InputState,
    raw_mouse_state: RawMouseState,
    cursor_state: Rc<RefCell<CursorState>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            input_state: InputState::default(),
            raw_mouse_state: RawMouseState::default(),
            cursor_state: Rc::new(RefCell::new(CursorState::default())),
        }
    }

    pub fn cursor_state(&self) -> Rc<RefCell<CursorState>> {
        self.cursor_state.clone()
    }

    pub fn handle(&mut self, term_evt: TzInputEvent, ctx: &mut EventContext<'_>) -> bool {
        let ctrl_c = matches!(&term_evt, TzInputEvent::Key(key) if matches!(key.key, KeyCode::Char('c' | 'C')) && key.modifiers.contains(TzModifiers::CTRL) && ctx.cfg.ctrl_c_quit);
        if ctrl_c {
            return true;
        }

        let raw_inputs = raw_input_from_termwiz(
            &term_evt,
            ctx.viewport,
            ctx.pixel_viewport,
            &mut self.raw_mouse_state,
        );
        {
            let _guard = RuntimeGuard::new(ctx.renderer.runtime.clone());
            for event in raw_inputs.iter().cloned() {
                ctx.renderer.input_bus.publish(event);
            }
        }

        let event_hit_position = mouse_position_from_termwiz(&term_evt, ctx.pixel_scale, ctx.cell_metrics);
        if let Some((x, y)) = cursor_position_from_termwiz(&term_evt) {
            let mut state = self.cursor_state.borrow_mut();
            if state.mode == CursorMode::FollowMouse {
                state.unit = match term_evt {
                    TzInputEvent::PixelMouse(_) => CursorUnit::Pixel,
                    _ => CursorUnit::Cell,
                };
                state.position = Some((x, y));
            }
        }
        let cursor_position = self
            .cursor_state
            .borrow()
            .position
            .and_then(|pos| cursor_hit_position(pos, self.cursor_state.borrow().unit, ctx.cell_metrics))
            .or(event_hit_position);

        let mut focus_hit: Option<(f32, f32)> = None;
        if mouse_press_from_termwiz(&term_evt) {
            focus_hit = cursor_position;
        }

        if let Some(ui_event) = ui_event_from_termwiz(
            &term_evt,
            ctx.pixel_scale,
            ctx.cell_metrics,
            &mut self.input_state,
        ) {
            ctx.renderer.doc.handle_ui_event(ui_event);
        }

        if let Some((x, y)) = focus_hit {
            if let Some(hit) = ctx.renderer.doc.inner.hit(x, y) {
                if let Some(node) = ctx.renderer.doc.inner.get_node(hit.node_id) {
                    if node.is_focussable() {
                        let _ = ctx.renderer.doc.inner.set_focus_to(hit.node_id);
                    }
                }
            }
        }

        if let Some((x, y)) = cursor_position {
            if let Some(target) = target_from_hit(&ctx.renderer.doc, x, y) {
                for evt in raw_inputs {
                    if evt.name == "wheel" || evt.name == "pixelwheel" {
                        let runtime_event = evt.data.into_platform_event(evt.bubbles);
                        ctx.renderer
                            .handle_event(target, evt.name, runtime_event, evt.bubbles);
                    }
                }
            }
        }

        false
    }
}

#[derive(Default)]
struct InputState {
    last_buttons: MouseEventButtons,
    last_button: MouseEventButton,
}

fn ui_event_from_termwiz(
    evt: &TzInputEvent,
    pixel_scale: f32,
    cell_metrics: CellMetrics,
    input_state: &mut InputState,
) -> Option<UiEvent> {
    match evt {
        TzInputEvent::Key(key) => {
            let (key_val, code) = map_code(key);
            let modifiers = map_modifiers(key.modifiers);
            let text = match key.key {
                KeyCode::Char(c) => Some(SmolStr::new(c.to_string())),
                _ => None,
            };
            Some(UiEvent::KeyDown(BlitzKeyEvent {
                key: key_val,
                code,
                modifiers,
                location: Location::Standard,
                is_auto_repeating: false,
                is_composing: false,
                state: KeyState::Pressed,
                text,
            }))
        }
        TzInputEvent::Mouse(mouse) => ui_event_from_mouse(
            mouse.x as f32 * cell_metrics.cell_w_px,
            mouse.y as f32 * cell_metrics.cell_h_px,
            mouse.mouse_buttons.clone(),
            mouse.modifiers,
            input_state,
        ),
        TzInputEvent::PixelMouse(mouse) => ui_event_from_mouse(
            scale_pixels(mouse.x_pixels, pixel_scale),
            scale_pixels(mouse.y_pixels, pixel_scale),
            mouse.mouse_buttons.clone(),
            mouse.modifiers,
            input_state,
        ),
        _ => None,
    }
}

fn ui_event_from_mouse(
    x: f32,
    y: f32,
    buttons: termwiz::input::MouseButtons,
    mods: termwiz::input::Modifiers,
    input_state: &mut InputState,
) -> Option<UiEvent> {
    if buttons.contains(termwiz::input::MouseButtons::VERT_WHEEL)
        || buttons.contains(termwiz::input::MouseButtons::HORZ_WHEEL)
    {
        return None;
    }

    let buttons = mouse_buttons_from_termwiz(&buttons);
    let modifiers = map_modifiers(mods);
    let button = if buttons == MouseEventButtons::None {
        input_state.last_button
    } else {
        mouse_button_from_event_buttons(buttons)
    };

    let mut event = BlitzMouseButtonEvent {
        x,
        y,
        button,
        buttons,
        mods: modifiers,
    };

    let released = input_state.last_buttons & !buttons;
    if released != MouseEventButtons::None {
        event.button = mouse_button_from_event_buttons(released);
        input_state.last_button = event.button;
        input_state.last_buttons = buttons;
        return Some(UiEvent::MouseUp(event));
    }

    let added = buttons & !input_state.last_buttons;
    if added != MouseEventButtons::None {
        event.button = mouse_button_from_event_buttons(added);
        input_state.last_button = event.button;
        input_state.last_buttons = buttons;
        return Some(UiEvent::MouseDown(event));
    }

    input_state.last_buttons = buttons;
    if buttons == MouseEventButtons::None {
        return Some(UiEvent::MouseMove(event));
    }

    Some(UiEvent::MouseMove(event))
}

fn mouse_buttons_from_termwiz(buttons: &termwiz::input::MouseButtons) -> MouseEventButtons {
    let mut mapped = MouseEventButtons::None;
    if buttons.contains(termwiz::input::MouseButtons::LEFT) {
        mapped.insert(MouseEventButtons::Primary);
    }
    if buttons.contains(termwiz::input::MouseButtons::RIGHT) {
        mapped.insert(MouseEventButtons::Secondary);
    }
    if buttons.contains(termwiz::input::MouseButtons::MIDDLE) {
        mapped.insert(MouseEventButtons::Auxiliary);
    }
    mapped
}

fn mouse_button_from_event_buttons(buttons: MouseEventButtons) -> MouseEventButton {
    if buttons.contains(MouseEventButtons::Primary) {
        MouseEventButton::Main
    } else if buttons.contains(MouseEventButtons::Secondary) {
        MouseEventButton::Secondary
    } else if buttons.contains(MouseEventButtons::Auxiliary) {
        MouseEventButton::Auxiliary
    } else {
        MouseEventButton::Main
    }
}

fn mouse_position_from_termwiz(
    evt: &TzInputEvent,
    pixel_scale: f32,
    cell_metrics: CellMetrics,
) -> Option<(f32, f32)> {
    match evt {
        TzInputEvent::Mouse(mouse) => Some((
            normalize_cell_coord(mouse.x) as f32 * cell_metrics.cell_w_px,
            normalize_cell_coord(mouse.y) as f32 * cell_metrics.cell_h_px,
        )),
        TzInputEvent::PixelMouse(mouse) => Some((
            scale_pixels(normalize_pixel_coord(mouse.x_pixels), pixel_scale),
            scale_pixels(normalize_pixel_coord(mouse.y_pixels), pixel_scale),
        )),
        _ => None,
    }
}

fn cursor_position_from_termwiz(evt: &TzInputEvent) -> Option<(f32, f32)> {
    match evt {
        TzInputEvent::Mouse(mouse) => {
            Some((normalize_cell_coord(mouse.x) as f32, normalize_cell_coord(mouse.y) as f32))
        }
        TzInputEvent::PixelMouse(mouse) => Some((
            normalize_pixel_coord(mouse.x_pixels) as f32,
            normalize_pixel_coord(mouse.y_pixels) as f32,
        )),
        _ => None,
    }
}

fn cursor_hit_position(
    position: (f32, f32),
    unit: CursorUnit,
    cell_metrics: CellMetrics,
) -> Option<(f32, f32)> {
    let (x, y) = position;
    match unit {
        CursorUnit::Cell => {
            let cell_w = if cell_metrics.cell_w_px > 0.0 {
                cell_metrics.cell_w_px
            } else {
                1.0
            };
            let cell_h = if cell_metrics.cell_h_px > 0.0 {
                cell_metrics.cell_h_px
            } else {
                1.0
            };
            Some((x * cell_w, y * cell_h))
        }
        CursorUnit::Pixel => Some((x, y)),
    }
}

fn normalize_cell_coord(value: u16) -> u16 {
    value.saturating_sub(1)
}

fn normalize_pixel_coord(value: u16) -> u16 {
    value.saturating_sub(1)
}

fn mouse_press_from_termwiz(evt: &TzInputEvent) -> bool {
    let buttons = match evt {
        TzInputEvent::Mouse(mouse) => &mouse.mouse_buttons,
        TzInputEvent::PixelMouse(mouse) => &mouse.mouse_buttons,
        _ => return false,
    };

    if buttons.contains(termwiz::input::MouseButtons::VERT_WHEEL)
        || buttons.contains(termwiz::input::MouseButtons::HORZ_WHEEL)
    {
        return false;
    }

    buttons.contains(termwiz::input::MouseButtons::LEFT)
        || buttons.contains(termwiz::input::MouseButtons::RIGHT)
        || buttons.contains(termwiz::input::MouseButtons::MIDDLE)
}

fn scale_pixels(value: u16, pixel_scale: f32) -> f32 {
    let scale = if pixel_scale > 0.0 { pixel_scale } else { 1.0 };
    (value as f32) / scale
}

fn target_from_hit(doc: &DioxusDocument, x: f32, y: f32) -> Option<ElementId> {
    let hit = doc.inner.hit(x, y)?;
    let node = doc.inner.get_node(hit.node_id)?;
    dioxus_id_from_node(node)
}

fn dioxus_id_from_node(node: &Node) -> Option<ElementId> {
    node.element_data()?
        .attrs
        .iter()
        .find(|attr| *attr.name.local == *"data-dioxus-id")
        .and_then(|attr| attr.value.parse::<usize>().ok())
        .map(ElementId)
}

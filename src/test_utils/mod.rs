use std::cell::RefCell;
use std::rc::Rc;

use termwiz::color::{ColorAttribute, SrgbaTuple};

use crate::capabilities::{detect, DetectedCapabilities, InlineImageProtocol, TerminalCapabilities};
use crate::cell_render::paint_surface;
use crate::config::{ColorMode, Config, PaletteEntry};
use crate::geometry::Rect;
use crate::render::{apply_soft_caret_overlay, DioxusRenderer};
use crate::{CellMetrics, RawVirtualDom, Surface};
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus_core::ComponentFunction;
use std::collections::VecDeque;
use crate::image::PlacedImage;
use crate::hooks::{CaretBus, CaretCommand, CaretMode, CaretSubscription};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaretSnapshot {
    pub visible: bool,
    pub position: Option<(u16, u16)>,
    pub mode: CaretMode,
}

impl Default for CaretSnapshot {
    fn default() -> Self {
        Self {
            visible: false,
            position: None,
            mode: CaretMode::Physical,
        }
    }
}

impl CaretSnapshot {
    fn apply(&mut self, cmd: CaretCommand) {
        match cmd {
            CaretCommand::Show => self.visible = true,
            CaretCommand::Hide => self.visible = false,
            CaretCommand::SetPosition(x, y) => self.position = Some((x, y)),
            CaretCommand::SetMode(mode) => self.mode = mode,
        }
    }
}

pub struct CaretRecorder {
    events: Rc<RefCell<Vec<CaretCommand>>>,
    _subscription: CaretSubscription,
}

impl CaretRecorder {
    pub fn new(bus: &CaretBus) -> Self {
        let events = Rc::new(RefCell::new(Vec::new()));
        let events_handle = Rc::clone(&events);
        let subscription = bus.subscribe(Rc::new(move |cmd| {
            events_handle.borrow_mut().push(cmd);
        }));
        Self {
            events,
            _subscription: subscription,
        }
    }

    pub fn last_position(&self) -> Option<(u16, u16)> {
        self.events
            .borrow()
            .iter()
            .rev()
            .find_map(|cmd| match cmd {
                CaretCommand::SetPosition(x, y) => Some((*x, *y)),
                _ => None,
            })
    }

    pub fn events(&self) -> Vec<CaretCommand> {
        self.events.borrow().clone()
    }

    pub fn snapshot(&self) -> CaretSnapshot {
        let mut snapshot = CaretSnapshot::default();
        for cmd in self.events.borrow().iter().copied() {
            snapshot.apply(cmd);
        }
        snapshot
    }
}

pub fn palette_entry_to_attr(
    entry: PaletteEntry,
    color_mode: ColorMode,
    truecolor: bool,
) -> ColorAttribute {
    match entry {
        PaletteEntry::Ansi(idx) | PaletteEntry::Palette256(idx) => {
            ColorAttribute::PaletteIndex(idx)
        }
        PaletteEntry::Rgb(r, g, b) => {
            let srgb = SrgbaTuple::from((r, g, b));
            let palette_idx_256 =
                16 + 36 * (r as u16 / 51) as u8 + 6 * (g as u16 / 51) as u8 + (b as u16 / 51) as u8;
            let base_idx = (if r >= 128 { 1 } else { 0 })
                | (if g >= 128 { 2 } else { 0 })
                | (if b >= 128 { 4 } else { 0 });
            match color_mode {
                ColorMode::BaseColors => ColorAttribute::PaletteIndex(base_idx),
                ColorMode::Ansi => ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256),
                ColorMode::Rgb => {
                    if truecolor {
                        ColorAttribute::TrueColorWithDefaultFallback(srgb)
                    } else {
                        ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256)
                    }
                }
            }
        }
    }
}

pub fn render_once_with_caret<P, F>(
    cfg: Config,
    raw: RawVirtualDom<P, F>,
    area: Rect,
) -> crate::error::Result<(Surface, CaretSnapshot)>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let detected = match detect() {
        Ok(detected) => detected,
        Err(_err) => DetectedCapabilities {
            termwiz: crate::capabilities::termwiz_capabilities()?,
            terminal: TerminalCapabilities {
                truecolor: false,
                inline_images: false,
                inline_protocol: InlineImageProtocol::None,
            },
        },
    };
    let metrics = CellMetrics {
        cell_w_px: 8.0,
        cell_h_px: 16.0,
    };
    let vdom = raw.into_virtual_dom();
    let viewport = Viewport::new(
        (area.width as f32 * metrics.cell_w_px).ceil().max(1.0) as u32,
        (area.height as f32 * metrics.cell_h_px).ceil().max(1.0) as u32,
        1.0,
        ColorScheme::Light,
    );
    let (mut renderer, _event_tx, _event_rx) = DioxusRenderer::new_with_viewport(vdom, viewport);
    let recorder = CaretRecorder::new(&renderer.caret_bus);

    renderer.update();
    for _ in 0..5 {
        if !renderer.layout_bus.registered_scopes().is_empty() {
            break;
        }
        renderer.update();
    }
    renderer.viewport_bus.publish(area);
    renderer.update();
    let mut surface = Surface::new(area.width, area.height);
    let mut images = VecDeque::<PlacedImage>::new();
    for _ in 0..5 {
        let _ = renderer.layout_root(area, metrics);
        renderer.publish_layout_rects(area, metrics);
        renderer.update();
        renderer.doc.vdom.process_events();
        renderer.update();
        if recorder.last_position().is_some() {
            break;
        }
    }
    paint_surface(
        &mut surface,
        &mut images,
        renderer.doc.inner.as_ref(),
        area,
        metrics,
        cfg.palette_roles,
        cfg.color_mode,
        detected.terminal.truecolor,
        cfg.custom_draw_mode,
        cfg.image_policy,
        cfg.image_downgrade,
        detected.terminal.inline_images,
    )?;
    let snapshot = recorder.snapshot();
    if snapshot.visible && snapshot.mode == CaretMode::Soft {
        apply_soft_caret_overlay(&mut surface, snapshot.position, cfg, &detected.terminal);
    }
    Ok((surface, snapshot))
}

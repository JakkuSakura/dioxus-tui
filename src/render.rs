use std::{any::Any, cell::RefCell, rc::Rc};

use crate::error::Result;
use blitz_dom::{Document as _, Node};
use blitz_traits::shell::{ColorScheme, Viewport};
use blitz_traits::events::{BlitzKeyEvent, BlitzMouseButtonEvent, KeyState, MouseEventButton, MouseEventButtons, UiEvent};
use dioxus_core::{ComponentFunction, ElementId, Event, Runtime, RuntimeGuard, VirtualDom};
use dioxus_html::PlatformEventData;
use dioxus_html::input_data::keyboard_types::Location;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use futures::{FutureExt, StreamExt};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use smol_str::SmolStr;
use termwiz::{
    color::{ColorAttribute, SrgbaTuple},
    surface::{Change, Position},
    terminal::{buffered::BufferedTerminal, ScreenSize, Terminal},
};
use tokio::time::sleep;
use termwiz::input::{InputEvent as TzInputEvent, KeyCode, Modifiers as TzModifiers};
use termwiz::terminal::new_terminal;

use crate::capabilities::{DetectedCapabilities, InlineImageProtocol, TerminalCapabilities};
use crate::capabilities::detect as detect_capabilities;
use crate::capabilities::termwiz_capabilities;
use crate::config::{ColorMode, Config, PaletteEntry, RenderingMode};
use crate::geometry::Rect;
use crate::hooks::{
    CursorBus, CursorCommand, CursorMode, CursorStyle, CursorUnit, map_code, map_modifiers,
    raw_input_from_termwiz, RawMouseState, TuiInputBus, ViewportBus,
};
use crate::layout::resolve_document;
use crate::scene::CellMetrics;
use crate::surface::Surface;
use crate::RawVirtualDom;
use crate::cell_render::paint_surface;
use crate::image::PlacedImage;
use tracing::debug;

#[cfg(feature = "blitz")]
use anyrender::ImageRenderer;
#[cfg(feature = "blitz")]
use blitz_paint::paint_scene;
#[cfg(feature = "blitz")]
use crate::layout::resolve_document_with_viewport_and_extra_css;

pub fn channel() -> (UnboundedSender<InputEvent>, UnboundedReceiver<InputEvent>) {
    unbounded()
}

pub(crate) struct RenderedFrame {
    pub(crate) surface: Surface,
    pub(crate) images: std::collections::VecDeque<PlacedImage>,
}

#[derive(Clone)]
pub struct TuiContext {
    tx: UnboundedSender<InputEvent>,
}

impl TuiContext {
    pub fn new(tx: UnboundedSender<InputEvent>) -> Self {
        Self { tx }
    }

    pub fn quit(&self) {
        let _ = self.tx.unbounded_send(InputEvent::Close);
    }

    pub fn inject_event(&self, event: termwiz::input::InputEvent) {
        let _ = self.tx.unbounded_send(InputEvent::UserInput(event));
    }
}

#[derive(Debug)]
pub enum InputEvent {
    UserInput(termwiz::input::InputEvent),
    Close,
}

pub(crate) struct DioxusRenderer {
    pub(crate) doc: DioxusDocument,
    pub(crate) input_bus: TuiInputBus,
    pub(crate) viewport_bus: ViewportBus,
    pub(crate) cursor_bus: CursorBus,
    pub(crate) runtime: std::rc::Rc<Runtime>,
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    pub(crate) hot_reload_rx: tokio::sync::mpsc::UnboundedReceiver<dioxus_hot_reload::HotReloadMsg>,
}

impl DioxusRenderer {
    pub fn new(
        vdom: VirtualDom,
    ) -> (
        Self,
        UnboundedSender<InputEvent>,
        UnboundedReceiver<InputEvent>,
    ) {
        let viewport = initial_viewport_size().map(|(w_cells, h_cells)| {
            Viewport::new(
                (w_cells as f32 * 8.0).ceil().max(1.0) as u32,
                (h_cells as f32 * 16.0).ceil().max(1.0) as u32,
                1.0,
                ColorScheme::Light,
            )
        });
        Self::new_inner(vdom, viewport)
    }

    pub fn new_with_viewport(
        vdom: VirtualDom,
        viewport: Viewport,
    ) -> (
        Self,
        UnboundedSender<InputEvent>,
        UnboundedReceiver<InputEvent>,
    ) {
        Self::new_inner(vdom, Some(viewport))
    }

    fn new_inner(
        vdom: VirtualDom,
        viewport: Option<Viewport>,
    ) -> (
        Self,
        UnboundedSender<InputEvent>,
        UnboundedReceiver<InputEvent>,
    ) {
        let (event_tx, event_rx) = channel();
        let ctx = TuiContext::new(event_tx.clone());
        let input_bus = TuiInputBus::new();
        let viewport_bus = ViewportBus::new();
        let cursor_bus = CursorBus::new();
        let vdom = vdom
            .with_root_context(ctx)
            .with_root_context(input_bus.clone())
            .with_root_context(viewport_bus.clone())
            .with_root_context(cursor_bus.clone());

        let mut doc = Self::build_document(vdom, viewport);
        doc.initial_build();
        let runtime = doc.vdom.runtime();

        (
            Self {
                doc,
                input_bus,
                viewport_bus,
                cursor_bus,
                runtime,
                #[cfg(all(feature = "hot-reload", debug_assertions))]
                hot_reload_rx: {
                    let (hot_reload_tx, hot_reload_rx) =
                        tokio::sync::mpsc::unbounded_channel::<dioxus_hot_reload::HotReloadMsg>();
                    dioxus_hot_reload::connect(move |msg| {
                        let _ = hot_reload_tx.send(msg);
                    });
                    hot_reload_rx
                },
            },
            event_tx,
            event_rx,
        )
    }

    fn build_document(vdom: VirtualDom, viewport: Option<Viewport>) -> DioxusDocument {
        DioxusDocument::new(
            vdom,
            DocumentConfig {
                viewport,
                ..Default::default()
            },
        )
    }

    pub(crate) fn update(&mut self) {
        while self.doc.poll(None) {}
    }

    fn handle_event(&mut self, id: ElementId, event: &str, value: Box<dyn Any>, bubbles: bool) {
        let platform_event = Rc::new(PlatformEventData::new(value));
        let runtime_event = Event::new(platform_event, bubbles).into_any();
        self.doc
            .vdom
            .runtime()
            .handle_event(event, runtime_event, id);
    }


    pub(crate) fn layout_root(&mut self, area: Rect, metrics: CellMetrics) -> Option<usize> {
        resolve_document(&mut self.doc, area, metrics)
    }
}

pub fn render_once<P, F>(cfg: Config, raw: RawVirtualDom<P, F>, area: Rect) -> Result<Surface>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    Ok(render_once_frame(cfg, raw, area)?.surface)
}

pub(crate) fn render_once_frame<P, F>(
    cfg: Config,
    raw: RawVirtualDom<P, F>,
    area: Rect,
) -> Result<RenderedFrame>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let detected = match detect_capabilities() {
        Ok(detected) => detected,
        Err(_err) => DetectedCapabilities {
            termwiz: termwiz_capabilities()?,
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

    renderer.update();
    renderer.viewport_bus.publish(area);
    renderer.update();
    let mut surface = Surface::new(area.width, area.height);
    let mut images = std::collections::VecDeque::<PlacedImage>::new();
    let _ = renderer.layout_root(area, metrics);
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
    Ok(RenderedFrame { surface, images })
}

pub(crate) fn debug_layout<P, F>(cfg: Config, raw: RawVirtualDom<P, F>, area: Rect) -> Result<()>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let _ = cfg;
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
    renderer.update();
    let _ = renderer.layout_root(area, metrics);

    println!("-- dioxus-tui layout debug --");
    let root_id = renderer.doc.inner.root_node().id;
    crate::layout::print_layout(renderer.doc.inner.as_ref(), root_id, 0, area, metrics);
    Ok(())
}

pub(crate) async fn run_renderer<P, F>(cfg: Config, raw: RawVirtualDom<P, F>) -> Result<()>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let detected = match detect_capabilities() {
        Ok(detected) => detected,
        Err(_err) => DetectedCapabilities {
            termwiz: termwiz_capabilities()?,
            terminal: TerminalCapabilities {
                truecolor: false,
                inline_images: false,
                inline_protocol: InlineImageProtocol::None,
            },
        },
    };

    let vdom = raw.into_virtual_dom();
    let (renderer, event_tx, event_rx) = DioxusRenderer::new(vdom);

    if cfg.rendering_mode == RenderingMode::Debug {
        let mut renderer = renderer;
        renderer.update();
        println!("-- dioxus-tui debug snapshot --");
        return Ok(());
    }

    run_tui_renderer(cfg, detected, renderer, event_rx, event_tx).await
}

async fn run_tui_renderer(
    cfg: Config,
    detected: DetectedCapabilities,
    mut renderer: DioxusRenderer,
    mut raw_event_reciever: UnboundedReceiver<InputEvent>,
    _event_tx: UnboundedSender<InputEvent>,
) -> Result<()> {
    let run_terminal = cfg.rendering_mode != RenderingMode::Headless;

    let mut terminal = if run_terminal {
        let mut term = new_terminal(detected.termwiz.clone())?;
        term.set_raw_mode()?;
        term.enter_alternate_screen()?;
        Some(BufferedTerminal::new(term)?)
    } else {
        None
    };

    let capabilities = detected.terminal;
    let mut last_surface: Option<Surface> = None;
    let mut last_area: Option<Rect> = None;
    #[cfg(feature = "blitz")]
    let mut last_pixel_viewport: Option<Rect> = None;
    #[cfg(not(feature = "blitz"))]
    let mut last_pixel_viewport: Option<Rect> = None;
    #[cfg(feature = "blitz")]
    let mut last_pixel_scale: f32 = 1.0;
    #[cfg(not(feature = "blitz"))]
    let last_pixel_scale: f32 = 1.0;
    let mut last_cell_metrics = CellMetrics {
        cell_w_px: 8.0,
        cell_h_px: 16.0,
    };
    let mut input_state = InputState::default();
    let mut raw_mouse_state = RawMouseState::default();
    let cursor_state = Rc::new(RefCell::new(CursorState::default()));
    let mut last_images: Option<std::collections::VecDeque<PlacedImage>> = None;

    let _cursor_subscription = {
        let cursor_state = cursor_state.clone();
        renderer.cursor_bus.subscribe(Rc::new(move |command| {
            let mut state = cursor_state.borrow_mut();
            match command {
                CursorCommand::Show => state.visible = true,
                CursorCommand::Hide => state.visible = false,
                CursorCommand::SetStyle(style) => state.style = style,
                CursorCommand::FollowMouse => state.mode = CursorMode::FollowMouse,
                CursorCommand::SetCellPosition(x, y) => {
                    state.mode = CursorMode::Manual;
                    state.unit = CursorUnit::Cell;
                    state.position = Some((x, y));
                    state.visible = true;
                }
                CursorCommand::SetPixelPosition(x, y) => {
                    state.mode = CursorMode::Manual;
                    state.unit = CursorUnit::Pixel;
                    state.position = Some((x, y));
                    state.visible = true;
                }
            }
        }))
    };

    renderer.update();

    let mut paint_error: Option<crate::error::Error> = None;

    let mut pixel_mouse_enabled = false;

    if let Some(_term) = &mut terminal {
        #[cfg(feature = "blitz")]
        let enable_pixel_mouse = cfg.sgr_pixel_mouse || cfg.rendering_mode == RenderingMode::BlitzTerminal;
        #[cfg(not(feature = "blitz"))]
        let enable_pixel_mouse = cfg.sgr_pixel_mouse;
        if enable_pixel_mouse {
            set_sgr_pixel_mouse(_term, true)?;
            pixel_mouse_enabled = true;
        }
    }

    let result = (async {
        loop {
            if let Some(term) = &mut terminal {
                if let Some(term_evt) = term.terminal().poll_input(Some(cfg.tick_rate))? {
                    if handle_termwiz_input(
                        term_evt,
                        &mut renderer,
                        cfg,
                        last_area,
                        last_pixel_viewport,
                        last_pixel_scale,
                        last_cell_metrics,
                        &mut input_state,
                        &mut raw_mouse_state,
                        &cursor_state,
                    ) {
                        return Ok(());
                    }
                }
            } else if cfg.tick_rate > std::time::Duration::ZERO {
                sleep(cfg.tick_rate).await;
            }

            while let Some(evt) = raw_event_reciever.next().now_or_never().flatten() {
                match evt {
                    InputEvent::Close => return Ok(()),
                    InputEvent::UserInput(term_evt) => {
                        if handle_termwiz_input(
                            term_evt,
                            &mut renderer,
                            cfg,
                            last_area,
                            last_pixel_viewport,
                            last_pixel_scale,
                            last_cell_metrics,
                            &mut input_state,
                            &mut raw_mouse_state,
                            &cursor_state,
                        ) {
                            return Ok(());
                        }
                    }
                }
            }

            renderer.update();

            if let Some(term) = &mut terminal {
                let (area, metrics) = terminal_size(term)?;
                last_cell_metrics = metrics;
                renderer.viewport_bus.publish(area);

                if cfg.sgr_pixel_mouse {
                    #[cfg(feature = "blitz")]
                    if cfg.rendering_mode == RenderingMode::BlitzTerminal {
                        // BlitzTerminal sets pixel viewport based on the render surface.
                        // Keep the existing value to avoid conflicting sizes.
                    } else {
                        let ScreenSize { xpixel, ypixel, .. } = term.terminal().get_screen_size()?;
                        if xpixel > 0 && ypixel > 0 {
                            let pixel_w = (xpixel.min(u16::MAX as usize)) as u16;
                            let pixel_h = (ypixel.min(u16::MAX as usize)) as u16;
                            last_pixel_viewport = Some(Rect::new(0, 0, pixel_w, pixel_h));
                        } else {
                            last_pixel_viewport = None;
                        }
                    }
                    #[cfg(not(feature = "blitz"))]
                    {
                        let ScreenSize { xpixel, ypixel, .. } = term.terminal().get_screen_size()?;
                        if xpixel > 0 && ypixel > 0 {
                            let pixel_w = (xpixel.min(u16::MAX as usize)) as u16;
                            let pixel_h = (ypixel.min(u16::MAX as usize)) as u16;
                            last_pixel_viewport = Some(Rect::new(0, 0, pixel_w, pixel_h));
                        } else {
                            last_pixel_viewport = None;
                        }
                    }
                }

                #[cfg(feature = "blitz")]
                if cfg.rendering_mode == RenderingMode::BlitzTerminal {
                    let ScreenSize {
                        cols,
                        rows,
                        xpixel,
                        ypixel,
                    } = term.terminal().get_screen_size()?;
                    if cols == 0 || rows == 0 {
                        return Err(crate::error::Error::Other(anyhow::anyhow!(
                            "terminal reported zero-sized cell grid"
                        )));
                    }
                    if xpixel == 0 || ypixel == 0 {
                        return Err(crate::error::Error::Other(anyhow::anyhow!(
                            "terminal did not report pixel dimensions (xpixel/ypixel=0); cannot use BlitzTerminal"
                        )));
                    }

                    let _cell_w_px = (xpixel as f32) / (cols as f32);
                    let cell_h_px = (ypixel as f32) / (rows as f32);
                    let supersample = cfg.blitz_hidpi_scale.max(1) as f32;
                    last_pixel_scale = supersample;
                    let render_w_px = ((xpixel as f32) * supersample).ceil().max(1.0) as u32;
                    let render_h_px = ((ypixel as f32) * supersample).ceil().max(1.0) as u32;

                    let pixel_w = (xpixel.min(u16::MAX as usize)) as u16;
                    let pixel_h = (ypixel.min(u16::MAX as usize)) as u16;
                    last_pixel_viewport = Some(Rect::new(0, 0, pixel_w, pixel_h));

                    let viewport = Viewport::new(render_w_px, render_h_px, supersample, ColorScheme::Light);
                    let font_px = (cell_h_px.round().max(1.0)) as u32;
                    let extra_css = format!(
                        ":root, html, body {{ font-family: monospace; font-size: {}px; line-height: {}px; }}",
                        font_px, font_px
                    );
                    let _ = resolve_document_with_viewport_and_extra_css(
                        &mut renderer.doc,
                        viewport.clone(),
                        Some(extra_css.as_str()),
                    );

                    // In launch mode, avoid expensive per-frame cropping. Render the full viewport.
                    let cropped_cells = area.height;
                    let cropped_px = render_h_px;

                    let mut image_renderer = <anyrender_vello_cpu::VelloCpuImageRenderer as ImageRenderer>::new(
                        render_w_px,
                        render_h_px,
                    );
                    let mut rgba = Vec::new();
                    image_renderer.render_to_vec(
                        |scene| {
                            paint_scene(
                                scene,
                                renderer.doc.inner.as_ref(),
                                viewport.scale_f64(),
                                render_w_px,
                                render_h_px,
                            );
                        },
                        &mut rgba,
                    );
                    if cropped_px < render_h_px {
                        let bytes_per_row = (render_w_px as usize) * 4;
                        let keep = (cropped_px as usize) * bytes_per_row;
                        if keep < rgba.len() {
                            rgba.truncate(keep);
                        }
                    }

                    if matches!(capabilities.inline_protocol, InlineImageProtocol::None) {
                        return Err(crate::error::Error::Other(anyhow::anyhow!(
                            "BlitzTerminal launch currently requires inline image protocol support"
                        )));
                    }
                    let png = crate::image::rgba_to_png_bytes(&rgba, render_w_px, cropped_px.min(render_h_px))?;
                    let encoder = inline_encoder_for_caps(&capabilities).unwrap_or(rasteroid::InlineEncoder::Ascii);
                    let mut payload = Vec::new();
                    rasteroid::inline_an_image(&png, &mut payload, None, Some((0, 0)), &encoder)
                        .map_err(|err| crate::error::Error::Other(anyhow::anyhow!("inline image error: {err}")))?;
                    term.add_change(Change::ClearScreen(ColorAttribute::Default));
                    term.add_change(Change::CursorPosition {
                        x: Position::Absolute(0),
                        y: Position::Absolute(0),
                    });
                    term.add_change(Change::Text(String::from_utf8_lossy(&payload).to_string()));
                    term.add_change(Change::CursorPosition {
                        x: Position::Absolute(0),
                        y: Position::Absolute((cropped_cells as usize).min(area.height as usize - 1)),
                    });
                    term.flush()?;

                    continue;
                }

                let mut surface = Surface::new(area.width, area.height);
                let mut images = std::collections::VecDeque::<PlacedImage>::new();
                last_area = Some(area);
                if let Some(_root) = renderer.layout_root(area, metrics) {
                    if let Err(err) = paint_surface(
                        &mut surface,
                        &mut images,
                        renderer.doc.inner.as_ref(),
                        area,
                        metrics,
                        cfg.palette_roles,
                        cfg.color_mode,
                        capabilities.truecolor,
                        cfg.custom_draw_mode,
                        cfg.image_policy,
                        cfg.image_downgrade,
                        capabilities.inline_images,
                    ) {
                        paint_error = Some(err);
                        break;
                    }
                }
                // Debug: dump first few lines of the surface for tracing.
                if cfg!(debug_assertions) {
                    let width = surface.width() as usize;
                    let dump: Vec<String> = surface
                        .content
                        .chunks(width)
                        .take(5)
                        .map(|row| row.iter().map(|c| c.ch).collect())
                        .collect();
                    debug!(?dump, "surface_dump");
                }
                #[cfg(feature = "blitz")]
                let is_blitz_gui = cfg.rendering_mode == RenderingMode::BlitzGui;
                #[cfg(not(feature = "blitz"))]
                let is_blitz_gui = false;

                if !is_blitz_gui {
                    let cursor_snapshot = cursor_state.borrow().clone();
                    apply_cursor_overlay(
                        &mut surface,
                        &cursor_snapshot,
                        cfg,
                        &capabilities,
                        metrics,
                    );
                }
                flush_surface(
                    term,
                    &surface,
                    last_surface.as_ref(),
                    &capabilities,
                    &images,
                    last_images.as_ref(),
                    metrics,
                )?;
                last_surface = Some(surface);
                last_images = Some(images);
            }
        }

        Ok(())
    })
    .await;

    let cleanup_result = (|| -> Result<()> {
        if let Some(term) = &mut terminal {
            if pixel_mouse_enabled {
                set_sgr_pixel_mouse(term, false)?;
            }
            term.terminal().exit_alternate_screen()?;
            term.terminal().set_cooked_mode()?;
            term.flush()?;
        }
        Ok(())
    })();

    if let Some(err) = paint_error {
        return Err(err);
    }

    match result {
        Err(err) => Err(err),
        Ok(()) => cleanup_result,
    }
}

#[derive(Default)]
struct InputState {
    last_buttons: MouseEventButtons,
    last_button: MouseEventButton,
}

#[derive(Clone)]
struct CursorState {
    visible: bool,
    style: CursorStyle,
    mode: CursorMode,
    unit: CursorUnit,
    position: Option<(f32, f32)>,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            visible: false,
            style: CursorStyle::Block,
            mode: CursorMode::FollowMouse,
            unit: CursorUnit::Cell,
            position: None,
        }
    }
}

fn handle_termwiz_input(
    term_evt: TzInputEvent,
    renderer: &mut DioxusRenderer,
    cfg: Config,
    last_area: Option<Rect>,
    last_pixel_viewport: Option<Rect>,
    pixel_scale: f32,
    cell_metrics: CellMetrics,
    input_state: &mut InputState,
    raw_mouse_state: &mut RawMouseState,
    cursor_state: &RefCell<CursorState>,
) -> bool {
    let ctrl_c = matches!(&term_evt, TzInputEvent::Key(key) if matches!(key.key, KeyCode::Char('c' | 'C')) && key.modifiers.contains(TzModifiers::CTRL) && cfg.ctrl_c_quit);
    if ctrl_c {
        return true;
    }
    let viewport = last_area.unwrap_or_else(|| Rect::new(0, 0, 0, 0));
    let pixel_viewport = last_pixel_viewport;
    let raw_inputs = raw_input_from_termwiz(&term_evt, viewport, pixel_viewport, raw_mouse_state);
    {
        let _guard = RuntimeGuard::new(renderer.runtime.clone());
        for event in raw_inputs.iter().cloned() {
            renderer.input_bus.publish(event);
        }
    }

    let event_position = mouse_position_from_termwiz(&term_evt, pixel_scale, cell_metrics);
    if let Some((x, y)) = event_position {
        let mut state = cursor_state.borrow_mut();
        if state.mode == CursorMode::FollowMouse {
            state.unit = match term_evt {
                TzInputEvent::PixelMouse(_) => CursorUnit::Pixel,
                _ => CursorUnit::Cell,
            };
            state.position = Some((x, y));
        }
    }
    let cursor_position = cursor_state.borrow().position.or(event_position);

    let mut focus_hit: Option<(f32, f32)> = None;
    if mouse_press_from_termwiz(&term_evt) {
        focus_hit = cursor_position;
    }

    if let Some(ui_event) = ui_event_from_termwiz(&term_evt, pixel_scale, cell_metrics, input_state) {
        renderer.doc.handle_ui_event(ui_event);
    }

    if let Some((x, y)) = focus_hit {
        if let Some(hit) = renderer.doc.inner.hit(x, y) {
            if let Some(node) = renderer.doc.inner.get_node(hit.node_id) {
                if node.is_focussable() {
                    let _ = renderer.doc.inner.set_focus_to(hit.node_id);
                }
            }
        }
    }

    if let Some((x, y)) = cursor_position {
        if let Some(target) = target_from_hit(&renderer.doc, x, y) {
            for evt in raw_inputs {
                if evt.name == "wheel" || evt.name == "pixelwheel" {
                    let runtime_event = evt.data.into_platform_event(evt.bubbles);
                    renderer.handle_event(target, evt.name, runtime_event, evt.bubbles);
                }
            }
        }
    }
    false
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
            mouse.x as f32 * cell_metrics.cell_w_px,
            mouse.y as f32 * cell_metrics.cell_h_px,
        )),
        TzInputEvent::PixelMouse(mouse) => Some((
            scale_pixels(mouse.x_pixels, pixel_scale),
            scale_pixels(mouse.y_pixels, pixel_scale),
        )),
        _ => None,
    }
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

fn apply_cursor_overlay(
    surface: &mut Surface,
    cursor: &CursorState,
    cfg: Config,
    capabilities: &TerminalCapabilities,
    cell_metrics: CellMetrics,
) {
    if !cursor.visible {
        return;
    }
    let Some((x, y)) = cursor.position else {
        return;
    };

    let (cell_x, cell_y) = match cursor.unit {
        CursorUnit::Cell => (x.floor(), y.floor()),
        CursorUnit::Pixel => {
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
            ((x / cell_w).floor(), (y / cell_h).floor())
        }
    };

    if cell_x < 0.0 || cell_y < 0.0 {
        return;
    }
    let cell_x = cell_x as u16;
    let cell_y = cell_y as u16;
    if cell_x >= surface.width() || cell_y >= surface.height() {
        return;
    }

    let idx = (cell_y as usize) * (surface.width() as usize) + (cell_x as usize);
    let Some(cell) = surface.content.get_mut(idx) else {
        return;
    };

    let accent = palette_entry_to_attr(cfg.palette_roles.accent, cfg.color_mode, capabilities.truecolor);
    let style = if cursor.unit == CursorUnit::Pixel && cursor.style == CursorStyle::Block {
        CursorStyle::Crosshair
    } else {
        cursor.style
    };

    match style {
        CursorStyle::Block => {
            cell.ch = ' ';
            cell.bg = Some(accent);
            cell.fg = None;
        }
        CursorStyle::Underline => {
            cell.underline = termwiz::cell::Underline::Single;
            cell.fg = Some(accent);
        }
        CursorStyle::Beam => {
            cell.ch = '▏';
            cell.fg = Some(accent);
            cell.bg = None;
        }
        CursorStyle::Crosshair => {
            cell.ch = '+';
            cell.fg = Some(accent);
            cell.bg = None;
        }
    }
}

fn palette_entry_to_attr(entry: PaletteEntry, color_mode: ColorMode, truecolor: bool) -> ColorAttribute {
    match entry {
        PaletteEntry::Ansi(idx) | PaletteEntry::Palette256(idx) => ColorAttribute::PaletteIndex(idx),
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

fn terminal_size<T: Terminal>(term: &mut BufferedTerminal<T>) -> Result<(Rect, CellMetrics)> {
    let ScreenSize {
        cols,
        rows,
        xpixel,
        ypixel,
    } = term.terminal().get_screen_size()?;
    let (cell_w_px, cell_h_px) = if cols > 0 && rows > 0 && xpixel > 0 && ypixel > 0 {
        (
            (xpixel as f32) / (cols as f32),
            (ypixel as f32) / (rows as f32),
        )
    } else {
        (8.0, 16.0)
    };
    Ok((
        Rect::new(0, 0, cols as u16, rows as u16),
        CellMetrics {
            cell_w_px,
            cell_h_px,
        },
    ))
}

fn set_sgr_pixel_mouse<T: Terminal>(term: &mut BufferedTerminal<T>, enabled: bool) -> Result<()> {
    let suffix = if enabled { "h" } else { "l" };
    term.add_change(Change::Text(format!("\x1b[?1016{suffix}")));
    term.flush()?;
    Ok(())
}

fn initial_viewport_size() -> Option<(u16, u16)> {
    let caps = detect_capabilities().ok()?;
    let mut term = new_terminal(caps.termwiz).ok()?;
    term.get_screen_size()
        .ok()
        .map(|s| (s.cols as u16, s.rows as u16))
}

pub(crate) fn surface_to_changes(
    surface: &Surface,
    prev: Option<&Surface>,
    masked_intervals_by_row: Option<&[Vec<(usize, usize)>]>,
) -> Vec<Change> {
    let full_redraw = prev.map(|p| p.dims() != surface.dims()).unwrap_or(true);
    let mut changes = Vec::new();

    if full_redraw {
        changes.push(Change::ClearScreen(ColorAttribute::Default));
    }

    // dirty lines: compare line-by-line, emit minimal cursor moves; future: extend to rect diff
    let width = surface.width() as usize;
    for (y, chunk) in surface.content.chunks(width).enumerate() {
        let is_dirty = if full_redraw {
            true
        } else if let Some(prev_surface) = prev {
            let prev_width = prev_surface.width() as usize;
            if prev_width != width || y >= prev_surface.height() as usize {
                true
            } else {
                let start = y * prev_width;
                let end = start + prev_width;
                &prev_surface.content[start..end] != chunk
            }
        } else {
            true
        };

        if !is_dirty {
            continue;
        }

        let mut current_fg = ColorAttribute::Default;
        let mut current_bg = ColorAttribute::Default;
        let mut current_intensity = termwiz::cell::Intensity::Normal;
        let mut current_underline = termwiz::cell::Underline::None;
        let mut current_italic = false;
        let mut current_blink = termwiz::cell::Blink::None;
        let mut buf = String::with_capacity(chunk.len());

        changes.push(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(y),
        });

        let intervals = masked_intervals_by_row
            .and_then(|rows| rows.get(y))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let mut interval_idx = 0usize;
        let mut masked_end: Option<usize> = None;

        let mut x = 0usize;
        while x < chunk.len() {
            if masked_end.is_none() {
                if let Some((start, end)) = intervals.get(interval_idx).copied() {
                    if x == start {
                        masked_end = Some(end.min(chunk.len()));
                    }
                }
            }

            let is_masked = masked_end.is_some_and(|end| x < end);
            if masked_end.is_some_and(|end| x == end) {
                masked_end = None;
                interval_idx += 1;
            }

            let cell = &chunk[x];
            // In masked regions (inline images), we still paint the cell background so that
            // letterboxing uses the panel background instead of the terminal default.
            // We force a blank glyph so we never overwrite the image with text.
            let fg = if is_masked {
                ColorAttribute::Default
            } else {
                cell.fg.unwrap_or(ColorAttribute::Default)
            };
            let bg = cell.bg.unwrap_or(ColorAttribute::Default);
            let intensity = cell.intensity;
            let underline = cell.underline;
            let italic = cell.italic;
            let blink = cell.blink;

            if fg != current_fg
                || bg != current_bg
                || intensity != current_intensity
                || underline != current_underline
                || italic != current_italic
                || blink != current_blink
            {
                if !buf.is_empty() {
                    changes.push(Change::Text(std::mem::take(&mut buf)));
                }
                if bg != current_bg {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Background(
                        bg,
                    )));
                    current_bg = bg;
                }
                if fg != current_fg {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Foreground(
                        fg,
                    )));
                    current_fg = fg;
                }
                if intensity != current_intensity {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Intensity(
                        intensity,
                    )));
                    current_intensity = intensity;
                }
                if underline != current_underline {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Underline(
                        underline,
                    )));
                    current_underline = underline;
                }
                if italic != current_italic {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Italic(italic)));
                    current_italic = italic;
                }
                if blink != current_blink {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Blink(blink)));
                    current_blink = blink;
                }
            }

            buf.push(if is_masked { ' ' } else { cell.ch });
            x += 1;
        }

        if !buf.is_empty() {
            changes.push(Change::Text(buf));
        }

        if current_fg != ColorAttribute::Default {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Foreground(
                ColorAttribute::Default,
            )));
        }
        if current_bg != ColorAttribute::Default {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Background(
                ColorAttribute::Default,
            )));
        }
        if current_intensity != termwiz::cell::Intensity::Normal {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Intensity(
                termwiz::cell::Intensity::Normal,
            )));
        }
        if current_underline != termwiz::cell::Underline::None {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Underline(
                termwiz::cell::Underline::None,
            )));
        }
        if current_italic {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Italic(false)));
        }
        if current_blink != termwiz::cell::Blink::None {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Blink(
                termwiz::cell::Blink::None,
            )));
        }
    }

    changes
}

fn image_mask_intervals_by_row(
    images: &std::collections::VecDeque<PlacedImage>,
    surface_width: usize,
    surface_height: usize,
) -> Vec<Vec<(usize, usize)>> {
    let mut rows: Vec<Vec<(usize, usize)>> = vec![Vec::new(); surface_height];
    for img in images {
        let x0 = (img.x_cell as usize).min(surface_width);
        let x1 = (img.x_cell as usize)
            .saturating_add(img.width_cells as usize)
            .min(surface_width);
        if x0 >= x1 {
            continue;
        }

        let y0 = (img.y_cell as usize).min(surface_height);
        let y1 = (img.y_cell as usize)
            .saturating_add(img.height_cells as usize)
            .min(surface_height);
        for y in y0..y1 {
            rows[y].push((x0, x1));
        }
    }

    for row in &mut rows {
        if row.len() <= 1 {
            continue;
        }
        row.sort_by_key(|(s, _e)| *s);
        let mut merged: Vec<(usize, usize)> = Vec::with_capacity(row.len());
        for (s, e) in row.drain(..) {
            if let Some((_ms, me)) = merged.last_mut() {
                if s <= *me {
                    *me = (*me).max(e);
                    continue;
                }
            }
            merged.push((s, e));
        }
        *row = merged;
    }

    rows
}

pub(crate) fn surface_to_cropped_stream_changes(surface: &Surface) -> Vec<Change> {
    let width = surface.width() as usize;
    let height = surface
        .content
        .len()
        .checked_div(width)
        .unwrap_or(0);
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let Some(bottom_row) = (0..height)
        .rev()
        .find(|&y| row_has_non_blank(&surface.content[y * width..(y + 1) * width]))
    else {
        return Vec::new();
    };

    let mut changes = Vec::new();
    // `render()` streams to stdout and must not assume the current terminal state.
    // Terminals are sticky: if the previous output set attributes, we must reset
    // them up-front to get predictable output.
    push_reset_attributes(&mut changes);
    // Force termwiz to flush the reset immediately; it buffers attribute changes
    // until it sees `Change::Text`.
    changes.push(Change::Text(String::new()));
    for y in 0..=bottom_row {
        let row = &surface.content[y * width..(y + 1) * width];

        let right_col = match (0..width).rev().find(|&x| !row[x].is_blank()) {
            Some(col) => col,
            None => {
                // Reset attributes so empty rows don't inherit styling.
                push_reset_attributes(&mut changes);
                // Always advance to a fresh line for stream output.
                changes.push(Change::Text("\r\n".to_string()));
                continue;
            }
        };

        let mut current_fg = ColorAttribute::Default;
        let mut current_bg = ColorAttribute::Default;
        let mut current_intensity = termwiz::cell::Intensity::Normal;
        let mut current_underline = termwiz::cell::Underline::None;
        let mut current_italic = false;
        let mut current_blink = termwiz::cell::Blink::None;
        let mut buf = String::new();

        for cell in row.iter().take(right_col + 1) {
            let fg = cell.fg.unwrap_or(ColorAttribute::Default);
            let bg = cell.bg.unwrap_or(ColorAttribute::Default);
            let intensity = cell.intensity;
            let underline = cell.underline;
            let italic = cell.italic;
            let blink = cell.blink;

            if fg != current_fg
                || bg != current_bg
                || intensity != current_intensity
                || underline != current_underline
                || italic != current_italic
                || blink != current_blink
            {
                if !buf.is_empty() {
                    changes.push(Change::Text(std::mem::take(&mut buf)));
                }
                if bg != current_bg {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Background(bg)));
                    current_bg = bg;
                }
                if fg != current_fg {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Foreground(fg)));
                    current_fg = fg;
                }
                if intensity != current_intensity {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Intensity(
                        intensity,
                    )));
                    current_intensity = intensity;
                }
                if underline != current_underline {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Underline(
                        underline,
                    )));
                    current_underline = underline;
                }
                if italic != current_italic {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Italic(italic)));
                    current_italic = italic;
                }
                if blink != current_blink {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Blink(blink)));
                    current_blink = blink;
                }
            }

            buf.push(cell.ch);
        }

        if !buf.is_empty() {
            changes.push(Change::Text(buf));
        }

        if current_fg != ColorAttribute::Default {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Foreground(
                ColorAttribute::Default,
            )));
        }
        if current_bg != ColorAttribute::Default {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Background(
                ColorAttribute::Default,
            )));
        }
        if current_intensity != termwiz::cell::Intensity::Normal {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Intensity(
                termwiz::cell::Intensity::Normal,
            )));
        }
        if current_underline != termwiz::cell::Underline::None {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Underline(
                termwiz::cell::Underline::None,
            )));
        }
        if current_italic {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Italic(false)));
        }
        if current_blink != termwiz::cell::Blink::None {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Blink(
                termwiz::cell::Blink::None,
            )));
        }

        // Ensure each row ends with default attributes and a newline.
        push_reset_attributes(&mut changes);
        changes.push(Change::Text("\r\n".to_string()));
    }

    changes
}

pub(crate) fn frame_to_cropped_stream_changes(
    frame: &RenderedFrame,
    caps: &TerminalCapabilities,
) -> Vec<Change> {
    // First, stream the full surface (including background for the image rect).
    // Then, place images at the end so that no subsequent output overwrites them.
    let mut changes = surface_to_cropped_stream_changes(&frame.surface);

    if frame.images.is_empty() || matches!(caps.inline_protocol, InlineImageProtocol::None) {
        return changes;
    }

    let width = frame.surface.width() as usize;
    let height = frame
        .surface
        .content
        .len()
        .checked_div(width)
        .unwrap_or(0);
    if width == 0 || height == 0 {
        return changes;
    }

    let Some(bottom_row) = (0..height)
        .rev()
        .find(|&y| row_has_non_blank(&frame.surface.content[y * width..(y + 1) * width]))
    else {
        return changes;
    };
    let printed_rows = bottom_row + 1;

    for img in &frame.images {
        let y = img.y_cell as usize;
        if y >= printed_rows {
            continue;
        }
        let up = printed_rows.saturating_sub(y);

        let encoder = inline_encoder_for_caps(caps).unwrap_or(rasteroid::InlineEncoder::Ascii);
        let mut payload = Vec::new();
        if rasteroid::inline_an_image(&img.png, &mut payload, None, Some((img.x_cell, img.y_cell)), &encoder).is_ok() {
            // Move to the image cell position relative to the end of output.
            changes.push(Change::Text(format!("\x1b[{}A\r", up)));
            changes.push(Change::Text(String::from_utf8_lossy(&payload).to_string()));
            // Return back to the end of output.
            changes.push(Change::Text(format!("\r\x1b[{}B", up)));
        }
    }

    changes
}

fn push_reset_attributes(changes: &mut Vec<Change>) {
    changes.push(Change::AllAttributes(termwiz::cell::CellAttributes::default()));
}

fn row_has_non_blank(row: &[crate::surface::Cell]) -> bool {
    row.iter().any(crate::surface::Cell::has_visible_content)
}

pub(crate) fn flush_surface<T: Terminal>(
    term: &mut BufferedTerminal<T>,
    surface: &Surface,
    prev: Option<&Surface>,
    caps: &TerminalCapabilities,
    images: &std::collections::VecDeque<PlacedImage>,
    prev_images: Option<&std::collections::VecDeque<PlacedImage>>,
    metrics: CellMetrics,
) -> Result<()> {
    let images_changed = prev_images.map(|p| p != images).unwrap_or(!images.is_empty());

    // If the image set changes, do a full redraw to avoid leaving stale image placements behind.
    let full_redraw = images_changed || prev.map(|p| p.dims() != surface.dims()).unwrap_or(true);

    let masked = if !matches!(caps.inline_protocol, InlineImageProtocol::None) && !images.is_empty() {
        Some(image_mask_intervals_by_row(
            images,
            surface.width() as usize,
            surface.height() as usize,
        ))
    } else {
        None
    };

    let changes = surface_to_changes(
        surface,
        if full_redraw { None } else { prev },
        masked.as_deref(),
    );

    for change in changes {
        term.add_change(change);
    }

    // Place images after text so they don't get overwritten by cleared cell regions.
    if let Some(encoder) = inline_encoder_for_caps(caps) {
        for img in images {
            let mut payload = Vec::new();
            if rasteroid::inline_an_image(&img.png, &mut payload, None, Some((img.x_cell, img.y_cell)), &encoder).is_ok() {
                term.add_change(Change::Text(String::from_utf8_lossy(&payload).to_string()));
            }
        }
    }

    let _ = (metrics,);
    term.flush()?;
    Ok(())
}

fn inline_encoder_for_caps(caps: &TerminalCapabilities) -> Option<rasteroid::InlineEncoder> {
    match caps.inline_protocol {
        InlineImageProtocol::Iterm2 => Some(rasteroid::InlineEncoder::Iterm),
        InlineImageProtocol::Sixel => Some(rasteroid::InlineEncoder::Sixel),
        InlineImageProtocol::None => None,
    }
}

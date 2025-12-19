use std::{any::Any, pin::Pin, rc::Rc};

use crate::error::Result;
use blitz_dom::Document as _;
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus_core::{ComponentFunction, ElementId, Event, VirtualDom};
use dioxus_html::PlatformEventData;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use futures::{pin_mut, StreamExt};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use termwiz::{
    caps::Capabilities,
    color::ColorAttribute,
    surface::{Change, Position},
    terminal::{buffered::BufferedTerminal, ScreenSize, Terminal},
};
use tokio::select;
use termwiz::input::{InputEvent as TzInputEvent, KeyCode, Modifiers as TzModifiers};
use termwiz::terminal::new_terminal;

use crate::capabilities::TerminalCapabilities;
use crate::config::{Config, RenderingMode};
use crate::geometry::Rect;
use crate::hooks::event_from_termwiz;
use crate::layout::resolve_document;
use crate::scene::CellMetrics;
use crate::surface::Surface;
use crate::RawVirtualDom;
use crate::cell_render::paint_surface;
use tracing::debug;

pub fn channel() -> (UnboundedSender<InputEvent>, UnboundedReceiver<InputEvent>) {
    unbounded()
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
        let vdom = vdom.with_root_context(ctx);

        let mut doc = Self::build_document(vdom, viewport);
        doc.initial_build();

        (
            Self {
                doc,
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

    fn update(&mut self) {
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

    fn poll_async(&mut self) -> Pin<Box<dyn futures::Future<Output = ()> + '_>> {
        #[cfg(all(feature = "hot-reload", debug_assertions))]
        return Box::pin(async {
            let hot_reload_wait = self.hot_reload_rx.recv();
            let mut hot_reload_msg = None;
            let wait_for_work = self.doc.vdom.wait_for_work();
            tokio::select! {
                Some(msg) = hot_reload_wait => {
                    #[cfg(all(feature = "hot-reload", debug_assertions))]
                    {
                        hot_reload_msg = Some(msg);
                    }
                    #[cfg(not(all(feature = "hot-reload", debug_assertions)))]
                    let () = msg;
                }
                _ = wait_for_work => {}
            }
            if let Some(msg) = hot_reload_msg {
                match msg {
                    dioxus_hot_reload::HotReloadMsg::UpdateTemplate(template) => {
                        self.doc.vdom.replace_template(template);
                    }
                    dioxus_hot_reload::HotReloadMsg::Shutdown => {
                        std::process::exit(0);
                    }
                    dioxus_hot_reload::HotReloadMsg::UpdateAsset(_) => {}
                }
            }
        });

        #[cfg(not(all(feature = "hot-reload", debug_assertions)))]
        Box::pin(self.doc.vdom.wait_for_work())
    }

    fn root_id(&self) -> Option<ElementId> {
        Some(ElementId(0))
    }

    fn layout_root(&mut self, area: Rect, metrics: CellMetrics) -> Option<usize> {
        resolve_document(&mut self.doc, area, metrics)
    }
}

pub fn render_once<P, F>(cfg: Config, raw: RawVirtualDom<P, F>, area: Rect) -> Result<Surface>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let metrics = CellMetrics {
        cell_w_px: 8.0,
        cell_h_px: 16.0,
    };
    let vdom = raw.into_virtual_dom();

    let viewport = Viewport::new(area.width.into(), area.height.into(), 1.0, ColorScheme::Light);
    let (mut renderer, _event_tx, _event_rx) = DioxusRenderer::new_with_viewport(vdom, viewport);

    renderer.update();
    let mut surface = Surface::new(area.width, area.height);
    let _ = renderer.layout_root(area, metrics);
    paint_surface(
        &mut surface,
        renderer.doc.inner.as_ref(),
        area,
        metrics,
        cfg.palette_roles,
        cfg.color_mode,
        true,
    );
    Ok(surface)
}

pub(crate) async fn run_renderer<P, F>(cfg: Config, raw: RawVirtualDom<P, F>) -> Result<()>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let vdom = raw.into_virtual_dom();
    let (renderer, event_tx, event_rx) = DioxusRenderer::new(vdom);

    if cfg.rendering_mode == RenderingMode::Debug {
        let mut renderer = renderer;
        renderer.update();
        println!("-- dioxus-tui debug snapshot --");
        return Ok(());
    }

    run_tui_renderer(cfg, renderer, event_rx, event_tx).await
}

async fn run_tui_renderer(
    cfg: Config,
    mut renderer: DioxusRenderer,
    mut raw_event_reciever: UnboundedReceiver<InputEvent>,
    event_tx: UnboundedSender<InputEvent>,
) -> Result<()> {
    let run_terminal = cfg.rendering_mode != RenderingMode::Headless;

    if run_terminal {
        let tx = event_tx.clone();
        let tick_rate = cfg.tick_rate;
        std::thread::spawn(move || {
            let res: Result<()> = (|| {
                let caps = Capabilities::new_from_env()?;
                let mut term = new_terminal(caps)?;
                term.set_raw_mode()?;

                loop {
                    match term.poll_input(Some(tick_rate))? {
                        Some(evt) => {
                            if tx.unbounded_send(InputEvent::UserInput(evt)).is_err() {
                                break;
                            }
                        }
                        None => {}
                    }
                }

                term.set_cooked_mode()?;
                Ok(())
            })();

            if let Err(err) = res {
                tracing::warn!("input thread terminated early: {err}");
            }
        });
    }

    let mut terminal = if run_terminal {
        let caps = Capabilities::new_from_env()?;
        let mut term = new_terminal(caps)?;
        term.set_raw_mode()?;
        term.enter_alternate_screen()?;
        Some(BufferedTerminal::new(term)?)
    } else {
        None
    };

    let capabilities = TerminalCapabilities::detect().unwrap_or(TerminalCapabilities {
        truecolor: false,
        inline_images: false,
    });
    let mut last_surface: Option<Surface> = None;
    let mut last_area: Option<Rect> = None;

    renderer.update();

    loop {
        let mut input_event: Option<InputEvent> = None;

        {
            let wait = renderer.poll_async();
            pin_mut!(wait);

            select! {
                _ = wait => {},
                evt = raw_event_reciever.next() => {
                    if let Some(evt) = evt {
                        input_event = Some(evt);
                    }
                }
            }
        }

        if let Some(evt) = input_event {
            match evt {
                InputEvent::Close => break,
                InputEvent::UserInput(term_evt) => {
                    let ctrl_c = matches!(&term_evt, TzInputEvent::Key(key) if matches!(key.key, KeyCode::Char('c' | 'C')) && key.modifiers.contains(TzModifiers::CTRL) && cfg.ctrl_c_quit);
                    if ctrl_c {
                        break;
                    }
                    if let Some(root) = renderer.root_id() {
                        let viewport = last_area.unwrap_or_else(|| Rect::new(0, 0, 0, 0));

                        for (target, name, data, bubbles) in
                            event_from_termwiz(term_evt, root, viewport)
                        {
                            let runtime_event = data.into_platform_event(bubbles);
                            renderer.handle_event(target, name, runtime_event, bubbles);
                        }
                    }
                }
            }
        }

        renderer.update();

        if let Some(term) = &mut terminal {
            let (area, metrics) = terminal_size(term)?;
            let mut surface = Surface::new(area.width, area.height);
            last_area = Some(area);
            if let Some(_root) = renderer.layout_root(area, metrics) {
                paint_surface(
                    &mut surface,
                    renderer.doc.inner.as_ref(),
                    area,
                    metrics,
                    cfg.palette_roles,
                    cfg.color_mode,
                    capabilities.truecolor,
                );
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
            let images = std::collections::VecDeque::new();
            flush_surface(
                term,
                &surface,
                last_surface.as_ref(),
                &capabilities,
                &images,
                metrics,
            )?;
            last_surface = Some(surface);
        }
    }

    if let Some(term) = &mut terminal {
        term.terminal().exit_alternate_screen()?;
        term.terminal().set_cooked_mode()?;
        term.flush()?;
    }

    Ok(())
}

fn terminal_size<T: Terminal>(term: &mut BufferedTerminal<T>) -> Result<(Rect, CellMetrics)> {
    let ScreenSize {
        cols,
        rows,
        ..
    } = term.terminal().get_screen_size()?;
    // `xpixel/ypixel` is unreliable or unavailable on some terminals, and we don't
    // strictly need it for cell-native rendering.
    let (cell_w_px, cell_h_px) = (8.0, 16.0);
    Ok((
        Rect::new(0, 0, cols as u16, rows as u16),
        CellMetrics {
            cell_w_px,
            cell_h_px,
        },
    ))
}

fn initial_viewport_size() -> Option<(u16, u16)> {
    let caps = Capabilities::new_from_env().ok()?;
    let mut term = new_terminal(caps).ok()?;
    term.get_screen_size()
        .ok()
        .map(|s| (s.cols as u16, s.rows as u16))
}

pub(crate) fn surface_to_changes(surface: &Surface, prev: Option<&Surface>) -> Vec<Change> {
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
        let mut buf = String::with_capacity(chunk.len());

        changes.push(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(y),
        });

        for cell in chunk.iter() {
            let fg = cell.fg.unwrap_or(ColorAttribute::Default);
            let bg = cell.bg.unwrap_or(ColorAttribute::Default);

            if fg != current_fg || bg != current_bg {
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
    }

    changes
}

pub(crate) fn surface_to_cropped_stream_changes(surface: &Surface) -> Vec<Change> {
    let width = surface.width() as usize;
    let height = surface.height() as usize;
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let Some(bottom_row) = (0..height)
        .rev()
        .find(|&y| row_has_visible_content(&surface.content[y * width..(y + 1) * width]))
    else {
        return Vec::new();
    };

    let mut changes = Vec::new();
    for y in 0..=bottom_row {
        let row = &surface.content[y * width..(y + 1) * width];
        let right_col = (0..width).rev().find(|&x| cell_has_visible_content(&row[x]));

        let mut current_fg = ColorAttribute::Default;
        let mut current_bg = ColorAttribute::Default;
        let mut buf = String::new();

        if let Some(right_col) = right_col {
            for x in 0..=right_col {
                let cell = &row[x];
                let fg = cell.fg.unwrap_or(ColorAttribute::Default);
                let bg = cell.bg.unwrap_or(ColorAttribute::Default);

                if fg != current_fg || bg != current_bg {
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
                }

                buf.push(cell.ch);
            }

            if !buf.is_empty() {
                changes.push(Change::Text(buf));
            }
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

        if y != bottom_row {
            changes.push(Change::Text("\n".to_string()));
        }
    }

    changes
}

fn row_has_visible_content(row: &[crate::surface::Cell]) -> bool {
    row.iter().any(cell_has_visible_content)
}

fn cell_has_visible_content(cell: &crate::surface::Cell) -> bool {
    // For `render()` (one-shot output), treat background-only cells as non-content
    // so we can trim trailing blank lines even when the UI paints a full-page background.
    cell.ch != ' ' && cell.ch != '\0'
}

pub(crate) fn flush_surface<T: Terminal>(
    term: &mut BufferedTerminal<T>,
    surface: &Surface,
    prev: Option<&Surface>,
    caps: &TerminalCapabilities,
    images: &std::collections::VecDeque<crate::scene::InlineImage>,
    metrics: CellMetrics,
) -> Result<()> {
    let changes = surface_to_changes(surface, prev);
    for change in changes {
        term.add_change(change);
    }
    let _ = (caps, images, metrics);
    term.flush()?;
    Ok(())
}

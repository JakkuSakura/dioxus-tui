use std::{any::Any, pin::Pin, rc::Rc};

use anyhow::Result;
use blitz_dom::Document;
use blitz_traits::shell::{ColorScheme, Viewport};
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event as TermEvent, KeyCode, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
};
use dioxus_core::{ElementId, Event, VirtualDom};
use dioxus_html::PlatformEventData;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use futures::{pin_mut, StreamExt};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use termwiz::{
    caps::Capabilities,
    color::ColorAttribute,
    surface::{Change, Position},
    terminal::{buffered::BufferedTerminal, ScreenSize, SystemTerminal, Terminal},
};
use tokio::select;

use crate::capabilities::TerminalCapabilities;
use crate::config::Config;
use crate::geometry::Rect;
use crate::hooks::event_from_crossterm;
use crate::image::emit_inline_images;
use crate::layout::{print_layout, resolve_document};
use crate::scene::{CellMetrics, TerminalScene};
use crate::surface::Surface;

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

    pub fn inject_event(&self, event: crossterm::event::Event) {
        let _ = self.tx.unbounded_send(InputEvent::UserInput(event));
    }
}

#[derive(Debug)]
pub enum InputEvent {
    UserInput(TermEvent),
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
        let (event_tx, event_rx) = channel();
        let ctx = TuiContext::new(event_tx.clone());
        let vdom = vdom.with_root_context(ctx);

        let viewport = {
            let (w, h) = size().unwrap_or((80, 24));
            Viewport::new(w.into(), h.into(), 1.0, ColorScheme::Light)
        };
        let mut doc = DioxusDocument::new(
            vdom,
            DocumentConfig {
                viewport: Some(viewport),
                ..Default::default()
            },
        );
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

    fn layout_root(&mut self, area: Rect) -> Option<usize> {
        resolve_document(&mut self.doc, area)
    }
}

pub(crate) async fn run_renderer(
    cfg: Config,
    mut renderer: DioxusRenderer,
    mut raw_event_reciever: UnboundedReceiver<InputEvent>,
    event_tx: UnboundedSender<InputEvent>,
) -> Result<()> {
    if cfg.rendering_mode == crate::config::RenderingMode::Debug {
        renderer.update();
        println!("-- dioxus-tui debug snapshot --");
        let (w, h) = size().unwrap_or((80, 24));
        let area = Rect::new(0, 0, w, h);
        if let Some(root) = renderer.layout_root(area) {
            print_layout(&renderer.doc.inner, root, 0, area);
        }
        return Ok(());
    }

    if cfg.rendering_mode != crate::config::RenderingMode::Headless {
        let tx = event_tx.clone();
        std::thread::spawn(move || {
            let tick_rate = cfg.tick_rate;
            loop {
                match crossterm::event::poll(tick_rate) {
                    Ok(true) => match crossterm::event::read() {
                        Ok(evt) => {
                            if tx.unbounded_send(InputEvent::UserInput(evt)).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    },
                    Ok(false) => {}
                    Err(_) => break,
                }
            }
        });
    }

    let mut terminal = (cfg.rendering_mode != crate::config::RenderingMode::Headless)
        .then(|| -> Result<BufferedTerminal<SystemTerminal>> {
            enable_raw_mode().unwrap();
            execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();
            let buffered =
                BufferedTerminal::new(SystemTerminal::new(Capabilities::new_from_env()?)?)?;
            Ok(buffered)
        })
        .transpose()?;

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
                    if matches!(term_evt, TermEvent::Key(key) if matches!(key.code, KeyCode::Char('c' | 'C')) && key.modifiers.contains(KeyModifiers::CONTROL) && cfg.ctrl_c_quit)
                    {
                        break;
                    }
                    if let Some(root) = renderer.root_id() {
                        let viewport = last_area.unwrap_or_else(|| Rect::new(0, 0, 0, 0));
                        for (target, name, data, bubbles) in
                            event_from_crossterm(term_evt, root, viewport)
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
            let mut images = std::collections::VecDeque::new();
            if let Some(root) = renderer.layout_root(area) {
                let mut scene = TerminalScene::new(
                    &mut surface,
                    &mut images,
                    metrics,
                    cfg.color_mode,
                    capabilities.truecolor,
                );
                blitz::paint::paint_scene(
                    &mut scene,
                    &renderer.doc.inner,
                    renderer.doc.inner.viewport().scale_f64(),
                    renderer.doc.inner.viewport().window_size.0,
                    renderer.doc.inner.viewport().window_size.1,
                );
            }
            if !capabilities.inline_images && !images.is_empty() {
                match cfg.image_policy {
                    crate::config::ImagePolicy::Degrade => {
                        paint_image_fallback(&mut surface, &images, metrics);
                    }
                    crate::config::ImagePolicy::Omit => {
                        // omit images entirely when unsupported
                    }
                }
            }
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

    if cfg.rendering_mode != crate::config::RenderingMode::Headless {
        disable_raw_mode().unwrap();
        execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        if let Some(term) = &mut terminal {
            term.terminal().flush()?;
        }
    }

    Ok(())
}

fn terminal_size(term: &mut BufferedTerminal<SystemTerminal>) -> Result<(Rect, CellMetrics)> {
    let ScreenSize {
        cols,
        rows,
        xpixel,
        ypixel,
    } = term.terminal().get_screen_size()?;
    let (cell_w_px, cell_h_px) = if xpixel > 0 && ypixel > 0 {
        (xpixel as f32 / cols as f32, ypixel as f32 / rows as f32)
    } else {
        (1.0, 1.0)
    };
    Ok((
        Rect::new(0, 0, cols as u16, rows as u16),
        CellMetrics {
            cell_w_px,
            cell_h_px,
        },
    ))
}

fn flush_surface(
    term: &mut BufferedTerminal<SystemTerminal>,
    surface: &Surface,
    prev: Option<&Surface>,
    caps: &TerminalCapabilities,
    images: &std::collections::VecDeque<crate::scene::InlineImage>,
    metrics: CellMetrics,
) -> Result<()> {
    let full_redraw = prev.map(|p| p.dims() != surface.dims()).unwrap_or(true);

    if full_redraw {
        term.add_change(Change::ClearScreen(ColorAttribute::Default));
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

        term.add_change(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(y),
        });

        for cell in chunk.iter() {
            let fg = cell.fg.unwrap_or(ColorAttribute::Default);
            let bg = cell.bg.unwrap_or(ColorAttribute::Default);

            if fg != current_fg || bg != current_bg {
                if !buf.is_empty() {
                    term.add_change(Change::Text(std::mem::take(&mut buf)));
                }
                if bg != current_bg {
                    term.add_change(Change::Attribute(
                        termwiz::cell::AttributeChange::Background(bg),
                    ));
                    current_bg = bg;
                }
                if fg != current_fg {
                    term.add_change(Change::Attribute(
                        termwiz::cell::AttributeChange::Foreground(fg),
                    ));
                    current_fg = fg;
                }
            }

            buf.push(cell.ch);
        }

        if !buf.is_empty() {
            term.add_change(Change::Text(buf));
        }

        if current_fg != ColorAttribute::Default {
            term.add_change(Change::Attribute(
                termwiz::cell::AttributeChange::Foreground(ColorAttribute::Default),
            ));
        }
        if current_bg != ColorAttribute::Default {
            term.add_change(Change::Attribute(
                termwiz::cell::AttributeChange::Background(ColorAttribute::Default),
            ));
        }
    }
    // Emit inline images after text to preserve ordering
    if caps.inline_images && !images.is_empty() {
        let mut buf = Vec::new();
        emit_inline_images(images, &mut buf, metrics.cell_w_px, metrics.cell_h_px)?;
        term.add_change(Change::Text(String::from_utf8_lossy(&buf).into_owned()));
    }
    term.flush()?;
    Ok(())
}

fn paint_image_fallback(
    surface: &mut Surface,
    images: &std::collections::VecDeque<crate::scene::InlineImage>,
    metrics: CellMetrics,
) {
    for img in images {
        let x_px = img.x_px;
        let y_px = img.y_px;
        let w_px = img.width_px as f32;
        let h_px = img.height_px as f32;
        let x0 = (x_px / metrics.cell_w_px).floor().max(0.0) as u16;
        let y0 = (y_px / metrics.cell_h_px).floor().max(0.0) as u16;
        let x1 = ((x_px + w_px) / metrics.cell_w_px).ceil().max(0.0) as u16;
        let y1 = ((y_px + h_px) / metrics.cell_h_px).ceil().max(0.0) as u16;
        for y in y0..y1.min(surface.height()) {
            for x in x0..x1.min(surface.width()) {
                let idx = y as usize * surface.width() as usize + x as usize;
                if let Some(slot) = surface.content.get_mut(idx) {
                    *slot = crate::surface::Cell {
                        ch: '░',
                        fg: None,
                        bg: None,
                    };
                }
            }
        }
    }
}

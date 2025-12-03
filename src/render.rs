use std::{any::Any, pin::Pin, rc::Rc, time::Duration};

use anyhow::Result;
use blitz_dom::Document;
use blitz_traits::shell::{ColorScheme, Viewport};
use crossterm::cursor::{RestorePosition, SavePosition, Show};
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
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use tokio::select;

use crate::config::Config;
use crate::hooks::event_from_crossterm;
use crate::layout::{build_layout, LayoutNode};
use crate::styles::{compute_styles, list_item_label, Attrs, ListStyle};

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

    fn layout_snapshot(&mut self, area: Rect) -> Option<LayoutNode> {
        build_layout(&mut self.doc, area)
    }
}

pub(crate) fn run_renderer(
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
        if let Some(layout_root) = renderer.layout_snapshot(area) {
            print_layout(&layout_root, 0);
        } else {
            println!("(no layout captured)");
        }
        return Ok(());
    }

    if cfg.rendering_mode != crate::config::RenderingMode::Headless {
        let tx = event_tx.clone();
        std::thread::spawn(move || {
            let tick_rate = Duration::from_millis(10);
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

    let mut terminal = (cfg.rendering_mode != crate::config::RenderingMode::Headless).then(|| {
        enable_raw_mode().unwrap();
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(std::io::stdout());
        Terminal::new(backend).unwrap()
    });
    if let Some(term) = &mut terminal {
        term.clear().unwrap();
    }

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
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
                            if matches!(term_evt, TermEvent::Key(key) if matches!(key.code, KeyCode::Char('c' | 'C')) && key.modifiers.contains(KeyModifiers::CONTROL) && cfg.ctrl_c_quit) {
                                break;
                            }
                            if let Some(root) = renderer.root_id() {
                                for (target, name, data, bubbles) in event_from_crossterm(term_evt, root) {
                                    let runtime_event = data.into_platform_event(bubbles);
                                    renderer.handle_event(target, name, runtime_event, bubbles);
                                }
                            }
                        }
                    }
                }

                renderer.update();

                if let Some(term) = &mut terminal {
                    execute!(term.backend_mut(), SavePosition).unwrap();
                    term.draw(|f| {
                        if let Some(layout) = renderer.layout_snapshot(f.area()) {
                            render_tree(f, &layout);
                        }
                    }).unwrap();
                    execute!(term.backend_mut(), RestorePosition, Show).unwrap();
                }
            }

            if let Some(term) = &mut terminal {
                disable_raw_mode().unwrap();
                execute!(term.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
                term.show_cursor().unwrap();
            }

            Ok(())
        })
}

/// Render the captured node tree into a ratatui frame using the computed layout.
pub fn render_tree(frame: &mut Frame, layout: &LayoutNode) {
    render_layout_node(frame, layout, true);
}

fn print_layout(node: &LayoutNode, depth: usize) {
    let indent = "  ".repeat(depth);
    let tag = node.tag.as_deref().unwrap_or("(root)");
    let text = node.text.as_deref().unwrap_or("");
    println!(
        "{indent}- {tag} id={:?} area=({}, {}) {}x{} text=\"{}\"",
        node.id, node.rect.x, node.rect.y, node.rect.width, node.rect.height, text
    );
    for child in node.children.iter() {
        print_layout(child, depth + 1);
    }
}

fn render_layout_node(frame: &mut Frame, node: &LayoutNode, is_root: bool) {
    let tag = node.tag.as_deref().unwrap_or("");
    let _stylesheet = (); // placeholder, layout hints not yet used

    fn collect_text(n: &LayoutNode) -> Option<String> {
        if let Some(t) = &n.text {
            return Some(t.clone());
        }
        for child in n.children.iter() {
            if let Some(t) = collect_text(child) {
                return Some(t);
            }
        }
        None
    }

    if is_root {
        for child in node.children.iter() {
            render_layout_node(frame, child, false);
        }
        return;
    }

    let text_opt = collect_text(node);
    let mut rect = node.rect;
    if rect.height == 0 && text_opt.is_some() {
        rect.height = 1;
    }
    if rect.width == 0 && text_opt.is_some() {
        rect.width = 1;
    }

    match tag {
        "p" | "span" => {
            let text = text_opt.unwrap_or_default();
            frame.render_widget(Paragraph::new(text).alignment(node.align), rect);
        }
        "li" => {
            let text = text_opt.clone().unwrap_or_default();
            let styles = compute_styles(tag, Attrs::new(&node.attrs));
            let default_style = ListStyle::Disc;
            let style_ref = styles.list_style.unwrap_or(default_style);
            let content = list_item_label(&style_ref, 0, &text);
            frame.render_widget(Paragraph::new(content).alignment(node.align), rect);
        }
        "ul" | "ol" => {
            let styles = compute_styles(tag, Attrs::new(&node.attrs));
            let default_style = if tag == "ol" {
                ListStyle::Decimal
            } else {
                ListStyle::Disc
            };
            let style_ref = styles.list_style.unwrap_or(default_style);
            for (idx, child) in node.children.iter().enumerate() {
                let text = collect_text(child).unwrap_or_default();
                let label = list_item_label(&style_ref, idx, &text);
                frame.render_widget(Paragraph::new(label).alignment(child.align), child.rect);
            }
        }
        "h1" | "h2" | "h3" => {
            let text = text_opt.unwrap_or_default();
            frame.render_widget(Paragraph::new(text).alignment(node.align), rect);
        }
        "" => {
            // Raw text node: render text only, then recurse into children if any.
            if let Some(text) = &node.text {
                frame.render_widget(Paragraph::new(text.clone()).alignment(node.align), rect);
            }
            for child in node.children.iter() {
                render_layout_node(frame, child, false);
            }
        }
        _ => {
            if let Some(text) = text_opt {
                frame.render_widget(Paragraph::new(text).alignment(node.align), rect);
            }
            for child in node.children.iter() {
                render_layout_node(frame, child, false);
            }
        }
    }
}

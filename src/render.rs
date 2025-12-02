use std::{any::Any, pin::Pin, rc::Rc, time::Duration};

use anyhow::Result;
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
use futures::{pin_mut, StreamExt};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use ratatui::widgets::{List, ListItem, Paragraph};
use ratatui::Frame;
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use tokio::select;

use crate::config::Config;
use crate::element::{DebugNode, DomState};
use crate::events::SerializedHtmlEventConverter;
use crate::hooks::event_from_crossterm;
use crate::layout::{build_layout, LayoutNode};
use crate::styles::{compute_styles, list_item_label, Attrs};

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
    pub(crate) vdom: VirtualDom,
    pub(crate) dom: DomState,
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    pub(crate) hot_reload_rx: tokio::sync::mpsc::UnboundedReceiver<dioxus_hot_reload::HotReloadMsg>,
}

impl DioxusRenderer {
    pub fn new(
        mut vdom: VirtualDom,
    ) -> (
        Self,
        UnboundedSender<InputEvent>,
        UnboundedReceiver<InputEvent>,
    ) {
        dioxus_html::set_event_converter(Box::new(SerializedHtmlEventConverter));
        let (event_tx, event_rx) = channel();
        let ctx = TuiContext::new(event_tx.clone());
        vdom = vdom.with_root_context(ctx);

        let mut dom = DomState::default();
        {
            let mut writer = dom.writer();
            vdom.rebuild(&mut writer);
        }

        (
            Self {
                vdom,
                dom,
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
        let mut writer = self.dom.writer();
        self.vdom.render_immediate(&mut writer);
    }

    fn handle_event(&mut self, id: ElementId, event: &str, value: Box<dyn Any>, bubbles: bool) {
        let platform_event = Rc::new(PlatformEventData::new(value));
        let runtime_event = Event::new(platform_event, bubbles).into_any();
        self.vdom.runtime().handle_event(event, runtime_event, id);
    }

    fn poll_async(&mut self) -> Pin<Box<dyn futures::Future<Output = ()> + '_>> {
        #[cfg(all(feature = "hot-reload", debug_assertions))]
        return Box::pin(async {
            let hot_reload_wait = self.hot_reload_rx.recv();
            let mut hot_reload_msg = None;
            let wait_for_work = self.vdom.wait_for_work();
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
                        self.vdom.replace_template(template);
                    }
                    dioxus_hot_reload::HotReloadMsg::Shutdown => {
                        std::process::exit(0);
                    }
                    dioxus_hot_reload::HotReloadMsg::UpdateAsset(_) => {}
                }
            }
        });

        #[cfg(not(all(feature = "hot-reload", debug_assertions)))]
        Box::pin(self.vdom.wait_for_work())
    }

    fn root_id(&self) -> Option<ElementId> {
        self.dom.root()
    }

    fn nodes_snapshot(&self) -> Vec<DebugNode> {
        self.dom.nodes()
    }
}

pub fn run_renderer(
    cfg: Config,
    mut renderer: DioxusRenderer,
    mut raw_event_reciever: UnboundedReceiver<InputEvent>,
    event_tx: UnboundedSender<InputEvent>,
) -> Result<()> {
    if cfg.rendering_mode == crate::config::RenderingMode::Debug {
        renderer.update();
        println!("-- dioxus-tui debug snapshot --");
        let nodes = renderer.nodes_snapshot();
        let root_id = renderer.root_id().or_else(|| nodes.first().map(|n| n.id));
        if let Some(root_id) = root_id {
            if let Some(root) = nodes.iter().find(|n| n.id == root_id) {
                let (w, h) = size().unwrap_or((80, 24));
                let layout_root = build_layout(&nodes, root, Rect::new(0, 0, w, h));
                print_layout(&layout_root, 0);
            } else {
                println!("(root node missing)");
            }
        } else {
            println!("(no nodes captured)");
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
                        let nodes = renderer.nodes_snapshot();
                        render_tree(f, &nodes, renderer.root_id());
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
pub fn render_tree(frame: &mut Frame, nodes: &[DebugNode], root_id: Option<ElementId>) {
    let root_id = root_id.or_else(|| nodes.first().map(|n| n.id));
    if let Some(root_id) = root_id {
        if let Some(root) = nodes.iter().find(|n| n.id == root_id) {
            let layout_tree = build_layout(nodes, root, frame.area());
            render_layout_node(frame, &layout_tree, true);
        }
    }
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
    let stylesheet = (); // placeholder, layout hints not yet used

    fn collect_text(n: &LayoutNode) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(t) = &n.text {
            parts.push(t.clone());
        }
        for child in n.children.iter() {
            if let Some(t) = collect_text(child) {
                parts.push(t);
            }
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" "))
        }
    }

    if is_root {
        for child in node.children.iter() {
            render_layout_node(frame, child, false);
        }
        return;
    }

    match tag {
        "p" | "span" => {
            let text = collect_text(node).unwrap_or_default();
            frame.render_widget(Paragraph::new(text).alignment(node.align), node.rect);
        }
        "li" => {
            let text = collect_text(node).unwrap_or_default();
            let styles = compute_styles(tag, Attrs::new(&node.attrs));
            let default_style = lightningcss::properties::list::ListStyleType::CounterStyle(
                lightningcss::properties::list::CounterStyle::Predefined(
                    lightningcss::properties::list::PredefinedCounterStyle::Disc,
                ),
            );
            let style_ref = styles.list_style.as_ref().unwrap_or(&default_style);
            let content = list_item_label(style_ref, 0, &text);
            frame.render_widget(Paragraph::new(content).alignment(node.align), node.rect);
        }
        "ul" | "ol" => {
            let styles = compute_styles(tag, Attrs::new(&node.attrs));
            let default_style = lightningcss::properties::list::ListStyleType::CounterStyle(
                lightningcss::properties::list::CounterStyle::Predefined(if tag == "ol" {
                    lightningcss::properties::list::PredefinedCounterStyle::Decimal
                } else {
                    lightningcss::properties::list::PredefinedCounterStyle::Disc
                }),
            );
            let style_ref = styles.list_style.as_ref().unwrap_or(&default_style);
            let items: Vec<ListItem> = node
                .children
                .iter()
                .enumerate()
                .map(|(idx, child)| {
                    let text = collect_text(child).unwrap_or_default();
                    let label = list_item_label(style_ref, idx, &text);
                    ListItem::new(label)
                })
                .collect();
            let list = List::new(items);
            frame.render_widget(list, node.rect);
        }
        "h1" | "h2" | "h3" => {
            let text = collect_text(node).unwrap_or_default();
            frame.render_widget(Paragraph::new(text).alignment(node.align), node.rect);
        }
        "div" => {
            // container, no border by default
            if let Some(text) = collect_text(node) {
                frame.render_widget(Paragraph::new(text).alignment(node.align), node.rect);
            }
            for child in node.children.iter() {
                render_layout_node(frame, child, false);
            }
        }
        "" => {
            // Raw text node: render text only, then recurse into children if any.
            if let Some(text) = &node.text {
                frame.render_widget(
                    Paragraph::new(text.clone()).alignment(node.align),
                    node.rect,
                );
            }
            for child in node.children.iter() {
                render_layout_node(frame, child, false);
            }
        }
        _ => {
            if let Some(text) = collect_text(node) {
                frame.render_widget(Paragraph::new(text).alignment(node.align), node.rect);
            }
            for child in node.children.iter() {
                render_layout_node(frame, child, false);
            }
        }
    }
}

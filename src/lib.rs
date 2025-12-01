#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]

mod element;
mod events;
mod hooks;
mod config;
pub mod components;

pub use config::{Config, RenderingMode, ColorMode};
pub use hooks::EventData;

use std::{any::Any, pin::Pin, rc::Rc, time::Duration};

use anyhow::Result;
use dioxus_core::{Element, ElementId, Event, VirtualDom};
use dioxus_html::PlatformEventData;
use element::DomState;
use events::SerializedHtmlEventConverter;
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use hooks::event_from_crossterm;
use crossterm::cursor::{RestorePosition, SavePosition, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event as TermEvent, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use futures::{pin_mut, StreamExt, Future};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::select;

fn channel() -> (UnboundedSender<InputEvent>, UnboundedReceiver<InputEvent>) {
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

pub mod launch {
    use super::*;

    pub type Config = super::Config;
    /// Launches the WebView and runs the event loop, with configuration and root props.
    pub fn launch(
        root: fn() -> Element,
        contexts: Vec<Box<dyn Fn() -> Box<dyn Any>>>,
        platform_config: Config,
    ) {
        let mut virtual_dom = VirtualDom::new(root);

        for context in contexts {
            virtual_dom.insert_any_root_context(context());
        }

        launch_vdom_cfg(virtual_dom, platform_config)
    }
}

pub fn launch(app: fn() -> Element) {
    launch_cfg(app, Config::default())
}

pub fn launch_cfg(app: fn() -> Element, cfg: Config) {
    launch_vdom_cfg(VirtualDom::new(app), cfg)
}

pub fn launch_cfg_with_props<P: Clone + 'static>(app: fn(P) -> Element, props: P, cfg: Config) {
    launch_vdom_cfg(VirtualDom::new_with_props(app, props), cfg)
}

pub fn launch_vdom_cfg(mut vdom: VirtualDom, cfg: Config) {
    dioxus_html::set_event_converter(Box::new(SerializedHtmlEventConverter));

    let (event_tx, event_rx) = channel();
    let ctx = TuiContext::new(event_tx.clone());
    vdom = vdom.with_root_context(ctx);

    let mut dom = DomState::default();
    {
        let mut writer = dom.writer();
        vdom.rebuild(&mut writer);
    }

    let renderer = DioxusRenderer {
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
    };

    render(cfg, renderer, event_rx, event_tx).unwrap();
}

fn render(
    cfg: Config,
    mut renderer: DioxusRenderer,
    mut raw_event_reciever: UnboundedReceiver<InputEvent>,
    event_tx: UnboundedSender<InputEvent>,
) -> Result<()> {
    if cfg.rendering_mode == config::RenderingMode::Debug {
        renderer.update();
        let lines = renderer.text_snapshot();
        println!("-- dioxus-tui debug snapshot --");
        if lines.is_empty() {
            println!("(no text nodes captured)");
        }
        for (i, entry) in lines.iter().enumerate() {
            let x = 0;
            let y = i as i32;
            let w = entry.text.len() as i32;
            let h = 1;
            let origin = entry.id.map(|id| format!("id={:?}", id)).unwrap_or_else(|| "static".into());
            println!("[{i}] {origin} pos=({x},{y}) size=({w},{h}) text=\"{}\"", entry.text);
        }
        return Ok(());
    }

    if cfg.rendering_mode != config::RenderingMode::Headless {
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

    let mut terminal = (cfg.rendering_mode != config::RenderingMode::Headless).then(|| {
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
                    let lines = renderer.text_snapshot();
                    let joined = lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n");
                    term.draw(|f| {
                        let area = f.area();
                        let paragraph = ratatui::widgets::Paragraph::new(joined.clone());
                        f.render_widget(paragraph, area);
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

pub trait Driver {
    fn update(&mut self);
    fn handle_event(&mut self, id: ElementId, event: &str, value: Box<dyn std::any::Any>, bubbles: bool);
    fn poll_async(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn root_id(&self) -> Option<ElementId>;
    fn text_snapshot(&self) -> Vec<crate::element::DebugText>;
}

pub(crate) struct DioxusRenderer {
    pub(crate) vdom: VirtualDom,
    pub(crate) dom: DomState,
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    pub(crate) hot_reload_rx: tokio::sync::mpsc::UnboundedReceiver<dioxus_hot_reload::HotReloadMsg>,
}

impl Driver for DioxusRenderer {
    fn update(&mut self) {
        let mut writer = self.dom.writer();
        self.vdom.render_immediate(&mut writer);
    }

    fn handle_event(&mut self, id: ElementId, event: &str, value: Box<dyn Any>, bubbles: bool) {
        let platform_event = Rc::new(PlatformEventData::new(value));
        let runtime_event = Event::new(platform_event, bubbles).into_any();
        self.vdom.runtime().handle_event(event, runtime_event, id);
    }

    fn poll_async(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
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

    fn text_snapshot(&self) -> Vec<crate::element::DebugText> {
        self.dom.texts()
    }
}

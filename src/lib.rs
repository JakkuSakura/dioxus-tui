#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]

mod element;
mod events;
pub mod components;
pub mod runtime;

pub use runtime::{Config, RenderingMode, TuiContext};

use std::{any::Any, pin::Pin, rc::Rc};

use dioxus_core::{Element, ElementId, Event, VirtualDom};
use dioxus_html::PlatformEventData;
use element::DomState;
use events::SerializedHtmlEventConverter;
use runtime::{render, Driver, InputEvent};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};

fn channel() -> (UnboundedSender<InputEvent>, UnboundedReceiver<InputEvent>) {
    unbounded()
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
}

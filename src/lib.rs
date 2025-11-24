#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]

extern crate self as dioxus_native_core;

mod element;
mod events;
pub mod components;
pub mod runtime;
pub use runtime::{Config, RenderingMode, Size, TuiContext};
pub mod engine;

// Re-export engine modules at the crate root so existing
// dioxus-native-core macro paths (dioxus_native_core::prelude, etc.)
// continue to resolve correctly via the crate alias above.
pub use crate::engine::{
    custom_element,
    dioxus,
    layout_attributes,
    node,
    node_ref,
    node_watcher,
    passes,
    real_dom,
    tree,
    utils,
    exports,
    prelude,
    FxDashMap,
    FxDashSet,
    NodeId,
    SendAnyMap,
};

use std::{
    any::Any,
    ops::Deref,
    rc::Rc,
    sync::{Arc, RwLock},
};

use dioxus_core::{consume_context_from_scope, Element, ElementId, Event, ScopeId, VirtualDom};
use dioxus_html::PlatformEventData;
use runtime::query::Query;
use runtime::{render, Driver};
use crate::engine::dioxus::DioxusState;
use crate::engine::prelude::*;

use element::DioxusTUIMutationWriter;

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

pub fn launch_vdom_cfg(vdom: VirtualDom, cfg: Config) {
    dioxus_html::set_event_converter(Box::new(events::SerializedHtmlEventConverter));

    render(cfg, |rdom, taffy, event_tx| {
        let dioxus_state = {
            let mut rdom = rdom.write().unwrap();
            DioxusState::create(&mut rdom)
        };
        let dioxus_state = Rc::new(RwLock::new(dioxus_state));
        let vdom = vdom
            .with_root_context(TuiContext::new(event_tx))
            .with_root_context(Query::new(rdom.clone(), taffy.clone()))
            .with_root_context(DioxusElementToNodeId {
                mapping: dioxus_state.clone(),
            });

        let queued_events = Vec::new();

        let mut myself = DioxusRenderer {
            vdom,
            dioxus_state,
            queued_events,
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

        {
            let mut rdom = rdom.write().unwrap();
            let mut dioxus_state = myself.dioxus_state.write().unwrap();

            let mut writer = DioxusTUIMutationWriter {
                query: myself
                    .vdom
                    .in_runtime(|| {
                        consume_context_from_scope::<Query>(ScopeId::ROOT)
                            .expect("Query context missing")
                    }),
                events: &mut myself.queued_events,
                native_core_writer: dioxus_state.create_mutation_writer(&mut rdom),
            };

            // Find any mount events
            myself.vdom.rebuild(&mut writer);
        }

        myself
    })
    .unwrap();
}

struct DioxusRenderer {
    vdom: VirtualDom,
    dioxus_state: Rc<RwLock<DioxusState>>,
    // Events that are queued up to be sent to the vdom next time the vdom is polled
    queued_events: Vec<(ElementId, &'static str, Box<dyn Any>, bool)>,
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    hot_reload_rx: tokio::sync::mpsc::UnboundedReceiver<dioxus_hot_reload::HotReloadMsg>,
}

impl Driver for DioxusRenderer {
    fn update(&mut self, rdom: &Arc<RwLock<RealDom>>) {
        let mut rdom = rdom.write().unwrap();
        let mut dioxus_state = self.dioxus_state.write().unwrap();

        let mut writer = DioxusTUIMutationWriter {
            query: self
                .vdom
                .in_runtime(|| {
                    consume_context_from_scope::<Query>(ScopeId::ROOT)
                        .expect("Query context missing")
                }),
            events: &mut self.queued_events,
            native_core_writer: dioxus_state.create_mutation_writer(&mut rdom),
        };

        // Find any mount events
        self.vdom.render_immediate(&mut writer);
    }

    fn handle_event(
        &mut self,
        rdom: &Arc<RwLock<RealDom>>,
        id: NodeId,
        event: &str,
        value: Rc<runtime::hooks::EventData>,
        bubbles: bool,
    ) {
        let id = {
            let guard = rdom.read().unwrap();
            let node = guard.get(id).unwrap();
            crate::engine::dioxus::NodeImmutableDioxusExt::<()>::mounted_id(&node)
        };
        if let Some(id) = id {
            let inner_value = value.deref().clone();
            let boxed_event = Box::new(inner_value);
            let platform_event = Rc::new(PlatformEventData::new(boxed_event));
            let runtime_event = Event::new(platform_event, bubbles).into_any();
            self.vdom
                .runtime()
                .handle_event(event, runtime_event, id);
        }
    }

    fn poll_async(&mut self) -> std::pin::Pin<Box<dyn futures::Future<Output = ()> + '_>> {
        // Add any queued events
        for (id, event, value, bubbles) in self.queued_events.drain(..) {
            let platform_event = Rc::new(PlatformEventData::new(value));
            let runtime_event = Event::new(platform_event, bubbles).into_any();
            self.vdom.runtime().handle_event(event, runtime_event, id);
        }

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
            // if we have a new template, replace the old one
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
}

#[derive(Clone)]
pub struct DioxusElementToNodeId {
    mapping: Rc<RwLock<DioxusState>>,
}

impl DioxusElementToNodeId {
    pub fn get_node_id(&self, element_id: ElementId) -> Option<NodeId> {
        self.mapping
            .read()
            .unwrap()
            .try_element_to_node_id(element_id)
    }
}

#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]

pub mod components;
mod config;
mod element;
mod events;
mod hooks;
mod layout;
mod render;

pub use config::{ColorMode, Config, RenderingMode};
pub use hooks::EventData;
pub use render::TuiContext;

use std::any::Any;

use dioxus_core::{Element, VirtualDom};
use render::{run_renderer, DioxusRenderer};

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
    let (renderer, event_tx, event_rx) = DioxusRenderer::new(vdom);
    run_renderer(cfg, renderer, event_rx, event_tx).unwrap();
}

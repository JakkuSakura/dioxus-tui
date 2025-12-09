#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]

pub mod capabilities;
mod config;
pub mod element;
pub mod debug_png;
pub mod geometry;
mod hooks;
pub mod image;
pub mod layout;
pub mod render;
pub mod scene;
pub mod surface;

pub use capabilities::TerminalCapabilities;
pub use config::{ColorMode, Config, ImagePolicy, PaletteEntry, PaletteRoles, RenderingMode};
pub use geometry::{Alignment, Rect};
pub use hooks::EventData;
pub use render::TuiContext;
pub use scene::{CellMetrics, InlineImage, TerminalScene};
pub use surface::Surface;

use std::any::Any;

use dioxus_core::{Element, VirtualDom};
use render::{run_renderer, DioxusRenderer};

pub mod launch {
    use super::*;

    pub type Config = super::Config;
    /// Launches the WebView and runs the event loop, with configuration and root props.
    pub async fn launch(
        root: fn() -> Element,
        contexts: Vec<Box<dyn Fn() -> Box<dyn Any>>>,
        platform_config: Config,
    ) {
        let mut virtual_dom = VirtualDom::new(root);

        for context in contexts {
            virtual_dom.insert_any_root_context(context());
        }

        launch_vdom_cfg(virtual_dom, platform_config).await
    }
}

pub async fn launch(app: fn() -> Element) {
    launch_cfg(app, Config::default()).await
}

pub async fn launch_cfg(app: fn() -> Element, cfg: Config) {
    launch_vdom_cfg(VirtualDom::new(app), cfg).await
}

pub async fn launch_cfg_with_props<P: Clone + 'static>(
    app: fn(P) -> Element,
    props: P,
    cfg: Config,
) {
    launch_vdom_cfg(VirtualDom::new_with_props(app, props), cfg).await
}

pub async fn launch_vdom_cfg(vdom: VirtualDom, cfg: Config) {
    let (renderer, event_tx, event_rx) = DioxusRenderer::new(vdom);
    run_renderer(cfg, renderer, event_rx, event_tx)
        .await
        .unwrap();
}

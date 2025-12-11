#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]

pub mod capabilities;
mod config;
pub mod element;
pub mod error;
pub mod geometry;
mod hooks;
pub mod image;
pub mod layout;
pub mod log;
pub mod render;
pub mod scene;
pub mod styles;
pub mod surface;

pub use capabilities::TerminalCapabilities;
pub use config::{ColorMode, Config, ImagePolicy, PaletteEntry, PaletteRoles, RenderingMode};
pub use error::Error;
pub use geometry::{Alignment, Rect};
pub use hooks::EventData;
pub use render::TuiContext;
pub use scene::{CellMetrics, InlineImage, TerminalScene};
pub use surface::Surface;

use std::any::Any;

use dioxus_core::{ComponentFunction, Element, VirtualDom};
use render::run_renderer;
use tokio::runtime::Builder as RuntimeBuilder;

pub mod launch {
    use super::*;

    pub type Config = super::Config;
    /// Launches the WebView and runs the event loop, with configuration and root props.
    pub fn launch(
        root: fn() -> Element,
        contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
        platform_config: Config,
    ) -> anyhow::Result<()> {
        let raw = RawVirtualDom::with_contexts(move |_| root(), (), contexts);
        launch_raw(raw, platform_config)
    }
}

pub fn launch(app: fn() -> Element) -> anyhow::Result<()> {
    launch_cfg(app, Config::default())
}

pub fn launch_cfg(app: fn() -> Element, cfg: Config) -> anyhow::Result<()> {
    let raw = RawVirtualDom::new(app);
    launch_raw(raw, cfg)
}

pub fn launch_cfg_with_props<P: Clone + Send + Sync + 'static>(
    app: fn(P) -> Element,
    props: P,
    cfg: Config,
) -> anyhow::Result<()> {
    let raw = RawVirtualDom::with_props(app, props);
    launch_raw(raw, cfg)
}

pub fn launch_raw<P: Clone + 'static, F>(
    raw: RawVirtualDom<P, F>,
    cfg: Config,
) -> anyhow::Result<()>
where
    F: ComponentFunction<P, ()> + 'static,
{
    if cfg.rendering_mode == RenderingMode::BlitzGui {
        return launch_blitz_gui_with_props(raw);
    }

    let rt = RuntimeBuilder::new_current_thread().enable_all().build()?;
    rt.block_on(run_renderer(cfg, raw))?;
    Ok(())
}

pub fn launch_blitz_gui(app: fn() -> Element) -> anyhow::Result<()> {
    let raw = RawVirtualDom::new(app);
    launch_blitz_gui_with_props(raw)
}

pub fn launch_blitz_gui_with_props<P: Clone + 'static, F: ComponentFunction<P, ()> + 'static>(
    raw: RawVirtualDom<P, F>,
) -> anyhow::Result<()> {
    let (app, props, contexts) = raw.into_parts();
    dioxus_native::launch_cfg_with_props(app, props, contexts, vec![]);
    Ok(())
}

pub struct RawVirtualDom<P, F> {
    app: F,
    props: P,
    contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
}

impl RawVirtualDom<(), fn() -> Element> {
    pub fn new(app: fn() -> Element) -> RawVirtualDom<(), impl ComponentFunction<(), ()>> {
        RawVirtualDom::with_props(move |_| app(), ())
    }
}

impl<P: Clone + 'static, F> RawVirtualDom<P, F>
where
    F: ComponentFunction<P, ()> + 'static,
{
    pub fn with_props(app: F, props: P) -> Self {
        Self {
            app,
            props,
            contexts: Vec::new(),
        }
    }

    pub fn with_contexts(
        app: F,
        props: P,
        contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
    ) -> Self {
        Self {
            app,
            props,
            contexts,
        }
    }

    pub fn into_virtual_dom(self) -> VirtualDom {
        let RawVirtualDom {
            app,
            props,
            contexts,
            ..
        } = self;
        let mut vdom = VirtualDom::new_with_props(app, props);
        for context in contexts {
            vdom.insert_any_root_context(context());
        }
        vdom
    }

    pub fn into_parts(self) -> (F, P, Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>) {
        let RawVirtualDom {
            app,
            props,
            contexts,
            ..
        } = self;
        (app, props, contexts)
    }
}

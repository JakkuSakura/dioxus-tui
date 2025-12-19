#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]

pub mod capabilities;
pub mod ansi;
mod cell_render;
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
use termwiz::{
    caps::Capabilities,
    terminal::{new_terminal, Terminal as _},
};

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

pub fn render_surface(app: fn() -> Element, width: u16, height: u16) -> anyhow::Result<Surface> {
    render_surface_cfg(app, Config::default(), width, height)
}

pub fn render_surface_cfg(
    app: fn() -> Element,
    cfg: Config,
    width: u16,
    height: u16,
) -> anyhow::Result<Surface> {
    let raw = RawVirtualDom::new(app);
    render_surface_raw(raw, cfg, Rect::new(0, 0, width, height))
}

/// Renders a single frame and writes it to stdout as ANSI-colored text.
///
/// This is a convenience for non-interactive output (no alternate screen, no input loop).
/// The viewport size is detected from the current terminal when possible, with a reasonable
/// fallback for non-TTY environments.
pub fn render(app: fn() -> Element) -> anyhow::Result<()> {
    render_cfg(app, Config::default())
}

pub fn render_cfg(app: fn() -> Element, cfg: Config) -> anyhow::Result<()> {
    let (width, height) = detect_output_size().unwrap_or((100, 40));
    let surface = render_surface_cfg(app, cfg, width, height)?;

    let mut out = std::io::stdout().lock();
    match ansi::write_surface_ansi_cropped(&mut out, &surface) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => {
            // Common when piping to tools like `head`; treat as a clean early-exit.
        }
        Err(err) => return Err(err.into()),
    }
    Ok(())
}

pub fn render_with_size(app: fn() -> Element, width: u16, height: u16) -> anyhow::Result<()> {
    render_cfg_with_size(app, Config::default(), width, height)
}

pub fn render_cfg_with_size(
    app: fn() -> Element,
    cfg: Config,
    width: u16,
    height: u16,
) -> anyhow::Result<()> {
    let surface = render_surface_cfg(app, cfg, width, height)?;

    let mut out = std::io::stdout().lock();
    match ansi::write_surface_ansi_cropped(&mut out, &surface) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => {
            // Common when piping to tools like `head`; treat as a clean early-exit.
        }
        Err(err) => return Err(err.into()),
    }
    Ok(())
}

pub fn render_surface_cfg_with_props<P: Clone + Send + Sync + 'static>(
    app: fn(P) -> Element,
    props: P,
    cfg: Config,
    width: u16,
    height: u16,
) -> anyhow::Result<Surface> {
    let raw = RawVirtualDom::with_props(app, props);
    render_surface_raw(raw, cfg, Rect::new(0, 0, width, height))
}

pub fn render_surface_raw<P: Clone + 'static, F>(
    raw: RawVirtualDom<P, F>,
    cfg: Config,
    area: Rect,
) -> anyhow::Result<Surface>
where
    F: ComponentFunction<P, ()> + 'static,
{
    let rt = RuntimeBuilder::new_current_thread().enable_all().build()?;
    let surface = rt.block_on(async move { render::render_once(cfg, raw, area) })?;
    Ok(surface)
}

fn detect_output_size() -> Option<(u16, u16)> {
    // Respect the conventional env vars first (useful in CI and non-TTY contexts).
    let width = std::env::var("COLUMNS").ok().and_then(|s| s.parse::<u16>().ok());
    let height = std::env::var("LINES").ok().and_then(|s| s.parse::<u16>().ok());
    if let (Some(width), Some(height)) = (width, height) {
        if width > 0 && height > 0 {
            return Some((width, height));
        }
    }

    // Best-effort: ask the current terminal.
    let caps = Capabilities::new_from_env().ok()?;
    let mut term = new_terminal(caps).ok()?;
    term.get_screen_size()
        .ok()
        .map(|s| (s.cols as u16, s.rows as u16))
}

pub fn launch_raw<P: Clone + 'static, F>(
    raw: RawVirtualDom<P, F>,
    cfg: Config,
) -> anyhow::Result<()>
where
    F: ComponentFunction<P, ()> + 'static,
{
    let rt = RuntimeBuilder::new_current_thread().enable_all().build()?;
    rt.block_on(run_renderer(cfg, raw))?;
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

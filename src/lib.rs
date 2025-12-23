#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]

pub mod capabilities;
mod blitz;
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

#[cfg(feature = "blitz")]
mod gui;

#[cfg(feature = "blitz")]
mod blitz_terminal;

pub use capabilities::TerminalCapabilities;
pub use config::{ColorMode, Config, ImageDowngrade, ImagePolicy, PaletteEntry, PaletteRoles, RenderingMode};
pub use error::Error;
pub use geometry::{Alignment, Rect};
pub use hooks::{
    EventData, RawInputEvent, TuiInputBus, ViewportBus, use_keyboard_input, use_mouse_input,
    use_raw_input, use_viewport, use_wheel_input,
};
pub use render::TuiContext;
pub use scene::{CellMetrics, InlineImage, TerminalScene};
pub use surface::Surface;

use std::any::Any;

use dioxus_core::{ComponentFunction, Element, VirtualDom};
use render::run_renderer;
use tokio::runtime::Builder as RuntimeBuilder;
use std::io::Write;

use termwiz::{
    render::RenderTty,
    terminal::{Terminal as _},
};
use termwiz::{render::terminfo::TerminfoRenderer, terminal::ScreenSize};

pub type ContextFactory = Box<dyn Fn() -> Box<dyn Any> + Send + Sync>;

pub struct RenderRequest {
    root: fn() -> Element,
    cfg: Config,
    size: Option<(u16, u16)>,
    contexts: Vec<ContextFactory>,
}

impl RenderRequest {
    pub fn new(root: fn() -> Element) -> Self {
        Self {
            root,
            cfg: Config::default(),
            size: None,
            contexts: Vec::new(),
        }
    }

    pub fn with_config(mut self, cfg: Config) -> Self {
        self.cfg = cfg;
        self
    }

    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.size = Some((width, height));
        self
    }

    pub fn with_contexts(mut self, contexts: Vec<ContextFactory>) -> Self {
        self.contexts = contexts;
        self
    }

}

impl From<fn() -> Element> for RenderRequest {
    fn from(root: fn() -> Element) -> Self {
        Self::new(root)
    }
}

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

pub fn render_surface_cfg(app: fn() -> Element, cfg: Config, width: u16, height: u16) -> anyhow::Result<Surface> {
    let raw = RawVirtualDom::new(app);
    render_surface_raw(raw, cfg, Rect::new(0, 0, width, height))
}

/// Renders a single frame and writes it to stdout as ANSI-colored text.
///
/// This is a convenience for non-interactive output (no alternate screen, no input loop).
/// The viewport size is detected from the current terminal when possible, with a reasonable
/// fallback for non-TTY environments.
pub fn render(request: impl Into<RenderRequest>) -> anyhow::Result<()> {
    let request = request.into();
    render_request(request)
}

pub fn render_cfg(app: fn() -> Element, cfg: Config) -> anyhow::Result<()> {
    render(RenderRequest::new(app).with_config(cfg))
}

pub fn render_with_size(app: fn() -> Element, width: u16, height: u16) -> anyhow::Result<()> {
    render(RenderRequest::new(app).with_size(width, height))
}

pub fn render_cfg_with_size(app: fn() -> Element, cfg: Config, width: u16, height: u16) -> anyhow::Result<()> {
    render(RenderRequest::new(app).with_config(cfg).with_size(width, height))
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

fn detect_output_width() -> Option<u16> {
    #[cfg(unix)]
    {
        use std::io::IsTerminal;
        use std::os::fd::AsRawFd;

        unsafe fn cols_from_fd(fd: i32) -> Option<u16> {
            let mut ws: libc::winsize = std::mem::zeroed();
            if libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) == 0 {
                if ws.ws_col > 0 {
                    return Some(ws.ws_col as u16);
                }
            }
            None
        }

        let stdout = std::io::stdout();
        if stdout.is_terminal() {
            if let Some(cols) = unsafe { cols_from_fd(stdout.as_raw_fd()) } {
                return Some(cols);
            }
        }

        // If stdout isn't a TTY (piped output), try the controlling terminal.
        if let Ok(tty) = std::fs::File::open("/dev/tty") {
            if let Some(cols) = unsafe { cols_from_fd(tty.as_raw_fd()) } {
                return Some(cols);
            }
        }
    }

    // Respect the conventional env var (useful in CI and non-TTY contexts).
    if let Some(width) = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|w| *w > 0)
    {
        return Some(width);
    }

    // Best-effort: ask the current terminal.
    // Prefer stdio here so `render()` can work in environments without `/dev/tty` access.
    let caps = capabilities::detect().ok()?;
    let mut term = termwiz::terminal::new_terminal(caps.termwiz).ok()?;
    term.get_screen_size().ok().map(|s| s.cols as u16)
}

#[cfg(feature = "blitz")]
fn detect_output_size() -> Option<(u16, u16)> {
    // Respect the conventional env vars first (useful in CI and non-TTY contexts).
    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|w| *w > 0);
    let height = std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|h| *h > 0);

    if let (Some(w), Some(h)) = (width, height) {
        return Some((w, h));
    }

    // Best-effort: ask the current terminal.
    let caps = capabilities::detect().ok()?;
    let mut term = termwiz::terminal::new_terminal(caps.termwiz).ok()?;
    let size = term.get_screen_size().ok()?;
    let cols = width.unwrap_or(size.cols as u16);
    let rows = height.unwrap_or(size.rows as u16);
    Some((cols.max(1), rows.max(1)))
}

fn render_request(request: RenderRequest) -> anyhow::Result<()> {
    // In render-mode, height should behave like "infinite" output (scrollback-friendly).
    // We render into a large virtual height and then trim trailing blank lines.
    const DEFAULT_RENDER_HEIGHT: u16 = 2000;

    let cfg = request.cfg;

    let width = request
        .size
        .map(|(w, _h)| w)
        .or_else(detect_output_width)
        .unwrap_or(100);

    #[cfg(feature = "blitz")]
    if cfg.rendering_mode == RenderingMode::BlitzTerminal {
        if let Ok(detected) = crate::capabilities::detect() {
            if blitz::terminal_image_supported(detected.terminal) {
                let (width_cells, height_cells) = request
                    .size
                    .or_else(detect_output_size)
                    .unwrap_or((100, 24));
                let raw = RawVirtualDom::with_contexts(
                    move |_| (request.root)(),
                    (),
                    request.contexts,
                );
                let rendered = crate::blitz_terminal::render_blitz_terminal(
                    cfg.rendering_mode,
                    detected.termwiz,
                    detected.terminal,
                    raw,
                    width_cells,
                    height_cells,
                    cfg,
                )?;
                if !rendered {
                    anyhow::bail!("BlitzTerminal rendering was selected but produced no output");
                }
                return Ok(());
            }
        }
    }

    if cfg.rendering_mode == RenderingMode::Debug {
        let height = request.size.map(|(_, h)| h).unwrap_or(80);
        let raw = RawVirtualDom::with_contexts(move |_| (request.root)(), (), request.contexts);
        render::debug_layout(cfg, raw, Rect::new(0, 0, width, height))?;
        return Ok(());
    }

    let height = request
        .size
        .map(|(_w, h)| h)
        .unwrap_or(DEFAULT_RENDER_HEIGHT);

    let raw = RawVirtualDom::with_contexts(move |_| (request.root)(), (), request.contexts);
    let frame = {
        let rt = RuntimeBuilder::new_current_thread().enable_all().build()?;
        rt.block_on(async move { render::render_once_frame(cfg, raw, Rect::new(0, 0, width, height)) })?
    };

    // `render()` is a one-shot, non-interactive API. It should behave like normal stdout output:
    // no alternate screen and no cursor addressing that overwrites existing content.
    //
    // We still use the same pipeline as `launch` up to `Surface`, and then we render the resulting
    // `Change` stream using termwiz's own `TerminfoRenderer`.
    let caps = crate::capabilities::detect()?;
    let changes = render::frame_to_cropped_stream_changes(&frame, &caps.terminal);
    let mut renderer = TerminfoRenderer::new(caps.termwiz);
    let mut out = std::io::stdout().lock();
    let mut tty = StdoutRenderTty {
        out: &mut out,
        size: ScreenSize {
            rows: height as usize,
            cols: width as usize,
            xpixel: 0,
            ypixel: 0,
        },
        broken_pipe: false,
    };
    renderer.render_to(&changes, &mut tty)?;
    if !tty.broken_pipe {
        match out.flush() {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => {}
            Err(err) => return Err(err.into()),
        }
    }
    Ok(())
}

struct StdoutRenderTty<'a> {
    out: &'a mut dyn Write,
    size: ScreenSize,
    broken_pipe: bool,
}

impl Write for StdoutRenderTty<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.broken_pipe {
            return Ok(buf.len());
        }
        match self.out.write(buf) {
            Ok(n) => Ok(n),
            Err(err) if err.kind() == std::io::ErrorKind::BrokenPipe => {
                self.broken_pipe = true;
                Ok(buf.len())
            }
            Err(err) => Err(err),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.broken_pipe {
            return Ok(());
        }
        self.out.flush()
    }
}

impl RenderTty for StdoutRenderTty<'_> {
    fn get_size_in_cells(&mut self) -> termwiz::Result<(usize, usize)> {
        Ok((self.size.cols, self.size.rows))
    }
}

pub fn launch_raw<P: Clone + 'static, F>(
    raw: RawVirtualDom<P, F>,
    cfg: Config,
) -> anyhow::Result<()>
where
    F: ComponentFunction<P, ()> + 'static,
{
    #[cfg(feature = "blitz")]
    if cfg.rendering_mode == RenderingMode::BlitzGui && blitz::gui_env_supported() {
        crate::gui::launch_blitz_gui(raw);
        return Ok(());
    }

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

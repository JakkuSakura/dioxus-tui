use anyhow::Result;
use termwiz::caps::{Capabilities, ColorLevel, ProbeHints};
use std::env;

#[derive(Debug, Clone)]
pub struct DetectedCapabilities {
    pub termwiz: Capabilities,
    pub terminal: TerminalCapabilities,
}

#[derive(Debug, Clone, Copy)]
pub struct TerminalCapabilities {
    pub truecolor: bool,
    pub inline_images: bool,
    pub inline_protocol: InlineImageProtocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineImageProtocol {
    Iterm2,
    Sixel,
    None,
}

impl TerminalCapabilities {
    pub(crate) fn from_termwiz(caps: &Capabilities) -> Self {
        let iterm2_images = caps.iterm2_image();
        let sixel_images = caps.sixel();
        let inline_protocol = if iterm2_images {
            InlineImageProtocol::Iterm2
        } else if sixel_images {
            InlineImageProtocol::Sixel
        } else {
            InlineImageProtocol::None
        };
        Self {
            truecolor: matches!(caps.color_level(), ColorLevel::TrueColor),
            inline_images: !matches!(inline_protocol, InlineImageProtocol::None),
            inline_protocol,
        }
    }
}

pub fn detect() -> Result<DetectedCapabilities> {
    let termwiz = termwiz_capabilities()?;
    let terminal = TerminalCapabilities::from_termwiz(&termwiz);
    Ok(DetectedCapabilities { termwiz, terminal })
}

pub(crate) fn termwiz_capabilities() -> Result<Capabilities> {
    let mut hints = ProbeHints::new_from_env();
    if is_wezterm() {
        // WezTerm supports the iTerm2 OSC 1337 image protocol, but termwiz only
        // auto-enables `iterm2_image` for iTerm.app. Override here so `Change::Image`
        // uses OSC 1337 instead of the blank-cell fallback.
        hints = hints.iterm2_image(Some(true));
    }
    Ok(Capabilities::new_with_hints(hints)?)
}

fn is_wezterm() -> bool {
    if env::var("WEZTERM_PANE").is_ok() || env::var("WEZTERM_EXECUTABLE").is_ok() {
        return true;
    }

    // If the user followed WezTerm's recommendation and configured `term = "wezterm"`,
    // treat that as a strong signal that we're running under WezTerm.
    if matches!(env::var("TERM").ok().as_deref(), Some("wezterm") | Some("wezterm-256color")) {
        return true;
    }

    matches!(
        env::var("TERM_PROGRAM").ok().as_deref(),
        Some("WezTerm") | Some("wezterm")
    )
}

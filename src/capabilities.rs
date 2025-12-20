use anyhow::Result;
use termwiz::caps::{Capabilities, ColorLevel, ProbeHints};
use std::env;

#[derive(Debug, Clone, Copy)]
pub struct TerminalCapabilities {
    pub truecolor: bool,
    pub inline_images: bool,
    pub iterm2_images: bool,
    pub sixel_images: bool,
}

impl TerminalCapabilities {
    pub fn detect() -> Result<Self> {
        let caps = termwiz_capabilities()?;
        let iterm2_images = caps.iterm2_image();
        let sixel_images = caps.sixel();
        Ok(Self {
            truecolor: matches!(caps.color_level(), ColorLevel::TrueColor),
            inline_images: iterm2_images || sixel_images,
            iterm2_images,
            sixel_images,
        })
    }
}

pub fn termwiz_capabilities() -> Result<Capabilities> {
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

    matches!(
        env::var("TERM_PROGRAM").ok().as_deref(),
        Some("WezTerm") | Some("wezterm")
    )
}

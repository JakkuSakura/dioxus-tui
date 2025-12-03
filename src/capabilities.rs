use anyhow::Result;
use termwiz::caps::{Capabilities, ColorLevel};

#[derive(Debug, Clone, Copy)]
pub struct TerminalCapabilities {
    pub truecolor: bool,
    pub inline_images: bool,
}

impl TerminalCapabilities {
    pub fn detect() -> Result<Self> {
        let caps = Capabilities::new_from_env()?;
        Ok(Self {
            truecolor: matches!(caps.color_level(), ColorLevel::TrueColor),
            inline_images: caps.iterm2_image() || caps.sixel(),
        })
    }
}

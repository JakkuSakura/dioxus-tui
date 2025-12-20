use std::env;

use crate::capabilities::TerminalCapabilities;

pub(crate) fn gui_env_supported() -> bool {
    if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
        return true;
    }

    // For unix-like platforms, require a display server.
    env::var_os("DISPLAY").is_some() || env::var_os("WAYLAND_DISPLAY").is_some()
}

pub(crate) fn terminal_image_supported(caps: TerminalCapabilities) -> bool {
    caps.inline_images
}

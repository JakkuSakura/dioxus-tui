use dioxus::prelude::*;
use dioxus_tui::{Config, RenderingMode};

fn main() {
    // Debug mode prints the captured DOM/text snapshot to stdout (no terminal UI).
    dioxus_tui::launch_cfg(app, Config::default().with_rendering_mode(RenderingMode::Debug));
}

fn app() -> Element {
    rsx! {
        div {
            "Ratatui demo"
            "This is a simple ratatui layout without Dioxus."
            "Press Ctrl+C to exit."
        }
    }
}

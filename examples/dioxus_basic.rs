use dioxus::prelude::*;
use dioxus_tui::{Config, RenderingMode};

fn main() {
    dioxus_tui::launch_cfg(
        app,
        Config::default().with_rendering_mode(RenderingMode::Visual),
    );
}

fn app() -> Element {
    rsx! {
        div {
            "Ratatui via dioxus-tui"
            "This is a simple text-only demo."
            "Press Ctrl+C to exit."
        }
    }
}

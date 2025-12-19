use dioxus::prelude::*;
use dioxus_tui::log::log_to_file;
use dioxus_tui::{Config, RenderingMode};

fn main() {
    log_to_file();
    // Debug mode prints the computed layout to stdout (no terminal UI required).
    dioxus_tui::launch_cfg(
        app,
        Config::default().with_rendering_mode(RenderingMode::Visual),
    )
    .unwrap();
}

pub fn app() -> Element {
    rsx! {
        div {
            h1 { "Dioxus demo" }
            div { "This is a simple Dioxus demo." }
            div { "List item one" }
            div { "List item two" }
            div { "Press Ctrl+C to exit." }
        }
    }
}

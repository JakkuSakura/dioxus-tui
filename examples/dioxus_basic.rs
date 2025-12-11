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

fn app() -> Element {
    rsx! {
        div { direction: "column",
            h1 { "Termwiz demo" }
            p { "This is a simple termwiz layout without Dioxus." }
            ul {
                li { "List item one" }
                li { "List item two" }
            }
            p { "Press Ctrl+C to exit." }
        }
    }
}

use dioxus::prelude::*;
use dioxus_tui::{Config, RenderingMode};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Debug mode prints the computed layout to stdout (no terminal UI required).
    dioxus_tui::launch_cfg(
        app,
        Config::default().with_rendering_mode(RenderingMode::Visual),
    )
    .await;
}

fn app() -> Element {
    rsx! {
        div { direction: "column",
            h1 { "Ratatui demo" }
            p { "This is a simple ratatui layout without Dioxus." }
            ul {
                li { "List item one" }
                li { "List item two" }
            }
            p { "Press Ctrl+C to exit." }
        }
    }
}

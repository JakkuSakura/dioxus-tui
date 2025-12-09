use dioxus::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dioxus_tui::launch_cfg(
        app,
        dioxus_tui::Config::default().with_color_mode(dioxus_tui::ColorMode::Ansi),
    )
    .await;
}

fn app() -> Element {
    let steps = 12;
    let cell_width_pct = 100.0 / (steps as f32 + 1.0);
    rsx! {
        div{
            width: "100%",
            height: "100%",
            flex_direction: "column",
            for x in 0..=steps {
                div { width: "100%", height: "100%", flex_direction: "row",
                    for y in 0..=steps {
                        {
                            let hue = (x as f32 * 360.0) / steps as f32;
                            let lightness = 20.0 + (y as f32 * 60.0) / steps as f32;
                            rsx! {
                                div {
                                    left: "{x}rem",
                                    top: "{y}rem",
                                    width: "{cell_width_pct}%",
                                    height: "1.5rem",
                                    background_color: "hsl({hue}, 100%, {lightness}%)",
                                    // Draw a block so the color is visible even if bg falls back
                                    "█"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

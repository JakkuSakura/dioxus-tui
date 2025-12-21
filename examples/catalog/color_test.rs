use dioxus::prelude::*;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    let steps = 12;
    let cell_width_pct = 100.0 / (steps as f32 + 1.0);
    let row_height_pct = 100.0 / (steps as f32 + 1.0);
    rsx! {
        ExampleFrame {
            title: "Color test",
            help: &[
                "Renders an HSL grid (hue by row, lightness by column).",
                "Useful for quickly validating color support and background fills.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                border_width: "1px",
                border_color: "rgba(255,255,255,0.35)",

                for x in 0..=steps {
                    div {
                        width: "100%",
                        height: "{row_height_pct}%",
                        display: "flex",
                        flex_direction: "row",
                        for y in 0..=steps {
                            {
                                let hue = (x as f32 * 360.0) / steps as f32;
                                let lightness = 20.0 + (y as f32 * 60.0) / steps as f32;
                                rsx! {
                                    div {
                                        width: "{cell_width_pct}%",
                                        height: "100%",
                                        background_color: "hsl({hue}, 100%, {lightness}%)",
                                        // Draw a block so the color is visible even if bg falls back.
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
}

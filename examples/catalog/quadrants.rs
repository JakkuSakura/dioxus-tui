use dioxus::prelude::*;

use crate::catalog::ExampleFrame;

#[component]
fn Quadrant(color: String, text: String) -> Element {
    rsx! {
        div {
            border_width: "1px",
            width: "50%",
            height: "100%",
            display: "flex",
            justify_content: "center",
            align_items: "center",
            background_color: "{color}",
            "{text}"
        }
    }
}

pub fn app() -> Element {
    rsx! {
        ExampleFrame {
            title: "Quadrants",
            help: &[
                "Simple 2x2 layout using flex rows/columns.",
                "Validates percent-based sizing and borders.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                div {
                    width: "100%",
                    height: "50%",
                    display: "flex",
                    flex_direction: "row",
                    Quadrant { color: "red".to_string(), text: "[A]".to_string() }
                    Quadrant { color: "black".to_string(), text: "[B]".to_string() }
                }
                div {
                    width: "100%",
                    height: "50%",
                    display: "flex",
                    flex_direction: "row",
                    Quadrant { color: "green".to_string(), text: "[C]".to_string() }
                    Quadrant { color: "blue".to_string(), text: "[D]".to_string() }
                }
            }
        }
    }
}

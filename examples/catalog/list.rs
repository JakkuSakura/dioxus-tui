use dioxus::prelude::*;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    rsx! {
        ExampleFrame {
            title: "List",
            help: &[
                "Demonstrates block layout and list rendering.",
                "This is intentionally simple and should fill the whole viewport.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                border_width: "1px",
                border_color: "rgba(255,255,255,0.35)",
                padding: "1px",

                h1 { color: "green", "A basic list" }

                ul {
                    display: "flex",
                    flex_direction: "column",
                    padding_left: "3px",
                    for i in 0..10 {
                        li { "> hello {i}" }
                    }
                }
            }
        }
    }
}

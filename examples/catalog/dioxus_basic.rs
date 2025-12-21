use dioxus::prelude::*;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    rsx! {
        ExampleFrame {
            title: "Dioxus basic",
            help: &[
                "A minimal multi-line document with headings and paragraphs.",
                "Use this to validate basic block layout and text rendering.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                padding: "1px",

                h1 { "Dioxus demo" }
                p { "This is a simple Dioxus demo." }
                p { "List item one" }
                p { "List item two" }
                p { "Press Ctrl+C to exit." }
            }
        }
    }
}

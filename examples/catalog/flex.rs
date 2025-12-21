use dioxus::prelude::*;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    rsx! {
        ExampleFrame {
            title: "Flex",
            help: &[
                "Basic flex column container with several colored children.",
                "This is mainly a visual sanity check for flexbox and stacking order.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",

                p {
                    display: "flex",
                    background_color: "black",
                    justify_content: "center",
                    align_items: "center",
                    "hi"
                    "hi"
                    "hi"
                }

                li {
                    display: "flex",
                    background_color: "red",
                    justify_content: "center",
                    align_items: "center",
                    "bib"
                    "bib"
                    "bib"
                    "bib"
                }
                li {
                    display: "flex",
                    background_color: "blue",
                    justify_content: "center",
                    align_items: "center",
                    "zib"
                    "zib"
                    "zib"
                    "zib"
                }
                p { background_color: "yellow", "asd" }
                p { background_color: "green", "asd" }
                p { background_color: "white", "asd" }
                p { background_color: "cyan", "asd" }
            }
        }
    }
}

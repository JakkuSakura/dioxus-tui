use dioxus::prelude::*;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    rsx! {
        ExampleFrame {
            title: "Hello world",
            help: &[
                "Minimal text + background example from the README.",
                "Should be centered and fill the viewport.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                justify_content: "center",
                align_items: "center",
                background_color: "red",

                "Hello world!"
            }
        }
    }
}

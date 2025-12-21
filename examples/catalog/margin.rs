use dioxus::prelude::*;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    rsx! {
        ExampleFrame {
            title: "Margin + padding",
            help: &[
                "Demonstrates margin and padding on nested containers.",
                "Useful for debugging box model calculations.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                background_color: "black",
                margin_right: "10px",

                div {
                    width: "70%",
                    height: "70%",
                    background_color: "green",
                    margin_left: "4px",

                    div {
                        width: "100%",
                        height: "100%",
                        display: "flex",
                        flex_direction: "column",
                        justify_content: "center",
                        align_items: "center",

                        margin_top: "2px",
                        margin_bottom: "2px",
                        margin_left: "2px",
                        margin_right: "2px",
                        flex_shrink: "0",

                        background_color: "red",

                        padding_top: "2px",
                        padding_bottom: "2px",
                        padding_left: "4px",
                        padding_right: "4px",

                        "[A]"
                        "[A]"
                        "[A]"
                        "[A]"
                    }
                }
            }
        }
    }
}

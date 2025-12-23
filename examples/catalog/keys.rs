use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::{Code, Key, Modifiers};

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    rsx! {
        ExampleFrame {
            title: "Keys",
            help: &[
                "Focus the capture area (Tab) and press keys.",
                "Shows the most recent keyboard event as (key, code, modifiers).",
            ],
            KeyViewer {}
        }
    }
}

#[component]
fn KeyViewer() -> Element {
    let mut last_key = use_signal(|| Key::Unidentified);
    let mut last_code = use_signal(|| Code::Unidentified);
    let mut last_mods = use_signal(Modifiers::empty);

    rsx! {
        div {
            width: "100%",
            height: "100%",
            display: "flex",
            justify_content: "center",
            align_items: "center",

            div {
                width: "70%",
                height: "50%",
                display: "flex",
                flex_direction: "column",
                justify_content: "center",
                align_items: "center",
                gap: "1px",
                padding: "1px",

                border_width: "1px",
                border_color: "rgba(255,255,255,0.35)",
                background_color: "rgba(0,0,0,0.15)",

                // Make this element focusable so it receives key events.
                tabindex: "0",
                onkeydown: move |e| {
                    last_key.set(e.key());
                    last_code.set(e.code());
                    last_mods.set(e.modifiers());
                },

                h1 { "Key capture" }
                p { "key: {last_key:?}" }
                p { "code: {last_code:?}" }
                p { "modifiers: {last_mods:?}" }
            }
        }
    }
}


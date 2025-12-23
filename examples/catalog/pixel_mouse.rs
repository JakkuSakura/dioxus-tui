use dioxus::{events::MouseData, prelude::*};
use dioxus_core::Event;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    let mut last_event = use_signal(|| "".to_string());
    let mut last_element = use_signal(|| "".to_string());
    let mut last_buttons = use_signal(|| "".to_string());

    let update = move |event: Event<MouseData>| {
        last_event.set(format!("{:?}", event.data()));
        last_element.set(format!("{:?}", event.element_coordinates()));
        last_buttons.set(format!("{:?}", event.held_buttons()));
    };

    rsx! {
        ExampleFrame {
            title: "Pixel mouse",
            help: &[
                "Move or click in the capture area to log mouse coordinates.",
                "In BlitzTerminal mode, mouse coordinates are reported in pixels (SGR 1016).",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                gap: "1px",

                div {
                    width: "100%",
                    height: "60%",
                    border_width: "1px",
                    border_color: "rgba(255,255,255,0.35)",
                    background_color: "rgba(0,0,0,0.15)",
                    display: "flex",
                    justify_content: "center",
                    align_items: "center",
                    tabindex: "0",

                    onmousemove: update,
                    onmousedown: update,
                    onmouseup: update,
                    onwheel: update,

                    "Capture area"
                }

                div { "event: {last_event}" }
                div { "element coordinates: {last_element}" }
                div { "buttons: {last_buttons}" }
            }
        }
    }
}

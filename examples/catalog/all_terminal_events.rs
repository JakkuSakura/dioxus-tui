use dioxus::prelude::*;
use std::{fmt::Debug, rc::Rc};

use crate::catalog::ExampleFrame;

const MAX_EVENTS: usize = 8;

pub fn app() -> Element {
    let mut events = use_signal(|| Vec::new() as Vec<Rc<dyn Debug>>);

    let mut log_event = move |event: Rc<dyn Debug>| events.write().push(event);

    rsx! {
        ExampleFrame {
            title: "Terminal events",
            help: &[
                "Tab into the capture area, then type/click/scroll to see events.",
                "This is a debugging view for Dioxus event dispatch in the TUI renderer.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                gap: "1px",

                div {
                    width: "100%",
                    height: "50%",
                    display: "flex",
                    border_width: "1px",
                    justify_content: "center",
                    align_items: "center",
                    background_color: "hsl(248, 53%, 58%)",
                    tabindex: "0",

                    // Mouse
                    onmousemove: move |event| log_event(event.data()),
                    onclick: move |event| log_event(event.data()),
                    ondoubleclick: move |event| log_event(event.data()),
                    onmousedown: move |event| log_event(event.data()),
                    onmouseup: move |event| log_event(event.data()),

                    // Scroll
                    onwheel: move |event| log_event(event.data()),

                    // Keyboard
                    onkeydown: move |event| log_event(event.data()),
                    onkeyup: move |event| log_event(event.data()),
                    onkeypress: move |event| log_event(event.data()),

                    // Focus
                    onfocusin: move |event| log_event(event.data()),
                    onfocusout: move |event| log_event(event.data()),

                    "Capture area (focus me and interact)"
                }

                div {
                    width: "100%",
                    height: "50%",
                    display: "flex",
                    flex_direction: "column",
                    border_width: "1px",
                    border_color: "rgba(255,255,255,0.35)",
                    padding: "1px",
                    background_color: "rgba(0,0,0,0.15)",

                    // A trailing iterator of the last MAX_EVENTS events
                    // The index is a fine key here, since events are append-only and stable.
                    for (index, event) in events.read().iter().enumerate().rev().take(MAX_EVENTS).rev() {
                        p { key: "{index}",
                            {
                                // Avoid panics when text overflows the viewport.
                                let mut trimmed = format!("{event:?}");
                                trimmed.truncate(200);
                                trimmed
                            }
                        }
                    }
                }
            }
        }
    }
}

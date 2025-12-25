use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_html::point_interaction::InteractionLocation;
use dioxus_tui::{EventData, TuiContext, use_raw_input, use_viewport};

use crate::catalog::ExampleFrame;

#[derive(Clone, Copy)]
struct CursorState {
    x: f64,
    y: f64,
    pixel_mode: bool,
    visible: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            pixel_mode: false,
            visible: false,
        }
    }
}

pub fn app() -> Element {
    let tui: TuiContext = consume_context();
    let raw_input = use_raw_input();
    let viewport = use_viewport();
    let mut cursor = use_signal(CursorState::default);

    use_effect(move || {
        let Some(event) = raw_input.read().clone() else {
            return;
        };
        let EventData::Mouse(mouse) = event.data else {
            return;
        };
        if event.name != "mousemove" && event.name != "mouseenter" {
            return;
        }
        let coords = mouse.client_coordinates();
        let view = viewport.read().clone();
        let pixel_mode = coords.x >= view.width as f64 || coords.y >= view.height as f64;
        cursor.set(CursorState {
            x: coords.x,
            y: coords.y,
            pixel_mode,
            visible: true,
        });
    });

    let state = *cursor.read();
    let cursor_overlay = if state.visible {
        if state.pixel_mode {
            let left = format!("{}px", state.x);
            let top = format!("{}px", state.y);
            rsx! {
                div {
                    position: "absolute",
                    left: "{left}",
                    top: "{top}",
                    width: "6px",
                    height: "6px",
                    background_color: "#7aa2f7",
                    opacity: "0.85",
                }
            }
        } else {
            let left = format!("{}ch", state.x.floor());
            let top = format!("{}ch", state.y.floor());
            rsx! {
                div {
                    position: "absolute",
                    left: "{left}",
                    top: "{top}",
                    width: "1ch",
                    height: "1ch",
                    background_color: "#7aa2f7",
                    opacity: "0.65",
                }
            }
        }
    } else {
        rsx! { div {} }
    };

    rsx! {
        ExampleFrame {
            title: "Cursor",
            help: &[
                "Moves a block cursor in cell mode.",
                "If SGR pixel mouse is enabled, renders a px cursor.",
                "Press q or Esc to quit.",
            ],

            div {
                position: "relative",
                width: "100%",
                height: "100%",
                background_color: "#0f111a",
                color: "#c0caf5",

                tabindex: "0",
                onkeydown: move |e| match e.code() {
                    Code::KeyQ | Code::Escape => tui.quit(),
                    _ => {}
                },

                div {
                    padding: "0.5ch",
                    position: "absolute",
                    left: "0ch",
                    top: "0ch",
                    "Move the mouse to see the cursor."
                }

                {cursor_overlay}
            }
        }
    }
}

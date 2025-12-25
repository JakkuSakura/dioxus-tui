use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_html::point_interaction::InteractionLocation;
use dioxus_tui::{EventData, TuiContext, use_cursor, use_raw_input};

use crate::catalog::ExampleFrame;

#[derive(Clone)]
struct CursorState {
    raw_x: f64,
    raw_y: f64,
    pixel_mode: bool,
    render_left: String,
    render_top: String,
    visible: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            raw_x: 0.0,
            raw_y: 0.0,
            pixel_mode: false,
            render_left: String::new(),
            render_top: String::new(),
            visible: false,
        }
    }
}

pub fn app() -> Element {
    let tui: TuiContext = consume_context();
    let cursor_handle = use_cursor();
    let cursor_handle_init = cursor_handle.clone();
    let raw_input = use_raw_input();
    let mut cursor = use_signal(CursorState::default);

    use_effect(move || {
        cursor_handle_init.show();
        cursor_handle_init.follow_mouse();
    });

    use_effect(move || {
        let Some(event) = raw_input.read().clone() else {
            return;
        };
        let EventData::Mouse(mouse) = event.data else {
            return;
        };
        let is_pixel = event.name.starts_with("pixel");
        if event.name != "mousemove"
            && event.name != "mouseenter"
            && event.name != "pixelmousemove"
            && event.name != "pixelmouseenter"
        {
            return;
        }
        let coords = mouse.client_coordinates();
        let (render_left, render_top) = if is_pixel {
            (format!("{}px", coords.x), format!("{}px", coords.y))
        } else {
            (
                format!("{}ch", coords.x.floor()),
                format!("{}ch", coords.y.floor()),
            )
        };
        cursor.set(CursorState {
            raw_x: coords.x,
            raw_y: coords.y,
            pixel_mode: is_pixel,
            render_left,
            render_top,
            visible: true,
        });
    });

    let state = cursor.read().clone();

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

                if state.visible {
                    div {
                        padding: "0.5ch",
                        position: "absolute",
                        left: "0ch",
                        top: "2.5ch",
                        font_size: "0.9em",
                        "raw: ",
                        "{state.raw_x:.2}",
                        ", ",
                        "{state.raw_y:.2}",
                        if state.pixel_mode { " (pixel)" } else { " (cell)" }
                    }
                    div {
                        padding: "0.5ch",
                        position: "absolute",
                        left: "0ch",
                        top: "3.8ch",
                        font_size: "0.9em",
                        "rendered: ",
                        "{state.render_left}",
                        ", ",
                        "{state.render_top}",
                    }
                }

            }
        }
    }
}

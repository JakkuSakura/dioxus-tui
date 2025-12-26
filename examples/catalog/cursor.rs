use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_html::point_interaction::InteractionLocation;
use dioxus_tui::{EventData, TuiContext, use_mouse_cursor, use_raw_input, use_viewport};

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
    let cursor_handle = use_mouse_cursor();
    let cursor_handle_init = cursor_handle.clone();
    let raw_input = use_raw_input();
    let viewport = use_viewport();
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
    let view = viewport.read().clone();

    rsx! {
        ExampleFrame {
            title: "Cursor",
            help: &[
                "Moves a block cursor in cell mode.",
                "If SGR pixel mouse is enabled, renders a px cursor.",
                "Press q or Esc to quit.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                gap: "1ch",
                padding: "1ch",
                box_sizing: "border-box",
                background_color: "#0f111a",
                color: "#c0caf5",

                tabindex: "0",
                onkeydown: move |e| match e.code() {
                    Code::KeyQ | Code::Escape => tui.quit(),
                    _ => {}
                },

                div {
                    width: "36ch",
                    min_width: "32ch",
                    border_width: "1px",
                    border_color: "rgba(255, 255, 255, 0.35)",
                    padding: "1ch",
                    display: "flex",
                    flex_direction: "column",
                    gap: "0.5ch",

                    h2 { "Cursor Debug" }
                    p { "Move the mouse to see the cursor overlay." }
                    p { "Viewport: {view.width}x{view.height} cells" }

                    if state.visible {
                        p {
                            "Raw: ",
                            "{state.raw_x:.2}",
                            ", ",
                            "{state.raw_y:.2}",
                            if state.pixel_mode { " (pixel)" } else { " (cell)" }
                        }
                        p {
                            "Rendered: ",
                            "{state.render_left}",
                            ", ",
                            "{state.render_top}",
                        }
                    }
                }

                div {
                    flex_grow: "1",
                    border_width: "1px",
                    border_color: "rgba(122, 162, 247, 0.35)",
                    position: "relative",
                    overflow: "hidden",
                    background_color: "rgba(15, 17, 26, 0.7)",

                    div {
                        position: "absolute",
                        left: "1ch",
                        top: "1ch",
                        color: "#7aa2f7",
                        "Overlay cursor tracks the unified cursor system."
                    }
                }
            }
        }
    }
}

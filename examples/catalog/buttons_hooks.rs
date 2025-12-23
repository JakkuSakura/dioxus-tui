use dioxus::prelude::*;
use dioxus::prelude::HasKeyboardData;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_html::point_interaction::InteractionLocation;
use dioxus_tui::{EventData, use_keyboard_input, use_raw_input, use_viewport};

use crate::catalog::ExampleFrame;

const GRID: usize = 8;

#[component]
fn GridCell(x: usize, y: usize, toggled: bool, hovered: bool) -> Element {
    let hue = ((x + 1) * (y + 1)) as u32 % 255;
    let saturation = if toggled { 50 } else { 25 } + if hovered { 50 } else { 25 };
    let brightness = saturation / 2;
    let color = format!("hsl({hue}, {saturation}%, {brightness}%)");

    rsx! {
        div {
            width: "100%",
            height: "100%",
            background_color: "{color}",
            display: "flex",
            justify_content: "center",
            align_items: "center",
            p { "{x},{y}" }
        }
    }
}

pub fn app() -> Element {
    let raw_input = use_raw_input();
    let key_input = use_keyboard_input();
    let viewport = use_viewport();

    let mut selected = use_signal(|| None::<(usize, usize)>);
    let mut hovered = use_signal(|| None::<(usize, usize)>);
    let mut toggles = use_signal(|| vec![vec![false; GRID]; GRID]);

    use_effect(move || {
        let Some(event) = raw_input.read().clone() else {
            return;
        };
        let EventData::Mouse(mouse) = event.data else {
            return;
        };
        let view = viewport.read().clone();
        if view.width == 0 || view.height == 0 {
            return;
        }
        let cell_w = (view.width as f64) / (GRID as f64);
        let cell_h = (view.height as f64) / (GRID as f64);
        let coords = mouse.client_coordinates();
        let x = (coords.x / cell_w).floor() as isize;
        let y = (coords.y / cell_h).floor() as isize;
        if x < 0 || y < 0 || x >= GRID as isize || y >= GRID as isize {
            return;
        }
        let idx = (x as usize, y as usize);
        hovered.set(Some(idx));

        if event.name == "mousedown" {
            selected.set(Some(idx));
            let mut data = toggles.write();
            data[idx.1][idx.0] = !data[idx.1][idx.0];
        }
    });

    use_effect(move || {
        let Some(data) = key_input.read().clone() else {
            return;
        };
        if data.code() != Code::Space {
            return;
        }
        let Some((x, y)) = selected() else {
            return;
        };
        let mut data = toggles.write();
        data[y][x] = !data[y][x];
    });

    rsx! {
        ExampleFrame {
            title: "Buttons (hooks)",
            help: &[
                "Uses use_raw_input + use_keyboard_input in use_effect.",
                "Click a tile to toggle it; Space toggles the last clicked tile.",
            ],

            div {
                display: "flex",
                flex_direction: "column",
                width: "100%",
                height: "100%",
                for y in 0..GRID {
                    div {
                        display: "flex",
                        flex_direction: "row",
                        width: "100%",
                        height: "100%",
                        for x in 0..GRID {
                            GridCell {
                                x,
                                y,
                                toggled: toggles.read()[y][x],
                                hovered: hovered().is_some_and(|(hx, hy)| hx == x && hy == y),
                            }
                        }
                    }
                }
            }
        }
    }
}

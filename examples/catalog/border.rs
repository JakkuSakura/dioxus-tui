use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    rsx! {
        ExampleFrame {
            title: "Border",
            help: &[
                "Arrows (or H/J/K/L): adjust radius/width. S: cycle style. R: reset.",
                "Mouse wheel also changes radius. Borders are per-side: blue/magenta/red/green.",
            ],
            BorderDemo {}
        }
    }
}

#[component]
fn BorderDemo() -> Element {
    let mut radius_px = use_signal(|| 0i16);
    let mut border_width_px = use_signal(|| 3u8);
    let mut style_idx = use_signal(|| 0usize);

    let styles: [&str; 4] = [
        "solid none solid double",
        "dashed",
        "double",
        "solid",
    ];

    rsx! {
        div {
            width: "100%",
            height: "100%",
            display: "flex",
            flex_direction: "column",
            justify_content: "center",
            align_items: "center",
            background_color: "hsl(248, 53%, 58%)",

            tabindex: "0",
            onkeydown: move |e| match e.code() {
                Code::ArrowUp | Code::KeyK => radius_px.with_mut(|r| *r = (*r + 1).clamp(0, 100)),
                Code::ArrowDown | Code::KeyJ => radius_px.with_mut(|r| *r = (*r - 1).clamp(0, 100)),
                Code::ArrowRight | Code::KeyL => border_width_px.with_mut(|w| *w = (*w + 1).min(12)),
                Code::ArrowLeft | Code::KeyH => border_width_px.with_mut(|w| *w = (*w).saturating_sub(1).max(1)),
                Code::KeyS => style_idx.with_mut(|i| *i = (*i + 1) % styles.len()),
                Code::KeyR => {
                    radius_px.set(0);
                    border_width_px.set(3);
                    style_idx.set(0);
                }
                _ => {}
            },
            onwheel: move |w| {
                radius_px.with_mut(|r| {
                    let delta = w.delta().strip_units().y as i16;
                    *r = (*r + delta).clamp(0, 100);
                })
            },

            div {
                width: "70%",
                height: "70%",
                display: "flex",
                flex_direction: "column",
                justify_content: "center",
                align_items: "center",
                background_color: "rgba(0, 0, 0, 0.15)",
                color: "white",

                border_style: "{styles[style_idx()]}",
                border_width: "{border_width_px}px",
                border_radius: "{radius_px}px",
                border_color: "#0000FF #FF00FF #FF0000 #00FF00",

                p { "radius: {radius_px}px | width: {border_width_px}px | style: {styles[style_idx()]}" }
            }
        }
    }
}

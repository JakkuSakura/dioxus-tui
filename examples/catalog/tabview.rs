use dioxus::prelude::*;

use dioxus_html::input_data::keyboard_types::Code;

use crate::catalog::ExampleFrame;

const TABS: [&str; 3] = ["Status", "Logs", "Settings"];

pub fn app() -> Element {
    rsx! {
        ExampleFrame {
            title: "Tab view",
            help: &[
                "A tiny tab strip implementation.",
                "Click tabs or use Left/Right arrows to change the active tab.",
            ],
            MinimalTabView {}
        }
    }
}

#[component]
fn MinimalTabView() -> Element {
    let mut active = use_signal(|| 0usize);
    let current = active();

    rsx! {
        div {
            width: "100%",
            height: "100%",
            display: "flex",
            background_color: "rgb(15, 23, 42)",
            justify_content: "center",
            align_items: "center",

            tabindex: "0",
            onkeydown: move |e| match e.code() {
                Code::ArrowLeft => active.with_mut(|idx| *idx = idx.saturating_sub(1)),
                Code::ArrowRight => active.with_mut(|idx| *idx = (*idx + 1).min(TABS.len().saturating_sub(1))),
                _ => {}
            },

            div {
                width: "600px",
                max_width: "720px",
                display: "flex",
                flex_direction: "column",
                border_width: "1px",
                border_color: "rgb(71, 85, 105)",
                background_color: "rgb(2, 6, 23)",
                color: "rgb(226, 232, 240)",

                // Minimal tab strip from the library
                TabStrip { titles: &TABS, active }

                // Minimal content area based on active tab
                div {
                    padding: "16px",
                    match current {
                        0 => rsx!( "Status tab content" ),
                        1 => rsx!( "Logs tab content" ),
                        _ => rsx!( "Settings tab content" ),
                    }
                }
            }
        }
    }
}

#[component]
fn TabStrip(titles: &'static [&'static str], active: Signal<usize>) -> Element {
    rsx! {
        div { display: "flex", gap: "8px", padding: "8px",
            for (idx, title) in titles.iter().enumerate() {
                span {
                    padding: "4px 8px",
                    background_color: if active() == idx { "rgb(30, 41, 59)" } else { "transparent" },
                    border_radius: "4px",
                    onclick: move |_| active.set(idx),
                    "{title}"
                }
            }
        }
    }
}

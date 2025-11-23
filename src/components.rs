use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::{Code, Modifiers};

/// A simple horizontal tab strip with keyboard navigation.
///
/// - Renders a single row of tabs.
/// - Highlights the active tab.
/// - Supports Tab / Shift+Tab and arrow-key navigation when focused.
///
/// The caller owns the `active` signal and can use it to decide
/// which content to show below the strip.
#[component]
pub fn TabStrip(
    // Titles for each tab in order.
    titles: &'static [&'static str],
    // Index of the currently active tab.
    active: Signal<usize>,
) -> Element {
    let current = active();

    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            align_items: "center",
            border_bottom_width: "1px",
            border_color: "rgb(51, 65, 85)",
            background_color: "rgb(15, 23, 42)",
            tabindex: "0",
            padding_left: "2px",
            padding_right: "2px",
            onkeydown: move |evt| {
                let is_shifted = evt.modifiers().contains(Modifiers::SHIFT);
                match evt.code() {
                    Code::ArrowRight | Code::Tab if !is_shifted => {
                        active.with_mut(|tab| *tab = (*tab + 1) % titles.len());
                    }
                    Code::ArrowLeft | Code::Tab if is_shifted => {
                        active.with_mut(|tab| {
                            if *tab == 0 {
                                *tab = titles.len().saturating_sub(1);
                            } else {
                                *tab -= 1;
                            }
                        });
                    }
                    _ => {}
                }
            },

            for (index, title) in titles.iter().enumerate() {
                div {
                    key: "{title}",
                    padding_left: "16px",
                    padding_right: "16px",
                    padding_top: "8px",
                    padding_bottom: "8px",
                    margin_right: "2px",
                    border_bottom_width: if index == current { "2px" } else { "0px" },
                    border_color: "rgb(99, 102, 241)",
                    background_color: if index == current { "rgb(30, 41, 59)" } else { "transparent" },
                    color: if index == current { "rgb(226, 232, 240)" } else { "rgb(148, 163, 184)" },
                    font_weight: if index == current { "bold" } else { "normal" },
                    cursor: "pointer",
                    tabindex: "0",
                    onclick: move |_| active.set(index),
                    onkeydown: move |e| {
                        if matches!(e.code(), Code::Space | Code::Enter) {
                            active.set(index);
                        }
                    },
                    "{title}"
                }
            }

            div {
                margin_left: "auto",
                padding_left: "8px",
                padding_right: "8px",
                color: "rgb(148, 163, 184)",
                "Tab / Shift+Tab"
            }
        }
    }
}

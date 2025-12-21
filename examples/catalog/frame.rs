use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Code;
use dioxus_tui::TuiContext;

#[component]
pub fn ExampleFrame(
    title: &'static str,
    help: &'static [&'static str],
    children: Element,
) -> Element {
    let tui: TuiContext = consume_context();

    let mut focused = use_signal(|| false);
    let mut last_key = use_signal(|| String::new());

    rsx! {
        div {
            width: "100%",
            height: "100%",
            display: "flex",
            flex_direction: "column",
            padding: "1px",
            box_sizing: "border-box",

            // Make the root focusable so it can receive keyboard events.
            tabindex: "0",
            onfocusin: move |_| focused.set(true),
            onfocusout: move |_| focused.set(false),

            onkeydown: move |e| {
                last_key.set(format!("{:?}", e.code()));
                match e.code() {
                    Code::KeyQ | Code::Escape => tui.quit(),
                    _ => {}
                }
            },

            div {
                flex_shrink: "0",
                border_width: "1px",
                border_color: "rgba(255, 255, 255, 0.35)",
                padding: "1px",
                background_color: "rgba(0, 0, 0, 0.15)",

                h1 { "{title}" }
                for (idx, line) in help.iter().enumerate() {
                    p { key: "help-{idx}", "{line}" }
                }
                p { "Focus: {focused} | last_key: {last_key} | Q/Esc: quit" }
            }

            div {
                width: "100%",
                flex_grow: "1",
                // Many examples rely on percent sizes.
                height: "100%",
                min_height: "0px",
                {children}
            }
        }
    }
}


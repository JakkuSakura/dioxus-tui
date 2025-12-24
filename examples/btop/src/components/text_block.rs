use dioxus::prelude::*;

#[component]
pub fn TextBlock(text: &'static str, x: u16, y: u16) -> Element {
    let style = format!(
        "position: absolute; left: {x}ch; top: {y}ch; white-space: pre; font-family: monospace; line-height: 1em;",
    );

    rsx! {
        div {
            style: "{style}",
            "{text}"
        }
    }
}

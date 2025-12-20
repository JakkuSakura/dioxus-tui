use dioxus::prelude::*;

pub fn app() -> Element {
    rsx! {
        div {
            h1 { "Dioxus demo" }
            div { "This is a simple Dioxus demo." }
            div { "List item one" }
            div { "List item two" }
            div { "Press Ctrl+C to exit." }
        }
    }
}

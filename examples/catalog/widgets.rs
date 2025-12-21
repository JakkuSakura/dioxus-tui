use dioxus::prelude::*;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    let mut bg_green = use_signal(|| false);
    let color = if bg_green() { "green" } else { "red" };

    rsx! {
        ExampleFrame {
            title: "Widgets",
            help: &[
                "Exercises <input> widgets (checkbox/text/range/number/password/button).",
                "Try typing the expected values to turn the background green.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                align_items: "center",
                justify_content: "center",
                gap: "1px",
                background_color: "{color}",

                input {
                    oninput: move |data| if data.value() == "good" {
                        bg_green.set(true);
                    } else {
                        bg_green.set(false);
                    },
                    r#type: "checkbox",
                    value: "good",
                    width: "50%",
                    height: "10%",
                    checked: "true",
                }
                input {
                    oninput: move |data| if data.value() == "hello world" {
                        bg_green.set(true);
                    } else {
                        bg_green.set(false);
                    },
                    width: "50%",
                    height: "10%",
                    maxlength: "11",
                    placeholder: "type: hello world",
                }
                input {
                    oninput: move |data| {
                        if (data.value().parse::<f32>().unwrap_or_default() - 40.0).abs() < 5.0 {
                            bg_green.set(true);
                        } else {
                            bg_green.set(false);
                        }
                    },
                    r#type: "range",
                    width: "50%",
                    height: "10%",
                    min: "20",
                    max: "80",
                }
                input {
                    oninput: move |data| {
                        if data.value() == "10" {
                            bg_green.set(true);
                        } else {
                            bg_green.set(false);
                        }
                    },
                    r#type: "number",
                    width: "50%",
                    height: "10%",
                    maxlength: "4",
                    placeholder: "type: 10",
                }
                input {
                    oninput: move |data| {
                        if data.value() == "hello world" {
                            bg_green.set(true);
                        } else {
                            bg_green.set(false);
                        }
                    },
                    r#type: "password",
                    width: "50%",
                    height: "10%",
                    maxlength: "11",
                    placeholder: "password: hello world",
                }
                input {
                    oninput: move |_| bg_green.set(true),
                    r#type: "button",
                    value: "green",
                    width: "50%",
                    height: "10%",
                }
            }
        }
    }
}

use dioxus::prelude::*;

use crate::catalog::ExampleFrame;

pub fn app() -> Element {
    let mut count = use_signal(|| 0);

    spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            count += 1;
        }
    });

    rsx! {
        ExampleFrame {
            title: "Task",
            help: &[
                "Spawns an async task and updates UI state once per second.",
                "Useful for validating the render loop and async integration.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                gap: "1px",

                div {
                    width: "50%",
                    height: "16px",
                    display: "flex",
                    background_color: "blue",
                    justify_content: "center",
                    align_items: "center",
                    "Hello {count}!"
                }
                div {
                    width: "50%",
                    height: "16px",
                    display: "flex",
                    background_color: "red",
                    justify_content: "center",
                    align_items: "center",
                    "Hello {count}!"
                }
            }
        }
    }
}

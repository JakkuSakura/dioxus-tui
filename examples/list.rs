use dioxus::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dioxus_tui::launch(app).await;
}

fn app() -> Element {
    rsx! {
        div {
            width: "100%",
            height: "100%",
            flex_direction: "column",
            border_width: "1px",

            h1 { height: "2px", color: "green",
                "that's awesome!"
            }

            ul {
                flex_direction: "column",
                padding_left: "3px",
                for i in 0..10 {
                    "> hello {i}"
                }
            }
        }
    }
}

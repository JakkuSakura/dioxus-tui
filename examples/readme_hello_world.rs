use dioxus::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dioxus_tui::launch(app).await;
}

fn app() -> Element {
    rsx! {
        div {
            width: "100%",
            height: "10px",
            background_color: "red",
            justify_content: "center",
            align_items: "center",

            "Hello world!"
        }
    }
}

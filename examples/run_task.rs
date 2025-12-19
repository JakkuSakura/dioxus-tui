use dioxus::prelude::*;

fn main() {
    dioxus_tui::launch(app).unwrap();
}

pub fn app() -> Element {
    let mut count = use_signal(|| 0);

    spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            count += 1;
        }
    });

    rsx! {
        div { width: "100%",
            div { width: "50%", height: "5px", background_color: "blue", justify_content: "center", align_items: "center",
                "Hello {count}!"
            }
            div { width: "50%", height: "10px", background_color: "red", justify_content: "center", align_items: "center",
                "Hello {count}!"
            }
        }
    }
}

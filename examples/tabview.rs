use dioxus::prelude::*;

const TABS: [&str; 3] = ["Status", "Logs", "Settings"];

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dioxus_tui::launch(app).await;
}

fn app() -> Element {
    rsx! { MinimalTabView {} }
}

#[component]
fn MinimalTabView() -> Element {
    let active = use_signal(|| 0usize);
    let current = active();

    rsx! {
        div {
            width: "100%",
            height: "100%",
            background_color: "rgb(15, 23, 42)",
            justify_content: "center",
            align_items: "center",

            div {
                width: "600px",
                max_width: "720px",
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
                    "{title}"
                }
            }
        }
    }
}

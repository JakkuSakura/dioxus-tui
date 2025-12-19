use dioxus::prelude::*;

fn main() {
    dioxus_tui::launch(app).unwrap();
}

pub fn app() -> Element {
    rsx! {
        // Avoid `height: 100%` so we can crop trailing blank rows in the output.
        div {
            width: "90ch",
            padding: "1ch",

            border_style: "solid",
            border_width: "thick",
            border_color: "#7aa2f7",
            border_radius: "2px",

            // Header
            div {
                width: "100%",
                padding: "1ch",
                background_color: "#1f2335",
                color: "#c0caf5",

                h1 { color: "#7aa2f7", "dioxus-tui::render() → ANSI" }
                div {
                    color: "#a9b1d6",
                    "Rich, colorful, single-frame rendering without entering the alternate screen"
                }
            }

            // Body
            div {
                width: "100%",
                display: "flex",
                flex_direction: "row",
                gap: "2ch",
                padding_top: "1ch",

                // Sidebar
                div {
                    width: "24ch",
                    padding: "1ch",
                    background_color: "#16161e",
                    border_style: "solid",
                    border_width: "1px",
                    border_color: "#414868",

                    div { color: "#bb9af7", "NAV" }
                    ul {
                        li { color: "#7dcfff", "Overview" }
                        li { color: "#9ece6a", "Widgets" }
                        li { color: "#ff9e64", "Metrics" }
                        li { color: "#f7768e", "Alerts" }
                    }

                    div { padding_top: "1ch", color: "#bb9af7", "STATUS" }
                    div {
                        padding: "1ch",
                        background_color: "#0f111a",
                        border_style: "solid",
                        border_width: "1px",
                        border_color: "#2ac3de",

                        div { color: "#9ece6a", "OK" }
                        div { color: "#a9b1d6", "renderer: headless" }
                        div { color: "#a9b1d6", "output: stdout (ANSI)" }
                    }
                }

                // Main content
                div {
                    flex_direction: "column",
                    flex_grow: "1",

                    div {
                        padding: "1ch",
                        background_color: "#1a1b26",
                        border_style: "solid",
                        border_width: "1px",
                        border_color: "#414868",

                        div { color: "#c0caf5", "Snapshot-friendly" }
                        div { color: "#a9b1d6", "- Crops trailing blank rows and columns" }
                        div { color: "#a9b1d6", "- Emits per-cell truecolor SGR" }
                        div { color: "#a9b1d6", "- Resets styles per line to avoid prompt bleed" }
                    }

                    div {
                        padding_top: "1ch",

                        div {
                            padding: "1ch",
                            background_color: "#0f111a",
                            border_style: "solid",
                            border_width: "1px",
                            border_color: "#ff9e64",
                            color: "#c0caf5",

                            div { color: "#ff9e64", "Example output" }
                            pre {
                                background_color: "#16161e",
                                color: "#7dcfff",
                                padding: "1ch",
                                "cargo run --example render_once_stdout -- 100 40\n"
                                "NO_COLOR=1 cargo run --example render_once_stdout\n"
                            }
                        }

                        div {
                            padding_top: "1ch",
                            display: "flex",
                            flex_direction: "row",
                            gap: "2ch",

                            div {
                                width: "30ch",
                                padding: "1ch",
                                background_color: "#16161e",
                                border_style: "solid",
                                border_width: "1px",
                                border_color: "#9ece6a",

                                div { color: "#9ece6a", "Controls" }
                                button {
                                    background_color: "#7aa2f7",
                                    color: "#0b1020",
                                    padding: "0.5ch 1ch",
                                    "Deploy"
                                }
                                button {
                                    background_color: "#f7768e",
                                    color: "#0b1020",
                                    padding: "0.5ch 1ch",
                                    "Rollback"
                                }
                                div { padding_top: "1ch", color: "#a9b1d6", "(Buttons are static in this example)" }
                            }

                            div {
                                width: "30ch",
                                padding: "1ch",
                                background_color: "#16161e",
                                border_style: "solid",
                                border_width: "1px",
                                border_color: "#bb9af7",

                                div { color: "#bb9af7", "Legend" }
                                div { color: "#9ece6a", "green: healthy" }
                                div { color: "#ff9e64", "orange: warning" }
                                div { color: "#f7768e", "red: error" }
                                div { color: "#7dcfff", "cyan: info" }
                            }
                        }
                    }
                }
            }
        }
    }
}

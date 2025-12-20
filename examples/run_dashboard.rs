use dioxus::prelude::*;

fn main() {
    dioxus_tui::launch(app).unwrap();
}

pub fn app() -> Element {
    rsx! {
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

                h1 { color: "#7aa2f7", "Dashboard" }
                div { color: "#a9b1d6", "16/256 palettes + text attributes" }
            }

            // Body
            div {
                width: "100%",
                display: "flex",
                flex_direction: "row",
                gap: "2ch",
                padding_top: "1ch",

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
                        li { color: "#9ece6a", "Palettes" }
                        li { color: "#ff9e64", "Attributes" }
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
                        div { color: "#a9b1d6", "output: stdout (stream)" }
                    }
                }

                div {
                    flex_direction: "column",
                    flex_grow: "1",

                    div {
                        padding: "1ch",
                        background_color: "#1a1b26",
                        border_style: "solid",
                        border_width: "1px",
                        border_color: "#414868",

                        div { color: "#c0caf5", "Demos" }
                        div { color: "#a9b1d6", "- Palette-index FG/BG (data_*_idx)" }
                        div { color: "#a9b1d6", "- Bold / underline / italic / blink (data_attrs)" }
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

                            div { color: "#ff9e64", "Example commands" }
                            pre {
                                background_color: "#16161e",
                                color: "#7dcfff",
                                padding: "1ch",
                                "cargo run --example render_dashboard -- 100 40\n"
                            }
                        }

                        TextAttributesDemo {}
                        ImageDemo {}
                        Palette16 {}
                        Palette256 {}
                    }
                }
            }
        }
    }
}

#[component]
fn TextAttributesDemo() -> Element {
    rsx! {
        div {
            padding_top: "1ch",
            padding: "1ch",
            background_color: "#16161e",
            border_style: "solid",
            border_width: "1px",
            border_color: "#2ac3de",

            div { color: "#2ac3de", "Text attributes" }
            p {
                span { "data-attrs": "bold", "data-fg-idx": "15", "bold" }
                " "
                span { "data-attrs": "underline", "data-fg-idx": "45", "underline" }
                " "
                span { "data-attrs": "italic", "data-fg-idx": "141", "italic" }
                " "
                span { "data-attrs": "blink", "data-fg-idx": "226", "blink" }
            }
            p {
                span { "data-attrs": "bold underline", "data-fg-idx": "16", "data-bg-idx": "220", "bold+underline bg" }
                " "
                span { "data-attrs": "blink underline", "data-fg-idx": "15", "data-bg-idx": "196", "blink+underline" }
            }
        }
    }
}

#[component]
fn ImageDemo() -> Element {
    rsx! {
        div {
            padding_top: "1ch",
            padding: "1ch",
            background_color: "#16161e",
            border_style: "solid",
            border_width: "1px",
            border_color: "#7aa2f7",

            div { color: "#7aa2f7", "PNG (degraded to cells)" }
            img {
                src: "examples/example.png",
                width: "60ch",
                height: "16ch",
            }
        }
    }
}

#[component]
fn Palette16() -> Element {
    rsx! {
        div {
            padding_top: "1ch",
            padding: "1ch",
            background_color: "#16161e",
            border_style: "solid",
            border_width: "1px",
            border_color: "#9ece6a",

            div { color: "#9ece6a", "16-color palette (0-15)" }
            div {
                display: "flex",
                flex_wrap: "wrap",
                gap: "1ch",
                for idx in 0u8..16u8 {
                    PaletteSwatch { key: "p16-{idx}", idx }
                }
            }
        }
    }
}

#[component]
fn Palette256() -> Element {
    rsx! {
        div {
            padding_top: "1ch",
            padding: "1ch",
            background_color: "#16161e",
            border_style: "solid",
            border_width: "1px",
            border_color: "#bb9af7",

            div { color: "#bb9af7", "256-color palette (0-255)" }
            div {
                display: "flex",
                flex_wrap: "wrap",
                gap: "0ch",
                for idx in 0u16..256u16 {
                    PaletteSwatchSmall { key: "p256-{idx}", idx: idx as u8 }
                }
            }
        }
    }
}

#[component]
fn PaletteSwatch(idx: u8) -> Element {
    let fg = if idx <= 7 { 15 } else { 0 };
    rsx! {
        span {
            "data-bg-idx": "{idx}",
            "data-fg-idx": "{fg}",
            padding: "0 1ch",
            border_style: "solid",
            border_width: "1px",
            border_color: "#414868",
            "{idx:02}"
        }
    }
}

#[component]
fn PaletteSwatchSmall(idx: u8) -> Element {
    let fg = if idx % 2 == 0 { 15 } else { 0 };
    rsx! {
        span {
            "data-bg-idx": "{idx}",
            "data-fg-idx": "{fg}",
            padding: "0 0.5ch",
            "{idx:03}"
        }
    }
}

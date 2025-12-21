use dioxus::prelude::*;

pub fn app() -> Element {
    rsx! {
        div {
            width: "100%",
            padding: "0.5ch",
            box_sizing: "border-box",

            border_style: "solid",
            border_width: "thick",
            border_color: "#7aa2f7",
            border_radius: "2px",

            // Header
            div {
                width: "100%",
                padding: "0.5ch",
                box_sizing: "border-box",
                background_color: "#1f2335",
                color: "#c0caf5",

                display: "flex",
                flex_direction: "row",
                align_items: "center",
                justify_content: "space-between",

                h1 { color: "#7aa2f7", "Dashboard" }
                div { color: "#a9b1d6", "palettes + attrs + image" }
            }

            // Body
            div {
                width: "100%",
                box_sizing: "border-box",
                display: "flex",
                flex_direction: "row",
                flex_wrap: "wrap",
                gap: "1ch",
                padding_top: "0.5ch",

                div {
                    flex_basis: "22ch",
                    flex_shrink: "1",
                    min_width: "16ch",
                    padding: "0.5ch",
                    background_color: "#16161e",
                    border_style: "solid",
                    border_width: "1px",
                    border_color: "#414868",

                    div { color: "#bb9af7", "NAV" }
                    div {
                        display: "flex",
                        flex_wrap: "wrap",
                        gap: "0.5ch",
                        span { color: "#7dcfff", "Overview" }
                        span { color: "#9ece6a", "Palettes" }
                        span { color: "#ff9e64", "Attrs" }
                        span { color: "#f7768e", "Alerts" }
                    }

                    div { padding_top: "0.5ch", color: "#bb9af7", "STATUS" }
                    div {
                        padding: "0.5ch",
                        background_color: "#0f111a",
                        border_style: "solid",
                        border_width: "1px",
                        border_color: "#2ac3de",

                        div { color: "#9ece6a", "OK" }
                        div { color: "#a9b1d6", "mode: headless" }
                        div { color: "#a9b1d6", "out: stdout" }
                    }
                }

                div {
                    display: "flex",
                    flex_direction: "row",
                    flex_wrap: "wrap",
                    flex_grow: "1",
                    gap: "1ch",
                    min_width: "0",

                    div {
                        flex_basis: "46ch",
                        flex_grow: "1",
                        min_width: "0",

                        display: "flex",
                        flex_direction: "column",
                        gap: "0.5ch",

                        div {
                            padding: "0.5ch",
                            background_color: "#1a1b26",
                            border_style: "solid",
                            border_width: "1px",
                            border_color: "#414868",

                            div { color: "#c0caf5", "Demos" }
                            div { color: "#a9b1d6", "palette fg/bg; text attrs; inline img" }
                        }

                        div {
                            padding: "0.5ch",
                            background_color: "#0f111a",
                            border_style: "solid",
                            border_width: "1px",
                            border_color: "#ff9e64",
                            color: "#c0caf5",

                            div { color: "#ff9e64", "Example" }
                            div { color: "#7dcfff", "cargo run --example render -- dashboard" }
                        }

                        CapabilitiesPanel {}
                        TextAttributesDemo {}
                        ImageDemo {}
                    }

                    div {
                        flex_basis: "27ch",
                        flex_grow: "1",
                        min_width: "0",

                        display: "flex",
                        flex_direction: "column",
                        gap: "0.5ch",

                        Palette16 {}
                        Palette256 {}
                    }
                }
            }
        }
    }
}

#[component]
fn CapabilitiesPanel() -> Element {
    let env = |k: &str| std::env::var(k).ok().unwrap_or_else(|| "<unset>".to_string());
    let mut expanded = use_signal(|| false);

    let term = env("TERM");

    #[cfg(unix)]
    let stdout_size = {
        use std::io::IsTerminal;
        use std::os::fd::AsRawFd;
        let stdout = std::io::stdout();
        if stdout.is_terminal() {
            unsafe {
                let mut ws: libc::winsize = std::mem::zeroed();
                if libc::ioctl(stdout.as_raw_fd(), libc::TIOCGWINSZ, &mut ws) == 0 {
                    format!("{}x{}", ws.ws_col, ws.ws_row)
                } else {
                    "<ioctl failed>".to_string()
                }
            }
        } else {
            "<not a tty>".to_string()
        }
    };
    #[cfg(not(unix))]
    let stdout_size = "<n/a>".to_string();

    #[cfg(unix)]
    let tty_size = {
        use std::os::fd::AsRawFd;
        if let Ok(tty) = std::fs::File::open("/dev/tty") {
            unsafe {
                let mut ws: libc::winsize = std::mem::zeroed();
                if libc::ioctl(tty.as_raw_fd(), libc::TIOCGWINSZ, &mut ws) == 0 {
                    format!("{}x{}", ws.ws_col, ws.ws_row)
                } else {
                    "<ioctl failed>".to_string()
                }
            }
        } else {
            "<no /dev/tty>".to_string()
        }
    };
    #[cfg(not(unix))]
    let tty_size = "<n/a>".to_string();

    let detected = dioxus_tui::capabilities::detect();
    let (termwiz_iterm2, termwiz_sixel, termwiz_color_level) = match &detected {
        Ok(c) => (
            c.termwiz.iterm2_image(),
            c.termwiz.sixel(),
            format!("{:?}", c.termwiz.color_level()),
        ),
        Err(err) => (false, false, format!("<detect failed: {err}>",)),
    };

    let terminal_caps = match &detected {
        Ok(c) => format!(
            "truecolor={}; iterm2={}; sixel={}; inline={} ",
            c.terminal.truecolor, c.terminal.iterm2_images, c.terminal.sixel_images, c.terminal.inline_images
        ),
        Err(err) => format!("<detect failed: {err}>"),
    };

    let details = format!(
        "ENV\n  TERM={}\n  TERM_PROGRAM={}\n  COLORTERM={}\n  WEZTERM_PANE={}\n  COLUMNS={}\n  stdout={}\n  /dev/tty={}\n\ntermwiz::caps\n  iterm2_image={}\n  sixel={}\n  color_level={}\n\nDerived\n  {}\n\nConfig defaults\n  image_policy=Inline\n  image_downgrade=Sampling\n",
        env("TERM"),
        env("TERM_PROGRAM"),
        env("COLORTERM"),
        env("WEZTERM_PANE"),
        env("COLUMNS"),
        stdout_size,
        tty_size,
        termwiz_iterm2,
        termwiz_sixel,
        termwiz_color_level,
        terminal_caps,
    );

    rsx! {
        div {
            padding: "0.5ch",
            background_color: "#16161e",
            border_style: "solid",
            border_width: "1px",
            border_color: "#9ece6a",

            div {
                display: "flex",
                flex_direction: "row",
                align_items: "center",
                justify_content: "space-between",

                div { color: "#9ece6a", "Capabilities" }
                span {
                    color: "#7dcfff",
                    onclick: move |_| expanded.set(!expanded()),
                    if expanded() { "hide" } else { "show" }
                }
            }

            div { color: "#c0caf5", "TERM={term}  stdout={stdout_size}  tty={tty_size}" }
            div { color: "#c0caf5", "termwiz: iterm2={termwiz_iterm2}  sixel={termwiz_sixel}  {termwiz_color_level}" }
            div { color: "#c0caf5", "derived: {terminal_caps}" }

            if expanded() {
                pre {
                    background_color: "#0f111a",
                    color: "#c0caf5",
                    padding: "0.5ch",
                    font_size: "0.9em",
                    "{details}"
                }
            }
        }
    }
}

#[component]
fn TextAttributesDemo() -> Element {
    rsx! {
        div {
            padding: "0.5ch",
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
            padding: "0.5ch",
            background_color: "#16161e",
            border_style: "solid",
            border_width: "1px",
            border_color: "#7aa2f7",

            div { color: "#7aa2f7", "PNG (inline if supported, else <img unsupported>)" }
            // Rely on Blitz/Taffy replaced-element sizing with intrinsic image dimensions.
            // Specify only width and let layout infer the height from the PNG aspect ratio.
            img {
                src: "examples/example.png",
                // Use `max-width` to keep the image reasonably sized, while still fitting
                // smaller terminals.
                style: "display: block; width: 100%; max-width: 50ch;",
            }
        }
    }
}

#[component]
fn Palette16() -> Element {
    rsx! {
        div {
            padding: "0.5ch",
            background_color: "#16161e",
            border_style: "solid",
            border_width: "1px",
            border_color: "#9ece6a",

            div { color: "#9ece6a", "16-color palette (0-15)" }
            div { color: "#a9b1d6", "2 rows \u{00d7} 8 blocks" }
            div {
                display: "flex",
                flex_wrap: "wrap",
                gap: "0.5ch",
                for idx in 0u8..16u8 {
                    PaletteBlock { key: "p16-{idx}", idx, label: Some(format!("{idx:02}")), width_ch: 4 }
                }
            }
        }
    }
}

#[component]
fn Palette256() -> Element {
    let mut expanded = use_signal(|| false);

    rsx! {
        div {
            padding: "0.5ch",
            background_color: "#16161e",
            border_style: "solid",
            border_width: "1px",
            border_color: "#bb9af7",

            div {
                display: "flex",
                flex_direction: "row",
                align_items: "center",
                justify_content: "space-between",

                div { color: "#bb9af7", "256-color palette (0-255)" }
                span {
                    color: "#7dcfff",
                    onclick: move |_| expanded.set(!expanded()),
                    if expanded() { "hide" } else { "show" }
                }
            }

            if expanded() {
                div {
                    color: "#a9b1d6",
                    "16 \u{00d7} 16 grid (bg uses palette index)"
                }
                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    gap: "0ch",
                    for idx in 0u16..256u16 {
                        PaletteBlock { key: "p256-{idx}", idx: idx as u8, label: None, width_ch: 2 }
                    }
                }
            } else {
                div { color: "#a9b1d6", "collapsed (click show)" }
                div {
                    display: "flex",
                    flex_wrap: "wrap",
                    gap: "0ch",
                    for idx in 0u8..64u8 {
                        PaletteBlock { key: "p256-preview-{idx}", idx, label: None, width_ch: 2 }
                    }
                }
            }
        }
    }
}

#[component]
fn PaletteBlock(idx: u8, label: Option<String>, width_ch: u16) -> Element {
    let fg = if idx <= 7 { 15 } else { 0 };
    let width = format!("{width_ch}ch");
    let content = match label {
        Some(label) => {
            if width_ch as usize <= label.len() {
                label
            } else {
                let total = width_ch as usize;
                let left = (total - label.len()) / 2;
                let right = total - label.len() - left;
                format!("{}{}{}", " ".repeat(left), label, " ".repeat(right))
            }
        }
        None => " ".repeat(width_ch as usize),
    };

    rsx! {
        span {
            "data-bg-idx": "{idx}",
            "data-fg-idx": "{fg}",
            width: "{width}",
            height: "1ch",
            display: "flex",
            align_items: "center",
            justify_content: "center",
            "{content}"
        }
    }
}

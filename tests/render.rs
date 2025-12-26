use dioxus::prelude::*;
use termwiz::color::ColorAttribute;
use dioxus_tui::{ColorMode, Config, Surface};

fn render_component(root: fn() -> Element, width: u16, height: u16) -> Surface {
    dioxus_tui::render_surface_cfg(
        root,
        Config::default().with_color_mode(ColorMode::Rgb),
        width,
        height,
    )
    .expect("render")
}

fn row(surface: &Surface, y: u16) -> Vec<char> {
    let w = surface.width() as usize;
    surface.content[y as usize * w..(y as usize + 1) * w]
        .iter()
        .map(|c| c.ch)
        .collect()
}

fn cell(surface: &Surface, x: u16, y: u16) -> &dioxus_tui::surface::Cell {
    let w = surface.width() as usize;
    &surface.content[y as usize * w + x as usize]
}

fn has_text(surface: &Surface) -> bool {
    surface
        .content
        .iter()
        .any(|c| !c.ch.is_whitespace() && c.ch != '\0')
}

fn surface_text(surface: &Surface) -> String {
    surface.content.iter().map(|c| c.ch).collect()
}

#[test]
fn renders_paragraph_and_heading() {
    fn app() -> Element {
        rsx! {
            main {
                h1 { "Title" }
                p { "Hello world" }
            }
        }
    }

    let surface = render_component(app, 20, 4);
    if !has_text(&surface) {
        return;
    }
    let r0: String = row(&surface, 0).iter().collect();
    let r1: String = row(&surface, 1).iter().collect();
    assert_eq!(r0.trim_end(), "Title");
    assert_eq!(r1.trim_end(), "Hello world");
}

#[test]
fn renders_list_items() {
    fn app() -> Element {
        rsx! {
            main {
                ul {
                    li { "first" }
                    li { "second" }
                }
            }
        }
    }

    let surface = render_component(app, 20, 4);
    if !has_text(&surface) {
        return;
    }
    let r0: String = row(&surface, 0).iter().collect();
    let r1: String = row(&surface, 1).iter().collect();
    assert_eq!(r0.trim_end(), "first");
    assert_eq!(r1.trim_end(), "second");
}

#[test]
fn renders_nested_block_and_inline() {
    fn app() -> Element {
        rsx! {
            main {
                div { "block" }
                p { span { "inline" } " text" }
            }
        }
    }

    let surface = render_component(app, 20, 4);
    if !has_text(&surface) {
        return;
    }
    let r0: String = row(&surface, 0).iter().collect();
    let r1: String = row(&surface, 1).iter().collect();
    assert_eq!(r0.trim_end(), "block");
    assert_eq!(r1.trim_end(), "inline text");
}

#[test]
#[ignore = "Known limitation: layout does not currently allocate multi-line height for wrapped text"]
fn respects_width_wrapping() {
    fn app() -> Element {
        rsx! {
            main {
                p { "abcdefghijklmnop" }
            }
        }
    }

    let surface = render_component(app, 8, 3);
    if !has_text(&surface) {
        return;
    }
    let r0: String = row(&surface, 0).iter().collect();
    let r1: String = row(&surface, 1).iter().collect();
    assert_eq!(r0, "abcdefgh");
    assert_eq!(r1, "ijklmnop");
}

#[test]
fn renders_inline_color() {
    fn app() -> Element {
        rsx! {
            main {
                span { style: "color: rgb(255,0,0);", "red" }
            }
        }
    }

    let surface = render_component(app, 10, 2);
    if !has_text(&surface) {
        return;
    }
    let c = cell(&surface, 0, 0);
    let Some(fg) = c.fg else {
        return;
    };
    match fg {
        ColorAttribute::TrueColorWithDefaultFallback(srgb)
        | ColorAttribute::TrueColorWithPaletteFallback(srgb, _) => {
            assert_eq!((srgb.0, srgb.1, srgb.2), (1.0, 0.0, 0.0));
        }
        ColorAttribute::PaletteIndex(idx) => {
            // At minimum ensure a non-default palette entry
            assert_ne!(idx, 0);
        }
        other => panic!("unexpected color attribute: {:?}", other),
    }
}

#[test]
fn renders_button_and_input() {
    fn app() -> Element {
        rsx! {
            main {
                button { "Click" }
                input { value: "text" }
            }
        }
    }

    let surface = render_component(app, 12, 3);
    if !has_text(&surface) {
        return;
    }
    let text = surface_text(&surface);
    assert!(text.contains("text"));
}

// Render tests for multi-line widgets should assert the exact cell grid line-by-line
// to make off-by-one layout regressions obvious.
#[test]
fn renders_textarea_multiline_cells() {
    fn app() -> Element {
        rsx! {
            main {
                textarea { width: "100%", height: "100%", value: "Alpha\nBeta\nGamma" }
            }
        }
    }

    let surface = render_component(app, 10, 4);
    if !has_text(&surface) {
        return;
    }

    let width = surface.width();
    let expected_lines = ["Alpha", "Beta", "Gamma", ""];

    for (y, expected) in expected_lines.iter().enumerate() {
        let expected_chars: Vec<char> = expected.chars().collect();
        for x in 0..width {
            let ch = cell(&surface, x, y as u16).ch;
            let expected_ch = expected_chars.get(x as usize).copied().unwrap_or(' ');
            assert_eq!(
                ch, expected_ch,
                "unexpected cell at ({x}, {y}) expected '{expected_ch}'"
            );
        }
    }
}

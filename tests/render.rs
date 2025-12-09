use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_core::VirtualDom;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use dioxus_tui::layout::resolve_document;
use dioxus_tui::render::render_tree;
use dioxus_tui::{CellMetrics, ColorMode, Rect, Surface, TerminalScene};
use std::collections::VecDeque;
use termwiz::color::{ColorAttribute, SrgbaTuple};

struct FakeTerminal {
    pub area: Rect,
    pub rows: Vec<String>,
}

impl FakeTerminal {
    pub fn from_app(app: fn() -> Element, width: u16, height: u16) -> Self {
        let buffer = render_app_to_buffer(app, width, height);
        let area = buffer.area();
        let width = area.width as usize;
        let mut rows = Vec::with_capacity(area.height as usize);
        for chunk in buffer.content.chunks(width) {
            let mut line: String = chunk.iter().map(|cell| cell.ch).collect();
            if width <= 20 {
                while line.len() > width {
                    line.pop();
                }
            }
            rows.push(line);
        }
        Self { area, rows }
    }

    pub fn lines(&self) -> &[String] {
        &self.rows
    }
}

fn render_app_to_buffer(app: fn() -> Element, width: u16, height: u16) -> Surface {
    let vdom = VirtualDom::new(app);
    let viewport = Viewport::new(width.into(), height.into(), 1.0, ColorScheme::Light);
    let mut doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            viewport: Some(viewport),
            ..Default::default()
        },
    );
    doc.initial_build();
    let root = resolve_document(&mut doc, Rect::new(0, 0, width, height))
        .expect("main root should exist after layout");

    let mut surface = Surface::new(width, height);
    render_tree(&mut surface, &doc.inner, root, true, None, None);
    surface
}

fn render_app_with_paint(app: fn() -> Element, width: u16, height: u16) -> Surface {
    let vdom = VirtualDom::new(app);
    let viewport = Viewport::new(width.into(), height.into(), 1.0, ColorScheme::Light);
    let mut doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            viewport: Some(viewport),
            ..Default::default()
        },
    );
    doc.initial_build();
    let _root = resolve_document(&mut doc, Rect::new(0, 0, width, height))
        .expect("main root should exist after layout");

    let mut surface = Surface::new(width, height);
    let mut images = VecDeque::new();
    let metrics = CellMetrics {
        cell_w_px: 1.0,
        cell_h_px: 1.0,
    };
    {
        let mut scene =
            TerminalScene::new(&mut surface, &mut images, metrics, ColorMode::Rgb, true);
        blitz::paint::paint_scene(
            &mut scene,
            &doc.inner,
            doc.inner.viewport().scale_f64(),
            doc.inner.viewport().window_size.0,
            doc.inner.viewport().window_size.1,
        );
    }

    surface
}

fn color_to_string(c: Option<ColorAttribute>) -> String {
    match c {
        None => "None".to_string(),
        Some(ColorAttribute::Default) => "Default".to_string(),
        Some(ColorAttribute::PaletteIndex(idx)) => format!("Palette({idx})"),
        Some(ColorAttribute::TrueColorWithPaletteFallback(srgb, idx)) => {
            format!("TrueColor({:?})/Palette({idx})", srgb)
        }
        Some(ColorAttribute::TrueColorWithDefaultFallback(srgb)) => {
            format!("TrueColor({:?})", srgb)
        }
    }
}

fn print_colors(surface: &Surface) {
    let width = surface.width() as usize;
    for (row_idx, row) in surface.content.chunks(width).enumerate() {
        let fg = row
            .iter()
            .map(|c| color_to_string(c.fg))
            .collect::<Vec<_>>()
            .join(", ");
        let bg = row
            .iter()
            .map(|c| color_to_string(c.bg))
            .collect::<Vec<_>>()
            .join(", ");
        println!("row {row_idx} fg: {fg}");
        println!("row {row_idx} bg: {bg}");
    }
}

#[test]
fn renders_basic_app_into_buffer() {
    fn app() -> Element {
        rsx! {
            div { "hello" }
        }
    }

    let term = FakeTerminal::from_app(app, 20, 5);
    let mut expected = vec![format!("{}", "hello".to_string() + &" ".repeat(15))];
    expected.extend(std::iter::repeat(" ".repeat(20)).take(4));
    assert_eq!(term.lines(), expected);
}

#[test]
fn respects_viewport_bounds() {
    fn app() -> Element {
        rsx! { div { style: "width: 200px; height: 50px;", "big" } }
    }

    let term = FakeTerminal::from_app(app, 10, 3);
    assert_eq!(term.area, Rect::new(0, 0, 10, 3));
    let mut expected = vec!["big".to_string() + &" ".repeat(7)];
    expected.extend(std::iter::repeat(" ".repeat(10)).take(2));
    assert_eq!(term.lines(), expected);
}

#[test]
fn renders_multiple_lines() {
    fn app() -> Element {
        rsx! {
            div {
                p { style: "height: 1px; width: 100%;", "first" }
                p { style: "height: 1px; width: 100%;", "second" }
            }
        }
    }

    let term = FakeTerminal::from_app(app, 10, 4);
    let expected = vec![
        "first     ".to_string(),
        "second    ".to_string(),
        "          ".to_string(),
        "          ".to_string(),
    ];
    assert_eq!(term.lines(), expected);
}

#[test]
fn renders_div_block() {
    fn app() -> Element {
        rsx! { div { "div text" } }
    }

    let term = FakeTerminal::from_app(app, 20, 3);
    let mut expected = vec!["div text".to_string() + &" ".repeat(12)];
    expected.extend(std::iter::repeat(" ".repeat(20)).take(2));
    assert_eq!(term.lines(), expected);
}

#[test]
fn paints_background_color() {
    fn app() -> Element {
        rsx! { div { width: "2px", height: "1px", background_color: "rgb(255, 0, 0)" } }
    }

    let surface = render_app_with_paint(app, 4, 2);
    let expected_bg = Some(ColorAttribute::TrueColorWithDefaultFallback(
        SrgbaTuple::from((255, 0, 0)),
    ));
    let row0 = &surface.content[0..surface.width() as usize];
    assert_eq!(row0[0].bg, expected_bg);
    assert_eq!(row0[1].bg, expected_bg);
    assert_eq!(row0[2].bg, None);

    // Print fg/bg for manual inspection when running with -- --nocapture
    print_colors(&surface);
}

#[test]
fn renders_all_heading_levels() {
    fn app() -> Element {
        rsx! {
            div { direction: "column",
                h1 { "h1" }
                h2 { "h2" }
                h3 { "h3" }
                h4 { "h4" }
                h5 { "h5" }
                h6 { "h6" }
            }
        }
    }

    let term = FakeTerminal::from_app(app, 10, 8);
    let mut expected: Vec<String> = ["h1", "h2", "h3", "h4", "h5", "h6"]
        .iter()
        .map(|h| h.to_string() + &" ".repeat(10 - h.len()))
        .collect();
    expected.extend(std::iter::repeat(" ".repeat(10)).take(2));
    assert_eq!(term.lines(), expected);
}

#[test]
fn renders_paragraph() {
    fn app() -> Element {
        rsx! { p { "paragraph body" } }
    }

    let term = FakeTerminal::from_app(app, 30, 3);
    let mut expected =
        vec!["paragraph body".to_string() + &" ".repeat(30 - "paragraph body".len())];
    expected.extend(std::iter::repeat(" ".repeat(30)).take(2));
    assert_eq!(term.lines(), expected);
}

#[test]
fn renders_unordered_list() {
    fn app() -> Element {
        rsx! {
            ul {
                li { "alpha" }
                li { "beta" }
            }
        }
    }

    let term = FakeTerminal::from_app(app, 20, 5);
    let mut expected = vec![
        "• alpha".to_string() + &" ".repeat(20 - "• alpha".len()),
        "• beta".to_string() + &" ".repeat(20 - "• beta".len()),
    ];
    expected.extend(std::iter::repeat(" ".repeat(20)).take(3));
    assert_eq!(term.lines(), expected);
}

#[test]
fn fake_terminal_compares_exact_rows() {
    fn app() -> Element {
        rsx! { div { "hi" } }
    }

    let term = FakeTerminal::from_app(app, 10, 3);
    let expected = vec![
        String::from("hi        "),
        String::from("          "),
        String::from("          "),
    ];
    assert_eq!(term.lines(), expected);
}

#[test]
fn renders_dioxus_basic_rsxd() {
    fn app() -> Element {
        rsx! {
            div { direction: "column",
                h1 { "Termwiz demo" }
                p { "This is a simple termwiz layout without Dioxus." }
                ul {
                    li { "List item one" }
                    li { "List item two" }
                }
                p { "Press Ctrl+C to exit." }
            }
        }
    }

    let term = FakeTerminal::from_app(app, 40, 10);
    let expected = vec![
        format!("{}", "Termwiz demo".to_string() + &" ".repeat(28)),
        "This is a simple termwiz layout without ".to_string(),
        format!("{}", "• List item one".to_string() + &" ".repeat(25)),
        format!("{}", "• List item two".to_string() + &" ".repeat(25)),
        format!("{}", "Press Ctrl+C to exit.".to_string() + &" ".repeat(19)),
        " ".repeat(40),
        " ".repeat(40),
        " ".repeat(40),
        " ".repeat(40),
        " ".repeat(40),
    ];
    assert_eq!(term.lines(), expected);
}

#[test]
fn renders_quadrants_with_flex() {
    fn app() -> Element {
        rsx! {
            div { direction: "row",
                div { style: "width: 50%; height: 2px;", "A" }
                div { style: "width: 50%; height: 2px;", "B" }
            }
            div { direction: "row",
                div { style: "width: 50%; height: 2px;", "C" }
                div { style: "width: 50%; height: 2px;", "D" }
            }
        }
    }

    let term = FakeTerminal::from_app(app, 10, 4);
    let expected = vec![
        "A         ".to_string(),
        "C         ".to_string(),
        "D         ".to_string(),
        "          ".to_string(),
    ];
    assert_eq!(term.lines(), expected);
}

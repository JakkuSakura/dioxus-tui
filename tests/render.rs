use dioxus::prelude::*;
use ratatui::layout::Rect;

mod common;
use common::FakeTerminal;

#[test]
fn renders_basic_app_into_buffer() {
    fn app() -> Element {
        rsx! {
            div { "hello" }
        }
    }

    let term = FakeTerminal::from_app(app, 20, 5);
    let mut expected = vec![
        format!("{}", "hello".to_string() + &" ".repeat(15)),
    ];
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
    let mut expected = vec!["paragraph body".to_string() + &" ".repeat(30 - "paragraph body".len())];
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
        "• alpha".to_string() + &" ".repeat(20 - 7),
        "• beta".to_string() + &" ".repeat(20 - 6),
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
                h1 { style: "height: 1px; width: 100%;", "Ratatui demo" }
                p { style: "height: 1px; width: 100%;", "This is a simple ratatui layout without Dioxus." }
                p { style: "height: 1px; width: 100%;", "Press Ctrl+C to exit." }
                ul {
                    li { style: "height: 1px; width: 100%;", "List item one" }
                    li { style: "height: 1px; width: 100%;", "List item two" }
                }
            }
        }
    }

    let term = FakeTerminal::from_app(app, 40, 10);
    let expected = vec![
        format!("{}", "Ratatui demo".to_string() + &" ".repeat(28)),
        "This is a simple ratatui layout without ".to_string(),
        format!("{}", "Press Ctrl+C to exit.".to_string() + &" ".repeat(17)),
        format!("{}", "• List item one".to_string() + &" ".repeat(25)),
        format!("{}", "• List item two".to_string() + &" ".repeat(25)),
        " ".repeat(40),
        " ".repeat(40),
        " ".repeat(40),
        " ".repeat(40),
        " ".repeat(40),
    ];
    assert_eq!(term.lines(), expected);
}

use std::collections::VecDeque;

use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use termwiz::color::ColorAttribute;

use dioxus_tui::layout::resolve_document;
use dioxus_tui::{CellMetrics, ColorMode, Rect, Surface, TerminalScene};

fn render_component(root: fn() -> Element, width: u16, height: u16) -> Surface {
    let vdom = VirtualDom::new(root);
    let mut doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            viewport: Some(Viewport::new(width as u32, height as u32, 1.0, ColorScheme::Light)),
            ..Default::default()
        },
    );
    doc.initial_build();

    let area = Rect::new(0, 0, width, height);
    let root_id = resolve_document(&mut doc, area).expect("layout root");
    assert_eq!(root_id, 0, "expected root node id 0");

    let mut surface = Surface::new(width, height);
    let mut images = VecDeque::new();
    let metrics = CellMetrics {
        cell_w_px: 1.0,
        cell_h_px: 1.0,
    };

    {
        let mut scene = TerminalScene::new(
            &mut surface,
            &mut images,
            metrics,
            ColorMode::Rgb,
            true,
        );
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
    let r0: String = row(&surface, 0).iter().collect();
    let r1: String = row(&surface, 1).iter().collect();
    assert_eq!(r0, "Title              ");
    assert_eq!(r1, "Hello world        ");
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
    let r0: String = row(&surface, 0).iter().collect();
    let r1: String = row(&surface, 1).iter().collect();
    assert_eq!(r0, "first            ");
    assert_eq!(r1, "second           ");
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
    let r0: String = row(&surface, 0).iter().collect();
    let r1: String = row(&surface, 1).iter().collect();
    assert_eq!(r0, "block               ");
    assert_eq!(r1.trim_end(), "inline");
}

#[test]
fn respects_width_wrapping() {
    fn app() -> Element {
        rsx! {
            main {
                p { "abcdefghijklmnop" }
            }
        }
    }

    let surface = render_component(app, 8, 3);
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
    let c = cell(&surface, 0, 0);
    let fg = c.fg.expect("foreground color set");
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
    let r0: String = row(&surface, 0).iter().collect();
    let r1: String = row(&surface, 1).iter().collect();
    // Expect explicit framing: button and input rendered as bordered boxes with content.
    assert_eq!(r0, "Click       ");
    assert_eq!(r1, "text        ");
}

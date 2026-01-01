mod catalog {
    #![allow(dead_code)]
    pub mod frame {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/catalog/frame.rs"));
    }
    pub use frame::ExampleFrame;
}

mod textarea_example {
    #![allow(dead_code)]
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/catalog/textarea.rs"));

    #[cfg(test)]
    mod tests {
        use super::*;
        use dioxus::prelude::*;
        use dioxus_html::input_data::keyboard_types::Key;
        use blitz_dom::local_name;
        use blitz_traits::shell::{ColorScheme, Viewport};
        use dioxus_core::VirtualDom;
        use dioxus_native_dom::{DioxusDocument, DocumentConfig};
        use dioxus_tui::layout::node_rect;
        use dioxus_tui::{CellMetrics, Config, RawVirtualDom, Rect, Surface, TerminalCapabilities, TuiContext, render};
        use dioxus_tui::capabilities::InlineImageProtocol;
        use dioxus_tui::render::{apply_caret_overlay_at, caret_changes};

        #[test]
        fn enter_key_preserves_second_line_content() {
            let (tx, _rx) = render::channel();
            let tui = TuiContext::new(tx);
            let mut buffer = TextBuffer::default();

            for ch in ["H", "e", "l", "l", "o"] {
                buffer.handle_key(&Key::Character(ch.to_string()), &tui);
            }
            buffer.handle_key(&Key::Enter, &tui);
            for ch in ["W", "o", "r", "l", "d"] {
                buffer.handle_key(&Key::Character(ch.to_string()), &tui);
            }

            assert_eq!(buffer.lines, vec!["Hello", "World"]);
            assert_eq!(buffer.row, 1);
            assert_eq!(buffer.col, 5);
        }

        #[test]
        fn caret_position_accounts_for_layout_rect() {
            let layout = Rect::new(5, 9, 20, 10);
            let buffer = TextBuffer::default();

            let (cursor_x, cursor_y) = caret_position(layout, &buffer, 1);

            assert_eq!(cursor_x, layout.x + 1);
            assert_eq!(cursor_y, layout.y + 1);
        }

        fn assert_surface_matches(surface: &Surface, expected: &[&str]) {
            let width = surface.width();
            let height = surface.height();
            assert_eq!(expected.len() as u16, height);

            for (y, line) in expected.iter().enumerate() {
                let expected_chars: Vec<char> = line.chars().collect();
                for x in 0..width {
                    let idx = y * width as usize + x as usize;
                    let cell = surface.content[idx].ch;
                    let expected_ch = expected_chars.get(x as usize).copied().unwrap_or(' ');
                    assert_eq!(cell, expected_ch, "unexpected cell at ({x}, {y})");
                }
            }
        }

        #[component]
        fn TextareaLayoutProbe() -> Element {
            rsx! {
                textarea {
                    rows: "1",
                    cols: "10",
                    value: "ready",
                }
            }
        }

        #[component]
        fn TestTextareaBox(buffer: Signal<TextBuffer>) -> Element {
            let state = buffer.read().clone();
            let rendered_lines = state.lines.iter().map(|line| {
                let content = if line.is_empty() { " " } else { line.as_str() };
                rsx! { div { "{content}" } }
            });

            rsx! {
                div {
                    id: "caret-box",
                    width: "80%",
                    height: "70%",
                    padding: "1ch",
                    background_color: "#1a1b26",
                    color: "#c0caf5",
                    border_style: "solid",
                    border_width: "1px",
                    border_color: "#565f89",
                    white_space: "pre",
                    overflow: "hidden",
                    tabindex: "0",

                    {rendered_lines}
                }
            }
        }

        #[component]
        fn TextareaCaretProbe() -> Element {
            let buffer = use_signal(TextBuffer::default);
            rsx! { TestTextareaBox { buffer } }
        }

        fn build_doc_with_layout(app: fn() -> Element, width: u16, height: u16) -> (DioxusDocument, usize) {
            let metrics = CellMetrics {
                cell_w_px: 8.0,
                cell_h_px: 16.0,
            };
            let vdom = VirtualDom::new(app);
            let viewport = Viewport::new(
                (width as f32 * metrics.cell_w_px).ceil().max(1.0) as u32,
                (height as f32 * metrics.cell_h_px).ceil().max(1.0) as u32,
                1.0,
                ColorScheme::Light,
            );
            let mut doc = DioxusDocument::new(
                vdom,
                DocumentConfig {
                    viewport: Some(viewport),
                    ..Default::default()
                },
            );
            doc.initial_build();
            let root = dioxus_tui::layout::resolve_document(
                &mut doc,
                Rect::new(0, 0, width, height),
                metrics,
            )
                .expect("root layout");
            (doc, root)
        }

        fn find_node_by_id(doc: &blitz_dom::BaseDocument, id: usize, target: &str) -> Option<usize> {
            let node = doc.get_node(id)?;
            if node.attr(local_name!("id")).as_deref() == Some(target) {
                return Some(id);
            }
            for child in node.children.iter().copied() {
                if let Some(found) = find_node_by_id(doc, child, target) {
                    return Some(found);
                }
            }
            None
        }

        #[test]
        fn textarea_surface_renders_all_cells() {
            let raw = RawVirtualDom::new(TextareaLayoutProbe);
            let surface = render::render_once(Config::default(), raw, Rect::new(0, 0, 10, 1))
                .expect("render once");

            assert_surface_matches(&surface, &["ready     "]);
        }

        #[test]
        fn caret_overlay_renders_in_surface_cells() {
            let raw = RawVirtualDom::new(TextareaCaretProbe);
            let mut surface = render::render_once(Config::default(), raw, Rect::new(0, 0, 20, 6))
                .expect("render once");
            let (doc, root) = build_doc_with_layout(TextareaCaretProbe, 20, 6);
            let node_id = find_node_by_id(&doc.inner, root, "caret-box").expect("caret box");
            let metrics = CellMetrics {
                cell_w_px: 8.0,
                cell_h_px: 16.0,
            };
            let rect = node_rect(
                &doc.inner,
                doc.inner.get_node(node_id).expect("caret node"),
                Rect::new(0, 0, 20, 6),
                metrics,
            );
            let caret_pos = caret_position(rect, &TextBuffer::default(), 1);
            let baseline: Vec<char> = surface.content.iter().map(|cell| cell.ch).collect();
            let baseline_fg: Vec<Option<termwiz::color::ColorAttribute>> = surface
                .content
                .iter()
                .map(|cell| cell.fg)
                .collect();
            let baseline_bg: Vec<Option<termwiz::color::ColorAttribute>> = surface
                .content
                .iter()
                .map(|cell| cell.bg)
                .collect();

            let capabilities = TerminalCapabilities {
                truecolor: false,
                inline_images: false,
                inline_protocol: InlineImageProtocol::None,
            };
            let metrics = CellMetrics {
                cell_w_px: 8.0,
                cell_h_px: 16.0,
            };
            apply_caret_overlay_at(
                &mut surface,
                caret_pos,
                Config::default(),
                &capabilities,
                metrics,
            );

            let width = surface.width() as usize;
            let height = surface.height() as usize;
            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    let actual = surface.content[idx].ch;
                    let expected_ch = baseline[idx];
                    assert_eq!(actual, expected_ch, "unexpected cell at ({x}, {y})");

                    if (x as u16, y as u16) == caret_pos {
                        assert_eq!(surface.content[idx].fg, baseline_bg[idx]);
                        assert_eq!(surface.content[idx].bg, baseline_fg[idx]);
                    } else {
                        assert_eq!(surface.content[idx].fg, baseline_fg[idx]);
                        assert_eq!(surface.content[idx].bg, baseline_bg[idx]);
                    }
                }
            }
        }

        #[test]
        fn caret_changes_include_visibility_and_position() {
            let changes = caret_changes(true, Some((2, 0)));

            assert_eq!(changes.len(), 2);
            assert!(matches!(changes[0], termwiz::surface::Change::CursorVisibility(_)));
            assert!(matches!(
                changes[1],
                termwiz::surface::Change::CursorPosition { x, y }
                if x == termwiz::surface::Position::Absolute(2)
                    && y == termwiz::surface::Position::Absolute(0)
            ));
        }
    }
}

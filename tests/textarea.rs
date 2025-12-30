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
        use dioxus_tui::{Config, RawVirtualDom, Rect, Surface, TuiContext, render, use_layout_rect};

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
            let value = use_signal(|| String::from("pending"));
            let mut value_update = value.clone();
            let layout_rect = use_layout_rect();
            let _layout_subscription = layout_rect.read().clone();

            use_effect(move || {
                if layout_rect.read().is_some() {
                    value_update.set(String::from("ready"));
                }
            });

            rsx! {
                textarea {
                    rows: "1",
                    cols: "10",
                    value: value,
                }
            }
        }

        #[test]
        fn textarea_effect_runs_after_layout_rect_publish() {
            let raw = RawVirtualDom::new(TextareaLayoutProbe);
            let surface = render::render_once(Config::default(), raw, Rect::new(0, 0, 10, 1))
                .expect("render once");

            assert_surface_matches(&surface, &["ready     "]);
        }
    }
}

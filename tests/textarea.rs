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
        use dioxus_html::input_data::keyboard_types::Key;
        use dioxus_tui::render;

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
    }
}

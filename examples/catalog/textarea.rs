use dioxus::prelude::*;
use dioxus::prelude::HasKeyboardData;
use dioxus_html::input_data::keyboard_types::Key;
use dioxus_tui::{TuiContext, use_keyboard_input, use_caret, use_viewport, use_layout_rect};

use crate::catalog::ExampleFrame;

#[derive(Clone)]
struct TextBuffer {
    lines: Vec<String>,
    row: usize,
    col: usize,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self {
            lines: vec!["".to_string()],
            row: 0,
            col: 0,
        }
    }
}

impl TextBuffer {
    fn line_len(line: &str) -> usize {
        line.chars().count()
    }

    fn byte_index(line: &str, col: usize) -> usize {
        if col == 0 {
            return 0;
        }
        line.char_indices()
            .nth(col)
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| line.len())
    }

    fn clamp_cursor(&mut self) {
        if self.lines.is_empty() {
            self.lines.push("".to_string());
        }
        if self.row >= self.lines.len() {
            self.row = self.lines.len() - 1;
        }
        let len = Self::line_len(&self.lines[self.row]);
        if self.col > len {
            self.col = len;
        }
    }

    fn insert_str(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == '\n' || ch == '\r' {
                self.insert_newline();
            } else {
                let line = &mut self.lines[self.row];
                let idx = Self::byte_index(line, self.col);
                line.insert_str(idx, &ch.to_string());
                self.col += 1;
            }
        }
    }

    fn insert_newline(&mut self) {
        let line = &mut self.lines[self.row];
        let idx = Self::byte_index(line, self.col);
        let tail = line[idx..].to_string();
        line.truncate(idx);
        self.row += 1;
        self.col = 0;
        self.lines.insert(self.row, tail);
    }

    fn backspace(&mut self) {
        if self.col > 0 {
            let line = &mut self.lines[self.row];
            let start = Self::byte_index(line, self.col - 1);
            let end = Self::byte_index(line, self.col);
            line.replace_range(start..end, "");
            self.col -= 1;
        } else if self.row > 0 {
            let current = self.lines.remove(self.row);
            self.row -= 1;
            let prev_len = Self::line_len(&self.lines[self.row]);
            self.lines[self.row].push_str(&current);
            self.col = prev_len;
        }
    }

    fn delete(&mut self) {
        let line_len = Self::line_len(&self.lines[self.row]);
        if self.col < line_len {
            let line = &mut self.lines[self.row];
            let start = Self::byte_index(line, self.col);
            let end = Self::byte_index(line, self.col + 1);
            line.replace_range(start..end, "");
        } else if self.row + 1 < self.lines.len() {
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
        }
    }

    fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.col = Self::line_len(&self.lines[self.row]);
        }
    }

    fn move_right(&mut self) {
        let line_len = Self::line_len(&self.lines[self.row]);
        if self.col < line_len {
            self.col += 1;
        } else if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            let len = Self::line_len(&self.lines[self.row]);
            self.col = self.col.min(len);
        }
    }

    fn move_down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.row += 1;
            let len = Self::line_len(&self.lines[self.row]);
            self.col = self.col.min(len);
        }
    }

    fn handle_key(&mut self, key: &Key, tui: &TuiContext) {
        match key {
            Key::Character(text) => {
                if text == "\t" {
                    self.insert_str("    ");
                } else if text == "\n" || text == "\r" {
                    self.insert_newline();
                } else {
                    self.insert_str(text);
                }
            }
            Key::Enter => self.insert_newline(),
            Key::Backspace => self.backspace(),
            Key::Delete => self.delete(),
            Key::ArrowLeft => self.move_left(),
            Key::ArrowRight => self.move_right(),
            Key::ArrowUp => self.move_up(),
            Key::ArrowDown => self.move_down(),
            Key::Home => self.col = 0,
            Key::End => self.col = Self::line_len(&self.lines[self.row]),
            Key::Escape => tui.quit(),
            _ => {}
        }
        self.clamp_cursor();
    }

}

fn caret_position(layout: dioxus_tui::Rect, state: &TextBuffer, padding: u16) -> (u16, u16) {
    let max_x = layout.x.saturating_add(layout.width.saturating_sub(1));
    let max_y = layout.y.saturating_add(layout.height.saturating_sub(1));
    let cursor_x = layout
        .x
        .saturating_add(padding)
        .saturating_add(state.col as u16)
        .min(max_x);
    let cursor_y = layout
        .y
        .saturating_add(padding)
        .saturating_add(state.row as u16)
        .min(max_y);
    (cursor_x, cursor_y)
}

pub fn app() -> Element {
    let tui: TuiContext = consume_context();
    let key_input = use_keyboard_input();
    let cursor_handle = use_caret();
    let cursor_handle_update = cursor_handle.clone();
    let viewport = use_viewport();
    let (layout_handle, layout_rect) = use_layout_rect();
    let mut buffer = use_signal(TextBuffer::default);

    use_effect(move || {
        let Some(data) = key_input.read().clone() else {
            return;
        };
        buffer.with_mut(|buf| buf.handle_key(&data.key(), &tui));
    });

    use_effect(move || {
        let state = buffer.read().clone();
        let _ = viewport.read().clone();
        let Some(layout) = layout_rect.read().clone() else {
            return;
        };
        let padding = 1u16;
        let (cursor_x, cursor_y) = caret_position(layout, &state, padding);
        cursor_handle_update.show();
        cursor_handle_update.set_cell_position(cursor_x, cursor_y);
    });

    let state = buffer.read().clone();
    let rendered_lines = state.lines.iter().map(|line| {
        let content = if line.is_empty() { " " } else { line.as_str() };
        rsx! { div { "{content}" } }
    });

    rsx! {
        ExampleFrame {
            title: "Textarea",
            help: &[
                "Type to insert text. Enter makes a new line.",
                "Use arrow keys, Backspace, Delete. Esc to quit.",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                align_items: "center",
                justify_content: "center",

                div {
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
                    "data-layout-id": "{layout_handle.id()}",

                    {rendered_lines}
                }
            }
        }
    }
}

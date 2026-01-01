use dioxus::prelude::*;
use dioxus::prelude::HasKeyboardData;
use dioxus_html::input_data::keyboard_types::Key;
use dioxus_tui::{CaretMode, TuiContext, use_keyboard_input, use_caret, use_layout_rect};

use crate::catalog::ExampleFrame;

#[derive(Clone)]
struct TextBuffer {
    lines: Vec<String>,
    row: usize,
    col: usize,
}

#[derive(Clone)]
struct CaretDebugInfo {
    layout: dioxus_tui::Rect,
    has_layout: bool,
    caret: (u16, u16),
    has_caret: bool,
    row: usize,
    col: usize,
    mode: CaretMode,
}

impl Default for CaretDebugInfo {
    fn default() -> Self {
        Self {
            layout: dioxus_tui::Rect::new(0, 0, 0, 0),
            has_layout: false,
            caret: (0, 0),
            has_caret: false,
            row: 0,
            col: 0,
            mode: CaretMode::Physical,
        }
    }
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

#[component]
fn TextareaBox(buffer: Signal<TextBuffer>, debug_info: Signal<CaretDebugInfo>) -> Element {
    let cursor_handle = use_caret();
    let cursor_handle_update = cursor_handle.clone();
    let layout_rect = use_layout_rect();
    let _layout_subscription = layout_rect.read().clone();
    let caret_mode = use_context::<Signal<CaretMode>>();
    let mut debug_update = debug_info.clone();

    use_effect(move || {
        let state = buffer.read().clone();
        let layout = layout_rect.read().clone();
        let padding = 1u16;
        cursor_handle_update.set_mode(*caret_mode.read());
        let (caret, has_caret) = if let Some(layout) = layout {
            let caret = caret_position(layout, &state, padding);
            cursor_handle_update.show();
            cursor_handle_update.set_cell_position(
                padding.saturating_add(state.col as u16),
                padding.saturating_add(state.row as u16),
            );
            (caret, true)
        } else {
            ((0, 0), false)
        };
        let (layout, has_layout) = layout.map_or((dioxus_tui::Rect::new(0, 0, 0, 0), false), |rect| {
            (rect, true)
        });
        debug_update.set(CaretDebugInfo {
            layout,
            has_layout,
            caret,
            has_caret,
            row: state.row,
            col: state.col,
            mode: *caret_mode.read(),
        });
    });

    let state = buffer.read().clone();
    let rendered_lines = state.lines.iter().map(|line| {
        let content = if line.is_empty() { " " } else { line.as_str() };
        rsx! { div { "{content}" } }
    });

    rsx! {
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

            {rendered_lines}
        }
    }
}

pub fn app() -> Element {
    let tui: TuiContext = consume_context();
    let key_input = use_keyboard_input();
    let mut buffer = use_signal(TextBuffer::default);
    let debug_info = use_signal(CaretDebugInfo::default);
    let caret_mode = use_signal(|| CaretMode::Physical);
    let mut caret_mode_update = caret_mode.clone();
    provide_context(caret_mode);

    use_effect(move || {
        let Some(data) = key_input.read().clone() else {
            return;
        };
        if data.key() == Key::F2 {
            caret_mode_update.with_mut(|mode| {
                *mode = match *mode {
                    CaretMode::Physical => CaretMode::Soft,
                    CaretMode::Soft => CaretMode::Physical,
                };
            });
            return;
        }
        buffer.with_mut(|buf| buf.handle_key(&data.key(), &tui));
    });

    let debug = debug_info.read().clone();
    let caret_mode_label = match debug.mode {
        CaretMode::Physical => "physical",
        CaretMode::Soft => "soft",
    };

    rsx! {
        ExampleFrame {
            title: "Textarea",
            help: &[
                "Type to insert text. Enter makes a new line.",
                "Use arrow keys, Backspace, Delete. Esc to quit.",
                "F2 toggles caret mode (soft/physical).",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                align_items: "center",
                justify_content: "center",

                TextareaBox { buffer, debug_info }

                div {
                    margin_top: "1ch",
                    width: "80%",
                    color: "#a9b1d6",
                    "Caret row/col: {debug.row}, {debug.col}"
                }
                div {
                    width: "80%",
                    color: "#a9b1d6",
                    "Caret mode: {caret_mode_label}"
                }
                if debug.has_layout {
                    div {
                        width: "80%",
                        color: "#a9b1d6",
                        "Layout rect: {debug.layout.x},{debug.layout.y} {debug.layout.width}x{debug.layout.height}"
                    }
                }
                if debug.has_caret {
                    div {
                        width: "80%",
                        color: "#a9b1d6",
                        "Caret cell: {debug.caret.0},{debug.caret.1}"
                    }
                }
            }
        }
    }
}

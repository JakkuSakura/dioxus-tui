use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Key;
use dioxus_tui::{CaretMode, use_caret};

#[derive(Clone, PartialEq)]
pub enum TextareaAction {
    None,
    Quit,
}

#[derive(Clone)]
pub struct TextBuffer {
    pub lines: Vec<String>,
    pub row: usize,
    pub col: usize,
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

    pub fn handle_key(&mut self, key: &Key) -> TextareaAction {
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
            Key::Escape => return TextareaAction::Quit,
            _ => {}
        }
        self.clamp_cursor();
        TextareaAction::None
    }
}

#[derive(Clone)]
pub struct TextareaViewModel {
    buffer: Signal<TextBuffer>,
}

impl TextareaViewModel {
    pub fn buffer(&self) -> Signal<TextBuffer> {
        self.buffer
    }

    pub fn handle_key(&self, key: &Key) -> TextareaAction {
        let mut buffer = self.buffer;
        let mut action = TextareaAction::None;
        buffer.with_mut(|buf| {
            action = buf.handle_key(key);
        });
        action
    }
}

pub fn use_textarea_view_model() -> TextareaViewModel {
    TextareaViewModel {
        buffer: use_signal(TextBuffer::default),
    }
}

#[derive(Clone, Props, PartialEq)]
pub struct TextareaViewProps {
    pub buffer: Signal<TextBuffer>,
    pub caret_mode: CaretMode,
    #[props(default = 1)]
    pub padding: u16,
}

#[component]
pub fn TextareaView(props: TextareaViewProps) -> Element {
    let TextareaViewProps {
        buffer,
        caret_mode,
        padding,
    } = props;
    let caret_handle = use_caret();
    let caret_handle_update = caret_handle.clone();
    use_effect(move || {
        let state = buffer.read().clone();
        caret_handle_update.set_mode(caret_mode);
        caret_handle_update.show();
        caret_handle_update.set_cell_position(
            padding.saturating_add(state.col as u16),
            padding.saturating_add(state.row as u16),
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use dioxus::prelude::VirtualDom;
    use dioxus_tui::{CaretBus, CaretCommand, CaretMode, LayoutBus};

    #[component]
    fn CaretProbe() -> Element {
        let model = use_textarea_view_model();
        rsx! {
            TextareaView {
                buffer: model.buffer(),
                caret_mode: CaretMode::Physical,
                padding: 1,
            }
        }
    }

    #[test]
    fn textarea_view_emits_caret_position() {
        let caret_bus = CaretBus::new();
        let layout_bus = LayoutBus::new();
        let events = Rc::new(RefCell::new(Vec::new()));
        let events_handle = Rc::clone(&events);
        let _subscription = caret_bus.subscribe(Rc::new(move |cmd| {
            events_handle.borrow_mut().push(cmd);
        }));

        let mut vdom = VirtualDom::new(CaretProbe)
            .with_root_context(caret_bus)
            .with_root_context(layout_bus);
        let _ = vdom.rebuild();
        let _ = vdom.work_with_deadline(|| false);

        let events = events.borrow();
        assert!(events.iter().any(|cmd| matches!(cmd, CaretCommand::Show)));
        assert!(events.iter().any(|cmd| matches!(cmd, CaretCommand::SetPosition(_, _))));
    }
}

//! Helper builders for producing TUI-native layout from Dioxus nodes.
//!
//! Note: these builders render content as many positioned nodes, which is slow.

use dioxus::prelude::*;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub dim: bool,
    pub underline: bool,
    pub italic: bool,
    pub blink: bool,
}

impl Style {
    pub fn to_css(&self) -> String {
        let mut parts = Vec::new();
        if let Some(fg) = &self.fg {
            parts.push(format!("color: {fg};"));
        }
        if let Some(bg) = &self.bg {
            parts.push(format!("background-color: {bg};"));
        }
        if self.bold {
            parts.push("font-weight: bold;".to_string());
        }
        if self.dim {
            parts.push("opacity: 0.7;".to_string());
        }
        if self.underline {
            parts.push("text-decoration: underline;".to_string());
        }
        if self.italic {
            parts.push("font-style: italic;".to_string());
        }
        if self.blink {
            parts.push("text-decoration: blink;".to_string());
        }
        parts.join(" ")
    }
}

#[derive(Clone)]
struct Span {
    x: usize,
    text: String,
    style: Style,
}

#[derive(Clone)]
pub struct PositionedSpan {
    pub x: usize,
    pub y: usize,
    pub text: String,
    pub style: Style,
}

#[derive(Clone, Default)]
pub struct LineBuilder {
    spans: Vec<Span>,
}

impl LineBuilder {
    pub fn new() -> Self {
        Self { spans: Vec::new() }
    }

    pub fn set_str(&mut self, x: usize, text: &str) {
        self.set_str_styled(x, text, Style::default());
    }

    pub fn set_str_styled(&mut self, x: usize, text: &str, style: Style) {
        self.spans.push(Span {
            x,
            text: text.to_string(),
            style,
        });
    }

    pub fn set_char(&mut self, x: usize, ch: char) {
        self.set_str(x, &ch.to_string());
    }

    pub fn set_repeat(&mut self, x: usize, ch: char, count: usize) {
        let text: String = std::iter::repeat(ch).take(count).collect();
        self.set_str(x, &text);
    }

    pub fn to_string(&self, width: usize) -> String {
        let mut chars = vec![' '; width];
        for span in &self.spans {
            for (idx, ch) in span.text.chars().enumerate() {
                let pos = span.x + idx;
                if pos < chars.len() {
                    chars[pos] = ch;
                }
            }
        }
        chars.into_iter().collect()
    }

    fn spans(&self) -> &[Span] {
        &self.spans
    }
}

pub struct RectBuilder {
    width: usize,
    height: usize,
    lines: Vec<LineBuilder>,
}

impl RectBuilder {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            lines: vec![LineBuilder::new(); height],
        }
    }

    pub fn line_mut(&mut self, y: usize) -> Option<&mut LineBuilder> {
        self.lines.get_mut(y)
    }

    pub fn set_str(&mut self, x: usize, y: usize, text: &str) {
        if let Some(line) = self.lines.get_mut(y) {
            line.set_str(x, text);
        }
    }

    pub fn set_char(&mut self, x: usize, y: usize, ch: char) {
        if let Some(line) = self.lines.get_mut(y) {
            line.set_char(x, ch);
        }
    }

    pub fn set_repeat(&mut self, x: usize, y: usize, ch: char, count: usize) {
        if let Some(line) = self.lines.get_mut(y) {
            line.set_repeat(x, ch, count);
        }
    }

    pub fn to_lines(&self) -> Vec<String> {
        self.lines
            .iter()
            .map(|line| line.to_string(self.width))
            .collect()
    }

    pub fn positioned_spans(&self, x_offset: usize, y_offset: usize) -> Vec<PositionedSpan> {
        let mut out = Vec::new();
        for (row_idx, line) in self.lines.iter().enumerate() {
            for span in line.spans() {
                out.push(PositionedSpan {
                    x: x_offset + span.x,
                    y: y_offset + row_idx,
                    text: span.text.clone(),
                    style: span.style.clone(),
                });
            }
        }
        out
    }

    pub fn render(&self, x: usize, y: usize) -> Element {
        let mut spans = Vec::new();
        for (row_idx, line) in self.lines.iter().enumerate() {
            for (span_idx, span) in line.spans().iter().enumerate() {
                let width = span.text.chars().count();
                let style = format!(
                    "position: absolute; left: {}ch; top: {}ch; width: {}ch; height: 1ch; white-space: pre; font-family: monospace; {}",
                    x + span.x,
                    y + row_idx,
                    width,
                    span.style.to_css()
                );
                spans.push(rsx! {
                    div {
                        key: "span-{row_idx}-{span_idx}",
                        style: "{style}",
                        "{span.text}"
                    }
                });
            }
        }

        let spans = spans.into_iter();

        rsx! {
            div {
                position: "absolute",
                left: "0ch",
                top: "0ch",
                width: "{self.width}ch",
                height: "{self.height}ch",
                {spans}
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct BorderStyle {
    pub tl: char,
    pub tr: char,
    pub bl: char,
    pub br: char,
    pub h: char,
    pub v: char,
}

impl BorderStyle {
    pub fn single() -> Self {
        Self {
            tl: '┌',
            tr: '┐',
            bl: '└',
            br: '┘',
            h: '─',
            v: '│',
        }
    }
}

pub struct BorderedRectBuilder {
    inner: RectBuilder,
    border: BorderStyle,
    title: Option<(String, Style)>,
}

impl BorderedRectBuilder {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: RectBuilder::new(width, height),
            border: BorderStyle::single(),
            title: None,
        }
    }

    pub fn inner_mut(&mut self) -> &mut RectBuilder {
        &mut self.inner
    }

    pub fn border_style(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    pub fn title(mut self, text: &str, style: Style) -> Self {
        self.title = Some((text.to_string(), style));
        self
    }

    pub fn build(&self) -> RectBuilder {
        let width = self.inner.width + 2;
        let height = self.inner.height + 2;
        let mut rect = RectBuilder::new(width, height);

        rect.set_char(0, 0, self.border.tl);
        rect.set_char(width - 1, 0, self.border.tr);
        rect.set_char(0, height - 1, self.border.bl);
        rect.set_char(width - 1, height - 1, self.border.br);
        rect.set_repeat(1, 0, self.border.h, width - 2);
        rect.set_repeat(1, height - 1, self.border.h, width - 2);

        for y in 1..height - 1 {
            rect.set_char(0, y, self.border.v);
            rect.set_char(width - 1, y, self.border.v);
        }

        if let Some((title, style)) = &self.title {
            if let Some(line) = rect.line_mut(0) {
                line.set_str_styled(2, title, style.clone());
            }
        }

        for (y, line) in self.inner.lines.iter().enumerate() {
            if let Some(dest) = rect.line_mut(y + 1) {
                for span in line.spans() {
                    dest.set_str_styled(span.x + 1, &span.text, span.style.clone());
                }
            }
        }

        rect
    }
}

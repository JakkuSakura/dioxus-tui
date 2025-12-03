use crate::geometry::Rect;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone)]
pub struct Surface {
    width: u16,
    height: u16,
    pub content: Vec<char>,
}

impl Surface {
    pub fn new(width: u16, height: u16) -> Self {
        let len = width.saturating_mul(height) as usize;
        Self {
            width,
            height,
            content: vec![' '; len],
        }
    }

    pub fn clear(&mut self) {
        self.content.fill(' ');
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn area(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    pub fn set_stringn(&mut self, x: u16, y: u16, text: impl AsRef<str>, width: usize) {
        if y >= self.height {
            return;
        }
        let max_cols = self.width as usize;
        let start = y as usize * max_cols;
        let mut col = x as usize;
        for ch in text.as_ref().chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1).max(1);
            if col >= max_cols || col + ch_width > x as usize + width {
                break;
            }

            if let Some(slot) = self.content.get_mut(start + col) {
                *slot = ch;
            }

            // Fill the remainder of a wide char with spaces to avoid stale glyphs
            if ch_width > 1 {
                for extra in 1..ch_width {
                    if let Some(slot) = self.content.get_mut(start + col + extra) {
                        *slot = ' ';
                    }
                }
            }

            col += ch_width;
        }
    }

    pub fn lines(&self) -> Vec<String> {
        let width = self.width as usize;
        self.content
            .chunks(width)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect()
    }

    pub fn dims(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}

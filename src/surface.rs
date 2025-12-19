use crate::geometry::Rect;
use termwiz::color::ColorAttribute;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<ColorAttribute>,
    pub bg: Option<ColorAttribute>,
}

impl Cell {
    pub fn is_blank(&self) -> bool {
        (self.ch == ' ' || self.ch == '\0') && self.fg.is_none() && self.bg.is_none()
    }

    pub fn has_glyph(&self) -> bool {
        self.ch != ' ' && self.ch != '\0'
    }

    pub fn has_visible_content(&self) -> bool {
        !self.is_blank()
    }
}

#[derive(Debug, Clone)]
pub struct Surface {
    width: u16,
    height: u16,
    pub content: Vec<Cell>,
}

impl Surface {
    pub fn new(width: u16, height: u16) -> Self {
        let len = width.saturating_mul(height) as usize;
        Self {
            width,
            height,
            content: vec![
                Cell {
                    ch: ' ',
                    fg: None,
                    bg: None,
                };
                len
            ],
        }
    }

    pub fn clear(&mut self) {
        for cell in self.content.iter_mut() {
            cell.ch = ' ';
            cell.fg = None;
            cell.bg = None;
        }
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
        self.set_stringn_colored(x, y, text, width, None, None);
    }

    pub fn set_stringn_colored(
        &mut self,
        x: u16,
        y: u16,
        text: impl AsRef<str>,
        width: usize,
        fg: Option<ColorAttribute>,
        bg: Option<ColorAttribute>,
    ) {
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
                slot.ch = ch;
                if let Some(fg) = fg {
                    slot.fg = Some(fg);
                }
                if let Some(bg) = bg {
                    slot.bg = Some(bg);
                }
            }

            if ch_width > 1 {
                for extra in 1..ch_width {
                    if let Some(slot) = self.content.get_mut(start + col + extra) {
                        slot.ch = ' ';
                        if let Some(fg) = fg {
                            slot.fg = Some(fg);
                        }
                        if let Some(bg) = bg {
                            slot.bg = Some(bg);
                        }
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
            .map(|chunk| chunk.iter().map(|c| c.ch).collect::<String>())
            .collect()
    }

    pub fn dims(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}

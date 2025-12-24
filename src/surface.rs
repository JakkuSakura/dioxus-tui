use crate::geometry::Rect;
use termwiz::cell::{Blink, Intensity, Underline};
use termwiz::color::ColorAttribute;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<ColorAttribute>,
    pub bg: Option<ColorAttribute>,
    pub intensity: Intensity,
    pub underline: Underline,
    pub italic: bool,
    pub blink: Blink,
}

impl Cell {
    pub fn is_blank(&self) -> bool {
        (self.ch == ' ' || self.ch == '\0')
            && self.fg.is_none()
            && self.bg.is_none()
            && self.intensity == Intensity::Normal
            && self.underline == Underline::None
            && !self.italic
            && self.blink == Blink::None
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
        // Use a usize multiplication so large render targets (e.g. render-mode with a tall
        // virtual height for scrollback) allocate a consistent backing buffer.
        let len = (width as usize) * (height as usize);
        Self {
            width,
            height,
            content: vec![
                Cell {
                    ch: ' ',
                    fg: None,
                    bg: None,
                    intensity: Intensity::Normal,
                    underline: Underline::None,
                    italic: false,
                    blink: Blink::None,
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
            cell.intensity = Intensity::Normal;
            cell.underline = Underline::None;
            cell.italic = false;
            cell.blink = Blink::None;
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
        self.set_stringn_styled(
            x,
            y,
            text,
            width,
            fg,
            bg,
            Intensity::Normal,
            Underline::None,
            false,
            Blink::None,
        );
    }

    pub fn set_stringn_styled(
        &mut self,
        x: u16,
        y: u16,
        text: impl AsRef<str>,
        width: usize,
        fg: Option<ColorAttribute>,
        bg: Option<ColorAttribute>,
        intensity: Intensity,
        underline: Underline,
        italic: bool,
        blink: Blink,
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
                slot.intensity = intensity;
                slot.underline = underline;
                slot.italic = italic;
                slot.blink = blink;
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
                        slot.intensity = intensity;
                        slot.underline = underline;
                        slot.italic = italic;
                        slot.blink = blink;
                    }
                }
            }

            col += ch_width;
        }
    }

    pub fn set_glyph_styled(
        &mut self,
        x: u16,
        y: u16,
        ch: char,
        fg: Option<ColorAttribute>,
        bg: Option<ColorAttribute>,
        intensity: Intensity,
        underline: Underline,
        italic: bool,
        blink: Blink,
    ) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y as usize * self.width as usize + x as usize;
        if let Some(slot) = self.content.get_mut(idx) {
            slot.ch = ch;
            if let Some(fg) = fg {
                slot.fg = Some(fg);
            }
            if let Some(bg) = bg {
                slot.bg = Some(bg);
            }
            slot.intensity = intensity;
            slot.underline = underline;
            slot.italic = italic;
            slot.blink = blink;
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

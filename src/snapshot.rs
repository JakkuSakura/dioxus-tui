#![allow(dead_code)]

use crate::surface::Surface;
use termwiz::color::ColorAttribute;

/// Canonical snapshot format for tests/examples: cell grid plus attributes.
pub struct Snapshot {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<CellSnapshot>,
}

pub struct CellSnapshot {
    pub ch: char,
    pub fg: Option<ColorAttribute>,
    pub bg: Option<ColorAttribute>,
}

impl Snapshot {
    pub fn from_surface(surface: &Surface) -> Self {
        let width = surface.width();
        let height = surface.height();
        let cells = surface
            .content
            .iter()
            .map(|c| CellSnapshot {
                ch: c.ch,
                fg: c.fg,
                bg: c.bg,
            })
            .collect();
        Self {
            width,
            height,
            cells,
        }
    }
}

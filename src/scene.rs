use std::collections::VecDeque;

use anyrender::{types::PaintRef, PaintScene};
use kurbo::{Affine, Point, Rect as KRect, Shape, Stroke};
use peniko::{color::Rgba8, BlendMode, Color, Fill, FontData, ImageBrushRef, StyleRef};
use unicode_width::UnicodeWidthChar;
use ttf_parser::{Face, GlyphId};

use crate::config::ColorMode;
use crate::geometry::Rect;
use crate::surface::Surface;
use termwiz::color::{ColorAttribute, SrgbaTuple};

#[derive(Debug, Clone, Copy)]
pub struct CellMetrics {
    pub cell_w_px: f32,
    pub cell_h_px: f32,
}

#[derive(Debug, Clone)]
pub struct InlineImage {
    pub data: Vec<u8>, // RGBA
    pub width_px: u32,
    pub height_px: u32,
    pub x_px: f32,
    pub y_px: f32,
}

pub struct TerminalScene<'a> {
    surface: &'a mut Surface,
    images: &'a mut VecDeque<InlineImage>,
    metrics: CellMetrics,
    clip_stack: Vec<Rect>,
    color_mode: ColorMode,
    truecolor: bool,
}

impl<'a> TerminalScene<'a> {
    pub fn new(
        surface: &'a mut Surface,
        images: &'a mut VecDeque<InlineImage>,
        metrics: CellMetrics,
        color_mode: ColorMode,
        truecolor: bool,
    ) -> Self {
        Self {
            surface,
            images,
            metrics,
            clip_stack: Vec::new(),
            color_mode,
            truecolor,
        }
    }

    fn in_clip(&self, x: u16, y: u16) -> bool {
        if self.clip_stack.is_empty() {
            return true;
        }
        let Rect {
            x: cx,
            y: cy,
            width,
            height,
        } = *self.clip_stack.last().unwrap();
        if width == 0 || height == 0 {
            return true;
        }
        let within_x = x >= cx && x < cx.saturating_add(width);
        let within_y = y >= cy && y < cy.saturating_add(height);
        within_x && within_y
    }

    fn paint_cell(
        &mut self,
        ch: char,
        x: u16,
        y: u16,
        fg: Option<ColorAttribute>,
        bg: Option<ColorAttribute>,
    ) {
        if !self.in_clip(x, y) {
            return;
        }
        let width = self.surface.width() as usize;
        let start = y as usize * width + x as usize;
        if let Some(slot) = self.surface.content.get_mut(start) {
            slot.ch = ch;
            slot.fg = fg;
            slot.bg = bg;
        }
    }

    fn paint_rect(
        &mut self,
        ch: char,
        x_px: f32,
        y_px: f32,
        w_px: f32,
        h_px: f32,
        fg: Option<ColorAttribute>,
        bg: Option<ColorAttribute>,
    ) {
        let x0 = (x_px / self.metrics.cell_w_px).floor().max(0.0) as u16;
        let y0 = (y_px / self.metrics.cell_h_px).floor().max(0.0) as u16;
        let x1 = ((x_px + w_px) / self.metrics.cell_w_px).ceil().max(0.0) as u16;
        let y1 = ((y_px + h_px) / self.metrics.cell_h_px).ceil().max(0.0) as u16;
        for y in y0..y1.min(self.surface.height()) {
            for x in x0..x1.min(self.surface.width()) {
                self.paint_cell(ch, x, y, fg, bg);
            }
        }
    }

    fn push_text(&mut self, text: &str, x_px: f32, y_px: f32, fg: Option<ColorAttribute>) {
        let cell_x = (x_px / self.metrics.cell_w_px).floor().max(0.0) as u16;
        let cell_y = (y_px / self.metrics.cell_h_px).floor().max(0.0) as u16;
        let width = self.surface.width() as usize;
        let mut col = cell_x as usize;
        let max_cols = self.surface.width() as usize;
        let max_rows = self.surface.height() as usize;
        if cell_y as usize >= max_rows {
            return;
        }
        for ch in text.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1).max(1);
            if col >= max_cols {
                break;
            }
            if !self.in_clip(col as u16, cell_y) {
                col = col.saturating_add(ch_width);
                continue;
            }
            let start = cell_y as usize * width + col;
            if let Some(slot) = self.surface.content.get_mut(start) {
                slot.ch = ch;
                slot.fg = fg;
            }
            if ch_width > 1 {
                for extra in 1..ch_width {
                    if let Some(slot) = self.surface.content.get_mut(start + extra) {
                        slot.ch = ' ';
                        slot.fg = fg;
                    }
                }
            }
            col = col.saturating_add(ch_width);
        }
    }

    fn push_image(&mut self, image: ImageBrushRef, x_px: f32, y_px: f32) {
        let data = image.image.data.as_ref().to_vec();
        self.images.push_back(InlineImage {
            data,
            width_px: image.image.width,
            height_px: image.image.height,
            x_px,
            y_px,
        });
    }

    fn to_color_attr(&self, color: Color) -> Option<ColorAttribute> {
        let Rgba8 { r, g, b, a } = color.to_rgba8();
        let srgb = SrgbaTuple::from((r, g, b));
        if a == 0 {
            return None;
        }

        let palette_idx_256 =
            16 + 36 * (r as u16 / 51) as u8 + 6 * (g as u16 / 51) as u8 + (b as u16 / 51) as u8;

        let base_idx = (if r >= 128 { 1 } else { 0 })
            | (if g >= 128 { 2 } else { 0 })
            | (if b >= 128 { 4 } else { 0 });

        Some(match self.color_mode {
            ColorMode::BaseColors => ColorAttribute::PaletteIndex(base_idx),
            ColorMode::Ansi => ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256),
            ColorMode::Rgb => {
                if self.truecolor {
                    ColorAttribute::TrueColorWithDefaultFallback(srgb)
                } else {
                    ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256)
                }
            }
        })
    }
}

fn glyph_id_to_char(font: &FontData, glyph_id: u32) -> Option<char> {
    let face = Face::parse(font.data.as_ref(), 0).ok()?;
    let target = GlyphId(glyph_id as u16);

    // Try ASCII fast path
    for code in 0u32..=0x7f {
        if let Some(gid) = face.glyph_index(char::from_u32(code)?) {
            if gid == target {
                return char::from_u32(code);
            }
        }
    }

    // Broader BMP search (limited to keep it tractable for tests).
    for code in 0x80u32..=0xffff {
        if let Some(ch) = char::from_u32(code) {
            if let Some(gid) = face.glyph_index(ch) {
                if gid == target {
                    return Some(ch);
                }
            }
        }
    }

    None
}

fn glyph_id_to_chars(font: &FontData, glyph_id: u32) -> Vec<char> {
    if let Some(c) = glyph_id_to_char(font, glyph_id) {
        match c as u32 {
            0xfb00 => vec!['f', 'f'],
            0xfb01 => vec!['f', 'i'],
            0xfb02 => vec!['f', 'l'],
            0xfb03 => vec!['f', 'f', 'i'],
            0xfb04 => vec!['f', 'f', 'l'],
            _ => vec![c],
        }
    } else {
        vec!['█']
    }
}

impl<'a> PaintScene for TerminalScene<'a> {
    fn reset(&mut self) {
        self.surface.clear();
        self.images.clear();
        self.clip_stack.clear();
    }

    fn push_layer(
        &mut self,
        _blend: impl Into<BlendMode>,
        _alpha: f32,
        _transform: Affine,
        clip: &impl Shape,
    ) {
        let bbox = clip.bounding_box();
        let x = bbox.x0.max(0.0) as u16;
        let y = bbox.y0.max(0.0) as u16;
        let w = bbox.width().ceil().max(0.0) as u16;
        let h = bbox.height().ceil().max(0.0) as u16;
        self.clip_stack.push(Rect::new(x, y, w, h));
    }

    fn pop_layer(&mut self) {
        let _ = self.clip_stack.pop();
    }

    fn stroke<'b>(
        &mut self,
        _style: &Stroke,
        _transform: Affine,
        _brush: impl Into<PaintRef<'b>>,
        _brush_transform: Option<Affine>,
        _shape: &impl Shape,
    ) {
        let bbox = _shape.bounding_box();
        let x0 = bbox.x0 as f32;
        let y0 = bbox.y0 as f32;
        let w = bbox.width() as f32;
        let h = bbox.height() as f32;
        self.paint_rect('▓', x0, y0, w, self.metrics.cell_h_px, None, None);
        self.paint_rect('▓', x0, y0 + h, w, self.metrics.cell_h_px, None, None);
        self.paint_rect('▓', x0, y0, self.metrics.cell_w_px, h, None, None);
        self.paint_rect('▓', x0 + w, y0, self.metrics.cell_w_px, h, None, None);
    }

    fn fill<'b>(
        &mut self,
        _style: Fill,
        transform: Affine,
        brush: impl Into<PaintRef<'b>>,
        _brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let bbox = shape.bounding_box();
        let (x_px, y_px) = {
            let p = transform * Point::new(bbox.x0, bbox.y0);
            (p.x as f32, p.y as f32)
        };
        match brush.into() {
            PaintRef::Solid(c) => {
                let bg = self.to_color_attr(c);
                let w_px = bbox.width() as f32;
                let h_px = bbox.height() as f32;
                if w_px > 0.0 && h_px > 0.0 {
                    self.paint_rect(' ', x_px, y_px, w_px, h_px, None, bg);
                }
            }
            PaintRef::Image(img) => self.push_image(img, x_px, y_px),
            _ => {}
        }
    }

    fn draw_glyphs<'b, 's: 'b>(
        &'s mut self,
        _font: &'b FontData,
        font_size: f32,
        _hint: bool,
        _normalized_coords: &'b [anyrender::types::NormalizedCoord],
        _style: impl Into<StyleRef<'b>>,
        brush: impl Into<PaintRef<'b>>,
        _brush_alpha: f32,
        transform: Affine,
        _glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = anyrender::types::Glyph>,
    ) {
        let fg = match brush.into() {
            PaintRef::Solid(c) => self.to_color_attr(c),
            _ => None,
        };
        let collected: Vec<anyrender::types::Glyph> = glyphs.collect();
        if collected.is_empty() {
            return;
        }
        let anchor = transform
            * Point::new(collected[0].x as f64, collected[0].y as f64 - font_size as f64);
        let mut col = (anchor.x / self.metrics.cell_w_px as f64).floor().max(0.0) as u16;
        let mut row = (anchor.y / self.metrics.cell_h_px as f64).floor().max(0.0) as u16;
        for glyph in collected {
            let chars = glyph_id_to_chars(_font, glyph.id);
            for ch in chars {
                let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1).max(1) as u16;
                if col >= self.surface.width() {
                    col = 0;
                    row = row.saturating_add(1);
                }
                if row >= self.surface.height() {
                    return;
                }
                if self.in_clip(col, row) {
                    self.paint_cell(ch, col, row, fg, None);
                    for extra in 1..ch_width {
                        let cx = col.saturating_add(extra);
                        if cx < self.surface.width() {
                            self.paint_cell(' ', cx, row, fg, None);
                        }
                    }
                }
                col = col.saturating_add(ch_width);
            }
        }
    }

    fn draw_box_shadow(
        &mut self,
        _transform: Affine,
        _rect: KRect,
        _brush: Color,
        _radius: f64,
        _std_dev: f64,
    ) {
        // ignore shadows for now
    }
}

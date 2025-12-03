use std::collections::VecDeque;

use anyrender::{types::PaintRef, PaintScene};
use peniko::{Color, ImageBrushRef};
use unicode_width::UnicodeWidthChar;

use crate::geometry::Rect;
use crate::surface::Surface;

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
}

impl<'a> TerminalScene<'a> {
    pub fn new(
        surface: &'a mut Surface,
        images: &'a mut VecDeque<InlineImage>,
        metrics: CellMetrics,
    ) -> Self {
        Self {
            surface,
            images,
            metrics,
            clip_stack: Vec::new(),
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
        let within_x = x >= cx && x < cx.saturating_add(width);
        let within_y = y >= cy && y < cy.saturating_add(height);
        within_x && within_y
    }

    fn push_text(&mut self, text: &str, x_px: f32, y_px: f32) {
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
                *slot = ch;
            }
            if ch_width > 1 {
                for extra in 1..ch_width {
                    if let Some(slot) = self.surface.content.get_mut(start + extra) {
                        *slot = ' ';
                    }
                }
            }
            col = col.saturating_add(ch_width);
        }
    }

    fn push_image(&mut self, image: ImageBrushRef, x_px: f32, y_px: f32) {
        let data = image.image.data.clone();
        self.images.push_back(InlineImage {
            data,
            width_px: image.image.width,
            height_px: image.image.height,
            x_px,
            y_px,
        });
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
        _blend: impl Into<anyrender::types::BlendMode>,
        _alpha: f32,
        _transform: kurbo::Affine,
        clip: &impl kurbo::Shape,
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

    fn stroke(
        &mut self,
        _style: &kurbo::Stroke,
        _transform: kurbo::Affine,
        _brush: impl Into<PaintRef<'_>>,
        _brush_transform: Option<kurbo::Affine>,
        _shape: &impl kurbo::Shape,
    ) {
        // no-op fallback
    }

    fn fill(
        &mut self,
        _style: peniko::Fill,
        transform: kurbo::Affine,
        brush: impl Into<PaintRef<'_>>,
        _brush_transform: Option<kurbo::Affine>,
        shape: &impl kurbo::Shape,
    ) {
        let bbox = shape.bounding_box();
        let (x_px, y_px) = {
            let p = transform * kurbo::Point::new(bbox.x0, bbox.y0);
            (p.x as f32, p.y as f32)
        };
        match brush.into() {
            PaintRef::Solid(_c) => {
                // For now, fill via text spaces; a more complete ANSI fill could be added.
                let cell_x = (x_px / self.metrics.cell_w_px).floor().max(0.0) as u16;
                let cell_y = (y_px / self.metrics.cell_h_px).floor().max(0.0) as u16;
                if self.in_clip(cell_x, cell_y) {
                    self.push_text(" ", x_px, y_px);
                }
            }
            PaintRef::Image(img) => self.push_image(img, x_px, y_px),
            _ => {}
        }
    }

    fn draw_glyphs<'b, 's: 'b>(
        &'s mut self,
        _font: &'b peniko::FontData,
        font_size: f32,
        _hint: bool,
        _normalized_coords: &'b [anyrender::types::NormalizedCoord],
        _style: impl Into<anyrender::types::StyleRef<'b>>,
        brush: impl Into<PaintRef<'b>>,
        _brush_alpha: f32,
        transform: kurbo::Affine,
        _glyph_transform: Option<kurbo::Affine>,
        glyphs: impl Iterator<Item = anyrender::types::Glyph>,
    ) {
        if !matches!(brush.into(), PaintRef::Solid(_)) {
            return;
        }
        for glyph in glyphs {
            let p =
                transform * kurbo::Point::new(glyph.x as f64, glyph.y as f64 - font_size as f64);
            self.push_text("█", p.x as f32, p.y as f32);
        }
    }

    fn draw_box_shadow(
        &mut self,
        _transform: kurbo::Affine,
        _rect: kurbo::Rect,
        _brush: Color,
        _radius: f64,
        _std_dev: f64,
    ) {
        // ignore shadows for now
    }
}

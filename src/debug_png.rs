use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Result;
use font8x8::UnicodeFonts;
use png::{BitDepth, ColorType, Encoder};
use termwiz::color::ColorAttribute;

use crate::surface::Surface;

fn attr_to_rgb(attr: Option<ColorAttribute>, default: (u8, u8, u8)) -> (u8, u8, u8) {
    match attr {
        Some(ColorAttribute::TrueColorWithDefaultFallback(srgb))
        | Some(ColorAttribute::TrueColorWithPaletteFallback(srgb, _)) => (srgb.0, srgb.1, srgb.2),
        Some(ColorAttribute::PaletteIndex(idx)) => palette_16(idx),
        _ => default,
    }
}

fn palette_16(idx: u8) -> (u8, u8, u8) {
    match idx & 0x0F {
        0 => (0, 0, 0),
        1 => (205, 0, 0),
        2 => (0, 205, 0),
        3 => (205, 205, 0),
        4 => (0, 0, 238),
        5 => (205, 0, 205),
        6 => (0, 205, 205),
        7 => (229, 229, 229),
        8 => (127, 127, 127),
        9 => (255, 0, 0),
        10 => (0, 255, 0),
        11 => (255, 255, 0),
        12 => (92, 92, 255),
        13 => (255, 0, 255),
        14 => (0, 255, 255),
        _ => (255, 255, 255),
    }
}

/// Write a `Surface` to a PNG image for debugging.
pub fn write_surface_png(surface: &Surface, cell_w: u32, cell_h: u32, path: impl AsRef<Path>) -> Result<()> {
    let width_px = surface.width() as u32 * cell_w;
    let height_px = surface.height() as u32 * cell_h;
    let mut buf = vec![0u8; (width_px * height_px * 4) as usize];

    for y in 0..surface.height() {
        for x in 0..surface.width() {
            let idx = y as usize * surface.width() as usize + x as usize;
            let cell = &surface.content[idx];
            let (bg_r, bg_g, bg_b) = attr_to_rgb(cell.bg, (0, 0, 0));
            let (fg_r, fg_g, fg_b) = attr_to_rgb(cell.fg, (255, 255, 255));

            // Fill background
            for py in 0..cell_h {
                for px in 0..cell_w {
                    let gx = x as u32 * cell_w + px;
                    let gy = y as u32 * cell_h + py;
                    let base = ((gy * width_px + gx) * 4) as usize;
                    buf[base] = bg_r;
                    buf[base + 1] = bg_g;
                    buf[base + 2] = bg_b;
                    buf[base + 3] = 255;
                }
            }

            // Draw glyph for basic ASCII using font8x8 bitmap.
            let ch = cell.ch as u32;
            if let Some(bitmap) = font8x8::BASIC_FONTS.get(ch) {
                let scale_x = cell_w / 8; // crude scale to fit cell
                let scale_y = cell_h / 8;
                for (row_idx, row_bits) in bitmap.iter().enumerate() {
                    for bit in 0..8 {
                        if (row_bits >> bit) & 1 == 1 {
                            let px = bit as u32 * scale_x;
                            let py = row_idx as u32 * scale_y;
                            for sy in 0..scale_y.max(1) {
                                for sx in 0..scale_x.max(1) {
                                    let gx = x as u32 * cell_w + px + sx;
                                    let gy = y as u32 * cell_h + py + sy;
                                    if gx < width_px && gy < height_px {
                                        let base = ((gy * width_px + gx) * 4) as usize;
                                        buf[base] = fg_r;
                                        buf[base + 1] = fg_g;
                                        buf[base + 2] = fg_b;
                                        buf[base + 3] = 255;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let file = File::create(path)?;
    let w = BufWriter::new(file);
    let mut encoder = Encoder::new(w, width_px, height_px);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&buf)?;
    writer.finish()?;
    Ok(())
}

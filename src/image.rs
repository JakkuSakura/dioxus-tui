use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use png::{BitDepth, ColorType, Encoder};
use std::io::Write;
use termwiz::escape::{osc, Action, CSI};

use crate::scene::InlineImage;

fn rgba_to_png_bytes(data: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut encoder = Encoder::new(&mut buf, width, height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(data)?;
        writer.finish()?;
    }
    Ok(buf)
}

pub fn emit_inline_images(
    images: &[InlineImage],
    out: &mut Vec<u8>,
    cell_w_px: f32,
    cell_h_px: f32,
) -> Result<()> {
    for img in images {
        let png = rgba_to_png_bytes(&img.data, img.width_px, img.height_px)?;
        let b64 = BASE64.encode(png);
        let x_cell = (img.x_px / cell_w_px).floor().max(0.0) as u32;
        let y_cell = (img.y_px / cell_h_px).floor().max(0.0) as u32;
        let w_cell = ((img.width_px as f32 / cell_w_px).ceil() as u32).max(1);
        let h_cell = ((img.height_px as f32 / cell_h_px).ceil() as u32).max(1);

        // Kitty inline image protocol: OSC 1337;File=...;width=..;height=..;inline=1:<base64> BEL
        let spec = format!(
            "File=inline=1;width={}cell;height={}cell;preserveAspectRatio=0",
            w_cell, h_cell
        );
        let payload = format!("\u{1b}]1337;{spec}:{}\u{7}", b64);
        // Move cursor to position
        let goto = CSI::CursorPosition {
            line: (y_cell + 1) as usize,
            col: (x_cell + 1) as usize,
        };
        goto.write_to(out)?;
        out.write_all(payload.as_bytes())?;
    }
    Ok(())
}

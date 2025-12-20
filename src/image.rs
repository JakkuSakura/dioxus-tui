use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use png::{BitDepth, ColorType, Decoder, Encoder, Transformations};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

use crate::scene::InlineImage;
use termwiz::image::{ImageData, ImageDataType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedImage {
    pub src: String,
    pub image: Arc<ImageData>,
    pub x_cell: u16,
    pub y_cell: u16,
    pub width_cells: u16,
    pub height_cells: u16,
}

#[derive(Debug)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

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

fn decode_png_rgba(png_bytes: &[u8]) -> Result<DecodedImage> {
    let mut decoder = Decoder::new(png_bytes);
    decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;

    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    let bytes = &buf[..info.buffer_size()];

    let rgba = match info.color_type {
        ColorType::Rgba => bytes.to_vec(),
        ColorType::Rgb => bytes
            .chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect(),
        ColorType::Grayscale => bytes
            .iter()
            .flat_map(|&v| [v, v, v, 255])
            .collect(),
        ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .flat_map(|ga| [ga[0], ga[0], ga[0], ga[1]])
            .collect(),
        ColorType::Indexed => {
            // With EXPAND, Indexed should already have been expanded.
            anyhow::bail!("unexpected indexed color type after EXPAND")
        }
    };

    Ok(DecodedImage {
        width: info.width,
        height: info.height,
        rgba,
    })
}

static IMAGE_CACHE: OnceLock<Mutex<HashMap<String, Arc<DecodedImage>>>> = OnceLock::new();

pub fn load_png_image(src: &str) -> Result<Arc<DecodedImage>> {
    let cache = IMAGE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(hit) = cache.lock().unwrap().get(src).cloned() {
        return Ok(hit);
    }

    let bytes = png_bytes_from_src(src)?;
    let decoded = Arc::new(decode_png_rgba(&bytes)?);

    cache
        .lock()
        .unwrap()
        .insert(src.to_string(), decoded.clone());
    Ok(decoded)
}

pub fn png_bytes_from_src(src: &str) -> Result<Vec<u8>> {
    if let Some(rest) = src.strip_prefix("data:image/png;base64,") {
        Ok(BASE64.decode(rest)?)
    } else {
        Ok(std::fs::read(src)?)
    }
}

pub fn placed_image_from_png(
    src: &str,
    x_cell: u16,
    y_cell: u16,
    width_cells: u16,
    height_cells: u16,
) -> Result<PlacedImage> {
    let bytes = png_bytes_from_src(src)?;
    let image = Arc::new(ImageData::with_data(ImageDataType::EncodedFile(bytes)));

    Ok(PlacedImage {
        src: src.to_string(),
        image,
        x_cell,
        y_cell,
        width_cells: width_cells.max(1),
        height_cells: height_cells.max(1),
    })
}

pub fn encode_sixel_rgba(rgba: &[u8], width: u32, height: u32) -> String {
    // Minimal sixel encoder with a small (<=64) RGB palette (4 levels per channel).
    // This is intentionally simple and good enough for demo/debug rendering.
    fn quantize(v: u8) -> u8 {
        match v {
            0..=63 => 0,
            64..=127 => 85,
            128..=191 => 170,
            _ => 255,
        }
    }

    fn pct(v: u8) -> u8 {
        ((v as u16 * 100) / 255) as u8
    }

    let mut palette: HashMap<(u8, u8, u8), u8> = HashMap::new();
    let mut next_idx: u8 = 0;
    let mut idx_map: Vec<Option<u8>> = Vec::with_capacity((width * height) as usize);

    for p in rgba.chunks_exact(4) {
        let a = p[3];
        if a < 16 {
            idx_map.push(None);
            continue;
        }
        let key = (quantize(p[0]), quantize(p[1]), quantize(p[2]));
        let idx = *palette.entry(key).or_insert_with(|| {
            let idx = next_idx;
            next_idx = next_idx.saturating_add(1);
            idx
        });
        idx_map.push(Some(idx));
    }

    let mut out = String::new();
    out.push_str("\x1bPq");
    out.push_str(&format!("\"1;1;{};{}", width, height));

    // Define palette
    let mut colors: Vec<((u8, u8, u8), u8)> = palette.into_iter().collect();
    colors.sort_by_key(|(_, idx)| *idx);
    for ((r, g, b), idx) in &colors {
        out.push_str(&format!("#{};2;{};{};{}", idx, pct(*r), pct(*g), pct(*b)));
    }

    let w = width as usize;
    let h = height as usize;
    let bands = (h + 5) / 6;

    for band in 0..bands {
        let y0 = band * 6;

        for (_rgb, idx) in &colors {
            // Build one scanline for this color.
            let mut line = Vec::with_capacity(w);
            for x in 0..w {
                let mut bits: u8 = 0;
                for dy in 0..6 {
                    let y = y0 + dy;
                    if y >= h {
                        continue;
                    }
                    let pos = y * w + x;
                    if idx_map.get(pos).copied().flatten() == Some(*idx) {
                        bits |= 1 << dy;
                    }
                }
                line.push((bits + 63) as u8);
            }

            if line.iter().all(|&b| b == 63) {
                continue;
            }

            out.push_str(&format!("#{}", idx));
            // naive RLE
            let mut i = 0;
            while i < line.len() {
                let b = line[i];
                let mut run = 1usize;
                while i + run < line.len() && line[i + run] == b {
                    run += 1;
                }
                if run > 3 {
                    out.push_str(&format!("!{}{}", run, b as char));
                } else {
                    for _ in 0..run {
                        out.push(b as char);
                    }
                }
                i += run;
            }
            out.push('$');
        }

        if band + 1 < bands {
            out.push('-');
        }
    }

    out.push_str("\x1b\\");
    out
}

pub fn emit_inline_images(
    images: &VecDeque<InlineImage>,
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
        let goto = format!("\u{1b}[{};{}H", y_cell + 1, x_cell + 1);
        out.write_all(goto.as_bytes())?;
        out.write_all(payload.as_bytes())?;
    }
    Ok(())
}

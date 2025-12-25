use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use png::{BitDepth, ColorType, Decoder, Encoder, Transformations};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedImage {
    pub src: String,
    pub png: Vec<u8>,
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

#[allow(dead_code)]
pub(crate) fn rgba_to_png_bytes(data: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
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
    Ok(PlacedImage {
        src: src.to_string(),
        png: bytes,
        x_cell,
        y_cell,
        width_cells: width_cells.max(1),
        height_cells: height_cells.max(1),
    })
}

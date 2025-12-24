use std::collections::HashMap;
use std::sync::{Arc, Mutex, LazyLock};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::config::{ColorMode, PaletteRoles};
use crate::geometry::Rect;
use crate::surface::Surface;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomDrawMode {
    Html,
    Native,
}

pub struct DrawContext<'a> {
    pub surface: &'a mut Surface,
    pub rect: Rect,
    pub color_mode: ColorMode,
    pub truecolor: bool,
    pub palette_roles: PaletteRoles,
}

type DrawCallback = Arc<dyn Fn(&mut DrawContext) + Send + Sync>;

static DRAW_ID: AtomicUsize = AtomicUsize::new(1);
static DRAW_REGISTRY: LazyLock<Mutex<HashMap<String, DrawCallback>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn on_draw<F>(f: F) -> String
where
    F: Fn(&mut DrawContext) + Send + Sync + 'static,
{
    let id = DRAW_ID.fetch_add(1, Ordering::Relaxed);
    let key = format!("draw-{id}");
    DRAW_REGISTRY
        .lock()
        .expect("draw registry")
        .insert(key.clone(), Arc::new(f));
    key
}

pub fn lookup_draw(id: &str) -> Option<DrawCallback> {
    DRAW_REGISTRY
        .lock()
        .expect("draw registry")
        .get(id)
        .cloned()
}

pub fn rgb_to_attr(
    r: u8,
    g: u8,
    b: u8,
    color_mode: ColorMode,
    truecolor: bool,
) -> termwiz::color::ColorAttribute {
    use termwiz::color::{ColorAttribute, SrgbaTuple};

    let srgb = SrgbaTuple::from((r, g, b));
    let palette_idx_256 = 16 + 36 * (r as u16 / 51) as u8 + 6 * (g as u16 / 51) as u8 + (b as u16 / 51) as u8;
    let base_idx = (if r >= 128 { 1 } else { 0 }) | (if g >= 128 { 2 } else { 0 }) | (if b >= 128 { 4 } else { 0 });

    match color_mode {
        ColorMode::BaseColors => ColorAttribute::PaletteIndex(base_idx),
        ColorMode::Ansi => ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256),
        ColorMode::Rgb => {
            if truecolor {
                ColorAttribute::TrueColorWithDefaultFallback(srgb)
            } else {
                ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256)
            }
        }
    }
}

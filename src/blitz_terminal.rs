#![cfg(feature = "blitz-terminal")]

use std::io::Write;

use anyhow::Result;
use anyrender::ImageRenderer;
use base64::Engine;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus_core::ComponentFunction;

use crate::capabilities::TerminalCapabilities;
use crate::geometry::Rect;
use crate::image::{encode_sixel_rgba, rgba_to_png_bytes};
use crate::render::DioxusRenderer;
use crate::{RawVirtualDom, RenderingMode};

const DEFAULT_CELL_W_PX: f32 = 8.0;
const DEFAULT_CELL_H_PX: f32 = 16.0;

pub(crate) fn render_blitz_terminal<P, F>(
    rendering_mode: RenderingMode,
    term_caps: TerminalCapabilities,
    raw: RawVirtualDom<P, F>,
    width_cells: u16,
    height_cells: u16,
) -> Result<bool>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    if rendering_mode != RenderingMode::BlitzTerminal {
        return Ok(false);
    }

    if !term_caps.inline_images {
        return Ok(false);
    }

    let width_px = (width_cells as f32 * DEFAULT_CELL_W_PX).ceil().max(1.0) as u32;
    let height_px = (height_cells as f32 * DEFAULT_CELL_H_PX).ceil().max(1.0) as u32;
    let viewport = Viewport::new(width_px, height_px, 1.0, ColorScheme::Light);

    let vdom = raw.into_virtual_dom();
    let (mut renderer, _event_tx, _event_rx) = DioxusRenderer::new_with_viewport(vdom, viewport);
    renderer.update();

    let area = Rect::new(0, 0, width_cells, height_cells);
    let metrics = crate::scene::CellMetrics {
        cell_w_px: DEFAULT_CELL_W_PX,
        cell_h_px: DEFAULT_CELL_H_PX,
    };
    let _ = renderer.layout_root(area, metrics);

    let mut image_renderer =
        <anyrender_vello_cpu::VelloCpuImageRenderer as ImageRenderer>::new(width_px, height_px);
    let mut rgba = Vec::new();
    image_renderer.render_to_vec(
        |scene| {
            paint_scene(scene, renderer.doc.inner.as_ref(), 1.0, width_px, height_px);
        },
        &mut rgba,
    );

    let mut out = std::io::stdout().lock();
    if term_caps.iterm2_images {
        let png = rgba_to_png_bytes(&rgba, width_px, height_px)?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(png);
        // Emit at the current cursor position; do not use absolute cursor addressing in `render()` mode.
        write!(
            out,
            "\u{1b}]1337;File=inline=1;width={}cell;height={}cell;preserveAspectRatio=0:{}\u{7}",
            width_cells, height_cells, b64
        )?;
    } else if term_caps.sixel_images {
        let sixel = encode_sixel_rgba(&rgba, width_px, height_px);
        out.write_all(sixel.as_bytes())?;
    } else {
        // Defensive: `inline_images` implies iterm2 or sixel, but keep a safe fallback.
        return Ok(false);
    }

    // Ensure subsequent output (e.g. shell prompt) appears below the image.
    for _ in 0..height_cells {
        out.write_all(b"\n")?;
    }
    out.flush()?;

    Ok(true)
}

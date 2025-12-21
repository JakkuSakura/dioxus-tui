#![cfg(feature = "blitz")]

use std::io::Write;

use anyhow::Result;
use anyrender::ImageRenderer;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus_core::ComponentFunction;
use termwiz::render::terminfo::TerminfoRenderer;
use termwiz::render::RenderTty;
use termwiz::surface::{Change, Image, Position};
use termwiz::terminal::ScreenSize;
use termwiz::image::TextureCoordinate;

use crate::capabilities::TerminalCapabilities;
use crate::geometry::Rect;
use crate::image::{encode_sixel_rgba, rgba_to_png_bytes};
use crate::render::DioxusRenderer;
use crate::{RawVirtualDom, RenderingMode};

const DEFAULT_CELL_W_PX: f32 = 8.0;
const DEFAULT_CELL_H_PX: f32 = 16.0;

pub(crate) fn render_blitz_terminal<P, F>(
    rendering_mode: RenderingMode,
    termwiz_caps: termwiz::caps::Capabilities,
    term_caps: TerminalCapabilities,
    raw: RawVirtualDom<P, F>,
    width_cells: u16,
    height_cells: u16,
) -> Result<bool>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let _ = rendering_mode;

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
        let image = Image {
            width: width_cells as usize,
            height: height_cells as usize,
            top_left: TextureCoordinate::new_f32(0.0, 0.0),
            bottom_right: TextureCoordinate::new_f32(1.0, 1.0),
            image: std::sync::Arc::new(termwiz::image::ImageData::with_data(
                termwiz::image::ImageDataType::EncodedFile(png),
            )),
        };

        let mut renderer = TerminfoRenderer::new(termwiz_caps);
        let mut tty = StdoutRenderTty {
            out: &mut out,
            size: ScreenSize {
                rows: height_cells as usize,
                cols: width_cells as usize,
                xpixel: 0,
                ypixel: 0,
            },
        };
        renderer.render_to(
            &[
                // Ensure we render at the current cursor position.
                Change::CursorPosition {
                    x: Position::Relative(0),
                    y: Position::Relative(0),
                },
                Change::Image(image),
            ],
            &mut tty,
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

struct StdoutRenderTty<'a> {
    out: &'a mut dyn Write,
    size: ScreenSize,
}

impl Write for StdoutRenderTty<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.out.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}

impl RenderTty for StdoutRenderTty<'_> {
    fn get_size_in_cells(&mut self) -> termwiz::Result<(usize, usize)> {
        Ok((self.size.cols, self.size.rows))
    }
}

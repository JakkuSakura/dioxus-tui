#![cfg(feature = "blitz")]

use std::io::Write;

use anyhow::Result;
use anyrender::ImageRenderer;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus_core::ComponentFunction;
use termwiz::render::terminfo::TerminfoRenderer;
use termwiz::render::RenderTty;
use termwiz::surface::Change;
use termwiz::terminal::ScreenSize;
use termwiz::terminal::Terminal as _;
use termwiz::image::TextureCoordinate;

use crate::capabilities::TerminalCapabilities;
use crate::cell_render::paint_surface;
use crate::config::{ColorMode, Config, ImagePolicy, RenderingMode};
use crate::geometry::Rect;
use crate::image::{encode_sixel_rgba, rgba_to_png_bytes};
use crate::layout::resolve_document_with_viewport_and_extra_css;
use crate::render::DioxusRenderer;
use crate::surface::Surface;
use crate::RawVirtualDom;

pub(crate) fn render_blitz_terminal<P, F>(
    rendering_mode: RenderingMode,
    termwiz_caps: termwiz::caps::Capabilities,
    term_caps: TerminalCapabilities,
    raw: RawVirtualDom<P, F>,
    width_cells: u16,
    height_cells: u16,
    cfg: Config,
) -> Result<bool>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let _ = rendering_mode;

    if !term_caps.inline_images {
        anyhow::bail!("BlitzTerminal requires inline image protocol support");
    }

    // Strict sizing: require terminal-provided pixel geometry.
    let mut term = termwiz::terminal::new_terminal(termwiz_caps.clone())
        .map_err(|e| anyhow::anyhow!("failed to initialize terminal for sizing: {e}"))?;
    let size = term
        .get_screen_size()
        .map_err(|e| anyhow::anyhow!("failed to query terminal size: {e}"))?;
    if size.cols == 0 || size.rows == 0 {
        anyhow::bail!("terminal reported zero-sized cell grid");
    }
    if size.xpixel == 0 || size.ypixel == 0 {
        anyhow::bail!(
            "terminal did not report pixel dimensions (xpixel/ypixel=0); cannot use BlitzTerminal"
        );
    }
    if size.cols as u16 != width_cells || size.rows as u16 != height_cells {
        anyhow::bail!(
            "terminal cell size mismatch: requested {}x{} but terminal is {}x{}",
            width_cells,
            height_cells,
            size.cols,
            size.rows
        );
    }

    let base_width_px = size.xpixel as u32;
    let base_height_px = size.ypixel as u32;
    let cell_w_px = (size.xpixel as f32) / (size.cols as f32);
    let cell_h_px = (size.ypixel as f32) / (size.rows as f32);

    let supersample = cfg.blitz_hidpi_scale.max(1) as f32;
    let render_width_px = ((base_width_px as f32) * supersample).ceil().max(1.0) as u32;
    let render_height_px = ((base_height_px as f32) * supersample).ceil().max(1.0) as u32;

    // Layout should be resolved at constant logical size:
    // logical = physical / hidpi_scale.
    // We keep logical == base by scaling physical and hidpi_scale by the same factor.
    let viewport = Viewport::new(render_width_px, render_height_px, supersample, ColorScheme::Light);

    let area = Rect::new(0, 0, width_cells, height_cells);
    // Use the base per-cell pixel sizes for the cell-based painter.
    // The viewport scaling handles HiDPI.
    let metrics = crate::scene::CellMetrics {
        cell_w_px,
        cell_h_px,
    };

    let vdom = raw.into_virtual_dom();
    let (mut renderer, _event_tx, _event_rx) =
        DioxusRenderer::new_with_viewport(vdom, viewport.clone());
    renderer.update();

    // Make the default text size map 1em ~= 1 cell height.
    // This keeps the rasterized output readable and aligned with the terminal grid.
    let font_px = (cell_h_px.round().max(1.0)) as u32;
    let extra_css = format!(
        ":root, html, body {{ font-family: monospace; font-size: {}px; line-height: {}px; }}",
        font_px, font_px
    );
    let _ = resolve_document_with_viewport_and_extra_css(
        &mut renderer.doc,
        viewport.clone(),
        Some(extra_css.as_str()),
    );

    // Crop trailing empty rows before emission.
    // Use the normal cell pipeline to decide what is "empty".
    let mut crop_cfg = cfg;
    crop_cfg.color_mode = ColorMode::Rgb;
    crop_cfg.image_policy = ImagePolicy::Sampling;

    let mut surface = Surface::new(width_cells, height_cells);
    let mut images = std::collections::VecDeque::new();
    paint_surface(
        &mut surface,
        &mut images,
        renderer.doc.inner.as_ref(),
        area,
        metrics,
        crop_cfg.palette_roles,
        crop_cfg.color_mode,
        term_caps.truecolor,
        Some(&renderer.draw_state),
        crop_cfg.custom_draw_mode,
        crop_cfg.image_policy,
        crop_cfg.image_downgrade,
        false,
    )?;

    let width = width_cells as usize;
    let height = height_cells as usize;
    let mut last_row_with_content: Option<usize> = None;
    for (y, row) in surface.content.chunks(width).enumerate() {
        if row.iter().any(crate::surface::Cell::has_visible_content) {
            last_row_with_content = Some(y);
        }
    }
    let mut last_row_with_image: Option<usize> = None;
    for img in &images {
        let bottom = (img.y_cell as usize)
            .saturating_add((img.height_cells as usize).saturating_sub(1));
        last_row_with_image = Some(last_row_with_image.map_or(bottom, |v| v.max(bottom)));
    }
    let max_row = last_row_with_content
        .into_iter()
        .chain(last_row_with_image)
        .max();
    let cropped_height_cells = max_row
        .map(|r| (r + 1).min(height))
        .unwrap_or(0)
        .max(1) as u16;
    let cropped_height_px = ((cropped_height_cells as f32) * (cell_h_px * supersample))
        .ceil()
        .max(1.0) as u32;

    // Paint at supersampled resolution.
    let mut image_renderer =
        <anyrender_vello_cpu::VelloCpuImageRenderer as ImageRenderer>::new(render_width_px, render_height_px);
    let mut rgba = Vec::new();
    image_renderer.render_to_vec(
        |scene| {
            paint_scene(
                scene,
                renderer.doc.inner.as_ref(),
                viewport.scale_f64(),
                render_width_px,
                render_height_px,
            );
        },
        &mut rgba,
    );
    if cropped_height_px < render_height_px {
        let bytes_per_row = (render_width_px as usize) * 4;
        let keep = (cropped_height_px as usize) * bytes_per_row;
        if keep < rgba.len() {
            rgba.truncate(keep);
        }
    }

    let mut out = std::io::stdout().lock();
    if term_caps.iterm2_images {
        let png = rgba_to_png_bytes(&rgba, render_width_px, cropped_height_px.min(render_height_px))?;

        let image = termwiz::surface::Image {
            width: width_cells as usize,
            height: cropped_height_cells as usize,
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
                xpixel: size.xpixel,
                ypixel: size.ypixel,
            },
        };
        renderer.render_to(&[Change::Image(image)], &mut tty)?;
    } else if term_caps.sixel_images {
        let sixel = encode_sixel_rgba(&rgba, render_width_px, cropped_height_px.min(render_height_px));
        out.write_all(sixel.as_bytes())?;
    } else {
        anyhow::bail!("BlitzTerminal requires iterm2 or sixel inline images");
    }

    // If the terminal moved the cursor (default), it should already be below the image.
    // Add a single newline to ensure the prompt does not sit on the last row.
    out.write_all(b"\r\n")?;
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

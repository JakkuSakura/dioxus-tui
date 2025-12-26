use std::{any::Any, cell::RefCell, rc::Rc};

use crate::capabilities::{DetectedCapabilities, InlineImageProtocol, TerminalCapabilities};
use crate::capabilities::detect as detect_capabilities;
use crate::capabilities::termwiz_capabilities;
use crate::cell_render::paint_surface;
use crate::config::RenderingMode;
use crate::event::{EventContext, EventDispatcher};
use crate::geometry::Rect;
use crate::hooks::{CaretCommand, CursorCommand, CursorMode, CursorUnit};
use crate::image::PlacedImage;
use crate::render::{
    apply_caret, apply_cursor_overlay, flush_surface, set_sgr_pixel_mouse, terminal_size,
    CaretState, DioxusRenderer, InputEvent,
};
use crate::scene::CellMetrics;
use crate::surface::Surface;
use crate::RawVirtualDom;
use crate::error::Result;
use dioxus_core::{ComponentFunction, Element};
use futures::{FutureExt, StreamExt};
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use termwiz::{
    surface::Change,
    terminal::{buffered::BufferedTerminal, ScreenSize, Terminal},
};
use termwiz::terminal::new_terminal;
use tokio::time::sleep;
use tracing::debug;

#[cfg(feature = "blitz")]
use blitz_traits::shell::{ColorScheme, Viewport};
#[cfg(feature = "blitz")]
use crate::render::inline_encoder_for_caps;
#[cfg(feature = "blitz")]
use anyrender::ImageRenderer;
#[cfg(feature = "blitz")]
use termwiz::{color::ColorAttribute, surface::Position};
#[cfg(feature = "blitz")]
use blitz_paint::paint_scene;
#[cfg(feature = "blitz")]
use crate::layout::resolve_document_with_viewport_and_extra_css;

pub type Config = crate::Config;

/// Launches the WebView and runs the event loop, with configuration and root props.
pub fn launch(
    root: fn() -> Element,
    contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
    platform_config: Config,
) -> anyhow::Result<()> {
    let raw = RawVirtualDom::with_contexts(move |_| root(), (), contexts);
    crate::launch_raw(raw, platform_config)
}

pub(crate) async fn run_renderer<P, F>(cfg: Config, raw: RawVirtualDom<P, F>) -> Result<()>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let detected = match detect_capabilities() {
        Ok(detected) => detected,
        Err(_err) => DetectedCapabilities {
            termwiz: termwiz_capabilities()?,
            terminal: TerminalCapabilities {
                truecolor: false,
                inline_images: false,
                inline_protocol: InlineImageProtocol::None,
            },
        },
    };

    let vdom = raw.into_virtual_dom();
    let (renderer, event_tx, event_rx) = DioxusRenderer::new(vdom);

    if cfg.rendering_mode == RenderingMode::Debug {
        let mut renderer = renderer;
        renderer.update();
        println!("-- dioxus-tui debug snapshot --");
        return Ok(());
    }

    run_tui_renderer(cfg, detected, renderer, event_rx, event_tx).await
}

async fn run_tui_renderer(
    cfg: Config,
    detected: DetectedCapabilities,
    mut renderer: DioxusRenderer,
    mut raw_event_reciever: UnboundedReceiver<InputEvent>,
    _event_tx: UnboundedSender<InputEvent>,
) -> Result<()> {
    let run_terminal = cfg.rendering_mode != RenderingMode::Headless;

    let mut terminal = if run_terminal {
        let mut term = new_terminal(detected.termwiz.clone())?;
        term.set_raw_mode()?;
        term.enter_alternate_screen()?;
        Some(BufferedTerminal::new(term)?)
    } else {
        None
    };

    let capabilities = detected.terminal;
    let mut last_surface: Option<Surface> = None;
    let mut last_area: Option<Rect> = None;
    #[cfg(feature = "blitz")]
    let mut last_pixel_viewport: Option<Rect> = None;
    #[cfg(not(feature = "blitz"))]
    let mut last_pixel_viewport: Option<Rect> = None;
    #[cfg(feature = "blitz")]
    let mut last_pixel_scale: f32 = 1.0;
    #[cfg(not(feature = "blitz"))]
    let last_pixel_scale: f32 = 1.0;
    let mut last_cell_metrics = CellMetrics {
        cell_w_px: 8.0,
        cell_h_px: 16.0,
    };
    let mut dispatcher = EventDispatcher::new();
    let cursor_state = dispatcher.cursor_state();
    let caret_state = Rc::new(RefCell::new(CaretState::default()));
    let mut last_images: Option<std::collections::VecDeque<PlacedImage>> = None;

    let _cursor_subscription = {
        let cursor_state = cursor_state.clone();
        renderer.cursor_bus.subscribe(Rc::new(move |command| {
            let mut state = cursor_state.borrow_mut();
            match command {
                CursorCommand::Show => state.visible = true,
                CursorCommand::Hide => state.visible = false,
                CursorCommand::SetStyle(style) => state.style = style,
                CursorCommand::FollowMouse => state.mode = CursorMode::FollowMouse,
                CursorCommand::SetCellPosition(x, y) => {
                    state.mode = CursorMode::Manual;
                    state.unit = CursorUnit::Cell;
                    state.position = Some((x, y));
                    state.visible = true;
                }
                CursorCommand::SetPixelPosition(x, y) => {
                    state.mode = CursorMode::Manual;
                    state.unit = CursorUnit::Pixel;
                    state.position = Some((x, y));
                    state.visible = true;
                }
            }
        }))
    };

    let _caret_subscription = {
        let caret_state = caret_state.clone();
        renderer.caret_bus.subscribe(Rc::new(move |command| {
            let mut state = caret_state.borrow_mut();
            match command {
                CaretCommand::Show => state.visible = true,
                CaretCommand::Hide => state.visible = false,
                CaretCommand::SetPosition(x, y) => {
                    state.position = Some((x, y));
                    state.visible = true;
                }
            }
        }))
    };

    renderer.update();

    let mut paint_error: Option<crate::error::Error> = None;
    let mut pixel_mouse_enabled = false;

    if let Some(term) = &mut terminal {
        #[cfg(feature = "blitz")]
        let enable_pixel_mouse = cfg.sgr_pixel_mouse || cfg.rendering_mode == RenderingMode::BlitzTerminal;
        #[cfg(not(feature = "blitz"))]
        let enable_pixel_mouse = cfg.sgr_pixel_mouse;
        if enable_pixel_mouse {
            set_sgr_pixel_mouse(term, true)?;
            pixel_mouse_enabled = true;
        }
    }

    let result = (async {
        loop {
            if let Some(term) = &mut terminal {
                if let Some(term_evt) = term.terminal().poll_input(Some(cfg.tick_rate))? {
                    let mut ctx = EventContext {
                        renderer: &mut renderer,
                        cfg,
                        viewport: last_area.unwrap_or_else(|| Rect::new(0, 0, 0, 0)),
                        pixel_viewport: last_pixel_viewport,
                        pixel_scale: last_pixel_scale,
                        cell_metrics: last_cell_metrics,
                    };
                    if dispatcher.handle(term_evt, &mut ctx) {
                        return Ok(());
                    }
                }
            } else if cfg.tick_rate > std::time::Duration::ZERO {
                sleep(cfg.tick_rate).await;
            }

            while let Some(evt) = raw_event_reciever.next().now_or_never().flatten() {
                match evt {
                    InputEvent::Close => return Ok(()),
                    InputEvent::UserInput(term_evt) => {
                        let mut ctx = EventContext {
                            renderer: &mut renderer,
                            cfg,
                            viewport: last_area.unwrap_or_else(|| Rect::new(0, 0, 0, 0)),
                            pixel_viewport: last_pixel_viewport,
                            pixel_scale: last_pixel_scale,
                            cell_metrics: last_cell_metrics,
                        };
                        if dispatcher.handle(term_evt, &mut ctx) {
                            return Ok(());
                        }
                    }
                }
            }

            renderer.update();

            if let Some(term) = &mut terminal {
                let (area, metrics) = terminal_size(term)?;
                last_cell_metrics = metrics;
                renderer.viewport_bus.publish(area);

                if cfg.sgr_pixel_mouse {
                    #[cfg(feature = "blitz")]
                    if cfg.rendering_mode == RenderingMode::BlitzTerminal {
                        // BlitzTerminal sets pixel viewport based on the render surface.
                        // Keep the existing value to avoid conflicting sizes.
                    } else {
                        let ScreenSize { xpixel, ypixel, .. } = term.terminal().get_screen_size()?;
                        if xpixel > 0 && ypixel > 0 {
                            let pixel_w = (xpixel.min(u16::MAX as usize)) as u16;
                            let pixel_h = (ypixel.min(u16::MAX as usize)) as u16;
                            last_pixel_viewport = Some(Rect::new(0, 0, pixel_w, pixel_h));
                        } else {
                            last_pixel_viewport = None;
                        }
                    }
                    #[cfg(not(feature = "blitz"))]
                    {
                        let ScreenSize { xpixel, ypixel, .. } = term.terminal().get_screen_size()?;
                        if xpixel > 0 && ypixel > 0 {
                            let pixel_w = (xpixel.min(u16::MAX as usize)) as u16;
                            let pixel_h = (ypixel.min(u16::MAX as usize)) as u16;
                            last_pixel_viewport = Some(Rect::new(0, 0, pixel_w, pixel_h));
                        } else {
                            last_pixel_viewport = None;
                        }
                    }
                }

                #[cfg(feature = "blitz")]
                if cfg.rendering_mode == RenderingMode::BlitzTerminal {
                    let ScreenSize {
                        cols,
                        rows,
                        xpixel,
                        ypixel,
                    } = term.terminal().get_screen_size()?;
                    if cols == 0 || rows == 0 {
                        return Err(crate::error::Error::Other(anyhow::anyhow!(
                            "terminal reported zero-sized cell grid"
                        )));
                    }
                    if xpixel == 0 || ypixel == 0 {
                        return Err(crate::error::Error::Other(anyhow::anyhow!(
                            "terminal did not report pixel dimensions (xpixel/ypixel=0); cannot use BlitzTerminal"
                        )));
                    }

                    let _cell_w_px = (xpixel as f32) / (cols as f32);
                    let cell_h_px = (ypixel as f32) / (rows as f32);
                    let supersample = cfg.blitz_hidpi_scale.max(1) as f32;
                    last_pixel_scale = supersample;
                    let render_w_px = ((xpixel as f32) * supersample).ceil().max(1.0) as u32;
                    let render_h_px = ((ypixel as f32) * supersample).ceil().max(1.0) as u32;

                    let pixel_w = (xpixel.min(u16::MAX as usize)) as u16;
                    let pixel_h = (ypixel.min(u16::MAX as usize)) as u16;
                    last_pixel_viewport = Some(Rect::new(0, 0, pixel_w, pixel_h));

                    let viewport = Viewport::new(render_w_px, render_h_px, supersample, ColorScheme::Light);
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

                    // In launch mode, avoid expensive per-frame cropping. Render the full viewport.
                    let cropped_cells = area.height;
                    let cropped_px = render_h_px;

                    let mut image_renderer = <anyrender_vello_cpu::VelloCpuImageRenderer as ImageRenderer>::new(
                        render_w_px,
                        render_h_px,
                    );
                    let mut rgba = Vec::new();
                    image_renderer.render_to_vec(
                        |scene| {
                            paint_scene(
                                scene,
                                renderer.doc.inner.as_ref(),
                                viewport.scale_f64(),
                                render_w_px,
                                render_h_px,
                            );
                        },
                        &mut rgba,
                    );
                    if cropped_px < render_h_px {
                        let bytes_per_row = (render_w_px as usize) * 4;
                        let keep = (cropped_px as usize) * bytes_per_row;
                        if keep < rgba.len() {
                            rgba.truncate(keep);
                        }
                    }

                    if matches!(capabilities.inline_protocol, InlineImageProtocol::None) {
                        return Err(crate::error::Error::Other(anyhow::anyhow!(
                            "BlitzTerminal launch currently requires inline image protocol support"
                        )));
                    }
                    let png = crate::image::rgba_to_png_bytes(&rgba, render_w_px, cropped_px.min(render_h_px))?;
                    let encoder = inline_encoder_for_caps(&capabilities)
                        .unwrap_or(rasteroid::InlineEncoder::Ascii);
                    let mut payload = Vec::new();
                    rasteroid::inline_an_image(&png, &mut payload, None, Some((0, 0)), &encoder)
                        .map_err(|err| {
                            crate::error::Error::Other(anyhow::anyhow!("inline image error: {err}"))
                        })?;
                    term.add_change(Change::ClearScreen(ColorAttribute::Default));
                    term.add_change(Change::CursorPosition {
                        x: Position::Absolute(0),
                        y: Position::Absolute(0),
                    });
                    term.add_change(Change::Text(String::from_utf8_lossy(&payload).to_string()));
                    term.add_change(Change::CursorPosition {
                        x: Position::Absolute(0),
                        y: Position::Absolute((cropped_cells as usize).min(area.height as usize - 1)),
                    });
                    term.flush()?;

                    continue;
                }

                let mut surface = Surface::new(area.width, area.height);
                let mut images = std::collections::VecDeque::<PlacedImage>::new();
                last_area = Some(area);
                if let Some(_root) = renderer.layout_root(area, metrics) {
                    if let Err(err) = paint_surface(
                        &mut surface,
                        &mut images,
                        renderer.doc.inner.as_ref(),
                        area,
                        metrics,
                        cfg.palette_roles,
                        cfg.color_mode,
                        capabilities.truecolor,
                        cfg.custom_draw_mode,
                        cfg.image_policy,
                        cfg.image_downgrade,
                        capabilities.inline_images,
                    ) {
                        paint_error = Some(err);
                        break;
                    }
                }
                // Debug: dump first few lines of the surface for tracing.
                if cfg!(debug_assertions) {
                    let width = surface.width() as usize;
                    let dump: Vec<String> = surface
                        .content
                        .chunks(width)
                        .take(5)
                        .map(|row| row.iter().map(|c| c.ch).collect())
                        .collect();
                    debug!(?dump, "surface_dump");
                }
                #[cfg(feature = "blitz")]
                let is_blitz_gui = cfg.rendering_mode == RenderingMode::BlitzGui;
                #[cfg(not(feature = "blitz"))]
                let is_blitz_gui = false;

                if !is_blitz_gui {
                    let cursor_snapshot = cursor_state.borrow().clone();
                    apply_cursor_overlay(
                        &mut surface,
                        &cursor_snapshot,
                        cfg,
                        &capabilities,
                        metrics,
                    );
                }
                flush_surface(
                    term,
                    &surface,
                    last_surface.as_ref(),
                    &capabilities,
                    &images,
                    last_images.as_ref(),
                    metrics,
                )?;
                apply_caret(term, &caret_state.borrow());
                last_surface = Some(surface);
                last_images = Some(images);
            }
        }

        Ok(())
    })
    .await;

    let cleanup_result = (|| -> Result<()> {
        if let Some(term) = &mut terminal {
            if pixel_mouse_enabled {
                set_sgr_pixel_mouse(term, false)?;
            }
            term.add_change(Change::CursorVisibility(termwiz::surface::CursorVisibility::Visible));
            term.flush()?;
            term.terminal().exit_alternate_screen()?;
            term.terminal().set_cooked_mode()?;
            term.flush()?;
        }
        Ok(())
    })();

    if let Some(err) = paint_error {
        return Err(err);
    }

    match result {
        Err(err) => Err(err),
        Ok(()) => cleanup_result,
    }
}

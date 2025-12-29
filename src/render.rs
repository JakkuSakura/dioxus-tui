use std::{any::Any, rc::Rc};

use crate::error::Result;
use blitz_dom::Document as _;
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus_core::{ComponentFunction, ElementId, Event, Runtime, VirtualDom};
use dioxus_html::PlatformEventData;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use termwiz::{
    color::{ColorAttribute, SrgbaTuple},
    surface::{Change, Position},
    terminal::{buffered::BufferedTerminal, ScreenSize, Terminal},
};
use termwiz::terminal::new_terminal;

use crate::capabilities::{DetectedCapabilities, InlineImageProtocol, TerminalCapabilities};
use crate::capabilities::detect as detect_capabilities;
use crate::capabilities::termwiz_capabilities;
use crate::config::{ColorMode, Config, PaletteEntry};
use crate::geometry::Rect;
use crate::hooks::{
    CaretBus, CursorBus, CursorStyle, CursorUnit, CursorState, LayoutBus, TuiInputBus, ViewportBus,
};
use crate::layout::resolve_document;
use crate::scene::CellMetrics;
use crate::surface::Surface;
use crate::RawVirtualDom;
use crate::cell_render::paint_surface;
use crate::image::PlacedImage;
use std::collections::HashMap;

pub fn channel() -> (UnboundedSender<InputEvent>, UnboundedReceiver<InputEvent>) {
    unbounded()
}

pub(crate) struct RenderedFrame {
    pub(crate) surface: Surface,
    pub(crate) images: std::collections::VecDeque<PlacedImage>,
}

#[derive(Clone)]
pub struct TuiContext {
    tx: UnboundedSender<InputEvent>,
}

impl TuiContext {
    pub fn new(tx: UnboundedSender<InputEvent>) -> Self {
        Self { tx }
    }

    pub fn quit(&self) {
        let _ = self.tx.unbounded_send(InputEvent::Close);
    }

    pub fn inject_event(&self, event: termwiz::input::InputEvent) {
        let _ = self.tx.unbounded_send(InputEvent::UserInput(event));
    }
}

#[derive(Debug)]
pub enum InputEvent {
    UserInput(termwiz::input::InputEvent),
    Close,
}

pub(crate) struct DioxusRenderer {
    pub(crate) doc: DioxusDocument,
    pub(crate) input_bus: TuiInputBus,
    pub(crate) viewport_bus: ViewportBus,
    pub(crate) cursor_bus: CursorBus,
    pub(crate) caret_bus: CaretBus,
    pub(crate) layout_bus: LayoutBus,
    pub(crate) runtime: std::rc::Rc<Runtime>,
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    pub(crate) hot_reload_rx: tokio::sync::mpsc::UnboundedReceiver<dioxus_hot_reload::HotReloadMsg>,
}

impl DioxusRenderer {
    pub fn new(
        vdom: VirtualDom,
    ) -> (
        Self,
        UnboundedSender<InputEvent>,
        UnboundedReceiver<InputEvent>,
    ) {
        let viewport = initial_viewport_size().map(|(w_cells, h_cells)| {
            Viewport::new(
                (w_cells as f32 * 8.0).ceil().max(1.0) as u32,
                (h_cells as f32 * 16.0).ceil().max(1.0) as u32,
                1.0,
                ColorScheme::Light,
            )
        });
        Self::new_inner(vdom, viewport)
    }

    pub fn new_with_viewport(
        vdom: VirtualDom,
        viewport: Viewport,
    ) -> (
        Self,
        UnboundedSender<InputEvent>,
        UnboundedReceiver<InputEvent>,
    ) {
        Self::new_inner(vdom, Some(viewport))
    }

    fn new_inner(
        vdom: VirtualDom,
        viewport: Option<Viewport>,
    ) -> (
        Self,
        UnboundedSender<InputEvent>,
        UnboundedReceiver<InputEvent>,
    ) {
        let (event_tx, event_rx) = channel();
        let ctx = TuiContext::new(event_tx.clone());
        let input_bus = TuiInputBus::new();
        let viewport_bus = ViewportBus::new();
        let cursor_bus = CursorBus::new();
        let caret_bus = CaretBus::new();
        let layout_bus = LayoutBus::new();
        let vdom = vdom
            .with_root_context(ctx)
            .with_root_context(input_bus.clone())
            .with_root_context(viewport_bus.clone())
            .with_root_context(cursor_bus.clone())
            .with_root_context(caret_bus.clone())
            .with_root_context(layout_bus.clone());

        let mut doc = Self::build_document(vdom, viewport);
        doc.initial_build();
        let runtime = doc.vdom.runtime();

        (
            Self {
                doc,
                input_bus,
                viewport_bus,
                cursor_bus,
                caret_bus,
                layout_bus,
                runtime,
                #[cfg(all(feature = "hot-reload", debug_assertions))]
                hot_reload_rx: {
                    let (hot_reload_tx, hot_reload_rx) =
                        tokio::sync::mpsc::unbounded_channel::<dioxus_hot_reload::HotReloadMsg>();
                    dioxus_hot_reload::connect(move |msg| {
                        let _ = hot_reload_tx.send(msg);
                    });
                    hot_reload_rx
                },
            },
            event_tx,
            event_rx,
        )
    }

    fn build_document(vdom: VirtualDom, viewport: Option<Viewport>) -> DioxusDocument {
        DioxusDocument::new(
            vdom,
            DocumentConfig {
                viewport,
                ..Default::default()
            },
        )
    }

    pub(crate) fn update(&mut self) {
        while self.doc.poll(None) {}
    }

    pub(crate) fn handle_event(
        &mut self,
        id: ElementId,
        event: &str,
        value: Box<dyn Any>,
        bubbles: bool,
    ) {
        let platform_event = Rc::new(PlatformEventData::new(value));
        let runtime_event = Event::new(platform_event, bubbles).into_any();
        self.doc
            .vdom
            .runtime()
            .handle_event(event, runtime_event, id);
    }


    pub(crate) fn layout_root(&mut self, area: Rect, metrics: CellMetrics) -> Option<usize> {
        resolve_document(&mut self.doc, area, metrics)
    }

    pub(crate) fn publish_layout_rects(&self, area: Rect, metrics: CellMetrics) {
        let mut rects = HashMap::new();
        for scope_id in self.layout_bus.registered_scopes() {
            let Some(scope) = self.doc.vdom.get_scope(scope_id) else {
                continue;
            };
            let Some(vnode) = scope.try_root_node() else {
                continue;
            };
            let Some(element_id) = vnode.mounted_root(0, &self.doc.vdom) else {
                continue;
            };
            let Some(node_id) = self.doc.vdom_state.try_element_to_node_id(element_id) else {
                continue;
            };
            let Some(node) = self.doc.inner.get_node(node_id) else {
                continue;
            };
            let rect = crate::layout::node_rect(self.doc.inner.as_ref(), node, area, metrics);
            rects.insert(scope_id, rect);
        }
        self.layout_bus.publish(rects);
    }
}

pub fn render_once<P, F>(cfg: Config, raw: RawVirtualDom<P, F>, area: Rect) -> Result<Surface>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    Ok(render_once_frame(cfg, raw, area)?.surface)
}

pub(crate) fn render_once_frame<P, F>(
    cfg: Config,
    raw: RawVirtualDom<P, F>,
    area: Rect,
) -> Result<RenderedFrame>
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
    let metrics = CellMetrics {
        cell_w_px: 8.0,
        cell_h_px: 16.0,
    };
    let vdom = raw.into_virtual_dom();

    let viewport = Viewport::new(
        (area.width as f32 * metrics.cell_w_px).ceil().max(1.0) as u32,
        (area.height as f32 * metrics.cell_h_px).ceil().max(1.0) as u32,
        1.0,
        ColorScheme::Light,
    );
    let (mut renderer, _event_tx, _event_rx) = DioxusRenderer::new_with_viewport(vdom, viewport);

    renderer.update();
    renderer.viewport_bus.publish(area);
    renderer.update();
    let mut surface = Surface::new(area.width, area.height);
    let mut images = std::collections::VecDeque::<PlacedImage>::new();
    let _ = renderer.layout_root(area, metrics);
    renderer.publish_layout_rects(area, metrics);
    paint_surface(
        &mut surface,
        &mut images,
        renderer.doc.inner.as_ref(),
        area,
        metrics,
        cfg.palette_roles,
        cfg.color_mode,
        detected.terminal.truecolor,
        cfg.custom_draw_mode,
        cfg.image_policy,
        cfg.image_downgrade,
        detected.terminal.inline_images,
    )?;
    Ok(RenderedFrame { surface, images })
}

pub(crate) fn debug_layout<P, F>(cfg: Config, raw: RawVirtualDom<P, F>, area: Rect) -> Result<()>
where
    P: Clone + 'static,
    F: ComponentFunction<P, ()> + 'static,
{
    let _ = cfg;
    let metrics = CellMetrics {
        cell_w_px: 8.0,
        cell_h_px: 16.0,
    };
    let vdom = raw.into_virtual_dom();

    let viewport = Viewport::new(
        (area.width as f32 * metrics.cell_w_px).ceil().max(1.0) as u32,
        (area.height as f32 * metrics.cell_h_px).ceil().max(1.0) as u32,
        1.0,
        ColorScheme::Light,
    );
    let (mut renderer, _event_tx, _event_rx) = DioxusRenderer::new_with_viewport(vdom, viewport);
    renderer.update();
    let _ = renderer.layout_root(area, metrics);

    println!("-- dioxus-tui layout debug --");
    let root_id = renderer.doc.inner.root_node().id;
    crate::layout::print_layout(renderer.doc.inner.as_ref(), root_id, 0, area, metrics);
    Ok(())
}

#[derive(Clone)]
pub(crate) struct CaretState {
    pub(crate) visible: bool,
    pub(crate) position: Option<(u16, u16)>,
}

impl Default for CaretState {
    fn default() -> Self {
        Self {
            visible: false,
            position: None,
        }
    }
}

pub(crate) fn apply_cursor_overlay(
    surface: &mut Surface,
    cursor: &CursorState,
    cfg: Config,
    capabilities: &TerminalCapabilities,
    cell_metrics: CellMetrics,
) {
    if !cursor.visible {
        return;
    }
    let Some((x, y)) = cursor.position else {
        return;
    };

    let (cell_x, cell_y) = match cursor.unit {
        CursorUnit::Cell => (x.floor(), y.floor()),
        CursorUnit::Pixel => {
            let cell_w = if cell_metrics.cell_w_px > 0.0 {
                cell_metrics.cell_w_px
            } else {
                1.0
            };
            let cell_h = if cell_metrics.cell_h_px > 0.0 {
                cell_metrics.cell_h_px
            } else {
                1.0
            };
            ((x / cell_w).floor(), (y / cell_h).floor())
        }
    };

    if cell_x < 0.0 || cell_y < 0.0 {
        return;
    }
    let cell_x = cell_x as u16;
    let cell_y = cell_y as u16;
    if cell_x >= surface.width() || cell_y >= surface.height() {
        return;
    }

    let idx = (cell_y as usize) * (surface.width() as usize) + (cell_x as usize);
    let Some(cell) = surface.content.get_mut(idx) else {
        return;
    };

    let accent = palette_entry_to_attr(cfg.palette_roles.accent, cfg.color_mode, capabilities.truecolor);
    let style = if cursor.unit == CursorUnit::Pixel && cursor.style == CursorStyle::Block {
        CursorStyle::Crosshair
    } else {
        cursor.style
    };

    match style {
        CursorStyle::Block => {
            cell.ch = ' ';
            cell.bg = Some(accent);
            cell.fg = None;
        }
        CursorStyle::Underline => {
            cell.underline = termwiz::cell::Underline::Single;
            cell.fg = Some(accent);
        }
        CursorStyle::Beam => {
            cell.ch = '▏';
            cell.fg = Some(accent);
            cell.bg = None;
        }
        CursorStyle::Crosshair => {
            cell.ch = '+';
            cell.fg = Some(accent);
            cell.bg = None;
        }
    }
}

pub(crate) fn apply_caret<T: Terminal>(
    term: &mut BufferedTerminal<T>,
    cursor: &CaretState,
) {
    let visibility = if cursor.visible {
        termwiz::surface::CursorVisibility::Visible
    } else {
        termwiz::surface::CursorVisibility::Hidden
    };
    term.add_change(Change::CursorVisibility(visibility));
    if let Some((x, y)) = cursor.position {
        term.add_change(Change::CursorPosition {
            x: Position::Absolute(x as usize),
            y: Position::Absolute(y as usize),
        });
    }
    let _ = term.flush();
}

fn palette_entry_to_attr(entry: PaletteEntry, color_mode: ColorMode, truecolor: bool) -> ColorAttribute {
    match entry {
        PaletteEntry::Ansi(idx) | PaletteEntry::Palette256(idx) => ColorAttribute::PaletteIndex(idx),
        PaletteEntry::Rgb(r, g, b) => {
            let srgb = SrgbaTuple::from((r, g, b));
            let palette_idx_256 =
                16 + 36 * (r as u16 / 51) as u8 + 6 * (g as u16 / 51) as u8 + (b as u16 / 51) as u8;
            let base_idx = (if r >= 128 { 1 } else { 0 })
                | (if g >= 128 { 2 } else { 0 })
                | (if b >= 128 { 4 } else { 0 });
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
    }
}

pub(crate) fn terminal_size<T: Terminal>(
    term: &mut BufferedTerminal<T>,
) -> Result<(Rect, CellMetrics)> {
    let ScreenSize {
        cols,
        rows,
        xpixel,
        ypixel,
    } = term.terminal().get_screen_size()?;
    let (cell_w_px, cell_h_px) = if cols > 0 && rows > 0 && xpixel > 0 && ypixel > 0 {
        (
            (xpixel as f32) / (cols as f32),
            (ypixel as f32) / (rows as f32),
        )
    } else {
        (8.0, 16.0)
    };
    Ok((
        Rect::new(0, 0, cols as u16, rows as u16),
        CellMetrics {
            cell_w_px,
            cell_h_px,
        },
    ))
}

pub(crate) fn set_sgr_pixel_mouse<T: Terminal>(
    term: &mut BufferedTerminal<T>,
    enabled: bool,
) -> Result<()> {
    let suffix = if enabled { "h" } else { "l" };
    term.add_change(Change::Text(format!("\x1b[?1016{suffix}")));
    term.flush()?;
    Ok(())
}

fn initial_viewport_size() -> Option<(u16, u16)> {
    let caps = detect_capabilities().ok()?;
    let mut term = new_terminal(caps.termwiz).ok()?;
    term.get_screen_size()
        .ok()
        .map(|s| (s.cols as u16, s.rows as u16))
}

pub(crate) fn surface_to_changes(
    surface: &Surface,
    prev: Option<&Surface>,
    masked_intervals_by_row: Option<&[Vec<(usize, usize)>]>,
) -> Vec<Change> {
    let full_redraw = prev.map(|p| p.dims() != surface.dims()).unwrap_or(true);
    let mut changes = Vec::new();

    if full_redraw {
        changes.push(Change::ClearScreen(ColorAttribute::Default));
    }

    // dirty lines: compare line-by-line, emit minimal cursor moves; future: extend to rect diff
    let width = surface.width() as usize;
    for (y, chunk) in surface.content.chunks(width).enumerate() {
        let is_dirty = if full_redraw {
            true
        } else if let Some(prev_surface) = prev {
            let prev_width = prev_surface.width() as usize;
            if prev_width != width || y >= prev_surface.height() as usize {
                true
            } else {
                let start = y * prev_width;
                let end = start + prev_width;
                &prev_surface.content[start..end] != chunk
            }
        } else {
            true
        };

        if !is_dirty {
            continue;
        }

        let mut current_fg = ColorAttribute::Default;
        let mut current_bg = ColorAttribute::Default;
        let mut current_intensity = termwiz::cell::Intensity::Normal;
        let mut current_underline = termwiz::cell::Underline::None;
        let mut current_italic = false;
        let mut current_blink = termwiz::cell::Blink::None;
        let mut buf = String::with_capacity(chunk.len());

        changes.push(Change::CursorPosition {
            x: Position::Absolute(0),
            y: Position::Absolute(y),
        });

        let intervals = masked_intervals_by_row
            .and_then(|rows| rows.get(y))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let mut interval_idx = 0usize;
        let mut masked_end: Option<usize> = None;

        let mut x = 0usize;
        while x < chunk.len() {
            if masked_end.is_none() {
                if let Some((start, end)) = intervals.get(interval_idx).copied() {
                    if x == start {
                        masked_end = Some(end.min(chunk.len()));
                    }
                }
            }

            let is_masked = masked_end.is_some_and(|end| x < end);
            if masked_end.is_some_and(|end| x == end) {
                masked_end = None;
                interval_idx += 1;
            }

            let cell = &chunk[x];
            // In masked regions (inline images), we still paint the cell background so that
            // letterboxing uses the panel background instead of the terminal default.
            // We force a blank glyph so we never overwrite the image with text.
            let fg = if is_masked {
                ColorAttribute::Default
            } else {
                cell.fg.unwrap_or(ColorAttribute::Default)
            };
            let bg = cell.bg.unwrap_or(ColorAttribute::Default);
            let intensity = cell.intensity;
            let underline = cell.underline;
            let italic = cell.italic;
            let blink = cell.blink;

            if fg != current_fg
                || bg != current_bg
                || intensity != current_intensity
                || underline != current_underline
                || italic != current_italic
                || blink != current_blink
            {
                if !buf.is_empty() {
                    changes.push(Change::Text(std::mem::take(&mut buf)));
                }
                if bg != current_bg {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Background(
                        bg,
                    )));
                    current_bg = bg;
                }
                if fg != current_fg {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Foreground(
                        fg,
                    )));
                    current_fg = fg;
                }
                if intensity != current_intensity {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Intensity(
                        intensity,
                    )));
                    current_intensity = intensity;
                }
                if underline != current_underline {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Underline(
                        underline,
                    )));
                    current_underline = underline;
                }
                if italic != current_italic {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Italic(italic)));
                    current_italic = italic;
                }
                if blink != current_blink {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Blink(blink)));
                    current_blink = blink;
                }
            }

            buf.push(if is_masked { ' ' } else { cell.ch });
            x += 1;
        }

        if !buf.is_empty() {
            changes.push(Change::Text(buf));
        }

        if current_fg != ColorAttribute::Default {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Foreground(
                ColorAttribute::Default,
            )));
        }
        if current_bg != ColorAttribute::Default {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Background(
                ColorAttribute::Default,
            )));
        }
        if current_intensity != termwiz::cell::Intensity::Normal {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Intensity(
                termwiz::cell::Intensity::Normal,
            )));
        }
        if current_underline != termwiz::cell::Underline::None {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Underline(
                termwiz::cell::Underline::None,
            )));
        }
        if current_italic {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Italic(false)));
        }
        if current_blink != termwiz::cell::Blink::None {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Blink(
                termwiz::cell::Blink::None,
            )));
        }
    }

    changes
}

fn image_mask_intervals_by_row(
    images: &std::collections::VecDeque<PlacedImage>,
    surface_width: usize,
    surface_height: usize,
) -> Vec<Vec<(usize, usize)>> {
    let mut rows: Vec<Vec<(usize, usize)>> = vec![Vec::new(); surface_height];
    for img in images {
        let x0 = (img.x_cell as usize).min(surface_width);
        let x1 = (img.x_cell as usize)
            .saturating_add(img.width_cells as usize)
            .min(surface_width);
        if x0 >= x1 {
            continue;
        }

        let y0 = (img.y_cell as usize).min(surface_height);
        let y1 = (img.y_cell as usize)
            .saturating_add(img.height_cells as usize)
            .min(surface_height);
        for y in y0..y1 {
            rows[y].push((x0, x1));
        }
    }

    for row in &mut rows {
        if row.len() <= 1 {
            continue;
        }
        row.sort_by_key(|(s, _e)| *s);
        let mut merged: Vec<(usize, usize)> = Vec::with_capacity(row.len());
        for (s, e) in row.drain(..) {
            if let Some((_ms, me)) = merged.last_mut() {
                if s <= *me {
                    *me = (*me).max(e);
                    continue;
                }
            }
            merged.push((s, e));
        }
        *row = merged;
    }

    rows
}

pub(crate) fn surface_to_cropped_stream_changes(surface: &Surface) -> Vec<Change> {
    let width = surface.width() as usize;
    let height = surface
        .content
        .len()
        .checked_div(width)
        .unwrap_or(0);
    if width == 0 || height == 0 {
        return Vec::new();
    }

    let Some(bottom_row) = (0..height)
        .rev()
        .find(|&y| row_has_non_blank(&surface.content[y * width..(y + 1) * width]))
    else {
        return Vec::new();
    };

    let mut changes = Vec::new();
    // `render()` streams to stdout and must not assume the current terminal state.
    // Terminals are sticky: if the previous output set attributes, we must reset
    // them up-front to get predictable output.
    push_reset_attributes(&mut changes);
    // Force termwiz to flush the reset immediately; it buffers attribute changes
    // until it sees `Change::Text`.
    changes.push(Change::Text(String::new()));
    for y in 0..=bottom_row {
        let row = &surface.content[y * width..(y + 1) * width];

        let right_col = match (0..width).rev().find(|&x| !row[x].is_blank()) {
            Some(col) => col,
            None => {
                // Reset attributes so empty rows don't inherit styling.
                push_reset_attributes(&mut changes);
                // Always advance to a fresh line for stream output.
                changes.push(Change::Text("\r\n".to_string()));
                continue;
            }
        };

        let mut current_fg = ColorAttribute::Default;
        let mut current_bg = ColorAttribute::Default;
        let mut current_intensity = termwiz::cell::Intensity::Normal;
        let mut current_underline = termwiz::cell::Underline::None;
        let mut current_italic = false;
        let mut current_blink = termwiz::cell::Blink::None;
        let mut buf = String::new();

        for cell in row.iter().take(right_col + 1) {
            let fg = cell.fg.unwrap_or(ColorAttribute::Default);
            let bg = cell.bg.unwrap_or(ColorAttribute::Default);
            let intensity = cell.intensity;
            let underline = cell.underline;
            let italic = cell.italic;
            let blink = cell.blink;

            if fg != current_fg
                || bg != current_bg
                || intensity != current_intensity
                || underline != current_underline
                || italic != current_italic
                || blink != current_blink
            {
                if !buf.is_empty() {
                    changes.push(Change::Text(std::mem::take(&mut buf)));
                }
                if bg != current_bg {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Background(bg)));
                    current_bg = bg;
                }
                if fg != current_fg {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Foreground(fg)));
                    current_fg = fg;
                }
                if intensity != current_intensity {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Intensity(
                        intensity,
                    )));
                    current_intensity = intensity;
                }
                if underline != current_underline {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Underline(
                        underline,
                    )));
                    current_underline = underline;
                }
                if italic != current_italic {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Italic(italic)));
                    current_italic = italic;
                }
                if blink != current_blink {
                    changes.push(Change::Attribute(termwiz::cell::AttributeChange::Blink(blink)));
                    current_blink = blink;
                }
            }

            buf.push(cell.ch);
        }

        if !buf.is_empty() {
            changes.push(Change::Text(buf));
        }

        if current_fg != ColorAttribute::Default {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Foreground(
                ColorAttribute::Default,
            )));
        }
        if current_bg != ColorAttribute::Default {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Background(
                ColorAttribute::Default,
            )));
        }
        if current_intensity != termwiz::cell::Intensity::Normal {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Intensity(
                termwiz::cell::Intensity::Normal,
            )));
        }
        if current_underline != termwiz::cell::Underline::None {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Underline(
                termwiz::cell::Underline::None,
            )));
        }
        if current_italic {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Italic(false)));
        }
        if current_blink != termwiz::cell::Blink::None {
            changes.push(Change::Attribute(termwiz::cell::AttributeChange::Blink(
                termwiz::cell::Blink::None,
            )));
        }

        // Ensure each row ends with default attributes and a newline.
        push_reset_attributes(&mut changes);
        changes.push(Change::Text("\r\n".to_string()));
    }

    changes
}

pub(crate) fn frame_to_cropped_stream_changes(
    frame: &RenderedFrame,
    caps: &TerminalCapabilities,
) -> Vec<Change> {
    // First, stream the full surface (including background for the image rect).
    // Then, place images at the end so that no subsequent output overwrites them.
    let mut changes = surface_to_cropped_stream_changes(&frame.surface);

    if frame.images.is_empty() || matches!(caps.inline_protocol, InlineImageProtocol::None) {
        return changes;
    }

    let width = frame.surface.width() as usize;
    let height = frame
        .surface
        .content
        .len()
        .checked_div(width)
        .unwrap_or(0);
    if width == 0 || height == 0 {
        return changes;
    }

    let Some(bottom_row) = (0..height)
        .rev()
        .find(|&y| row_has_non_blank(&frame.surface.content[y * width..(y + 1) * width]))
    else {
        return changes;
    };
    let printed_rows = bottom_row + 1;

    for img in &frame.images {
        let y = img.y_cell as usize;
        if y >= printed_rows {
            continue;
        }
        let up = printed_rows.saturating_sub(y);

        let encoder = inline_encoder_for_caps(caps).unwrap_or(rasteroid::InlineEncoder::Ascii);
        let mut payload = Vec::new();
        if rasteroid::inline_an_image(&img.png, &mut payload, None, Some((img.x_cell, img.y_cell)), &encoder).is_ok() {
            // Move to the image cell position relative to the end of output.
            changes.push(Change::Text(format!("\x1b[{}A\r", up)));
            changes.push(Change::Text(String::from_utf8_lossy(&payload).to_string()));
            // Return back to the end of output.
            changes.push(Change::Text(format!("\r\x1b[{}B", up)));
        }
    }

    changes
}

fn push_reset_attributes(changes: &mut Vec<Change>) {
    changes.push(Change::AllAttributes(termwiz::cell::CellAttributes::default()));
}

fn row_has_non_blank(row: &[crate::surface::Cell]) -> bool {
    row.iter().any(crate::surface::Cell::has_visible_content)
}

pub(crate) fn flush_surface<T: Terminal>(
    term: &mut BufferedTerminal<T>,
    surface: &Surface,
    prev: Option<&Surface>,
    caps: &TerminalCapabilities,
    images: &std::collections::VecDeque<PlacedImage>,
    prev_images: Option<&std::collections::VecDeque<PlacedImage>>,
    metrics: CellMetrics,
) -> Result<()> {
    let images_changed = prev_images.map(|p| p != images).unwrap_or(!images.is_empty());

    // If the image set changes, do a full redraw to avoid leaving stale image placements behind.
    let full_redraw = images_changed || prev.map(|p| p.dims() != surface.dims()).unwrap_or(true);

    let masked = if !matches!(caps.inline_protocol, InlineImageProtocol::None) && !images.is_empty() {
        Some(image_mask_intervals_by_row(
            images,
            surface.width() as usize,
            surface.height() as usize,
        ))
    } else {
        None
    };

    let changes = surface_to_changes(
        surface,
        if full_redraw { None } else { prev },
        masked.as_deref(),
    );

    for change in changes {
        term.add_change(change);
    }

    // Place images after text so they don't get overwritten by cleared cell regions.
    if let Some(encoder) = inline_encoder_for_caps(caps) {
        for img in images {
            let mut payload = Vec::new();
            if rasteroid::inline_an_image(&img.png, &mut payload, None, Some((img.x_cell, img.y_cell)), &encoder).is_ok() {
                term.add_change(Change::Text(String::from_utf8_lossy(&payload).to_string()));
            }
        }
    }

    let _ = (metrics,);
    term.flush()?;
    Ok(())
}

pub(crate) fn inline_encoder_for_caps(
    caps: &TerminalCapabilities,
) -> Option<rasteroid::InlineEncoder> {
    match caps.inline_protocol {
        InlineImageProtocol::Iterm2 => Some(rasteroid::InlineEncoder::Iterm),
        InlineImageProtocol::Sixel => Some(rasteroid::InlineEncoder::Sixel),
        InlineImageProtocol::None => None,
    }
}

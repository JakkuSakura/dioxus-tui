use blitz_dom::{local_name, BaseDocument, Node};
use termwiz::cell::{Blink, Intensity, Underline};
use termwiz::color::{ColorAttribute, SrgbaTuple};

use crate::config::{PaletteEntry, PaletteRoles};
use crate::config::ColorMode;
use crate::geometry::Rect;
use crate::layout::node_rect;
use crate::scene::CellMetrics;
use crate::surface::Surface;
use crate::config::{ImageDowngrade, ImagePolicy};
use crate::image::{load_png_image, placed_image_from_png, PlacedImage};
use std::collections::VecDeque;
use style::color::AbsoluteColor;
use unicode_width::UnicodeWidthChar;

pub fn paint_surface(
    surface: &mut Surface,
    images: &mut VecDeque<PlacedImage>,
    doc: &BaseDocument,
    area: Rect,
    metrics: CellMetrics,
    palette_roles: PaletteRoles,
    color_mode: ColorMode,
    truecolor: bool,
    draw_mode: crate::draw::CustomDrawMode,
    image_policy: ImagePolicy,
    image_downgrade: ImageDowngrade,
    inline_images_supported: bool,
) -> crate::error::Result<()> {
    surface.clear();
    images.clear();

    let fallback_fg = palette_entry_to_attr(palette_roles.fg_primary, color_mode, truecolor);

    let root = doc.root_element();
    if let Some(bg) = root_background(doc, root, color_mode, truecolor) {
        fill_rect(surface, surface.area(), None, Some(bg));
    }

    paint_node(
        surface,
        images,
        doc,
        root,
        area,
        metrics,
        color_mode,
        truecolor,
        fallback_fg,
        TextStyle::default(),
        draw_mode,
        palette_roles,
        image_policy,
        image_downgrade,
        inline_images_supported,
    )?;

    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct TextStyle {
    fg: Option<ColorAttribute>,
    bg: Option<ColorAttribute>,
    intensity: Intensity,
    underline: Underline,
    italic: bool,
    blink: Blink,
    preserve_whitespace: bool,
    pre_full_width: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            intensity: Intensity::Normal,
            underline: Underline::None,
            italic: false,
            blink: Blink::None,
            preserve_whitespace: false,
            pre_full_width: false,
        }
    }
}

impl TextStyle {
    fn merged(self, other: TextStyle) -> TextStyle {
        TextStyle {
            fg: other.fg.or(self.fg),
            bg: other.bg.or(self.bg),
            intensity: if other.intensity != Intensity::Normal {
                other.intensity
            } else {
                self.intensity
            },
            underline: if other.underline != Underline::None {
                other.underline
            } else {
                self.underline
            },
            italic: self.italic || other.italic,
            blink: if other.blink != Blink::None { other.blink } else { self.blink },
            preserve_whitespace: self.preserve_whitespace || other.preserve_whitespace,
            pre_full_width: self.pre_full_width || other.pre_full_width,
        }
    }
}

fn paint_node(
    surface: &mut Surface,
    images: &mut VecDeque<PlacedImage>,
    doc: &BaseDocument,
    node: &Node,
    area: Rect,
    metrics: CellMetrics,
    color_mode: ColorMode,
    truecolor: bool,
    fallback_fg: ColorAttribute,
    inherited: TextStyle,
    draw_mode: crate::draw::CustomDrawMode,
    palette_roles: PaletteRoles,
    image_policy: ImagePolicy,
    image_downgrade: ImageDowngrade,
    inline_images_supported: bool,
) -> crate::error::Result<()> {
    match &node.data {
        blitz_dom::node::NodeData::Element(_) | blitz_dom::node::NodeData::AnonymousBlock(_) => {
            let local_style = style_overrides(node, color_mode, truecolor);
            let node_style = inherited.merged(local_style);

            if draw_mode == crate::draw::CustomDrawMode::Native {
                if let Some(draw_id) = attr_value(node, "data-draw-id") {
                    if let Some(cb) = crate::draw::lookup_draw(draw_id) {
                        let rect = node_rect(doc, node, area, metrics);
                        if rect.width > 0 && rect.height > 0 {
                            let mut ctx = crate::draw::DrawContext {
                                surface,
                                rect,
                                color_mode,
                                truecolor,
                                palette_roles,
                            };
                            cb(&mut ctx);
                        }
                        return Ok(());
                    }
                }
            }

            if node.data.is_element_with_tag_name(&local_name!("img")) {
                let rect = node_rect(doc, node, area, metrics);
                paint_img(
                    surface,
                    images,
                    node,
                    rect,
                    metrics,
                    color_mode,
                    truecolor,
                    image_policy,
                    image_downgrade,
                    inline_images_supported,
                )?;
                return Ok(());
            }

            let rect = node_rect(doc, node, area, metrics);
            if rect.width > 0 && rect.height > 0 {
                if let Some(bg) = node_background(node, color_mode, truecolor).or(node_style.bg) {
                    fill_rect(surface, rect, None, Some(bg));
                }
            }

            // Render inline text content within this node's box.
            if rect.width > 0 && rect.height > 0 {
                let fg = Some(
                    node_color(node, color_mode, truecolor)
                        .or(node_style.fg)
                        .unwrap_or(fallback_fg),
                );
                let style = TextStyle { fg, ..node_style };
                // Constrain inline text rendering to this node's box.
                // Otherwise, text may spill into sibling regions and later get overwritten,
                // which looks like truncation/hidden content.
                let text_bounds = rect;

                if node.data.is_element_with_tag_name(&local_name!("input")) {
                    if let Some(value) = node.attr(local_name!("value")) {
                        let _ = write_wrapped(surface, text_bounds, (rect.x, rect.y), value, style);
                    }
                } else if node.data.is_element_with_tag_name(&local_name!("textarea")) {
                    if let Some(value) = node.attr(local_name!("value")) {
                        let _ = write_wrapped(surface, text_bounds, (rect.x, rect.y), value, style);
                    } else {
                        paint_inline_text(
                            surface,
                            doc,
                            node,
                            text_bounds,
                            style,
                            color_mode,
                            truecolor,
                            fallback_fg,
                        );
                    }
                } else if node.data.is_element_with_tag_name(&local_name!("button")) {
                    let label = node.text_content();
                    let _ = write_wrapped(surface, text_bounds, (rect.x, rect.y), label.as_str(), style);
                } else {
                    paint_inline_text(
                        surface,
                        doc,
                        node,
                        text_bounds,
                        style,
                        color_mode,
                        truecolor,
                        fallback_fg,
                    );
                }
            }

            // Render block children as their own boxes.
            for child_id in node.children.iter().copied() {
                let Some(child) = doc.get_node(child_id) else {
                    continue;
                };
                if is_blockish(child) {
                    paint_node(
                        surface,
                        images,
                        doc,
                        child,
                        area,
                        metrics,
                        color_mode,
                        truecolor,
                        fallback_fg,
                        node_style,
                        draw_mode,
                        palette_roles,
                        image_policy,
                        image_downgrade,
                        inline_images_supported,
                    )?;
                }
            }
        }
        blitz_dom::node::NodeData::Text(_) => {}
        blitz_dom::node::NodeData::Document | blitz_dom::node::NodeData::Comment => {
            for child_id in node.children.iter().copied() {
                if let Some(child) = doc.get_node(child_id) {
                    paint_node(
                        surface,
                        images,
                        doc,
                        child,
                        area,
                        metrics,
                        color_mode,
                        truecolor,
                        fallback_fg,
                        inherited,
                        draw_mode,
                        palette_roles,
                        image_policy,
                        image_downgrade,
                        inline_images_supported,
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn paint_inline_text(
    surface: &mut Surface,
    doc: &BaseDocument,
    node: &Node,
    bounds: Rect,
    inherited: TextStyle,
    color_mode: ColorMode,
    truecolor: bool,
    fallback_fg: ColorAttribute,
) {
    let mut cursor_x = bounds.x;
    let mut cursor_y = bounds.y;
    let end_y = if inherited.pre_full_width {
        surface.height()
    } else {
        bounds.y.saturating_add(bounds.height)
    };

    for child_id in node.children.iter().copied() {
        if cursor_y >= end_y {
            break;
        }
        let Some(child) = doc.get_node(child_id) else {
            continue;
        };
        match &child.data {
            blitz_dom::node::NodeData::Text(text) => {
                (cursor_x, cursor_y) = write_wrapped(
                    surface,
                    bounds,
                    (cursor_x, cursor_y),
                    text.content.as_str(),
                    inherited,
                );
            }
            blitz_dom::node::NodeData::Element(_) | blitz_dom::node::NodeData::AnonymousBlock(_) => {
                if !is_blockish(child) {
                    if child.data.is_element_with_tag_name(&local_name!("input")) {
                        if let Some(value) = child.attr(local_name!("value")) {
                            (cursor_x, cursor_y) =
                                write_wrapped(surface, bounds, (cursor_x, cursor_y), value, inherited);
                        }
                        continue;
                    }
                    if child.data.is_element_with_tag_name(&local_name!("textarea")) {
                        if let Some(value) = child.attr(local_name!("value")) {
                            (cursor_x, cursor_y) =
                                write_wrapped(surface, bounds, (cursor_x, cursor_y), value, inherited);
                        }
                        continue;
                    }

                    let local_style = style_overrides(child, color_mode, truecolor);
                    let child_style = inherited.merged(local_style);
                    let child_fg = Some(
                        node_color(child, color_mode, truecolor)
                            .or(child_style.fg)
                            .unwrap_or(fallback_fg),
                    );
                    let child_style = TextStyle { fg: child_fg, ..child_style };
                    (cursor_x, cursor_y) = paint_inline_children(
                        surface,
                        doc,
                        child,
                        bounds,
                        (cursor_x, cursor_y),
                        child_style,
                        color_mode,
                        truecolor,
                        fallback_fg,
                    );
                }
            }
            blitz_dom::node::NodeData::Document | blitz_dom::node::NodeData::Comment => {}
        }
    }
}

fn paint_inline_children(
    surface: &mut Surface,
    doc: &BaseDocument,
    node: &Node,
    bounds: Rect,
    cursor: (u16, u16),
    inherited: TextStyle,
    color_mode: ColorMode,
    truecolor: bool,
    fallback_fg: ColorAttribute,
) -> (u16, u16) {
    let mut cursor = cursor;
    for child_id in node.children.iter().copied() {
        let Some(child) = doc.get_node(child_id) else {
            continue;
        };
        match &child.data {
            blitz_dom::node::NodeData::Text(text) => {
                cursor = write_wrapped(surface, bounds, cursor, text.content.as_str(), inherited);
            }
            blitz_dom::node::NodeData::Element(_) | blitz_dom::node::NodeData::AnonymousBlock(_) => {
                if !is_blockish(child) {
                    if child.data.is_element_with_tag_name(&local_name!("input")) {
                        if let Some(value) = child.attr(local_name!("value")) {
                            cursor = write_wrapped(surface, bounds, cursor, value, inherited);
                        }
                        continue;
                    }
                    if child.data.is_element_with_tag_name(&local_name!("textarea")) {
                        if let Some(value) = child.attr(local_name!("value")) {
                            cursor = write_wrapped(surface, bounds, cursor, value, inherited);
                        }
                        continue;
                    }

                    let local_style = style_overrides(child, color_mode, truecolor);
                    let child_style = inherited.merged(local_style);
                    let child_fg = Some(
                        node_color(child, color_mode, truecolor)
                            .or(child_style.fg)
                            .unwrap_or(fallback_fg),
                    );
                    let child_style = TextStyle { fg: child_fg, ..child_style };
                    cursor = paint_inline_children(
                        surface,
                        doc,
                        child,
                        bounds,
                        cursor,
                        child_style,
                        color_mode,
                        truecolor,
                        fallback_fg,
                    );
                }
            }
            blitz_dom::node::NodeData::Document | blitz_dom::node::NodeData::Comment => {}
        }
    }
    cursor
}

fn is_blockish(node: &Node) -> bool {
    if node.is_or_contains_block() {
        return true;
    }

    let Some(el) = node.element_data() else {
        return false;
    };

    // Fallback classification when computed styles are unavailable.
    // This keeps basic HTML working even if stylo doesn't attach primary styles
    // to a node for any reason.
    matches!(
        el.name.local.as_ref(),
        "html"
            | "body"
            | "main"
            | "div"
            | "p"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "ul"
            | "ol"
            | "li"
            | "pre"
            | "button"
            | "input"
            | "textarea"
            | "select"
            | "form"
            | "section"
            | "header"
            | "footer"
            | "article"
            | "nav"
            | "table"
            | "tr"
            | "td"
            | "th"
            | "thead"
            | "tbody"
            | "tfoot"
            | "hr"
            | "img"
    )
}

fn parse_inline_style_value<'a>(style: &'a str, key: &str) -> Option<&'a str> {
    let key = key.trim().to_ascii_lowercase();
    for decl in style.split(';') {
        let decl = decl.trim();
        if decl.is_empty() {
            continue;
        }
        let mut parts = decl.splitn(2, ':');
        let k = parts.next()?.trim().to_ascii_lowercase();
        let v = parts.next()?.trim();
        if k == key {
            return Some(v);
        }
    }
    None
}

fn paint_img(
    surface: &mut Surface,
    images: &mut VecDeque<PlacedImage>,
    node: &Node,
    rect: Rect,
    metrics: CellMetrics,
    color_mode: ColorMode,
    truecolor: bool,
    image_policy: ImagePolicy,
    image_downgrade: ImageDowngrade,
    inline_images_supported: bool,
) -> crate::error::Result<()> {
    if rect.width == 0 || rect.height == 0 {
        return Ok(());
    }

    if matches!(image_policy, ImagePolicy::Omit) {
        return Ok(());
    }

    let Some(src) = node.attr(local_name!("src")) else {
        return Ok(());
    };

    fn parse_dim_cells(value: &str, cell_px: f32) -> Option<u16> {
        let s = value.trim();
        if let Some(rest) = s.strip_suffix("ch") {
            return rest.trim().parse::<u16>().ok();
        }
        if let Some(rest) = s.strip_suffix("px") {
            let px = rest.trim().parse::<f32>().ok()?;
            let cells = (px / cell_px).ceil().max(1.0) as u16;
            return Some(cells);
        }
        // Best-effort: allow raw integers interpreted as cells.
        s.parse::<u16>().ok()
    }

    // Blitz/Taffy doesn't reliably size replaced elements (`img`).
    // Treat explicit `width`/`height` attributes as authoritative sizing hints
    // for both inline and sampled render paths.
    let style = node.attr(local_name!("style"));
    let width_style_cells = style
        .and_then(|s| parse_inline_style_value(s, "width"))
        .and_then(|v| parse_dim_cells(v, metrics.cell_w_px));
    let height_style_cells = style
        .and_then(|s| parse_inline_style_value(s, "height"))
        .and_then(|v| parse_dim_cells(v, metrics.cell_h_px));

    // Dioxus treats `img.width`/`img.height` as HTML attributes, not CSS.
    // Prefer CSS sizing via `style` when present.
    let width_attr_cells = node
        .attr(local_name!("width"))
        .and_then(|v| parse_dim_cells(v, metrics.cell_w_px));
    let height_attr_cells = node
        .attr(local_name!("height"))
        .and_then(|v| parse_dim_cells(v, metrics.cell_h_px));

    let width_hint = width_style_cells.or(width_attr_cells);
    let height_hint = height_style_cells.or(height_attr_cells);

    let mut desired_w = width_hint.unwrap_or(rect.width);
    let mut desired_h = height_hint.unwrap_or(rect.height);

    // If only one dimension is specified, infer the other from the intrinsic aspect ratio.
    // This is important for terminals where replaced element layout is unreliable.
    if width_hint.is_some() ^ height_hint.is_some() {
        if let Ok(decoded) = load_png_image(src) {
            if let Some(w) = width_hint {
                let w_px = w as f32 * metrics.cell_w_px;
                let h_px = w_px * (decoded.height as f32 / decoded.width as f32);
                desired_h = (h_px / metrics.cell_h_px).ceil().max(1.0) as u16;
            } else if let Some(h) = height_hint {
                let h_px = h as f32 * metrics.cell_h_px;
                let w_px = h_px * (decoded.width as f32 / decoded.height as f32);
                desired_w = (w_px / metrics.cell_w_px).ceil().max(1.0) as u16;
            }
        }
    }

    // If sizing is still degenerate (common for replaced elements), try deriving a
    // reasonable size from the intrinsic PNG dimensions.
    if desired_w <= 1 || desired_h <= 1 {
        if let Ok(decoded) = load_png_image(src) {
            let intrinsic_w_cells = ((decoded.width as f32) / metrics.cell_w_px)
                .ceil()
                .max(1.0) as u16;
            let intrinsic_h_cells = ((decoded.height as f32) / metrics.cell_h_px)
                .ceil()
                .max(1.0) as u16;

            if desired_w <= 1 {
                desired_w = intrinsic_w_cells;
            }
            if desired_h <= 1 {
                desired_h = intrinsic_h_cells;
            }
        }
    }

    if rect.x + desired_w > surface.width() {
        desired_w = surface.width().saturating_sub(rect.x);
    }
    if rect.y + desired_h > surface.height() {
        desired_h = surface.height().saturating_sub(rect.y);
    }
    if desired_w == 0 || desired_h == 0 {
        return Ok(());
    }

    let fallback = || {
        node.attr(local_name!("alt"))
            .filter(|s| !s.is_empty())
            .unwrap_or("<img unsupported>")
    };

    let paint_alt_text = |surface: &mut Surface| {
        let bounds = Rect::new(
            rect.x,
            rect.y,
            surface.width().saturating_sub(rect.x),
            surface.height().saturating_sub(rect.y),
        );
        let _ = write_wrapped(surface, bounds, (rect.x, rect.y), fallback(), TextStyle::default());
    };

    let should_sample = match image_policy {
        ImagePolicy::AltText => {
            paint_alt_text(surface);
            false
        }
        ImagePolicy::Omit => false,
        ImagePolicy::Sampling => true,
        ImagePolicy::Inline => {
            if inline_images_supported {
                let placed = placed_image_from_png(src, rect.x, rect.y, desired_w, desired_h)
                    .map_err(crate::error::Error::Other)?;
                // Preserve any already-painted background (from ancestors), but clear glyphs
                // so the inline image isn't obscured by text.
                clear_glyphs_preserve_bg(surface, Rect::new(rect.x, rect.y, desired_w, desired_h));
                images.push_back(placed);
                false
            } else {
                // Inline unsupported: apply downgrade policy.
                match image_downgrade {
                    ImageDowngrade::AltText => {
                        paint_alt_text(surface);
                        false
                    }
                    ImageDowngrade::Sampling => true,
                    ImageDowngrade::Omit => false,
                    ImageDowngrade::Error => {
                        return Err(crate::error::Error::Other(anyhow::anyhow!(
                            "inline images not supported by terminal"
                        )));
                    }
                }
            }
        }
        ImagePolicy::Error => {
            if !inline_images_supported {
                return Err(crate::error::Error::Other(anyhow::anyhow!(
                    "inline images not supported by terminal"
                )));
            }
            let placed = placed_image_from_png(src, rect.x, rect.y, desired_w, desired_h)
                .map_err(crate::error::Error::Other)?;
            clear_glyphs_preserve_bg(surface, Rect::new(rect.x, rect.y, desired_w, desired_h));
            images.push_back(placed);
            false
        }
    };

    if !should_sample {
        return Ok(());
    }

    let img = match load_png_image(src) {
        Ok(img) => img,
        Err(err) => {
            // If sampling fails to load, fall back to alt text for best-effort modes,
            // or propagate error if configured.
            let hard_error = matches!(image_policy, ImagePolicy::Error)
                || matches!(image_downgrade, ImageDowngrade::Error);
            if hard_error {
                return Err(crate::error::Error::Other(err.into()));
            }
            paint_alt_text(surface);
            return Ok(());
        }
    };

    // Degrade: use half-block characters, encoding two vertical samples per cell.
    let cell_w = desired_w as u32;
    let cell_h = desired_h as u32;
    let sample_w = cell_w.max(1);
    let sample_h = (cell_h * 2).max(1);

    for cy in 0..cell_h {
        for cx in 0..cell_w {
            let px0 = (cx * img.width / sample_w).min(img.width.saturating_sub(1));
            let py_top = ((cy * 2) * img.height / sample_h).min(img.height.saturating_sub(1));
            let py_bot = ((cy * 2 + 1) * img.height / sample_h).min(img.height.saturating_sub(1));

            let (top_r, top_g, top_b, top_a) = pixel_rgba(&img.rgba, img.width, px0, py_top);
            let (bot_r, bot_g, bot_b, bot_a) = pixel_rgba(&img.rgba, img.width, px0, py_bot);

            // If both samples are effectively transparent, don't override.
            if top_a < 16 && bot_a < 16 {
                continue;
            }

            let fg = if top_a < 16 {
                None
            } else {
                Some(rgb_to_attr(top_r, top_g, top_b, color_mode, truecolor))
            };
            let bg = if bot_a < 16 {
                None
            } else {
                Some(rgb_to_attr(bot_r, bot_g, bot_b, color_mode, truecolor))
            };

            let x = rect.x.saturating_add(cx as u16);
            let y = rect.y.saturating_add(cy as u16);
            if x >= surface.width() || y >= surface.height() {
                continue;
            }
            let idx = y as usize * surface.width() as usize + x as usize;
            if let Some(slot) = surface.content.get_mut(idx) {
                slot.ch = '▀';
                slot.fg = fg;
                slot.bg = bg;
            }
        }
    }

    Ok(())
}

fn clear_glyphs_preserve_bg(surface: &mut Surface, rect: Rect) {
    let w = surface.width() as usize;
    for y in rect.y..rect.y.saturating_add(rect.height).min(surface.height()) {
        let row = y as usize * w;
        for x in rect.x..rect.x.saturating_add(rect.width).min(surface.width()) {
            if let Some(slot) = surface.content.get_mut(row + x as usize) {
                slot.ch = ' ';
                slot.fg = None;
                slot.intensity = Intensity::Normal;
                slot.underline = Underline::None;
                slot.italic = false;
                slot.blink = Blink::None;
            }
        }
    }
}

fn pixel_rgba(buf: &[u8], width: u32, x: u32, y: u32) -> (u8, u8, u8, u8) {
    let idx = ((y * width + x) * 4) as usize;
    if idx + 3 >= buf.len() {
        return (0, 0, 0, 0);
    }
    (buf[idx], buf[idx + 1], buf[idx + 2], buf[idx + 3])
}

fn rgb_to_attr(r: u8, g: u8, b: u8, color_mode: ColorMode, truecolor: bool) -> ColorAttribute {
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

fn write_wrapped(
    surface: &mut Surface,
    bounds: Rect,
    cursor: (u16, u16),
    text: &str,
    style: TextStyle,
) -> (u16, u16) {
    if style.preserve_whitespace || text.contains('\n') {
        return write_preformatted(surface, bounds, cursor, text, style);
    }

    let (mut x, mut y) = cursor;
    let end_x = bounds.x.saturating_add(bounds.width).min(surface.width());
    let end_y = bounds.y.saturating_add(bounds.height).min(surface.height());

    if bounds.width == 0 || bounds.height == 0 {
        return (x, y);
    }

    let max_line_width = bounds.width;

    let mut word = String::new();
    let mut pending_space = false;

    let flush_word = |surface: &mut Surface, x: &mut u16, y: &mut u16, word: &mut String| {
        if word.is_empty() {
            return;
        }

        let word_width: u16 = word
            .chars()
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(1).max(1) as u16)
            .sum();

        if *x > bounds.x && x.saturating_add(word_width) > end_x {
            *x = bounds.x;
            *y = y.saturating_add(1);
        }

        if *y >= end_y {
            word.clear();
            return;
        }

        // If the word is longer than the entire line, fall back to hard wrapping.
        if word_width > max_line_width {
            for ch in word.chars() {
                if *y >= end_y {
                    break;
                }
                let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1).max(1) as u16;
                if x.saturating_add(ch_width) > end_x {
                    *x = bounds.x;
                    *y = y.saturating_add(1);
                    if *y >= end_y {
                        break;
                    }
                }
                let line_width = end_x.saturating_sub(*x) as usize;
                surface.set_stringn_styled(
                    *x,
                    *y,
                    ch.to_string(),
                    line_width,
                    style.fg,
                    style.bg,
                    style.intensity,
                    style.underline,
                    style.italic,
                    style.blink,
                );
                *x = x.saturating_add(ch_width);
            }
            word.clear();
            return;
        }

        let line_width = end_x.saturating_sub(*x) as usize;
        surface.set_stringn_styled(
            *x,
            *y,
            word.as_str(),
            line_width,
            style.fg,
            style.bg,
            style.intensity,
            style.underline,
            style.italic,
            style.blink,
        );
        *x = x.saturating_add(word_width);
        word.clear();
    };

    for ch in text.chars() {
        if y >= end_y {
            break;
        }

        if ch == '\n' {
            flush_word(surface, &mut x, &mut y, &mut word);
            pending_space = false;
            x = bounds.x;
            y = y.saturating_add(1);
            continue;
        }

        if ch.is_whitespace() {
            flush_word(surface, &mut x, &mut y, &mut word);
            pending_space = true;
            continue;
        }

        if pending_space {
            if x > bounds.x {
                let space_width = 1u16;
                if x.saturating_add(space_width) > end_x {
                    x = bounds.x;
                    y = y.saturating_add(1);
                }
                if y < end_y {
                    let line_width = end_x.saturating_sub(x) as usize;
                    surface.set_stringn_styled(
                        x,
                        y,
                        " ",
                        line_width,
                        style.fg,
                        style.bg,
                        style.intensity,
                        style.underline,
                        style.italic,
                        style.blink,
                    );
                    x = x.saturating_add(space_width);
                }
            }
            pending_space = false;
        }

        word.push(ch);
    }

    flush_word(surface, &mut x, &mut y, &mut word);
    (x, y)
}

fn write_preformatted(
    surface: &mut Surface,
    bounds: Rect,
    cursor: (u16, u16),
    text: &str,
    style: TextStyle,
) -> (u16, u16) {
    let (mut x, mut y) = cursor;
    let (start_x, _start_y) = if style.pre_full_width {
        (0, 0)
    } else {
        (bounds.x, bounds.y)
    };
    let end_x = if style.pre_full_width {
        surface.width()
    } else {
        bounds.x.saturating_add(bounds.width)
    };
    let end_y = if style.pre_full_width {
        surface.height()
    } else {
        bounds.y.saturating_add(bounds.height)
    };

    if bounds.width == 0 || bounds.height == 0 {
        return (x, y);
    }

    let mut write_char = |ch: char, x: &mut u16, y: &mut u16| {
        if *y >= end_y {
            return;
        }
        let ch_width = 1u16;
        if x.saturating_add(ch_width) > end_x {
            *x = start_x;
            *y = y.saturating_add(1);
            if *y >= end_y {
                return;
            }
        }
        surface.set_glyph_styled(
            *x,
            *y,
            ch,
            style.fg,
            style.bg,
            style.intensity,
            style.underline,
            style.italic,
            style.blink,
        );
        *x = x.saturating_add(ch_width);
    };

    for ch in text.chars() {
        if y >= end_y {
            break;
        }

        if ch == '\n' {
            x = start_x;
            y = y.saturating_add(1);
            continue;
        }

        if ch == '\t' {
            for _ in 0..4 {
                write_char(' ', &mut x, &mut y);
                if y >= end_y {
                    break;
                }
            }
            continue;
        }

        if ch.is_whitespace() {
            write_char(' ', &mut x, &mut y);
            continue;
        }

        write_char(ch, &mut x, &mut y);
    }

    (x, y)
}

fn style_overrides(node: &Node, color_mode: ColorMode, truecolor: bool) -> TextStyle {
    let mut out = TextStyle::default();

    if let Some(idx) = attr_value(node, "data-fg-idx")
        .or_else(|| attr_value(node, "data_fg_idx"))
        .and_then(|s| s.parse::<u8>().ok())
    {
        out.fg = Some(ColorAttribute::PaletteIndex(idx));
    }
    if let Some(idx) = attr_value(node, "data-bg-idx")
        .or_else(|| attr_value(node, "data_bg_idx"))
        .and_then(|s| s.parse::<u8>().ok())
    {
        out.bg = Some(ColorAttribute::PaletteIndex(idx));
    }

    if let Some(attrs) = attr_value(node, "data-attrs").or_else(|| attr_value(node, "data_attrs")) {
        for token in attrs.split(|c: char| c == ',' || c.is_whitespace()) {
            match token.trim().to_ascii_lowercase().as_str() {
                "bold" => out.intensity = Intensity::Bold,
                "dim" => out.intensity = Intensity::Half,
                "underline" | "underscore" => out.underline = Underline::Single,
                "italic" => out.italic = true,
                "blink" => out.blink = Blink::Slow,
                _ => {}
            }
        }
    }

    if node.data.is_element_with_tag_name(&local_name!("a")) {
        out.underline = Underline::Single;
    }
    if node.data.is_element_with_tag_name(&local_name!("b"))
        || node.data.is_element_with_tag_name(&local_name!("strong"))
    {
        out.intensity = Intensity::Bold;
    }
    if node.data.is_element_with_tag_name(&local_name!("i"))
        || node.data.is_element_with_tag_name(&local_name!("em"))
    {
        out.italic = true;
    }
    if node.data.is_element_with_tag_name(&local_name!("blink")) {
        out.blink = Blink::Slow;
    }

    if node.data.is_element_with_tag_name(&local_name!("pre"))
        || node.data.is_element_with_tag_name(&local_name!("textarea"))
    {
        out.preserve_whitespace = true;
    }
    if let Some(style) = attr_value(node, "style")
        .and_then(|s| parse_inline_style_value(s, "white-space"))
    {
        let value = style.trim().to_ascii_lowercase();
        if value.starts_with("pre") {
            out.preserve_whitespace = true;
        }
    }
    if let Some(flag) = attr_value(node, "data-pre") {
        if flag == "true" || flag == "1" {
            out.preserve_whitespace = true;
        } else if flag == "full" {
            out.preserve_whitespace = true;
            out.pre_full_width = true;
        }
    }

    // If we don't have an explicit palette index override, allow CSS colors to set fg/bg.
    if out.fg.is_none() {
        out.fg = node_color(node, color_mode, truecolor);
    }
    if out.bg.is_none() {
        out.bg = node_background(node, color_mode, truecolor);
    }

    out
}

fn attr_value<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
    node.attrs()?.iter().find_map(|attr| {
        (attr.name.local.as_ref() == name).then_some(attr.value.as_str())
    })
}

fn root_background(
    doc: &BaseDocument,
    root: &Node,
    color_mode: ColorMode,
    truecolor: bool,
) -> Option<ColorAttribute> {
    let html_bg = node_background(root, color_mode, truecolor);
    if html_bg.is_some() {
        return html_bg;
    }

    root.children
        .iter()
        .copied()
        .find_map(|id| {
            doc.get_node(id)
                .filter(|node| node.data.is_element_with_tag_name(&local_name!("body")))
        })
        .and_then(|body| node_background(body, color_mode, truecolor))
}

fn node_background(node: &Node, color_mode: ColorMode, truecolor: bool) -> Option<ColorAttribute> {
    let style = node.primary_styles()?;
    let current_color = style.clone_color();
    let bg = style
        .get_background()
        .background_color
        .resolve_to_absolute(&current_color);
    absolute_to_color_attr(bg, color_mode, truecolor)
}

fn node_color(node: &Node, color_mode: ColorMode, truecolor: bool) -> Option<ColorAttribute> {
    let style = node.primary_styles()?;
    let fg = style.clone_color();
    absolute_to_color_attr(fg, color_mode, truecolor)
}

fn absolute_to_color_attr(
    color: AbsoluteColor,
    color_mode: ColorMode,
    truecolor: bool,
) -> Option<ColorAttribute> {
    let color = color.into_srgb_legacy();
    if color.is_transparent() {
        return None;
    }

    let comps = *color.raw_components();
    let r = (comps[0] * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (comps[1] * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (comps[2] * 255.0).round().clamp(0.0, 255.0) as u8;
    let srgb = SrgbaTuple::from((r, g, b));

    let palette_idx_256 =
        16 + 36 * (r as u16 / 51) as u8 + 6 * (g as u16 / 51) as u8 + (b as u16 / 51) as u8;
    let base_idx = (if r >= 128 { 1 } else { 0 })
        | (if g >= 128 { 2 } else { 0 })
        | (if b >= 128 { 4 } else { 0 });

    Some(match color_mode {
        ColorMode::BaseColors => ColorAttribute::PaletteIndex(base_idx),
        ColorMode::Ansi => ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256),
        ColorMode::Rgb => {
            if truecolor {
                ColorAttribute::TrueColorWithDefaultFallback(srgb)
            } else {
                ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256)
            }
        }
    })
}

fn fill_rect(
    surface: &mut Surface,
    rect: Rect,
    fg: Option<ColorAttribute>,
    bg: Option<ColorAttribute>,
) {
    let w = surface.width() as usize;
    for y in rect.y..rect.y.saturating_add(rect.height).min(surface.height()) {
        let row = y as usize * w;
        for x in rect.x..rect.x.saturating_add(rect.width).min(surface.width()) {
            if let Some(slot) = surface.content.get_mut(row + x as usize) {
                slot.ch = ' ';
                slot.fg = fg;
                slot.bg = bg;
                slot.intensity = Intensity::Normal;
                slot.underline = Underline::None;
                slot.italic = false;
                slot.blink = Blink::None;
            }
        }
    }
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

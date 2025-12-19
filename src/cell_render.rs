use blitz_dom::{local_name, BaseDocument, Node};
use termwiz::color::{ColorAttribute, SrgbaTuple};

use crate::config::{PaletteEntry, PaletteRoles};
use crate::config::ColorMode;
use crate::geometry::Rect;
use crate::layout::node_rect;
use crate::scene::CellMetrics;
use crate::surface::Surface;
use style::color::AbsoluteColor;
use unicode_width::UnicodeWidthChar;

pub fn paint_surface(
    surface: &mut Surface,
    doc: &BaseDocument,
    area: Rect,
    metrics: CellMetrics,
    palette_roles: PaletteRoles,
    color_mode: ColorMode,
    truecolor: bool,
) {
    surface.clear();

    let fallback_fg = palette_entry_to_attr(palette_roles.fg_primary, color_mode, truecolor);

    let root = doc.root_element();
    if let Some(bg) = root_background(doc, root, color_mode, truecolor) {
        fill_rect(surface, surface.area(), None, Some(bg));
    }

    paint_node(
        surface,
        doc,
        root,
        area,
        metrics,
        color_mode,
        truecolor,
        fallback_fg,
    );
}

fn paint_node(
    surface: &mut Surface,
    doc: &BaseDocument,
    node: &Node,
    area: Rect,
    metrics: CellMetrics,
    color_mode: ColorMode,
    truecolor: bool,
    fallback_fg: ColorAttribute,
) {
    match &node.data {
        blitz_dom::node::NodeData::Element(_) | blitz_dom::node::NodeData::AnonymousBlock(_) => {
            let rect = node_rect(doc, node, area, metrics);
            if rect.width > 0 && rect.height > 0 {
                if let Some(bg) = node_background(node, color_mode, truecolor) {
                    fill_rect(surface, rect, None, Some(bg));
                }
            }

            // Render inline text content within this node's box.
            if rect.width > 0 && rect.height > 0 {
                let fg = Some(node_color(node, color_mode, truecolor).unwrap_or(fallback_fg));
                let text_width = if is_blockish(node) {
                    area.width.saturating_sub(rect.x)
                } else {
                    rect.width
                };
                let text_bounds = Rect::new(
                    rect.x,
                    rect.y,
                    text_width,
                    area.height.saturating_sub(rect.y),
                );

                if node.data.is_element_with_tag_name(&local_name!("input")) {
                    if let Some(value) = node.attr(local_name!("value")) {
                        let _ = write_wrapped(surface, text_bounds, (rect.x, rect.y), value, fg);
                    }
                } else if node.data.is_element_with_tag_name(&local_name!("button")) {
                    let label = node.text_content();
                    let _ = write_wrapped(surface, text_bounds, (rect.x, rect.y), label.as_str(), fg);
                } else {
                    paint_inline_text(
                        surface,
                        doc,
                        node,
                        text_bounds,
                        fg,
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
                        doc,
                        child,
                        area,
                        metrics,
                        color_mode,
                        truecolor,
                        fallback_fg,
                    );
                }
            }
        }
        blitz_dom::node::NodeData::Text(_) => {}
        blitz_dom::node::NodeData::Document | blitz_dom::node::NodeData::Comment => {
            for child_id in node.children.iter().copied() {
                if let Some(child) = doc.get_node(child_id) {
                    paint_node(
                        surface,
                        doc,
                        child,
                        area,
                        metrics,
                        color_mode,
                        truecolor,
                        fallback_fg,
                    );
                }
            }
        }
    }
}

fn paint_inline_text(
    surface: &mut Surface,
    doc: &BaseDocument,
    node: &Node,
    bounds: Rect,
    fg: Option<ColorAttribute>,
    color_mode: ColorMode,
    truecolor: bool,
    fallback_fg: ColorAttribute,
) {
    let mut cursor_x = bounds.x;
    let mut cursor_y = bounds.y;
    let end_y = bounds.y.saturating_add(bounds.height);

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
                    fg,
                );
            }
            blitz_dom::node::NodeData::Element(_) | blitz_dom::node::NodeData::AnonymousBlock(_) => {
                if !is_blockish(child) {
                    if child.data.is_element_with_tag_name(&local_name!("input")) {
                        if let Some(value) = child.attr(local_name!("value")) {
                            (cursor_x, cursor_y) =
                                write_wrapped(surface, bounds, (cursor_x, cursor_y), value, fg);
                        }
                        continue;
                    }

                    let child_fg = Some(
                        node_color(child, color_mode, truecolor)
                            .or(fg)
                            .unwrap_or(fallback_fg),
                    );
                    (cursor_x, cursor_y) = paint_inline_children(
                        surface,
                        doc,
                        child,
                        bounds,
                        (cursor_x, cursor_y),
                        child_fg,
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
    fg: Option<ColorAttribute>,
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
                cursor = write_wrapped(surface, bounds, cursor, text.content.as_str(), fg);
            }
            blitz_dom::node::NodeData::Element(_) | blitz_dom::node::NodeData::AnonymousBlock(_) => {
                if !is_blockish(child) {
                    if child.data.is_element_with_tag_name(&local_name!("input")) {
                        if let Some(value) = child.attr(local_name!("value")) {
                            cursor = write_wrapped(surface, bounds, cursor, value, fg);
                        }
                        continue;
                    }

                    let child_fg = Some(
                        node_color(child, color_mode, truecolor)
                            .or(fg)
                            .unwrap_or(fallback_fg),
                    );
                    cursor = paint_inline_children(
                        surface,
                        doc,
                        child,
                        bounds,
                        cursor,
                        child_fg,
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
    )
}

fn write_wrapped(
    surface: &mut Surface,
    bounds: Rect,
    cursor: (u16, u16),
    text: &str,
    fg: Option<ColorAttribute>,
) -> (u16, u16) {
    let (mut x, mut y) = cursor;
    let end_x = bounds.x.saturating_add(bounds.width);
    let end_y = bounds.y.saturating_add(bounds.height);

    for ch in text.chars() {
        if y >= end_y {
            break;
        }
        if ch == '\n' {
            x = bounds.x;
            y = y.saturating_add(1);
            continue;
        }
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1).max(1) as u16;
        if x.saturating_add(ch_width) > end_x {
            x = bounds.x;
            y = y.saturating_add(1);
            if y >= end_y {
                break;
            }
        }
        let line_width = end_x.saturating_sub(x) as usize;
        surface.set_stringn_colored(x, y, ch.to_string(), line_width, fg, None);
        x = x.saturating_add(ch_width);
    }

    (x, y)
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

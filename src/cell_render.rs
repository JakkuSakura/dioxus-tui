use blitz_dom::{local_name, BaseDocument, Node};
use termwiz::cell::{Blink, Intensity, Underline};
use termwiz::color::{ColorAttribute, SrgbaTuple};

use crate::config::{PaletteEntry, PaletteRoles};
use crate::config::ColorMode;
use crate::geometry::Rect;
use crate::layout::node_rect;
use crate::scene::CellMetrics;
use crate::surface::Surface;
use crate::config::ImagePolicy;
use crate::image::load_png_image;
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
    image_policy: ImagePolicy,
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
        TextStyle::default(),
        image_policy,
    );
}

#[derive(Clone, Copy, Debug)]
struct TextStyle {
    fg: Option<ColorAttribute>,
    bg: Option<ColorAttribute>,
    intensity: Intensity,
    underline: Underline,
    italic: bool,
    blink: Blink,
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
        }
    }
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
    inherited: TextStyle,
    image_policy: ImagePolicy,
) {
    match &node.data {
        blitz_dom::node::NodeData::Element(_) | blitz_dom::node::NodeData::AnonymousBlock(_) => {
            let local_style = style_overrides(node, color_mode, truecolor);
            let node_style = inherited.merged(local_style);

            if node.data.is_element_with_tag_name(&local_name!("img")) {
                let rect = node_rect(doc, node, area, metrics);
                paint_img(surface, node, rect, color_mode, truecolor, image_policy);
                return;
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
                        let _ = write_wrapped(surface, text_bounds, (rect.x, rect.y), value, style);
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
                        doc,
                        child,
                        area,
                        metrics,
                        color_mode,
                        truecolor,
                        fallback_fg,
                        node_style,
                        image_policy,
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
                        inherited,
                        image_policy,
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
    inherited: TextStyle,
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

fn paint_img(
    surface: &mut Surface,
    node: &Node,
    rect: Rect,
    color_mode: ColorMode,
    truecolor: bool,
    image_policy: ImagePolicy,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    if matches!(image_policy, ImagePolicy::Omit) {
        return;
    }

    let Some(src) = node.attr(local_name!("src")) else {
        return;
    };
    let Ok(img) = load_png_image(src) else {
        return;
    };

    // Degrade: use half-block characters, encoding two vertical samples per cell.
    let cell_w = rect.width as u32;
    let cell_h = rect.height as u32;
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
        surface.set_stringn_styled(
            x,
            y,
            ch.to_string(),
            line_width,
            style.fg,
            style.bg,
            style.intensity,
            style.underline,
            style.italic,
            style.blink,
        );
        x = x.saturating_add(ch_width);
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

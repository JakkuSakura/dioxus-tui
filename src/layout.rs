use crate::geometry::{Alignment, Rect as UiRect};
use crate::scene::CellMetrics;
use crate::styles::DEFAULT_TUI_CSS;
use blitz_dom::{BaseDocument, Node};
use blitz_traits::shell::Viewport;
use dioxus_native_dom::DioxusDocument;

pub fn resolve_document(doc: &mut DioxusDocument, area: UiRect, metrics: CellMetrics) -> Option<usize> {
    // Ensure UA stylesheet is present for consistent defaults.
    doc.inner.add_user_agent_stylesheet(DEFAULT_TUI_CSS);

    let root_id = doc.inner.root_node().id;

    let width_px = (area.width as f32 * metrics.cell_w_px).ceil().max(1.0) as u32;
    let height_px = (area.height as f32 * metrics.cell_h_px).ceil().max(1.0) as u32;
    let viewport = Viewport::new(width_px, height_px, 1.0, doc.inner.viewport().color_scheme);
    doc.inner.set_viewport(viewport);
    doc.inner.resolve(0.0);

    if let Some(root) = doc.inner.get_node_mut(root_id) {
        let layout = &mut root.final_layout;
        if layout.size.width <= 1.0 {
            layout.size.width = width_px as f32;
        }
        if layout.size.height <= 1.0 {
            layout.size.height = height_px as f32;
        }
    }
    doc.inner.get_node(root_id).map(|_| root_id)
}

pub fn node_rect(doc: &BaseDocument, node: &Node, area: UiRect, metrics: CellMetrics) -> UiRect {
    let mut rect = {
        let layout = node.final_layout;
        let (abs_x, abs_y) = absolute_location_px(doc, node);
        let x0 = (abs_x / metrics.cell_w_px).floor().max(0.0) as i64;
        let y0 = (abs_y / metrics.cell_h_px).floor().max(0.0) as i64;
        let x1 = ((abs_x + layout.size.width) / metrics.cell_w_px)
            .ceil()
            .max(0.0) as i64;
        let y1 = ((abs_y + layout.size.height) / metrics.cell_h_px)
            .ceil()
            .max(0.0) as i64;

        let x = (x0 as u16).min(area.width.saturating_sub(1));
        let y = (y0 as u16).min(area.height.saturating_sub(1));
        let mut w = x1.saturating_sub(x0).max(1) as u16;
        let mut h = y1.saturating_sub(y0).max(1) as u16;
        if x + w > area.width {
            w = area.width.saturating_sub(x);
        }
        if y + h > area.height {
            h = area.height.saturating_sub(y);
        }
        UiRect::new(x, y, w, h)
    };

    if rect.x >= area.width {
        rect.x = area.width.saturating_sub(1);
        rect.width = 0;
    }
    if rect.y >= area.height {
        rect.y = area.height.saturating_sub(1);
        rect.height = 0;
    }
    if rect.x + rect.width > area.width {
        rect.width = area.width.saturating_sub(rect.x);
    }
    if rect.y + rect.height > area.height {
        rect.height = area.height.saturating_sub(rect.y);
    }

    rect
}

fn absolute_location_px(doc: &BaseDocument, node: &Node) -> (f32, f32) {
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut current = Some(node.id);
    while let Some(id) = current {
        let Some(n) = doc.get_node(id) else {
            break;
        };
        x += n.final_layout.location.x;
        y += n.final_layout.location.y;
        current = n.layout_parent.get().or(n.parent);
    }
    (x, y)
}

pub fn node_alignment(node: &Node) -> Alignment {
    node.element_data()
        .and_then(|el| {
            el.attrs.iter().find_map(|a| {
                let name = a.name.local.as_ref();
                if matches!(name, "text_align" | "align" | "align_items") {
                    Some(a.value.clone())
                } else {
                    None
                }
            })
        })
        .map(|v| match v.to_lowercase().as_str() {
            "center" => Alignment::Center,
            "right" => Alignment::Right,
            _ => Alignment::Left,
        })
        .unwrap_or(Alignment::Left)
}

pub fn print_layout(doc: &BaseDocument, node_id: usize, depth: usize, area: UiRect, metrics: CellMetrics) {
    let Some(node) = doc.get_node(node_id) else {
        return;
    };
    let indent = "  ".repeat(depth);
    let tag = node
        .element_data()
        .map(|el| el.name.local.to_string())
        .unwrap_or_else(|| "#text".to_string());
    let text = node
        .text_data()
        .map(|t| t.content.clone())
        .unwrap_or_default();
    let rect = node_rect(doc, node, area, metrics);
    println!(
        "{indent}- {tag} id={} area=({}, {}) {}x{} text=\"{}\"",
        node_id, rect.x, rect.y, rect.width, rect.height, text
    );
    for child in node.children.iter() {
        print_layout(doc, *child, depth + 1, area, metrics);
    }
}

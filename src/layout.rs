use crate::geometry::{Alignment, Rect as UiRect};
use crate::styles::DEFAULT_TUI_CSS;
use blitz_dom::{BaseDocument, Node};
use blitz_traits::shell::Viewport;
use dioxus_native_dom::DioxusDocument;

pub fn resolve_document(doc: &mut DioxusDocument, area: UiRect) -> Option<usize> {
    // Ensure UA stylesheet is present for consistent defaults.
    doc.inner.add_user_agent_stylesheet(DEFAULT_TUI_CSS);

    let root_id = doc.inner.root_node().id;

    let viewport = Viewport::new(
        area.width.into(),
        area.height.into(),
        1.0,
        doc.inner.viewport().color_scheme,
    );
    doc.inner.set_viewport(viewport);
    doc.inner.resolve(0.0);

    if let Some(root) = doc.inner.get_node_mut(root_id) {
        let layout = &mut root.final_layout;
        if layout.size.width <= 1.0 {
            layout.size.width = area.width as f32;
        }
        if layout.size.height <= 1.0 {
            layout.size.height = area.height as f32;
        }
    }
    doc.inner.get_node(root_id).map(|_| root_id)
}

pub fn node_rect(node: &Node, area: UiRect) -> UiRect {
    let mut rect = {
        let layout = node.final_layout;
        let x = layout.location.x.max(0.0).round() as u16;
        let y = layout.location.y.max(0.0).round() as u16;
        let mut w = layout.size.width.max(0.0).ceil() as u16;
        let mut h = layout.size.height.max(0.0).ceil() as u16;
        if w == 0 {
            w = 1;
        }
        if h == 0 {
            h = 1;
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

pub fn print_layout(doc: &BaseDocument, node_id: usize, depth: usize, area: UiRect) {
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
    let rect = node_rect(node, area);
    println!(
        "{indent}- {tag} id={} area=({}, {}) {}x{} text=\"{}\"",
        node_id, rect.x, rect.y, rect.width, rect.height, text
    );
    for child in node.children.iter() {
        print_layout(doc, *child, depth + 1, area);
    }
}

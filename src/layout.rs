use std::collections::HashMap;

use blitz_dom::BaseDocument;
use blitz_traits::shell::Viewport;
use dioxus_native_dom::DioxusDocument;
use ratatui::layout::{Alignment, Rect as UiRect};

pub struct LayoutNode {
    pub id: dioxus_core::ElementId,
    pub rect: UiRect,
    pub children: Vec<LayoutNode>,
    pub tag: Option<String>,
    pub text: Option<String>,
    pub attrs: std::collections::HashMap<String, String>,
    pub align: Alignment,
}

fn clamp_rect(rect: &mut UiRect, area: UiRect) {
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
}

fn parse_alignment(attrs: &HashMap<String, String>) -> Alignment {
    attrs
        .get("text_align")
        .or_else(|| attrs.get("align"))
        .map(|v| match v.to_lowercase().as_str() {
            "center" => Alignment::Center,
            "right" => Alignment::Right,
            _ => Alignment::Left,
        })
        .unwrap_or(Alignment::Left)
}

fn collect_attrs(node: &blitz_dom::Node) -> HashMap<String, String> {
    node.element_data()
        .map(|el| {
            el.attrs
                .iter()
                .map(|a| (a.name.local.to_string(), a.value.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn layout_from_blitz(blitz_id: usize, doc: &BaseDocument, area: UiRect) -> LayoutNode {
    let node = doc.get_node(blitz_id).expect("invalid blitz node id");

    let attrs = collect_attrs(node);
    let mut rect = {
        let layout = node.final_layout;
        let x = layout.location.x.max(0.0) as u16;
        let y = layout.location.y.max(0.0) as u16;
        let w = layout.size.width.max(0.0) as u16;
        let h = layout.size.height.max(0.0) as u16;
        UiRect::new(x, y, w, h)
    };
    clamp_rect(&mut rect, area);

    LayoutNode {
        id: dioxus_core::ElementId(blitz_id),
        tag: node.element_data().map(|el| el.name.local.to_string()),
        text: node.text_data().map(|t| t.content.clone()),
        attrs: attrs.clone(),
        align: parse_alignment(&attrs),
        rect,
        children: node
            .children
            .iter()
            .map(|child| layout_from_blitz(*child, doc, area))
            .collect(),
    }
}

fn find_main_container(doc: &BaseDocument) -> usize {
    fn walk(doc: &BaseDocument, id: usize) -> Option<usize> {
        let node = doc.get_node(id)?;
        if let Some(el) = node.element_data() {
            let is_main = el
                .attrs
                .iter()
                .any(|a| a.name.local.as_ref() == "id" && a.value == "main");
            if is_main || el.name.local.as_ref() == "main" {
                return Some(id);
            }
        }
        for child in node.children.iter() {
            if let Some(found) = walk(doc, *child) {
                return Some(found);
            }
        }
        None
    }

    walk(doc, doc.root_node().id).unwrap_or_else(|| doc.root_node().id)
}

/// Resolve layout on the Blitz document and return the rendered tree rooted at the app's main container.
pub fn build_layout(doc: &mut DioxusDocument, area: UiRect) -> Option<LayoutNode> {
    let viewport = Viewport::new(
        area.width.into(),
        area.height.into(),
        1.0,
        doc.inner.viewport().color_scheme,
    );
    doc.inner.set_viewport(viewport);
    doc.inner.resolve(0.0);

    let main_id = find_main_container(&doc.inner);
    doc.inner
        .get_node(main_id)
        .map(|_| layout_from_blitz(main_id, &doc.inner, area))
}

use std::collections::HashMap;

use crate::element::DomNode;
use blitz_dom::{
    local_name, ns, Attribute, BaseDocument, DocumentConfig, DocumentMutator, LocalName, QualName,
    DEFAULT_CSS,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus_native_dom::DioxusDocument;
use dioxus_native_dom::DEFAULT_CSS;
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

fn parse_alignment(node: &DomNode) -> Alignment {
    node.attrs
        .get("text_align")
        .or_else(|| node.attrs.get("align"))
        .map(|v| match v.to_lowercase().as_str() {
            "center" => Alignment::Center,
            "right" => Alignment::Right,
            _ => Alignment::Left,
        })
        .unwrap_or(Alignment::Left)
}

fn blitz_name(tag: &str) -> QualName {
    QualName::new(None, ns!(html), LocalName::from(tag))
}

fn blitz_style_for(node: &DomNode) -> String {
    // Translate our limited attrs into CSS Stylo understands.
    let mut rules: Vec<String> = Vec::new();

    // Respect explicit display if provided via attrs
    let mut has_display = false;
    if let Some(display) = node.attrs.get("display") {
        rules.push(format!("display: {}", display));
        has_display = true;
    }

    // Opt into flex layout only when explicitly requested.
    let flex_dir = node
        .attrs
        .get("flex_direction")
        .or_else(|| node.attrs.get("direction"));
    if flex_dir.is_some()
        || node.attrs.contains_key("justify_content")
        || node.attrs.contains_key("align_items")
    {
        rules.push("display: flex".into());
        has_display = true;
        rules.push(format!(
            "flex-direction: {}",
            flex_dir.unwrap_or(&"column".to_string())
        ));
    }

    if let Some(justify) = node.attrs.get("justify_content") {
        rules.push(format!("justify-content: {}", justify));
    }

    if let Some(align) = node
        .attrs
        .get("align_items")
        .or_else(|| node.attrs.get("align_content"))
    {
        rules.push(format!("align-items: {}", align));
    }

    if let Some(width) = node.attrs.get("width") {
        rules.push(format!("width: {width}"));
    }
    if let Some(height) = node.attrs.get("height") {
        rules.push(format!("height: {height}"));
    }

    // Default block width for common block tags to ensure they occupy the available space.
    if matches!(
        node.tag.as_deref(),
        Some("div" | "p" | "h1" | "h2" | "h3" | "ul" | "ol" | "li")
    ) {
        if !node.attrs.contains_key("width") {
            rules.push("width: 100%".into());
        }
    }

    if let Some(text_align) = node
        .attrs
        .get("text_align")
        .or_else(|| node.attrs.get("align"))
    {
        rules.push(format!("text-align: {text_align}"));
    }

    if !has_display {
        rules.push("display: block".into());
    }

    rules.join("; ")
}

/// Build layout using Blitz (Stylo + Taffy) to mirror web semantics.
pub fn build_layout(nodes: &[DomNode], root: &DomNode, area: UiRect) -> LayoutNode {
    // Build a Blitz DioxusDocument once and feed the entire tree (we don't use the dioxus VDOM).
    // Build a Blitz document without relying on a live dioxus VDOM; we only need the DOM container.
    let dummy_vdom = dioxus_core::VirtualDom::new(|| dioxus::prelude::rsx! { div { "" } });
    let mut doc = DioxusDocument::new(
        dummy_vdom,
        DocumentConfig {
            viewport: Some(Viewport::new(
                area.width.into(),
                area.height.into(),
                1.0,
                ColorScheme::Light,
            )),
            ..Default::default()
        },
    );
    // Apply Blitz UA stylesheet for sensible defaults.
    doc.inner.add_user_agent_stylesheet(DEFAULT_CSS);

    // Recreate the DOM tree inside Blitz in one traversal under <body>.
    let id_map: HashMap<dioxus_core::ElementId, &DomNode> =
        nodes.iter().map(|n| (n.id, n)).collect();
    let mut blitz_ids: HashMap<dioxus_core::ElementId, usize> = HashMap::new();
    let body_id = doc.body_element_id;

    fn build_blitz_subtree(
        node: &DomNode,
        mutator: &mut DocumentMutator<'_>,
        id_map: &HashMap<dioxus_core::ElementId, &DomNode>,
        blitz_ids: &mut HashMap<dioxus_core::ElementId, usize>,
    ) -> usize {
        // Text-only node: create a text node directly.
        if node.tag.is_none() {
            let text = node
                .text
                .as_ref()
                .map(|t| t.text.clone())
                .unwrap_or_default();
            let text_id = mutator.create_text_node(&text);
            blitz_ids.insert(node.id, text_id);
            return text_id;
        }

        let tag = node.tag.as_deref().unwrap_or("div");
        let qual = blitz_name(tag);

        let mut attrs: Vec<Attribute> = node
            .attrs
            .iter()
            .map(|(name, value)| Attribute {
                name: QualName::new(None, ns!(html), LocalName::from(name.as_str())),
                value: value.clone(),
            })
            .collect();

        let style = blitz_style_for(node);
        if !style.is_empty() {
            attrs.push(Attribute {
                name: QualName::new(None, ns!(html), local_name!("style")),
                value: style,
            });
        }

        let element_id = mutator.create_element(qual, attrs);
        blitz_ids.insert(node.id, element_id);

        if let Some(text) = &node.text {
            let text_id = mutator.create_text_node(&text.text);
            mutator.append_children(element_id, &[text_id]);
        }

        for child in node.children.iter() {
            if let Some(child_node) = id_map.get(child) {
                let child_id = build_blitz_subtree(child_node, mutator, id_map, blitz_ids);
                mutator.append_children(element_id, &[child_id]);
            }
        }

        element_id
    }

    {
        let mut mutator = doc.mutate();
        let root_blitz_id = build_blitz_subtree(root, &mut mutator, &id_map, &mut blitz_ids);
        mutator.append_children(body_id, &[root_blitz_id]);
    }

    doc.inner.resolve(0.0);

    // Build reverse map Blitz node -> Dioxus ElementId when available.
    let mut blitz_to_dom: HashMap<usize, dioxus_core::ElementId> = HashMap::new();
    for (dom_id, blitz_id) in blitz_ids.iter() {
        blitz_to_dom.insert(*blitz_id, *dom_id);
    }

    // Recursively assemble layout starting from the app root blitz id so we only include the user subtree.
    fn assemble_from_blitz(
        blitz_id: usize,
        doc: &BaseDocument,
        blitz_to_dom: &HashMap<usize, dioxus_core::ElementId>,
        id_map: &HashMap<dioxus_core::ElementId, &DomNode>,
        area: UiRect,
    ) -> LayoutNode {
        let node = doc.get_node(blitz_id).expect("invalid blitz node id");

        let dom_id = blitz_to_dom
            .get(&blitz_id)
            .cloned()
            .unwrap_or(dioxus_core::ElementId(blitz_id as u64));
        let dom_node = id_map.get(&dom_id).copied();

        let tag = node.element_data().map(|el| el.name.local.to_string());
        let text = node.text_data().map(|t| t.content.clone());
        let attrs = dom_node.map(|n| n.attrs.clone()).unwrap_or_default();

        let mut layout_node = LayoutNode {
            id: dom_id,
            rect: UiRect::default(),
            children: Vec::new(),
            tag,
            text,
            attrs,
            align: dom_node.map(parse_alignment).unwrap_or(Alignment::Left),
        };

        let layout = node.final_layout;
        let mut x = layout.location.x.max(0.0);
        let mut y = layout.location.y.max(0.0);
        x = x.min((area.width.saturating_sub(1)) as f32);
        y = y.min((area.height.saturating_sub(1)) as f32);
        let mut w = layout.size.width.max(0.0);
        let mut h = layout.size.height.max(0.0);
        w = w.min((area.width as f32 - x).max(0.0));
        h = h.min((area.height as f32 - y).max(0.0));
        layout_node.rect = UiRect::new(x as u16, y as u16, w as u16, h as u16);

        for child_id in node.children.iter() {
            layout_node.children.push(assemble_from_blitz(
                *child_id,
                doc,
                blitz_to_dom,
                id_map,
                area,
            ));
        }

        if layout_node.rect.width == 0 {
            layout_node.rect.width = 1;
        }
        if layout_node.rect.height == 0 {
            layout_node.rect.height = 1;
        }

        clamp_rect(&mut layout_node.rect, area);
        for child in &mut layout_node.children {
            clamp_rect(&mut child.rect, area);
        }

        layout_node
    }

    let root_blitz_id = blitz_ids.get(&root.id).copied().unwrap_or(body_id);
    assemble_from_blitz(root_blitz_id, &doc.inner, &blitz_to_dom, &id_map, area)
}

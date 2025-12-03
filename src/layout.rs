use std::collections::HashMap;

use crate::element::DomNode;
use blitz_dom::{
    local_name, ns, Attribute, BaseDocument, DocumentConfig, DocumentMutator, LocalName, QualName,
};
use blitz_traits::shell::{ColorScheme, Viewport};
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
    if flex_dir.is_some() || node.attrs.contains_key("justify_content") || node.attrs.contains_key("align_items") {
        rules.push("display: flex".into());
        has_display = true;
        rules.push(format!("flex-direction: {}", flex_dir.unwrap_or(&"column".to_string())));
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
    if matches!(node.tag.as_deref(), Some("div" | "p" | "h1" | "h2" | "h3" | "ul" | "ol" | "li")) {
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
    let mut doc = BaseDocument::new(DocumentConfig {
        viewport: Some(Viewport::new(
            area.width.into(),
            area.height.into(),
            1.0,
            ColorScheme::Light,
        )),
        ..Default::default()
    });

    let id_map: HashMap<dioxus_core::ElementId, &DomNode> = nodes.iter().map(|n| (n.id, n)).collect();
    let mut blitz_ids: HashMap<dioxus_core::ElementId, usize> = HashMap::new();

    fn build_blitz_subtree(
        node: &DomNode,
        mutator: &mut DocumentMutator<'_>,
        id_map: &HashMap<dioxus_core::ElementId, &DomNode>,
        blitz_ids: &mut HashMap<dioxus_core::ElementId, usize>,
    ) -> usize {
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

    let root_container_id = doc.root_node().id;
    {
        let mut mutator = doc.mutate();
        let root_blitz_id = build_blitz_subtree(root, &mut mutator, &id_map, &mut blitz_ids);
        mutator.append_children(root_container_id, &[root_blitz_id]);

        // Ensure the root fills the viewport to avoid zero-sized descendants when no author styles are present.
        mutator.set_style_property(root_blitz_id, "display", "block");
        mutator.set_style_property(root_blitz_id, "width", "100%");
        mutator.set_style_property(root_blitz_id, "height", "100%");
    }

    doc.resolve(0.0);

    fn dom_text(node: &DomNode, id_map: &HashMap<dioxus_core::ElementId, &DomNode>) -> String {
        let mut out = String::new();
        if let Some(t) = &node.text {
            out.push_str(&t.text);
        }
        for child_id in &node.children {
            if let Some(child) = id_map.get(child_id) {
                let child_text = dom_text(child, id_map);
                if !child_text.is_empty() {
                    if !out.is_empty() {
                        out.push(' ');
                    }
                    out.push_str(&child_text);
                }
            }
        }
        out
    }

    fn assemble_layout(
        node: &DomNode,
        id_map: &HashMap<dioxus_core::ElementId, &DomNode>,
        blitz_ids: &HashMap<dioxus_core::ElementId, usize>,
        doc: &BaseDocument,
        area: UiRect,
    ) -> LayoutNode {
        let mut layout_node = LayoutNode {
            id: node.id,
            rect: UiRect::default(),
            children: Vec::new(),
            tag: node.tag.clone(),
            text: node.text.as_ref().map(|t| t.text.clone()),
            attrs: node.attrs.clone(),
            align: parse_alignment(node),
        };

        if layout_node.text.is_none() {
            let fallback = dom_text(node, id_map);
            if !fallback.is_empty() {
                layout_node.text = Some(fallback);
            }
        }

        if let Some(node_id) = blitz_ids.get(&node.id).and_then(|id| doc.get_node(*id)) {
            let layout = node_id.final_layout;
            let mut x = layout.location.x.max(0.0);
            let mut y = layout.location.y.max(0.0);
            x = x.min((area.width.saturating_sub(1)) as f32);
            y = y.min((area.height.saturating_sub(1)) as f32);
            let mut w = layout.size.width.max(0.0);
            let mut h = layout.size.height.max(0.0);
            w = w.min((area.width as f32 - x).max(0.0));
            h = h.min((area.height as f32 - y).max(0.0));
            layout_node.rect = UiRect::new(x as u16, y as u16, w as u16, h as u16);
        }

        // Minimum size for text leaves to ensure visibility even if Blitz reports zero.
        if let Some(text) = &layout_node.text {
            let line_count = text.lines().count().max(1) as u16;
            let max_line = text.lines().map(|l| l.len() as u16).max().unwrap_or(1);
            if layout_node.rect.width == 0 {
                let remaining_w = area.width.saturating_sub(layout_node.rect.x);
                layout_node.rect.width = max_line.min(remaining_w).max(1);
            }
            if layout_node.rect.height == 0 {
                layout_node.rect.height = line_count;
            } else {
                layout_node.rect.height = layout_node.rect.height.max(line_count);
            }
        }

        for child in node.children.iter() {
            if let Some(child_node) = id_map.get(child) {
                layout_node.children
                    .push(assemble_layout(child_node, id_map, blitz_ids, doc, area));
            }
        }

        // Stack children vertically relative to the parent; recurse to keep subtree positions consistent.
        fn restack(node: &mut LayoutNode) {
            if node.children.is_empty() {
                return;
            }

            let mut cursor_y = node.rect.y;
            for child in &mut node.children {
                child.rect.x = node.rect.x;
                if !child.attrs.contains_key("width") && child.rect.width == 0 {
                    child.rect.width = node.rect.width.max(1);
                }
                child.rect.y = cursor_y;
                if child.rect.height == 0 {
                    child.rect.height = 1;
                }
                cursor_y = child.rect.y.saturating_add(child.rect.height);

                restack(child);
            }

            let mut max_right = node.rect.x as i32 + node.rect.width as i32;
            let mut max_bottom = node.rect.y as i32 + node.rect.height as i32;
            for child in &node.children {
                max_right = max_right.max(child.rect.x as i32 + child.rect.width as i32);
                max_bottom = max_bottom.max(child.rect.y as i32 + child.rect.height as i32);
            }
            let new_w = (max_right - node.rect.x as i32).max(0) as u16;
            let new_h = (max_bottom - node.rect.y as i32).max(0) as u16;
            node.rect.width = node.rect.width.max(new_w);
            node.rect.height = node.rect.height.max(new_h);
        }

        restack(&mut layout_node);

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

        clamp_rect(&mut layout_node.rect, area);
        for child in &mut layout_node.children {
            clamp_rect(&mut child.rect, area);
        }

        // Final guard: ensure a minimum visible size (clamped to viewport).
        let parent_right = layout_node.rect.x.saturating_add(layout_node.rect.width);
        let parent_bottom = layout_node.rect.y.saturating_add(layout_node.rect.height);
        for child in &mut layout_node.children {
            let remaining_w = parent_right.saturating_sub(child.rect.x);
            let remaining_h = parent_bottom.saturating_sub(child.rect.y);
            child.rect.width = child.rect.width.max(1).min(remaining_w.max(1));
            child.rect.height = child.rect.height.max(1).min(remaining_h.max(1));
        }

        layout_node.rect.width = layout_node.rect.width.max(1).min(area.width);
        layout_node.rect.height = layout_node.rect.height.max(1).min(area.height);

        layout_node
    }

    assemble_layout(root, &id_map, &blitz_ids, &doc, area)
}

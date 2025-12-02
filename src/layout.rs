use std::collections::HashMap;

use crate::element::ViewNode;
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

fn parse_alignment(node: &ViewNode) -> Alignment {
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

fn blitz_style_for(node: &ViewNode) -> String {
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
pub fn build_layout(nodes: &[ViewNode], root: &ViewNode, area: UiRect) -> LayoutNode {
    let mut doc = BaseDocument::new(DocumentConfig {
        viewport: Some(Viewport::new(
            area.width.into(),
            area.height.into(),
            1.0,
            ColorScheme::Light,
        )),
        ..Default::default()
    });

    let id_map: HashMap<dioxus_core::ElementId, &ViewNode> =
        nodes.iter().map(|n| (n.id, n)).collect();
    let mut blitz_ids: HashMap<dioxus_core::ElementId, usize> = HashMap::new();

    fn build_blitz_subtree(
        node: &ViewNode,
        mutator: &mut DocumentMutator<'_>,
        id_map: &HashMap<dioxus_core::ElementId, &ViewNode>,
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

    fn assemble_layout(
        node: &ViewNode,
        id_map: &HashMap<dioxus_core::ElementId, &ViewNode>,
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

        // Fallback size for text leaves that Blitz reports as zero-sized
        if layout_node.text.is_some() {
            if layout_node.rect.width == 0 {
                layout_node.rect.width = 1;
            }
            if layout_node.rect.height == 0 {
                layout_node.rect.height = 1;
            }
        }

        for child in node.children.iter() {
            if let Some(child_node) = id_map.get(child) {
                layout_node.children
                    .push(assemble_layout(child_node, id_map, blitz_ids, doc, area));
            }
        }

        // If the container got a zero (or tiny) size but its children have extents, expand to fit them within the viewport.
        let mut max_x = layout_node.rect.x as i32 + layout_node.rect.width as i32;
        let mut max_y = layout_node.rect.y as i32 + layout_node.rect.height as i32;
        for child in &layout_node.children {
            max_x = max_x.max(child.rect.x as i32 + child.rect.width as i32);
            max_y = max_y.max(child.rect.y as i32 + child.rect.height as i32);
        }
        let new_w = (max_x - layout_node.rect.x as i32).max(0) as u16;
        let new_h = (max_y - layout_node.rect.y as i32).max(0) as u16;
        if new_w > layout_node.rect.width {
            layout_node.rect.width = new_w.min(area.width.saturating_sub(layout_node.rect.x));
        }
        if new_h > layout_node.rect.height {
            layout_node.rect.height = new_h.min(area.height.saturating_sub(layout_node.rect.y));
        }

        layout_node
    }

    assemble_layout(root, &id_map, &blitz_ids, &doc, area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ViewNode, ViewText};
    use dioxus_core::ElementId;

    fn text_node(id: usize, text: &str) -> ViewNode {
        ViewNode {
            id: ElementId(id),
            text: Some(ViewText {
                text: text.to_string(),
            }),
            ..Default::default()
        }
    }

    fn element(id: usize, tag: &str, children: Vec<ElementId>) -> ViewNode {
        ViewNode {
            id: ElementId(id),
            tag: Some(tag.to_string()),
            children,
            ..Default::default()
        }
    }

    #[test]
    fn root_fills_viewport() {
        let mut nodes = Vec::new();
        // root div with a single text child
        let root = element(1, "div", vec![ElementId(2)]);
        let child = text_node(2, "hello");
        nodes.push(root.clone());
        nodes.push(child);

        let area = UiRect::new(0, 0, 40, 10);
        let layout = build_layout(&nodes, &root, area);

        assert_eq!(layout.rect.width, area.width);
        assert_eq!(layout.rect.height, area.height);
        // Child should have non-zero size and live within the root
        let child_rect = &layout.children[0].rect;
        assert!(child_rect.width > 0 && child_rect.height > 0);
        assert!(child_rect.x < area.width && child_rect.y < area.height);
    }

    #[test]
    fn block_children_stack_vertically() {
        // h1 + two paragraphs
        let mut nodes = Vec::new();
        let h1 = element(2, "h1", vec![ElementId(3)]);
        let p1 = element(4, "p", vec![ElementId(5)]);
        let p2 = element(6, "p", vec![ElementId(7)]);
        let root = ViewNode {
            id: ElementId(1),
            tag: Some("div".into()),
            children: vec![h1.id, p1.id, p2.id],
            ..Default::default()
        };

        nodes.push(root.clone());
        nodes.push(h1.clone());
        nodes.push(text_node(3, "Title"));
        nodes.push(p1.clone());
        nodes.push(text_node(5, "First paragraph."));
        nodes.push(p2.clone());
        nodes.push(text_node(7, "Second paragraph."));

        let area = UiRect::new(0, 0, 60, 20);
        let layout = build_layout(&nodes, &root, area);

        fn collect_text_leaves(node: &LayoutNode, out: &mut Vec<UiRect>) {
            if node.text.is_some() {
                out.push(node.rect);
            }
            for child in &node.children {
                collect_text_leaves(child, out);
            }
        }

        let mut leaves = Vec::new();
        collect_text_leaves(&layout, &mut leaves);
        assert!(leaves.iter().all(|r| r.width > 0 && r.height > 0));

        // y positions of direct children should be non-decreasing (stacked)
        let mut last_y = 0;
        for child in &layout.children {
            assert!(child.rect.y >= last_y);
            last_y = child.rect.y;
        }
    }
}

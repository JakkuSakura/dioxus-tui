use std::collections::HashMap;

use crate::element::DebugNode;
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

fn parse_alignment(node: &DebugNode) -> Alignment {
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

fn blitz_style_for(node: &DebugNode) -> String {
    // Translate our limited attrs into CSS Stylo understands.
    let mut rules: Vec<String> = vec!["display: flex".into(), "flex: 1 1 auto".into()];

    if let Some(dir) = node
        .attrs
        .get("flex_direction")
        .or_else(|| node.attrs.get("direction"))
    {
        rules.push(format!("flex-direction: {}", dir));
    } else {
        rules.push("flex-direction: column".into());
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

    if node.text.is_some() {
        rules.push("flex: 0 0 auto".into());
    }

    rules.join("; ")
}

/// Build layout using Blitz (Stylo + Taffy) to mirror web semantics.
pub fn build_layout(nodes: &[DebugNode], root: &DebugNode, area: UiRect) -> LayoutNode {
    let mut doc = BaseDocument::new(DocumentConfig {
        viewport: Some(Viewport::new(
            area.width.into(),
            area.height.into(),
            1.0,
            ColorScheme::Light,
        )),
        ..Default::default()
    });

    let id_map: HashMap<dioxus_core::ElementId, &DebugNode> =
        nodes.iter().map(|n| (n.id, n)).collect();
    let mut blitz_ids: HashMap<dioxus_core::ElementId, usize> = HashMap::new();

    fn build_blitz_subtree(
        node: &DebugNode,
        mutator: &mut DocumentMutator<'_>,
        id_map: &HashMap<dioxus_core::ElementId, &DebugNode>,
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
    }

    doc.resolve(0.0);

    fn assemble_layout(
        node: &DebugNode,
        id_map: &HashMap<dioxus_core::ElementId, &DebugNode>,
        blitz_ids: &HashMap<dioxus_core::ElementId, usize>,
        doc: &BaseDocument,
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
            layout_node.rect = UiRect::new(
                layout.location.x.max(0.0) as u16,
                layout.location.y.max(0.0) as u16,
                layout.size.width.max(0.0) as u16,
                layout.size.height.max(0.0) as u16,
            );
        }

        for child in node.children.iter() {
            if let Some(child_node) = id_map.get(child) {
                layout_node.children.push(assemble_layout(child_node, id_map, blitz_ids, doc));
            }
        }

        layout_node
    }

    assemble_layout(root, &id_map, &blitz_ids, &doc)
}

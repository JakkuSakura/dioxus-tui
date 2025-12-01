use std::collections::{HashMap, HashSet};

use crate::element::DebugNode;
use ratatui::layout::{Alignment, Rect as UiRect};
use taffy::prelude::*;
use taffy::{NodeId, Taffy};

pub struct LayoutNode {
    pub id: dioxus_core::ElementId,
    pub rect: UiRect,
    pub children: Vec<LayoutNode>,
    pub tag: Option<String>,
    pub text: Option<String>,
    pub attrs: std::collections::HashMap<String, String>,
    pub align: Alignment,
}

/// Build layout using Taffy (flexbox) to better mirror web semantics.
pub fn build_layout(nodes: &[DebugNode], root: &DebugNode, area: UiRect) -> LayoutNode {
    let mut taffy = Taffy::new();
    let mut id_map: HashMap<dioxus_core::ElementId, &DebugNode> =
        nodes.iter().map(|n| (n.id, n)).collect();

    fn style_from_attrs(attrs: &HashMap<String, String>) -> Style {
        let mut style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        };

        if let Some(dir) = attrs
            .get("flex_direction")
            .or_else(|| attrs.get("direction"))
            .map(|s| s.to_lowercase())
        {
            style.flex_direction = match dir.as_str() {
                "row" => FlexDirection::Row,
                _ => FlexDirection::Column,
            };
        }

        if let Some(justify) = attrs.get("justify_content").map(|s| s.to_lowercase()) {
            style.justify_content = match justify.as_str() {
                "center" => Some(JustifyContent::Center),
                "end" | "flex-end" => Some(JustifyContent::FlexEnd),
                "space-between" => Some(JustifyContent::SpaceBetween),
                _ => Some(JustifyContent::FlexStart),
            };
        }

        if let Some(align) = attrs
            .get("align_items")
            .or_else(|| attrs.get("align_content"))
            .map(|s| s.to_lowercase())
        {
            style.align_items = match align.as_str() {
                "center" => Some(AlignItems::Center),
                "end" | "flex-end" => Some(AlignItems::FlexEnd),
                "stretch" => Some(AlignItems::Stretch),
                _ => Some(AlignItems::FlexStart),
            };
        }

        if let Some(w) = attrs.get("width") {
            style.size.width = parse_dimension(w);
        }
        if let Some(h) = attrs.get("height") {
            style.size.height = parse_dimension(h);
        }
        style
    }

    fn parse_dimension(raw: &str) -> Dimension {
        let s = raw.trim().to_lowercase();
        if s.ends_with('%') {
            if let Ok(pct) = s.trim_end_matches('%').trim().parse::<f32>() {
                return Dimension::percent((pct / 100.0).clamp(0.0, 1.0));
            }
        }
        if let Some(px) = s.strip_suffix("px").and_then(|v| v.trim().parse::<f32>().ok()) {
            return Dimension::length(px);
        }
        if let Ok(px) = s.parse::<f32>() {
            return Dimension::length(px);
        }
        Dimension::auto()
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

    fn build_tree(
        taffy: &mut TaffyTree<MeasureFunc>,
        node: &DebugNode,
        map: &HashMap<dioxus_core::ElementId, &DebugNode>,
        visited: &mut HashSet<dioxus_core::ElementId>,
    ) -> (NodeId, LayoutNode) {
        if !visited.insert(node.id) {
            let layout_node = LayoutNode {
                id: node.id,
                rect: UiRect::default(),
                children: Vec::new(),
                tag: node.tag.clone(),
                text: node.text.as_ref().map(|t| t.text.clone()),
                attrs: node.attrs.clone(),
                align: parse_alignment(node),
            };
            let handle = taffy.new_leaf(style_from_attrs(&node.attrs)).unwrap();
            return (handle, layout_node);
        }

        let mut child_handles = Vec::new();
        let mut layout_children = Vec::new();
        for child_id in node.children.iter() {
            if let Some(child) = map.get(child_id) {
                let (handle, layout_child) = build_tree(taffy, child, map, visited);
                child_handles.push(handle);
                layout_children.push(layout_child);
            }
        }

        let style = style_from_attrs(&node.attrs);
        let handle = if child_handles.is_empty() {
            taffy.new_leaf(style).unwrap()
        } else {
            taffy.new_with_children(style, &child_handles).unwrap()
        };

        let layout_node = LayoutNode {
            id: node.id,
            rect: UiRect::default(),
            children: layout_children,
            tag: node.tag.clone(),
            text: node.text.as_ref().map(|t| t.text.clone()),
            attrs: node.attrs.clone(),
            align: parse_alignment(node),
        };

        (handle, layout_node)
    }

    // Build tree and compute layout
    let (root_handle, mut layout_root) = build_tree(&mut taffy, root, &id_map, &mut HashSet::new());
    let size = taffy::geometry::Size {
        width: AvailableSpace::Definite(area.width as f32),
        height: AvailableSpace::Definite(area.height as f32),
    };
    let _ = taffy.compute_layout(root_handle, size);

    fn apply_layout(taffy: &TaffyTree<MeasureFunc>, handle: NodeId, layout_node: &mut LayoutNode) {
        if let Ok(layout) = taffy.layout(handle) {
            layout_node.rect = UiRect::new(
                layout.location.x as u16,
                layout.location.y as u16,
                layout.size.width as u16,
                layout.size.height as u16,
            );
        }

        let children = taffy.children(handle).unwrap_or_default();
        for (i, child_handle) in children.into_iter().enumerate() {
            if let Some(child_node) = layout_node.children.get_mut(i) {
                apply_layout(taffy, child_handle, child_node);
            }
        }
    }

    apply_layout(&taffy, root_handle, &mut layout_root);
    layout_root
}

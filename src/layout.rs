use crate::element::DebugNode;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};

pub struct LayoutNode {
    pub id: dioxus_core::ElementId,
    pub rect: Rect,
    pub children: Vec<LayoutNode>,
    pub tag: Option<String>,
    pub text: Option<String>,
    pub attrs: std::collections::HashMap<String, String>,
    pub align: Alignment,
}

/// Build a very naive vertical layout: each node gets an equal slice of its parent's height.
pub fn build_layout(nodes: &[DebugNode], root: &DebugNode, area: Rect) -> LayoutNode {
    let children = root
        .children
        .iter()
        .filter_map(|id| nodes.iter().find(|n| n.id == *id))
        .collect::<Vec<_>>();

    if children.is_empty() {
        return LayoutNode {
            id: root.id,
            rect: area,
            children: Vec::new(),
            tag: root.tag.clone(),
            text: root.text.as_ref().map(|t| t.text.clone()),
        };
    }

    let direction = layout_direction(root);
    let constraints = child_constraints(&children, direction, area);
    let chunks = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area);

    let mut laid_out = Vec::new();
    for (i, child) in children.iter().enumerate() {
        laid_out.push(build_layout(nodes, child, chunks[i]));
    }

    LayoutNode {
        id: root.id,
        rect: area,
        children: laid_out,
        tag: root.tag.clone(),
        text: root.text.as_ref().map(|t| t.text.clone()),
        attrs: root.attrs.clone(),
        align: parse_alignment(root),
    }
}

fn layout_direction(node: &DebugNode) -> Direction {
    if let Some(dir) = node.attrs.get("flex_direction").or_else(|| node.attrs.get("direction")) {
        match dir.to_lowercase().as_str() {
            "row" => Direction::Horizontal,
            _ => Direction::Vertical,
        }
    } else {
        Direction::Vertical
    }
}

fn child_constraints(children: &[&DebugNode], direction: Direction, area: Rect) -> Vec<Constraint> {
    let mut constraints = Vec::new();
    let total = if direction == Direction::Vertical { area.height } else { area.width };
    for child in children {
        // Prefer explicit size attribute if provided (as pixels)
        let key = if direction == Direction::Vertical { "height" } else { "width" };
        if let Some(val) = child.attrs.get(key) {
            if let Some(px) = parse_length(val, total) {
                constraints.push(px);
                continue;
            }
        }
        constraints.push(Constraint::Length(total / children.len() as u16));
    }
    constraints
}

fn parse_length(raw: &str, total: u16) -> Option<Constraint> {
    let s = raw.trim().to_lowercase();
    if s.ends_with('%') {
        let num = s.trim_end_matches('%').trim();
        if let Ok(pct) = num.parse::<f32>() {
            let len = ((pct / 100.0) * total as f32).max(0.0) as u16;
            return Some(Constraint::Length(len));
        }
    } else if let Some(stripped) = s.strip_suffix("px") {
        if let Ok(px) = stripped.trim().parse::<u16>() {
            return Some(Constraint::Length(px));
        }
    } else if let Ok(px) = s.parse::<u16>() {
        return Some(Constraint::Length(px));
    }
    None
}

fn parse_alignment(node: &DebugNode) -> Alignment {
    node
        .attrs
        .get("text_align")
        .or_else(|| node.attrs.get("align"))
        .map(|v| match v.to_lowercase().as_str() {
            "center" => Alignment::Center,
            "right" => Alignment::Right,
            _ => Alignment::Left,
        })
        .unwrap_or(Alignment::Left)
}

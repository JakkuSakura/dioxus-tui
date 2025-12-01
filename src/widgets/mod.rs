use crate::element::{DebugNode, DebugText};
use crate::layout::{build_layout, LayoutNode};
use dioxus_core::ElementId;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{Frame, style::{Style, Modifier}};

/// Render a minimal three-row layout similar to the ratatui_basic demo.
pub fn render_basic(frame: &mut Frame, lines: &[DebugText]) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let get_line = |idx: usize| lines.get(idx).map(|l| l.text.as_str()).unwrap_or("");

    let header = Paragraph::new(get_line(0)).block(Block::default().borders(Borders::ALL).title("Title"));
    let info = Paragraph::new(get_line(1)).block(Block::default().borders(Borders::ALL).title("Info"));
    let body = Paragraph::new(get_line(2)).block(Block::default().borders(Borders::ALL).title("Body"));

    frame.render_widget(header, chunks[0]);
    frame.render_widget(info, chunks[1]);
    frame.render_widget(body, chunks[2]);
}

pub fn render_tree(frame: &mut Frame, nodes: &[DebugNode], root_id: Option<ElementId>) {
    if let Some(root_id) = root_id {
        if let Some(root) = nodes.iter().find(|n| n.id == root_id) {
            let layout_tree = build_layout(nodes, root, frame.area());
            render_layout_node(frame, &layout_tree);
        }
    }
}

fn render_layout_node(frame: &mut Frame, node: &LayoutNode) {
    let tag = node.tag.as_deref().unwrap_or("");

    match tag {
        "p" => {
            if let Some(text) = &node.text {
                frame.render_widget(Paragraph::new(text.clone()).alignment(node.align), node.rect);
            }
        }
        "ul" | "ol" => {
            let items: Vec<ListItem> = node.children.iter().map(|child| {
                let content = child.text.clone().unwrap_or_default();
                ListItem::new(content)
            }).collect();
            let list = List::new(items).block(Block::default().borders(Borders::ALL).title(tag));
            frame.render_widget(list, node.rect);
        }
        "h1" | "h2" | "h3" => {
            if let Some(text) = &node.text {
                let style = Style::default().add_modifier(Modifier::BOLD);
                let paragraph = Paragraph::new(text.clone()).style(style).alignment(node.align);
                frame.render_widget(paragraph, node.rect);
            }
        }
        _ => {
            let mut block = Block::default();
            if !tag.is_empty() {
                block = block.borders(Borders::ALL).title(tag.to_string());
            }
            frame.render_widget(block.clone(), node.rect);
            let inner = block.inner(node.rect);
            if let Some(text) = &node.text {
                frame.render_widget(Paragraph::new(text.clone()).alignment(node.align), inner);
            }
            for child in node.children.iter() {
                render_layout_node(frame, child);
            }
        }
    }
}

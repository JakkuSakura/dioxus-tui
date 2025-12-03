use blitz_dom::{BaseDocument, Node};
use blitz_traits::shell::Viewport;
use dioxus_native_dom::DioxusDocument;
use ratatui::layout::{Alignment, Rect as UiRect};
use std::collections::HashMap;

// Minimal UA overrides for TUI rendering.
const TUI_UA_CSS: &str = r#"
html, body, main, div, p, h1, h2, h3, h4, h5, h6, ul, ol, li {
    margin: 0;
    padding: 0;
}

body, html, main, div, p, li {
    display: block;
}

h1, h2, h3, h4, h5, h6 {
    font-size: 1em;
    font-weight: bold;
}

ul { list-style-type: disc; padding-left: 1ch; }
ol { list-style-type: decimal; padding-left: 2ch; }
li { list-style-position: outside; }
"#;

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

pub fn resolve_document(doc: &mut DioxusDocument, area: UiRect) -> Option<usize> {
    // Ensure UA stylesheet is present
    doc.inner.add_user_agent_stylesheet(TUI_UA_CSS);

    let viewport = Viewport::new(
        area.width.into(),
        area.height.into(),
        1.0,
        doc.inner.viewport().color_scheme,
    );
    doc.inner.set_viewport(viewport);
    doc.inner.resolve(0.0);

    let main_id = find_main_container(&doc.inner);
    doc.inner.get_node(main_id).map(|_| main_id)
}

pub fn node_rect(node: &Node, area: UiRect) -> UiRect {
    let mut rect = {
        let layout = node.final_layout;
        let x = layout.location.x.max(0.0).round() as u16;
        let y = layout.location.y.max(0.0).round() as u16;
        let mut w = layout.size.width.max(0.0).ceil() as u16;
        let mut h = layout.size.height.max(0.0).ceil() as u16;
        if w == 0 && (!node.children.is_empty() || node.text_data().is_some()) {
            w = 1;
        }
        if h == 0 && (!node.children.is_empty() || node.text_data().is_some()) {
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
                if name == "text_align" || name == "align" {
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

pub fn collect_attrs(node: &Node) -> HashMap<String, String> {
    node.element_data()
        .map(|el| {
            el.attrs
                .iter()
                .map(|a| (a.name.local.to_string(), a.value.clone()))
                .collect()
        })
        .unwrap_or_default()
}

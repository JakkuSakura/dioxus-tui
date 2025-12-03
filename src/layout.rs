use std::collections::HashMap;

use blitz_dom::{ns, BaseDocument, LocalName, QualName};
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

fn blitz_style_for(tag: Option<&str>, attrs: &HashMap<String, String>) -> String {
    let mut rules: Vec<String> = Vec::new();

    if let Some(display) = attrs.get("display") {
        rules.push(format!("display: {display}"));
    }

    let flex_dir = attrs
        .get("flex_direction")
        .or_else(|| attrs.get("direction"));
    if flex_dir.is_some()
        || attrs.contains_key("justify_content")
        || attrs.contains_key("align_items")
    {
        rules.push("display: flex".into());
        rules.push(format!(
            "flex-direction: {}",
            flex_dir.cloned().unwrap_or_else(|| "column".to_string())
        ));
    }

    if let Some(justify) = attrs.get("justify_content") {
        rules.push(format!("justify-content: {justify}"));
    }
    if let Some(align) = attrs
        .get("align_items")
        .or_else(|| attrs.get("align_content"))
    {
        rules.push(format!("align-items: {align}"));
    }

    if let Some(width) = attrs.get("width") {
        rules.push(format!("width: {width}"));
    }
    if let Some(height) = attrs.get("height") {
        rules.push(format!("height: {height}"));
    }

    if matches!(
        tag,
        Some(
            "div"
                | "p"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "ul"
                | "ol"
                | "li"
                | "main"
                | "html"
                | "body",
        )
    ) {
        if !attrs.contains_key("width") {
            rules.push("width: 100%".into());
        }
        if !attrs.contains_key("display") {
            rules.push("display: block".into());
        }
    }

    if let Some(text_align) = attrs.get("text_align").or_else(|| attrs.get("align")) {
        rules.push(format!("text-align: {text_align}"));
    }

    rules.join("; ")
}

fn layout_from_blitz(blitz_id: usize, doc: &BaseDocument, area: UiRect) -> LayoutNode {
    let node = doc.get_node(blitz_id).expect("invalid blitz node id");

    let attrs = collect_attrs(node);
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
    let mut styles: Vec<(usize, String)> = Vec::new();
    {
        fn walk(
            id: usize,
            doc: &BaseDocument,
            inherited_tag: Option<String>,
            out: &mut Vec<(usize, String)>,
        ) {
            let Some(node) = doc.get_node(id) else {
                return;
            };
            let tag = node
                .element_data()
                .map(|el| el.name.local.to_string())
                .or(inherited_tag.clone());
            let attrs = collect_attrs(node);
            let style = blitz_style_for(tag.as_deref(), &attrs);
            if !style.is_empty() {
                out.push((id, style));
            }

            for child in node.children.iter() {
                walk(*child, doc, tag.clone(), out);
            }
        }

        let root = doc.inner.root_node().id;
        walk(root, &doc.inner, None, &mut styles);
    }

    if !styles.is_empty() {
        let mut mutator = doc.inner.mutate();
        for (id, style) in styles.into_iter() {
            mutator.set_attribute(
                id,
                QualName::new(None, ns!(html), LocalName::from("style")),
                &style,
            );
        }
    }

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

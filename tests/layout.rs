use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_core::VirtualDom;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use dioxus_tui::layout::{node_rect, resolve_document};
use dioxus_tui::Rect as UiRect;

fn build_doc_with_layout(app: fn() -> Element, width: u16, height: u16) -> (DioxusDocument, usize) {
    let vdom = VirtualDom::new(app);
    let viewport = Viewport::new(width.into(), height.into(), 1.0, ColorScheme::Light);
    let mut doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            viewport: Some(viewport),
            ..Default::default()
        },
    );
    doc.initial_build();
    let root = resolve_document(&mut doc, UiRect::new(0, 0, width, height))
        .expect("main root should exist after layout");
    (doc, root)
}

fn find_by_tag(doc: &blitz_dom::BaseDocument, id: usize, tag: &str, out: &mut Vec<usize>) {
    if let Some(node) = doc.get_node(id) {
        if node
            .element_data()
            .map(|el| el.name.local.as_ref() == tag)
            .unwrap_or(false)
        {
            out.push(id);
        }
        for child in node.children.iter() {
            find_by_tag(doc, *child, tag, out);
        }
    }
}

#[test]
fn layout_root_matches_viewport() {
    fn app() -> Element {
        rsx! { div { "hello" } }
    }

    let width = 80u16;
    let height = 25u16;
    let area = UiRect::new(0, 0, width, height);
    let (doc, root) = build_doc_with_layout(app, width, height);

    let root_node = doc.inner.get_node(root).unwrap();
    let rect = node_rect(root_node, area);
    assert_eq!(rect.width, width);
    assert!(rect.height > 0 && rect.height <= height);

    let first_child = root_node
        .children
        .first()
        .and_then(|id| doc.inner.get_node(*id));
    assert!(first_child.is_some());
    let child_rect = node_rect(first_child.unwrap(), area);
    assert!(child_rect.width > 0 && child_rect.height > 0);
}

#[test]
fn block_elements_stack_and_have_space() {
    fn app() -> Element {
        rsx! {
            div { direction: "column",
                h1 { "Title" }
                p { "First paragraph" }
                p { "Second paragraph" }
            }
        }
    }

    let area = UiRect::new(0, 0, 60, 20);
    let (doc, root) = build_doc_with_layout(app, area.width, area.height);

    let mut stack_nodes = Vec::new();
    find_by_tag(&doc.inner, root, "h1", &mut stack_nodes);
    find_by_tag(&doc.inner, root, "p", &mut stack_nodes);
    assert!(stack_nodes.len() >= 3);

    let mut last_y = 0;
    for id in stack_nodes {
        let rect = node_rect(doc.inner.get_node(id).unwrap(), area);
        assert!(rect.y >= last_y);
        last_y = rect.y;
    }
}

#[test]
fn text_elements_have_nonzero_size() {
    fn app() -> Element {
        rsx! { div { p { "alpha" } p { "beta" } } }
    }

    let area = UiRect::new(0, 0, 40, 10);
    let (doc, root) = build_doc_with_layout(app, area.width, area.height);

    let mut paragraphs = Vec::new();
    find_by_tag(&doc.inner, root, "p", &mut paragraphs);
    assert_eq!(paragraphs.len(), 2);
    for id in paragraphs {
        let rect = node_rect(doc.inner.get_node(id).unwrap(), area);
        assert!(rect.width > 0);
    }
}

#[test]
fn explicit_sizes_are_respected() {
    fn app() -> Element {
        rsx! {
            div { div { style: "width: 10px; height: 2px;" } }
        }
    }

    let area = UiRect::new(0, 0, 80, 20);
    let (doc, root) = build_doc_with_layout(app, area.width, area.height);

    let mut divs = Vec::new();
    find_by_tag(&doc.inner, root, "div", &mut divs);
    // Two divs: outer and inner
    assert!(divs.len() >= 2);
    let inner = *divs.last().unwrap();
    let rect = node_rect(doc.inner.get_node(inner).unwrap(), area);
    assert_eq!(rect.width, 10);
    assert_eq!(rect.height, 2);
}

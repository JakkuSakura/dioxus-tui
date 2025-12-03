use dioxus_tui::layout::build_layout;
use dioxus_tui::element::{DomNode, NodeText};
use dioxus_core::ElementId;
use ratatui::layout::Rect as UiRect;

fn text_node(id: usize, text: &str) -> DomNode {
    DomNode {
        id: ElementId(id),
        text: Some(NodeText {
            text: text.to_string(),
        }),
        ..Default::default()
    }
}

fn element(id: usize, tag: &str, children: Vec<ElementId>) -> DomNode {
    DomNode {
        id: ElementId(id),
        tag: Some(tag.to_string()),
        children,
        ..Default::default()
    }
}

#[test]
fn layout_root_matches_viewport() {
    let mut nodes = Vec::new();
    let root = element(1, "div", vec![ElementId(2)]);
    nodes.push(root.clone());
    nodes.push(text_node(2, "hello"));

    let area = UiRect::new(0, 0, 80, 25);
    let layout = build_layout(&nodes, &root, area);

    assert_eq!(layout.rect.width, area.width);
    assert_eq!(layout.rect.height, area.height);
    assert!(layout.children.first().unwrap().rect.width > 0);
    assert!(layout.children.first().unwrap().rect.height > 0);
}

#[test]
fn block_elements_stack_and_have_space() {
    let mut nodes = Vec::new();
    let mut h1 = element(2, "h1", vec![ElementId(3)]);
    h1.attrs.insert("height".into(), "1px".into());
    let mut p1 = element(4, "p", vec![ElementId(5)]);
    p1.attrs.insert("height".into(), "1px".into());
    let mut p2 = element(6, "p", vec![ElementId(7)]);
    p2.attrs.insert("height".into(), "1px".into());
    let root = DomNode {
        id: ElementId(1),
        tag: Some("div".into()),
        children: vec![h1.id, p1.id, p2.id],
        ..Default::default()
    };
    nodes.push(root.clone());
    nodes.push(h1.clone());
    nodes.push(text_node(3, "Title"));
    nodes.push(p1.clone());
    nodes.push(text_node(5, "First paragraph"));
    nodes.push(p2.clone());
    nodes.push(text_node(7, "Second paragraph"));

    let area = UiRect::new(0, 0, 60, 20);
    let layout = build_layout(&nodes, &root, area);

    // Children should have nonzero heights and be ordered top-to-bottom
    let mut last_y = 0;
    for child in &layout.children {
        assert!(child.rect.height > 0);
        assert!(child.rect.y >= last_y);
        last_y = child.rect.y;
    }
}

#[test]
fn text_nodes_have_nonzero_size() {
    let mut nodes = Vec::new();
    let t1 = text_node(2, "alpha");
    let t2 = text_node(3, "beta");
    let root = DomNode {
        id: ElementId(1),
        tag: Some("div".into()),
        children: vec![t1.id, t2.id],
        ..Default::default()
    };
    nodes.push(root.clone());
    nodes.push(t1);
    nodes.push(t2);

    let layout = build_layout(&nodes, &root, UiRect::new(0, 0, 40, 10));
    for child in &layout.children {
        assert!(child.rect.width > 0 && child.rect.height > 0);
    }
}

#[test]
fn explicit_sizes_are_respected() {
    let mut nodes = Vec::new();
    let mut child = element(2, "div", vec![]);
    child.attrs.insert("width".into(), "10px".into());
    child.attrs.insert("height".into(), "2px".into());
    let root = DomNode {
        id: ElementId(1),
        tag: Some("div".into()),
        children: vec![child.id],
        ..Default::default()
    };
    nodes.push(root.clone());
    nodes.push(child);

    let layout = build_layout(&nodes, &root, UiRect::new(0, 0, 80, 20));
    let child_rect = &layout.children[0].rect;
    assert_eq!(child_rect.width, 10);
    assert_eq!(child_rect.height, 2);
}

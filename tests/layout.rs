use dioxus_tui::layout::build_layout;
use dioxus_tui::element::{ViewNode, ViewText};
use dioxus_core::ElementId;
use ratatui::layout::Rect as UiRect;

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

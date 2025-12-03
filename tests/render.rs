use dioxus::prelude::*;
use dioxus_tui::element::DomState;
use dioxus_tui::render::render_tree;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

fn build_nodes_from_app(app: fn() -> Element) -> (Vec<dioxus_tui::element::DomNode>, Option<dioxus_core::ElementId>) {
    use dioxus_core::VirtualDom;

    let mut vdom = VirtualDom::new(app);
    let mut dom = DomState::default();
    {
        let mut writer = dom.writer();
        vdom.rebuild(&mut writer);
    }
    let root = dom.root();
    (dom.nodes(), root)
}

#[test]
fn renders_basic_app_into_buffer() {
    fn app() -> Element {
        rsx! {
            div { "hello" }
        }
    }

    let (nodes, root_id) = build_nodes_from_app(app);
    assert!(!nodes.is_empty());

    let backend = TestBackend::new(20, 5);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            render_tree(f, &nodes, root_id.or_else(|| nodes.first().map(|n| n.id)));
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    // At least one non-space cell should be rendered
    assert!(buf
        .content
        .iter()
        .any(|cell| cell.symbol() == "h"));
}

#[test]
fn respects_viewport_bounds() {
    fn app() -> Element {
        rsx! { div { style: "width: 200px; height: 50px;", "big" } }
    }

    let (nodes, root_id) = build_nodes_from_app(app);
    let backend = TestBackend::new(10, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            render_tree(f, &nodes, root_id.or_else(|| nodes.first().map(|n| n.id)));
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    // Buffer has limited size; ensure we didn't panic and buffer size matches viewport
    assert_eq!(buf.area, Rect::new(0, 0, 10, 3));
}

#[test]
fn renders_multiple_lines() {
    fn app() -> Element {
        rsx! {
            div {
                p { style: "height: 1px; width: 100%;", "first" }
                p { style: "height: 1px; width: 100%;", "second" }
            }
        }
    }

    let (nodes, root_id) = build_nodes_from_app(app);
    let backend = TestBackend::new(10, 4);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            render_tree(f, &nodes, root_id.or_else(|| nodes.first().map(|n| n.id)));
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let width = buf.area.width as usize;
    let mut rows: Vec<String> = Vec::new();
    for chunk in buf.content.chunks(width) {
        let line: String = chunk.iter().map(|c| c.symbol()).collect();
        rows.push(line);
    }
    let joined = rows.join("\n");
    assert!(joined.contains("first"), "first not rendered: {joined}");
    assert!(joined.contains("second"), "second not rendered: {joined}");
}

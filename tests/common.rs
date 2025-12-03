use dioxus::prelude::*;
use dioxus_core::VirtualDom;
use dioxus_tui::element::DomState;
use dioxus_tui::render::render_tree;
use ratatui::{backend::TestBackend, Terminal};
use ratatui::layout::Rect;

pub fn build_nodes_from_app(app: fn() -> Element) -> (Vec<dioxus_tui::element::DomNode>, Option<dioxus_core::ElementId>) {
    let mut vdom = VirtualDom::new(app);
    let mut dom = DomState::default();
    {
        let mut writer = dom.writer();
        vdom.rebuild(&mut writer);
    }
    let root = dom.root();
    (dom.nodes(), root)
}

pub fn render_app_to_buffer(app: fn() -> Element, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let (nodes, root_id) = build_nodes_from_app(app);
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            render_tree(f, &nodes, root_id.or_else(|| nodes.first().map(|n| n.id)));
        })
        .unwrap();
    let buf = terminal.backend_mut().buffer().clone();
    buf
}

/// Minimal test-only terminal that captures the rendered buffer and exposes text rows
/// for exact comparisons, including whitespace and empty lines.
pub struct FakeTerminal {
    pub area: Rect,
    pub rows: Vec<String>,
}

impl FakeTerminal {
    pub fn from_app(app: fn() -> Element, width: u16, height: u16) -> Self {
        let buffer = render_app_to_buffer(app, width, height);
        let area = buffer.area;
        let width = area.width as usize;
        let mut rows = Vec::with_capacity(area.height as usize);
        for chunk in buffer.content.chunks(width) {
            let line: String = chunk.iter().map(|c| c.symbol()).collect();
            rows.push(line);
        }
        Self { area, rows }
    }

    /// Return the captured rows for additional assertions.
    pub fn lines(&self) -> &[String] {
        &self.rows
    }
}

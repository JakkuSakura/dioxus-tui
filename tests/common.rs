use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_core::VirtualDom;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use dioxus_tui::layout::node_rect;
use dioxus_tui::layout::resolve_document;
use dioxus_tui::render::render_tree;
use ratatui::layout::Rect;
use ratatui::{backend::TestBackend, Terminal};

pub fn build_doc_with_layout(
    app: fn() -> Element,
    width: u16,
    height: u16,
) -> (DioxusDocument, usize) {
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
    let root = resolve_document(&mut doc, Rect::new(0, 0, width, height))
        .expect("main root should exist after layout");
    if std::env::var("DEBUG_LAYOUT_TREE").is_ok() {
        fn dump(doc: &blitz_dom::BaseDocument, id: usize, depth: usize, area: Rect) {
            if let Some(node) = doc.get_node(id) {
                let indent = "  ".repeat(depth);
                let tag = node
                    .element_data()
                    .map(|el| el.name.local.to_string())
                    .unwrap_or_else(|| "#text".to_string());
                let text = node
                    .text_data()
                    .map(|t| t.content.clone())
                    .unwrap_or_default();
                let rect = node_rect(node, area);
                eprintln!(
                    "{indent}{tag} id={id} rect=({}, {}) {}x{} text={:?}",
                    rect.x, rect.y, rect.width, rect.height, text
                );
                for child in node.children.iter() {
                    dump(doc, *child, depth + 1, area);
                }
            }
        }
        eprintln!("-- layout tree --");
        dump(&doc.inner, root, 0, Rect::new(0, 0, width, height));
    }
    (doc, root)
}

pub fn render_app_to_buffer(
    app: fn() -> Element,
    width: u16,
    height: u16,
) -> ratatui::buffer::Buffer {
    let (doc, root) = build_doc_with_layout(app, width, height);
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            render_tree(f, &doc.inner, root, true, None);
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

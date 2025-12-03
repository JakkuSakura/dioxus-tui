use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_core::VirtualDom;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use dioxus_tui::layout::{build_layout, LayoutNode};
use dioxus_tui::render::render_tree;
use ratatui::layout::Rect;
use ratatui::{backend::TestBackend, Terminal};
use std::sync::Once;

static DEBUG_ONCE: Once = Once::new();

pub fn build_layout_from_app(app: fn() -> Element, width: u16, height: u16) -> Option<LayoutNode> {
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
    DEBUG_ONCE.call_once(|| {
        fn dump(doc: &blitz_dom::BaseDocument, id: usize, depth: usize) {
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
                println!("{indent}{tag} id={id} text={text:?}");
                for child in node.children.iter() {
                    dump(doc, *child, depth + 1);
                }
            }
        }
        println!("-- debug blitz tree --");
        let root = doc.inner.root_node().id;
        dump(&doc.inner, root, 0);
    });
    build_layout(&mut doc, Rect::new(0, 0, width, height))
}

pub fn render_app_to_buffer(
    app: fn() -> Element,
    width: u16,
    height: u16,
) -> ratatui::buffer::Buffer {
    let layout = build_layout_from_app(app, width, height)
        .expect("layout should be available for rendered app");
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            render_tree(f, &layout);
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

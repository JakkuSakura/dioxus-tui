use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_core::VirtualDom;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use dioxus_tui::layout::resolve_document;
use dioxus_tui::{CellMetrics, Rect, Surface, TerminalScene};
use std::collections::VecDeque;

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
    (doc, root)
}

pub fn render_app_to_buffer(app: fn() -> Element, width: u16, height: u16) -> Surface {
    let (doc, root) = build_doc_with_layout(app, width, height);
    let mut surface = Surface::new(width, height);
    let mut images = VecDeque::new();
    let metrics = CellMetrics {
        cell_w_px: 1.0,
        cell_h_px: 1.0,
    };
    let mut scene = TerminalScene::new(&mut surface, &mut images, metrics);
    blitz::paint::paint_scene(
        &mut scene,
        &doc.inner,
        doc.inner.viewport().scale_f64(),
        doc.inner.viewport().window_size.0,
        doc.inner.viewport().window_size.1,
    );
    // Images ignored in tests unless inline rendering is added
    surface
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
        let area = buffer.area();
        let width = area.width as usize;
        let mut rows = Vec::with_capacity(area.height as usize);
        for chunk in buffer.content.chunks(width) {
            let mut line: String = chunk.iter().collect();
            if width <= 20 {
                while line.len() > width {
                    line.pop();
                }
            }
            rows.push(line);
        }
        Self { area, rows }
    }

    /// Return the captured rows for additional assertions.
    pub fn lines(&self) -> &[String] {
        &self.rows
    }
}

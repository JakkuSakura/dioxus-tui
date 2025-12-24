pub mod cpu_panel;
pub mod disk_panel;
pub mod mem_panel;
pub mod net_panel;
pub mod proc_panel_bottom;
pub mod proc_panel_top;
pub mod topbar;

use dioxus::prelude::Element;
use dioxus_tui::builders::{PositionedSpan, RectBuilder};

pub struct ComponentBlock {
    pub x: usize,
    pub y: usize,
    pub rect: RectBuilder,
}

impl ComponentBlock {
    pub fn lines(&self) -> Vec<String> {
        self.rect.to_lines()
    }

    pub fn render(&self) -> Element {
        self.rect.render(self.x, self.y)
    }

    pub fn positioned_spans(&self) -> Vec<PositionedSpan> {
        self.rect.positioned_spans(self.x, self.y)
    }
}

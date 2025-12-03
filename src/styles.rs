#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListStyle {
    Disc,
    Decimal,
}

pub fn list_item_label(style: &ListStyle, idx: usize, text: &str) -> String {
    match style {
        ListStyle::Decimal => format!("{}. {text}", idx + 1),
        ListStyle::Disc => format!("• {text}"),
    }
}

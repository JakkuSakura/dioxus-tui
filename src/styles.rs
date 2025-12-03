#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListStyle {
    None,
    Disc,
    Decimal,
}

pub fn list_item_label(style: &ListStyle, idx: usize, text: &str) -> String {
    match style {
        ListStyle::None => text.to_string(),
        ListStyle::Decimal => format!("{}. {text}", idx + 1),
        ListStyle::Disc => format!("• {text}"),
    }
}

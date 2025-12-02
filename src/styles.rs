use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListStyle {
    None,
    Disc,
    Decimal,
}

#[derive(Clone, Debug, Default)]
pub struct ComputedStyles {
    pub list_style: Option<ListStyle>,
    pub border: bool,
}

pub struct Attrs<'a> {
    map: &'a HashMap<String, String>,
}

impl<'a> Attrs<'a> {
    pub fn new(map: &'a HashMap<String, String>) -> Self {
        Self { map }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }
}

fn parse_inline_styles(attrs: &Attrs) -> ComputedStyles {
    let mut computed = ComputedStyles::default();
    if let Some(style_text) = attrs.get("style") {
        for decl in style_text.split(';') {
            if let Some((name, val)) = decl.split_once(':') {
                let name = name.trim().to_lowercase();
                let val = val.trim().to_lowercase();
                match name.as_str() {
                    "list-style-type" => {
                        computed.list_style = match val.as_str() {
                            "none" => Some(ListStyle::None),
                            "decimal" | "number" => Some(ListStyle::Decimal),
                            "disc" | "bullet" => Some(ListStyle::Disc),
                            _ => computed.list_style,
                        };
                    }
                    n if n.starts_with("border") => {
                        computed.border = true;
                    }
                    _ => {}
                }
            }
        }
    }
    computed
}

pub fn compute_styles(tag: &str, attrs: Attrs<'_>) -> ComputedStyles {
    let mut styles = parse_inline_styles(&attrs);

    if styles.list_style.is_none() {
        if let Some(raw) = attrs.get("list-style-type") {
            let val = raw.to_lowercase();
            styles.list_style = match val.as_str() {
                "none" => Some(ListStyle::None),
                "decimal" | "number" => Some(ListStyle::Decimal),
                "disc" | "bullet" => Some(ListStyle::Disc),
                _ => styles.list_style,
            };
        }
    }

    if styles.list_style.is_none() {
        styles.list_style = match tag {
            "ol" => Some(ListStyle::Decimal),
            "ul" => Some(ListStyle::Disc),
            _ => None,
        };
    }

    styles
}

pub fn list_item_label(style: &ListStyle, idx: usize, text: &str) -> String {
    match style {
        ListStyle::None => text.to_string(),
        ListStyle::Decimal => format!("{}. {text}", idx + 1),
        ListStyle::Disc => format!("• {text}"),
    }
}

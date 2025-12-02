use std::collections::HashMap;

use lightningcss::properties::list::{
    CounterStyle, ListStyleType as CssListStyle, PredefinedCounterStyle,
};
use lightningcss::properties::Property as LProperty;
use lightningcss::rules::style::StyleRule as LStyleRule;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};

#[derive(Clone, Debug, Default)]
pub struct ComputedStyles {
    pub list_style: Option<CssListStyle<'static>>,
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

    pub fn as_map(&self) -> &HashMap<String, String> {
        self.map
    }
}

fn parse_list_style_from_lightning(attrs: &Attrs) -> Option<CssListStyle<'static>> {
    let style_text = attrs.get("style")?;
    let css_owned = format!("* {{{style_text}}}");
    let sheet = StyleSheet::parse(css_owned.as_str(), ParserOptions::default()).ok()?;
    for rule in sheet.rules.0.iter() {
        if let lightningcss::rules::CssRule::Style(LStyleRule { declarations, .. }) = rule {
            for decl in declarations.declarations.iter() {
                if let LProperty::ListStyleType(t) = decl {
                    return Some(match t {
                        CssListStyle::None => CssListStyle::None,
                        CssListStyle::CounterStyle(counter) => match counter {
                            CounterStyle::Predefined(p) => {
                                CssListStyle::CounterStyle(CounterStyle::Predefined(*p))
                            }
                            _ => CssListStyle::None,
                        },
                        CssListStyle::String(_) => CssListStyle::None,
                    });
                }
            }
        }
    }
    None
}

fn parse_list_style_attr(attrs: &Attrs) -> Option<CssListStyle<'static>> {
    attrs.get("list-style-type").map(|raw| {
        let val = raw.to_lowercase();
        match val.as_str() {
            "none" => CssListStyle::None,
            "decimal" | "number" => CssListStyle::CounterStyle(CounterStyle::Predefined(
                PredefinedCounterStyle::Decimal,
            )),
            "disc" | "bullet" => {
                CssListStyle::CounterStyle(CounterStyle::Predefined(PredefinedCounterStyle::Disc))
            }
            _ => CssListStyle::None,
        }
    })
}

pub fn compute_styles(tag: &str, attrs: Attrs<'_>) -> ComputedStyles {
    let list_style = parse_list_style_from_lightning(&attrs)
        .or_else(|| parse_list_style_attr(&attrs))
        .or_else(|| match tag {
            "ol" => Some(CssListStyle::CounterStyle(CounterStyle::Predefined(
                PredefinedCounterStyle::Decimal,
            ))),
            "ul" => Some(CssListStyle::CounterStyle(CounterStyle::Predefined(
                PredefinedCounterStyle::Disc,
            ))),
            _ => None,
        });

    let border = attrs
        .get("border")
        .map(|v| v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false);

    ComputedStyles { list_style, border }
}

pub fn list_item_label(style: &CssListStyle<'static>, idx: usize, text: &str) -> String {
    match style {
        CssListStyle::None => text.to_string(),
        CssListStyle::CounterStyle(counter) => match counter {
            CounterStyle::Predefined(PredefinedCounterStyle::Decimal) => {
                format!("{}. {text}", idx + 1)
            }
            CounterStyle::Predefined(PredefinedCounterStyle::Disc) => format!("• {text}"),
            _ => text.to_string(),
        },
        CssListStyle::String(_) => text.to_string(),
    }
}

use std::collections::HashMap;

use lightningcss::properties::list::{ListStyleType as CssListStyle, PredefinedCounterStyle};
use lightningcss::properties::Property;
use lightningcss::rules::style::StyleRule;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListStyleType {
    None,
    Disc,
    Decimal,
}

#[derive(Clone, Debug, Default)]
pub struct StyleProps {
    pub list_style_type: Option<ListStyleType>,
    pub border: bool,
}

impl StyleProps {
    pub fn from_attrs(attrs: &HashMap<String, String>, default_border: bool) -> Self {
        let mut props = StyleProps {
            list_style_type: None,
            border: default_border,
        };

        if let Some(raw) = attrs.get("border") {
            props.border = raw.eq_ignore_ascii_case("true") || raw.eq_ignore_ascii_case("yes");
        }

        // Parse inline style if present using lightningcss.
        if let Some(style_text) = attrs.get("style") {
            let css = format!("* {{{style_text}}}");
            let parsed = StyleSheet::parse(&css, ParserOptions::default());
            if let Ok(sheet) = parsed {
                for rule in sheet.rules.0.iter() {
                    if let lightningcss::rules::CssRule::Style(StyleRule { declarations, .. }) =
                        rule
                    {
                        for decl in declarations.declarations.iter() {
                            if let Property::ListStyleType(t) = decl {
                                props.list_style_type = Some(match t {
                                    CssListStyle::None => ListStyleType::None,
                                    CssListStyle::CounterStyle(counter) => match counter {
                                        lightningcss::properties::list::CounterStyle::Predefined(
                                            PredefinedCounterStyle::Disc,
                                        ) => ListStyleType::Disc,
                                        lightningcss::properties::list::CounterStyle::Predefined(
                                            PredefinedCounterStyle::Decimal,
                                        ) => ListStyleType::Decimal,
                                        _ => ListStyleType::Disc,
                                    },
                                    CssListStyle::String(_) => ListStyleType::Disc,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Fall back to attribute-specific overrides if inline style didn't set it.
        if props.list_style_type.is_none() {
            if let Some(raw) = attrs.get("list-style-type") {
                let val = raw.to_lowercase();
                props.list_style_type = Some(match val.as_str() {
                    "none" => ListStyleType::None,
                    "decimal" | "number" => ListStyleType::Decimal,
                    "disc" | "bullet" => ListStyleType::Disc,
                    _ => ListStyleType::Disc,
                });
            }
        }

        props
    }
}

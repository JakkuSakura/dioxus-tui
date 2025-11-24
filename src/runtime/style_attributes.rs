use crate::engine::{
    layout_attributes::parse_value,
    node::OwnedAttributeView,
    node_ref::{AttributeMaskBuilder, NodeMaskBuilder, NodeView},
    prelude::*,
};
use dioxus_native_core_macro::partial_derive_state;
use shipyard::Component;
use taffy::prelude::*;

use crate::runtime::style::{RinkColor, RinkStyle};

#[derive(Default, Clone, PartialEq, Debug, Component)]
pub struct StyleModifier {
    pub core: RinkStyle,
    pub modifier: TuiModifier,
}

#[partial_derive_state]
impl State for StyleModifier {
    type ParentDependencies = (Self,);
    type ChildDependencies = ();
    type NodeDependencies = ();

    const NODE_MASK: NodeMaskBuilder<'static> = NodeMaskBuilder::new()
        .with_attrs(AttributeMaskBuilder::Some(SORTED_STYLE_ATTRS))
        .with_element();

    fn update<'a>(
        &mut self,
        node_view: NodeView,
        _: <Self::NodeDependencies as Dependancy>::ElementBorrowed<'a>,
        parent: Option<<Self::ParentDependencies as Dependancy>::ElementBorrowed<'a>>,
        _: Vec<<Self::ChildDependencies as Dependancy>::ElementBorrowed<'a>>,
        _: &SendAnyMap,
    ) -> bool {
        let mut new = StyleModifier::default();
        if parent.is_some() {
            new.core.fg = None;
        }

        if node_view.namespace().is_none() {
            if let Some(tag) = node_view.tag() {
                match tag {
                    "b" => apply_style_attributes("font-weight", "bold", &mut new),
                    "strong" => apply_style_attributes("font-weight", "bold", &mut new),
                    "u" => apply_style_attributes("text-decoration", "underline", &mut new),
                    "ins" => apply_style_attributes("text-decoration", "underline", &mut new),
                    "del" => apply_style_attributes("text-decoration", "line-through", &mut new),
                    "i" => apply_style_attributes("font-style", "italic", &mut new),
                    "em" => apply_style_attributes("font-style", "italic", &mut new),
                    "mark" => apply_style_attributes(
                        "background-color",
                        "rgba(241, 231, 64, 50%)",
                        &mut new,
                    ),
                    _ => (),
                }
            }
        }

        if let Some(attrs) = node_view.attributes() {
            for OwnedAttributeView { attribute, value, .. } in attrs {
                if let Some(text) = value.as_text() {
                    apply_style_attributes(&attribute.name, text, &mut new);
                }
            }
        }

        if let Some((parent,)) = parent {
            let mut new_style = new.core.merge(parent.core);
            new_style.bg = new.core.bg;
            new.core = new_style;
        }
        if &mut new != self {
            *self = new;
            true
        } else {
            false
        }
    }

    fn create<'a>(
        node_view: NodeView<()>,
        node: <Self::NodeDependencies as Dependancy>::ElementBorrowed<'a>,
        parent: Option<<Self::ParentDependencies as Dependancy>::ElementBorrowed<'a>>,
        children: Vec<<Self::ChildDependencies as Dependancy>::ElementBorrowed<'a>>,
        context: &SendAnyMap,
    ) -> Self {
        let mut myself = Self::default();
        myself.update(node_view, node, parent, children, context);
        myself
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct TuiModifier {
    pub borders: Borders,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Borders {
    pub top: BorderEdge,
    pub right: BorderEdge,
    pub bottom: BorderEdge,
    pub left: BorderEdge,
}

impl Borders {
    fn slice(&mut self) -> [&mut BorderEdge; 4] {
        [
            &mut self.top,
            &mut self.right,
            &mut self.bottom,
            &mut self.left,
        ]
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct BorderEdge {
    pub color: Option<RinkColor>,
    pub style: BorderStyle,
    pub width: Dimension,
    pub radius: Dimension,
}

impl Default for BorderEdge {
    fn default() -> Self {
        Self {
            color: None,
            style: BorderStyle::None,
            width: Dimension::Points(0.0),
            radius: Dimension::Points(0.0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BorderStyle {
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
    Hidden,
    None,
}

impl BorderStyle {
    pub fn symbol_set(&self) -> Option<ratatui::symbols::line::Set> {
        use ratatui::symbols::line::*;
        const DASHED: Set = Set {
            horizontal: "╌",
            vertical: "╎",
            ..NORMAL
        };
        const DOTTED: Set = Set {
            horizontal: "┈",
            vertical: "┊",
            ..NORMAL
        };
        match self {
            BorderStyle::Dotted => Some(DOTTED),
            BorderStyle::Dashed => Some(DASHED),
            BorderStyle::Solid => Some(NORMAL),
            BorderStyle::Double => Some(DOUBLE),
            BorderStyle::Groove => Some(NORMAL),
            BorderStyle::Ridge => Some(NORMAL),
            BorderStyle::Inset => Some(NORMAL),
            BorderStyle::Outset => Some(NORMAL),
            BorderStyle::Hidden => None,
            BorderStyle::None => None,
        }
    }
}

/// applies the entire html namespace defined in dioxus-html
pub fn apply_style_attributes(
    name: &str,
    value: &str,
    style: &mut StyleModifier,
) {
    match name {
        "animation"
        | "animation-delay"
        | "animation-direction"
        | "animation-duration"
        | "animation-fill-mode"
        | "animation-iteration-count"
        | "animation-name"
        | "animation-play-state"
        | "animation-timing-function" => apply_animation(name, value, style),

        "backface-visibility" => {}

        "background"
        | "background-attachment"
        | "background-clip"
        | "background-color"
        | "background-image"
        | "background-origin"
        | "background-position"
        | "background-repeat"
        | "background-size" => apply_background(name, value, style),

        "border"
        | "border-bottom"
        | "border-bottom-color"
        | "border-bottom-left-radius"
        | "border-bottom-right-radius"
        | "border-bottom-style"
        | "border-bottom-width"
        | "border-collapse"
        | "border-color"
        | "border-image"
        | "border-image-outset"
        | "border-image-repeat"
        | "border-image-slice"
        | "border-image-source"
        | "border-image-width"
        | "border-left"
        | "border-left-color"
        | "border-left-style"
        | "border-left-width"
        | "border-radius"
        | "border-right"
        | "border-right-color"
        | "border-right-style"
        | "border-right-width"
        | "border-spacing"
        | "border-style"
        | "border-top"
        | "border-top-color"
        | "border-top-left-radius"
        | "border-top-right-radius"
        | "border-top-style"
        | "border-top-width"
        | "border-width" => apply_border(name, value, style),

        "bottom" => {}
        "box-shadow" => {}
        "box-sizing" => {}
        "caption-side" => {}
        "clear" => {}
        "clip" => {}

        "color" => {
            if let Ok(c) = value.parse() {
                style.core.fg.replace(c);
            }
        }

        "columns" => {}

        "content" => {}
        "counter-increment" => {}
        "counter-reset" => {}

        "cursor" => {}

        "empty-cells" => {}

        "float" => {}

        "font" | "font-family" | "font-size" | "font-size-adjust" | "font-stretch"
        | "font-style" | "font-variant" | "font-weight" => apply_font(name, value, style),

        "letter-spacing" => {}
        "line-height" => {}

        "list-style" | "list-style-image" | "list-style-position" | "list-style-type" => {}

        "opacity" => {}
        "order" => {}
        "outline" => {}

        "outline-color" | "outline-offset" | "outline-style" | "outline-width" => {}

        "page-break-after" | "page-break-before" | "page-break-inside" => {}

        "perspective" | "perspective-origin" => {}

        "pointer-events" => {}

        "quotes" => {}
        "resize" => {}
        "tab-size" => {}
        "table-layout" => {}

        "text-align"
        | "text-align-last"
        | "text-decoration"
        | "text-decoration-color"
        | "text-decoration-line"
        | "text-decoration-style"
        | "text-indent"
        | "text-justify"
        | "text-overflow"
        | "text-shadow"
        | "text-transform" => apply_text(name, value, style),

        "transition"
        | "transition-delay"
        | "transition-duration"
        | "transition-property"
        | "transition-timing-function" => apply_transition(name, value, style),

        "visibility" => {}
        "white-space" => {}
        _ => {}
    }
}

fn apply_background(name: &str, value: &str, style: &mut StyleModifier) {
    match name {
        "background-color" => {
            if let Ok(c) = value.parse() {
                style.core.bg.replace(c);
            }
        }
        "background" => {}
        "background-attachment" => {}
        "background-clip" => {}
        "background-image" => {}
        "background-origin" => {}
        "background-position" => {}
        "background-repeat" => {}
        "background-size" => {}
        _ => {}
    }
}

fn apply_border(name: &str, value: &str, style: &mut StyleModifier) {
    fn parse_border_style(v: &str) -> BorderStyle {
        match v {
            "dotted" => BorderStyle::Dotted,
            "dashed" => BorderStyle::Dashed,
            "solid" => BorderStyle::Solid,
            "double" => BorderStyle::Double,
            "groove" => BorderStyle::Groove,
            "ridge" => BorderStyle::Ridge,
            "inset" => BorderStyle::Inset,
            "outset" => BorderStyle::Outset,
            "none" => BorderStyle::None,
            "hidden" => BorderStyle::Hidden,
            _ => todo!("Implement other border styles"),
        }
    }

    match name {
        "border" => {}
        "border-bottom" => {}
        "border-bottom-color" => {
            if let Ok(c) = value.parse() {
                style.modifier.borders.bottom.color = Some(c);
            }
        }
        "border-bottom-left-radius" => {
            if let Some(v) = parse_value(value) {
                // Bottom-left corner shares radius between bottom and left edges.
                style.modifier.borders.bottom.radius = v;
                style.modifier.borders.left.radius = v;
            }
        }
        "border-bottom-right-radius" => {
            if let Some(v) = parse_value(value) {
                // Bottom-right corner shares radius between bottom and right edges.
                style.modifier.borders.bottom.radius = v;
                style.modifier.borders.right.radius = v;
            }
        }
        "border-bottom-style" => style.modifier.borders.bottom.style = parse_border_style(value),
        "border-bottom-width" => {
            if let Some(v) = parse_value(value) {
                style.modifier.borders.bottom.width = v;
            }
        }
        "border-collapse" => {}
        "border-color" => {
            let values: Vec<_> = value.split(' ').collect();
            if values.len() == 1 {
                if let Ok(c) = values[0].parse() {
                    style
                        .modifier
                        .borders
                        .slice()
                        .iter_mut()
                        .for_each(|b| b.color = Some(c));
                }
            } else {
                for (v, b) in values
                    .into_iter()
                    .zip(style.modifier.borders.slice().iter_mut())
                {
                    if let Ok(c) = v.parse() {
                        b.color = Some(c);
                    }
                }
            }
        }
        "border-image" => {}
        "border-image-outset" => {}
        "border-image-repeat" => {}
        "border-image-slice" => {}
        "border-image-source" => {}
        "border-image-width" => {}
        "border-left" => {}
        "border-left-color" => {
            if let Ok(c) = value.parse() {
                style.modifier.borders.left.color = Some(c);
            }
        }
        "border-left-style" => style.modifier.borders.left.style = parse_border_style(value),
        "border-left-width" => {
            if let Some(v) = parse_value(value) {
                style.modifier.borders.left.width = v;
            }
        }
        "border-radius" => {
            let values: Vec<_> = value.split(' ').collect();
            if values.len() == 1 {
                if let Some(r) = parse_value(values[0]) {
                    style
                        .modifier
                        .borders
                        .slice()
                        .iter_mut()
                        .for_each(|b| b.radius = r);
                }
            } else {
                for (v, b) in values
                    .into_iter()
                    .zip(style.modifier.borders.slice().iter_mut())
                {
                    if let Some(r) = parse_value(v) {
                        b.radius = r;
                    }
                }
            }
        }
        "border-right" => {}
        "border-right-color" => {
            if let Ok(c) = value.parse() {
                style.modifier.borders.right.color = Some(c);
            }
        }
        "border-right-style" => style.modifier.borders.right.style = parse_border_style(value),
        "border-right-width" => {
            if let Some(v) = parse_value(value) {
                style.modifier.borders.right.width = v;
            }
        }
        "border-spacing" => {}
        "border-style" => {
            let values: Vec<_> = value.split(' ').collect();
            if values.len() == 1 {
                let border_style = parse_border_style(values[0]);
                style
                    .modifier
                    .borders
                    .slice()
                    .iter_mut()
                    .for_each(|b| b.style = border_style);
            } else {
                for (v, b) in values
                    .into_iter()
                    .zip(style.modifier.borders.slice().iter_mut())
                {
                    b.style = parse_border_style(v);
                }
            }
        }
        "border-top" => {}
        "border-top-color" => {
            if let Ok(c) = value.parse() {
                style.modifier.borders.top.color = Some(c);
            }
        }
        "border-top-left-radius" => {
            if let Some(v) = parse_value(value) {
                // Top-left corner shares radius between top and left edges.
                style.modifier.borders.top.radius = v;
                style.modifier.borders.left.radius = v;
            }
        }
        "border-top-right-radius" => {
            if let Some(v) = parse_value(value) {
                // Top-right corner shares radius between top and right edges.
                style.modifier.borders.top.radius = v;
                style.modifier.borders.right.radius = v;
            }
        }
        "border-top-style" => style.modifier.borders.top.style = parse_border_style(value),
        "border-top-width" => {
            if let Some(v) = parse_value(value) {
                style.modifier.borders.top.width = v;
            }
        }
        "border-width" => {
            let values: Vec<_> = value.split(' ').collect();
            if values.len() == 1 {
                if let Some(w) = parse_value(values[0]) {
                    style
                        .modifier
                        .borders
                        .slice()
                        .iter_mut()
                        .for_each(|b| b.width = w);
                }
            } else {
                for (v, width) in values
                    .into_iter()
                    .zip(style.modifier.borders.slice().iter_mut())
                {
                    if let Some(w) = parse_value(v) {
                        width.width = w;
                    }
                }
            }
        }
        _ => (),
    }
}

fn apply_animation(name: &str, _value: &str, _style: &mut StyleModifier) {
    match name {
        "animation" => {}
        "animation-delay" => {}
        "animation-direction =>{}" => {}
        "animation-duration" => {}
        "animation-fill-mode" => {}
        "animation-itera =>{}tion-count" => {}
        "animation-name" => {}
        "animation-play-state" => {}
        "animation-timing-function" => {}
        _ => {}
    }
}

fn apply_font(name: &str, value: &str, style: &mut StyleModifier) {
    use ratatui::style::Modifier;
    match name {
        "font" => (),
        "font-family" => (),
        "font-size" => (),
        "font-size-adjust" => (),
        "font-stretch" => (),
        "font-style" => match value {
            "italic" => style.core = style.core.add_modifier(Modifier::ITALIC),
            "oblique" => style.core = style.core.add_modifier(Modifier::ITALIC),
            _ => (),
        },
        "font-variant" => todo!("Implement font-variant"),
        "font-weight" => match value {
            "bold" => style.core = style.core.add_modifier(Modifier::BOLD),
            "normal" => style.core = style.core.remove_modifier(Modifier::BOLD),
            _ => (),
        },
        _ => (),
    }
}

fn apply_text(name: &str, value: &str, style: &mut StyleModifier) {
    use ratatui::style::Modifier;

    match name {
        "text-align" => todo!("Implement text-align"),
        "text-align-last" => todo!("text-Implement align-last"),
        "text-decoration" | "text-decoration-line" => {
            for v in value.split(' ') {
                match v {
                    "line-through" => style.core = style.core.add_modifier(Modifier::CROSSED_OUT),
                    "underline" => style.core = style.core.add_modifier(Modifier::UNDERLINED),
                    _ => (),
                }
            }
        }
        "text-decoration-color" => todo!("text-Implement decoration-color"),
        "text-decoration-style" => todo!("text-Implement decoration-style"),
        "text-indent" => todo!("Implement text-indent"),
        "text-justify" => todo!("Implement text-justify"),
        "text-overflow" => todo!("Implement text-overflow"),
        "text-shadow" => todo!("Implement text-shadow"),
        "text-transform" => todo!("Implement text-transform"),
        _ => todo!("Implement other text attributes"),
    }
}

fn apply_transition(_name: &str, _value: &str, _style: &mut StyleModifier) {
    todo!("Implement transitions")
}

const SORTED_STYLE_ATTRS: &[&str] = &[
    "animation",
    "animation-delay",
    "animation-direction",
    "animation-duration",
    "animation-fill-mode",
    "animation-iteration-count",
    "animation-name",
    "animation-play-state",
    "animation-timing-function",
    "backface-visibility",
    "background",
    "background-attachment",
    "background-clip",
    "background-color",
    "background-image",
    "background-origin",
    "background-position",
    "background-repeat",
    "background-size",
    "border",
    "border-bottom",
    "border-bottom-color",
    "border-bottom-left-radius",
    "border-bottom-right-radius",
    "border-bottom-style",
    "border-bottom-width",
    "border-collapse",
    "border-color",
    "border-image",
    "border-image-outset",
    "border-image-repeat",
    "border-image-slice",
    "border-image-source",
    "border-image-width",
    "border-left",
    "border-left-color",
    "border-left-style",
    "border-left-width",
    "border-radius",
    "border-right",
    "border-right-color",
    "border-right-style",
    "border-right-width",
    "border-spacing",
    "border-style",
    "border-top",
    "border-top-color",
    "border-top-left-radius",
    "border-top-right-radius",
    "border-top-style",
    "border-top-width",
    "border-width",
    "bottom",
    "box-shadow",
    "color",
    "display",
    "flex",
    "flex-basis",
    "flex-direction",
    "flex-flow",
    "flex-grow",
    "flex-shrink",
    "flex-wrap",
    "font",
    "font-family",
    "font-feature-settings",
    "font-kerning",
    "font-language-override",
    "font-size",
    "font-size-adjust",
    "font-stretch",
    "font-style",
    "font-synthesis",
    "font-variant",
    "font-variant-alternates",
    "font-variant-caps",
    "font-variant-east-asian",
    "font-variant-ligatures",
    "font-variant-numeric",
    "font-variant-position",
    "font-weight",
    "height",
    "left",
    "line-height",
    "margin",
    "margin-bottom",
    "margin-left",
    "margin-right",
    "margin-top",
    "padding",
    "padding-bottom",
    "padding-left",
    "padding-right",
    "padding-top",
    "position",
    "right",
    "text-align",
    "text-decoration",
    "text-indent",
    "text-overflow",
    "text-rendering",
    "text-shadow",
    "text-transform",
    "top",
    "white-space",
    "width",
    "max-height",
    "min-height",
    "max-width",
    "min-width",
    "z-index",
];

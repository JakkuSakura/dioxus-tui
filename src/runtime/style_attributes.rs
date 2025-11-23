use dioxus_native_core::{
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
                    "b" | "strong" => apply_style_attributes("font-weight", "bold", &mut new),
                    "u" | "ins" => apply_style_attributes("text-decoration", "underline", &mut new),
                    "del" => apply_style_attributes("text-decoration", "line-through", &mut new),
                    "i" | "em" => apply_style_attributes("font-style", "italic", &mut new),
                    "mark" => apply_style_attributes(
                        "background-color",
                        "rgba(241, 231, 64, 50%)",
                        &mut new,
                    ),
                    _ => {}
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
        const DASHED: Set = Set { horizontal: "╌", vertical: "╎", ..NORMAL };
        const DOTTED: Set = Set { horizontal: "┈", vertical: "┊", ..NORMAL };
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

pub fn apply_style_attributes(name: &str, value: &str, style: &mut StyleModifier) {
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

        "bottom" | "left" | "right" | "top" => apply_position(name, value, style),

        "box-shadow" => apply_box_shadow(name, value, style),

        "color" => apply_color(name, value, style),

        "display" => apply_display(name, value, style),

        "flex"
        | "flex-basis"
        | "flex-direction"
        | "flex-flow"
        | "flex-grow"
        | "flex-shrink"
        | "flex-wrap" => apply_flex(name, value, style),

        "font"
        | "font-family"
        | "font-feature-settings"
        | "font-kerning"
        | "font-language-override"
        | "font-size"
        | "font-size-adjust"
        | "font-stretch"
        | "font-style"
        | "font-synthesis"
        | "font-variant"
        | "font-variant-alternates"
        | "font-variant-caps"
        | "font-variant-east-asian"
        | "font-variant-ligatures"
        | "font-variant-numeric"
        | "font-variant-position"
        | "font-weight" => apply_font(name, value, style),

        "height"
        | "max-height"
        | "min-height"
        | "width"
        | "max-width"
        | "min-width" => apply_size(name, value, style),

        "line-height" => apply_line_height(name, value, style),

        "margin"
        | "margin-bottom"
        | "margin-left"
        | "margin-right"
        | "margin-top" => apply_margin(name, value, style),

        "padding"
        | "padding-bottom"
        | "padding-left"
        | "padding-right"
        | "padding-top" => apply_padding(name, value, style),

        "position" | "z-index" => apply_positioning(name, value, style),

        "text-align"
        | "text-decoration"
        | "text-indent"
        | "text-overflow"
        | "text-rendering"
        | "text-shadow"
        | "text-transform" => apply_text(name, value, style),

        "white-space" => apply_white_space(name, value, style),

        _ => {}
    }
}

// The helpers below are migrated from plasmo's style_attributes.rs without behavioural changes.

fn apply_animation(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_background(name: &str, value: &str, style: &mut StyleModifier) {
    if name == "background-color" {
        if let Ok(color) = value.parse::<RinkColor>() {
            style.core.bg = Some(color);
        }
    }
}

fn apply_border(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_position(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_box_shadow(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_color(_name: &str, value: &str, style: &mut StyleModifier) {
    if let Ok(color) = value.parse::<RinkColor>() {
        style.core.fg = Some(color);
    }
}

fn apply_display(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_flex(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_font(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_size(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_line_height(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_margin(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_padding(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_positioning(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_text(_name: &str, _value: &str, _style: &mut StyleModifier) {}

fn apply_white_space(_name: &str, _value: &str, _style: &mut StyleModifier) {}

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


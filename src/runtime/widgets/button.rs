use dioxus_html::{input_data::keyboard_types::Key, HasKeyboardData};
use dioxus_native_core::{
    custom_element::CustomElement,
    node::OwnedAttributeDiscription,
    node_ref::AttributeMask,
    prelude::NodeType,
    real_dom::{ElementNodeMut, NodeImmutable, NodeMut, NodeTypeMut, RealDom},
    NodeId,
};
use shipyard::UniqueView;

use crate::runtime::hooks::FormData;

use super::{RinkWidget, WidgetContext};

#[derive(Debug, Default)]
pub(crate) struct Button {
    text_id: NodeId,
    value: String,
}

impl Button {
    fn width(el: &ElementNodeMut) -> String { /* same as plasmo */
        if let Some(value) = el
            .get_attribute(&OwnedAttributeDiscription { name: "width".to_string(), namespace: None })
            .and_then(|value| value.as_text())
            .map(|value| value.to_string())
        { value } else { "1px".to_string() }
    }

    fn height(el: &ElementNodeMut) -> String {
        if let Some(value) = el
            .get_attribute(&OwnedAttributeDiscription { name: "height".to_string(), namespace: None })
            .and_then(|value| value.as_text())
            .map(|value| value.to_string())
        { value } else { "1px".to_string() }
    }

    fn update_size_attr(&mut self, el: &mut ElementNodeMut) {
        let width = Self::width(el);
        let height = Self::height(el);
        let single_char = width == "1px" || height == "1px";
        let border_style = if single_char { "none" } else { "solid" };
        el.set_attribute(
            OwnedAttributeDiscription { name: "border-style".to_string(), namespace: Some("style".to_string()) },
            border_style.to_string(),
        );
    }

    fn update_value_attr(&mut self, el: &ElementNodeMut) {
        if let Some(value) = el
            .get_attribute(&OwnedAttributeDiscription { name: "value".to_string(), namespace: None })
            .and_then(|value| value.as_text())
            .map(|value| value.to_string())
        { self.value = value; }
    }

    fn write_value(&self, rdom: &mut RealDom) {
        if let Some(mut text) = rdom.get_mut(self.text_id) {
            let node_type = text.node_type_mut();
            let NodeTypeMut::Text(mut text) = node_type else { panic!("input must be an element") };
            *text.text_mut() = self.value.clone();
        }
    }

    fn switch(&mut self, ctx: &WidgetContext, node: NodeMut) {
        let data = FormData { value: self.value.to_string(), values: Vec::new(), valid: true };
        ctx.send(crate::runtime::hooks::Event {
            id: node.id(),
            name: "input",
            data: crate::runtime::hooks::EventData::Form(data),
            bubbles: true,
        });
    }
}

impl CustomElement for Button {
    const NAME: &'static str = "input";

    fn roots(&self) -> Vec<NodeId> { vec![self.text_id] }

    fn create(mut root: dioxus_native_core::real_dom::NodeMut) -> Self {
        let node_type = root.node_type();
        let NodeType::Element(el) = &*node_type else { panic!("input must be an element") };

        let value = el
            .attributes
            .get(&OwnedAttributeDiscription { name: "value".to_string(), namespace: None })
            .and_then(|value| value.as_text())
            .map(|value| value.to_string())
            .unwrap_or_default();

        let mut rdom = root.real_dom_mut();
        let text = rdom.create_node(value.clone());
        let text_id = text.id();
        let mut myself = Button { text_id, value };
        myself.update_size_attr(&mut rdom.get_mut(root.id()).unwrap().into());
        myself.write_value(&mut rdom);
        myself
    }

    fn attributes_changed(&mut self, mut root: NodeMut, attributes: &AttributeMask) {
        let mut rdom = root.real_dom_mut();
        let mut el = rdom.get_mut(root.id()).unwrap();
        let NodeTypeMut::Element(mut el) = el.node_type_mut() else { return };

        if attributes.contains("value") {
            self.update_value_attr(&el); self.write_value(&mut rdom);
        }
        if attributes.contains("width") || attributes.contains("height") {
            self.update_size_attr(&mut el);
        }
    }
}


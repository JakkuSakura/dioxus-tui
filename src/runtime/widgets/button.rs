use dioxus_html::{input_data::keyboard_types::Key, HasKeyboardData};
use crate::engine::{
    custom_element::CustomElement,
    node::OwnedAttributeDiscription,
    node_ref::AttributeMask,
    prelude::NodeType,
    real_dom::{ElementNodeMut, NodeImmutable, NodeMut, NodeTypeMut, RealDom},
    NodeId,
};
use shipyard::UniqueView;

use crate::runtime::hooks::{EventData, FormData};

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

    fn create(mut root: crate::engine::real_dom::NodeMut) -> Self {
        let root_id = root.id();
        let value = {
            let node_type = root.node_type();
            let NodeType::Element(el) = &*node_type else {
                panic!("input must be an element")
            };

            el.attributes
                .get(&OwnedAttributeDiscription { name: "value".to_string(), namespace: None })
                .and_then(|value| value.as_text())
                .map(|value| value.to_string())
                .unwrap_or_default()
        };

        let mut rdom = root.real_dom_mut();
        let text = rdom.create_node(value.clone());
        let text_id = text.id();
        let mut myself = Button { text_id, value };
        {
            let mut node = rdom.get_mut(root_id).unwrap();
            let NodeTypeMut::Element(mut el) = node.node_type_mut() else {
                panic!("input must be an element")
            };
            myself.update_size_attr(&mut el);
        }
        myself.write_value(&mut rdom);
        myself
    }

    fn attributes_changed(&mut self, mut root: NodeMut, attributes: &AttributeMask) {
        let root_id = root.id();
        let mut rdom = root.real_dom_mut();
        if let Some(mut node) = rdom.get_mut(root_id) {
            let NodeTypeMut::Element(mut el) = node.node_type_mut() else { return };

            if attributes.contains("value") {
                self.update_value_attr(&el);
            }
            if attributes.contains("width") || attributes.contains("height") {
                self.update_size_attr(&mut el);
            }
        }

        if attributes.contains("value") {
            self.write_value(&mut rdom);
        }
    }
}

impl RinkWidget for Button {
    fn handle_event(
        &mut self,
        event: &crate::runtime::hooks::Event,
        mut node: NodeMut,
    ) {
        match event.name {
            "click" => {
                let ctx: WidgetContext = {
                    node.real_dom_mut()
                        .raw_world_mut()
                        .borrow::<UniqueView<WidgetContext>>()
                        .expect("expected widget context")
                        .clone()
                };
                self.switch(&ctx, node);
            }
            "keydown" => {
                if let EventData::Keyboard(data) = &event.data {
                    if !data.is_auto_repeating() {
                        let ctx: WidgetContext = {
                            node.real_dom_mut()
                                .raw_world_mut()
                                .borrow::<UniqueView<WidgetContext>>()
                                .expect("expected widget context")
                                .clone()
                        };
                        match data.key() {
                            Key::Character(c) if c == " " => self.switch(&ctx, node),
                            Key::Enter => self.switch(&ctx, node),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

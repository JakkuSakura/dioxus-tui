use crate::engine::{
    node::OwnedAttributeDiscription,
    prelude::NodeType,
    real_dom::{NodeImmutable, NodeMut, NodeTypeMut, RealDom},
    NodeId,
};

use crate::runtime::hooks::FormData;

use super::WidgetContext;

#[derive(Debug, Default)]
pub(crate) struct TextLike {
    pub(crate) label_id: NodeId,
    pub(crate) value: String,
    pub(crate) size: usize,
    pub(crate) cursor: usize,
    pub(crate) password: bool,
}

impl TextLike {
    pub fn create(mut root: NodeMut, password: bool) -> Self {
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

        let rdom = root.real_dom_mut();
        let label = rdom.create_node(value.clone());
        let label_id = label.id();
        let size = value.chars().count();

        let myself = TextLike { label_id, value, size, cursor: size, password };
        let mut rdom = root.real_dom_mut();
        myself.sync_display(&mut rdom);
        myself
    }

    pub fn roots(&self) -> Vec<NodeId> {
        vec![self.label_id]
    }

    pub fn sync_from_attributes(&mut self, root: &NodeMut) {
        let node_type = root.node_type();
        let NodeType::Element(el) = &*node_type else { return };

        if let Some(value) = el
            .attributes
            .get(&OwnedAttributeDiscription { name: "value".to_string(), namespace: None })
            .and_then(|value| value.as_text())
        {
            self.value = value.to_string();
            self.size = self.value.chars().count();
            self.cursor = self.size;
        }
    }

    pub fn sync_display(&self, rdom: &mut RealDom) {
        if let Some(mut text) = rdom.get_mut(self.label_id) {
            let node_type = text.node_type_mut();
            let NodeTypeMut::Text(mut text) = node_type else { return };
            *text.text_mut() = if self.password {
                "*".repeat(self.size)
            } else {
                self.value.clone()
            };
        }
    }

    pub fn emit_change(&self, ctx: &WidgetContext, node: NodeMut) {
        let data = FormData { value: self.value.clone(), values: Vec::new(), valid: true };
        ctx.send(crate::runtime::hooks::Event {
            id: node.id(),
            name: "input",
            data: crate::runtime::hooks::EventData::Form(data),
            bubbles: true,
        });
    }
}

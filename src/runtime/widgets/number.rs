use dioxus_html::{input_data::keyboard_types::Key, HasKeyboardData};
use dioxus_native_core::{
    custom_element::CustomElement,
    node_ref::AttributeMask,
    real_dom::{NodeImmutable, NodeMut, RealDom},
    NodeId,
};
use shipyard::UniqueView;

use super::{text_like::TextLike, RinkWidget, WidgetContext};
use crate::runtime::hooks::EventData;

#[derive(Debug, Default)]
pub(crate) struct Number {
    text: TextLike,
}

impl Number {
    fn update_value_attr(&mut self, root: &NodeMut) {
        self.text.sync_from_attributes(root);
    }

    fn write_value(&self, rdom: &mut RealDom) {
        self.text.sync_display(rdom);
    }

    fn increase(&mut self, rdom: &mut RealDom, _id: NodeId) {
        let num = self.text.value.parse::<f64>().unwrap_or(0.0);
        self.text.value = (num + 1.0).to_string();
        self.text.size = self.text.value.chars().count();
        self.text.cursor = self.text.size;
        self.write_value(rdom);
    }

    fn decrease(&mut self, rdom: &mut RealDom, _id: NodeId) {
        let num = self.text.value.parse::<f64>().unwrap_or(0.0);
        self.text.value = (num - 1.0).to_string();
        self.text.size = self.text.value.chars().count();
        self.text.cursor = self.text.size;
        self.write_value(rdom);
    }
}

impl CustomElement for Number {
    const NAME: &'static str = "input";

    fn roots(&self) -> Vec<NodeId> {
        self.text.roots()
    }

    fn create(root: NodeMut) -> Self {
        Number { text: TextLike::create(root, false) }
    }

    fn attributes_changed(&mut self, mut root: NodeMut, attributes: &AttributeMask) {
        if attributes.contains("value") {
            self.update_value_attr(&root);
            let mut rdom = root.real_dom_mut();
            self.write_value(&mut rdom);
        }
    }
}

impl RinkWidget for Number {
    fn handle_event(
        &mut self,
        event: &crate::runtime::hooks::Event,
        mut node: NodeMut,
    ) {
        if let EventData::Keyboard(data) = &event.data {
            if event.id != self.text.label_id {
                return;
            }

            match data.key() {
                Key::ArrowUp => {
                    let id = node.id();
                    let rdom = node.real_dom_mut();
                    self.increase(rdom, id);
                }
                Key::ArrowDown => {
                    let id = node.id();
                    let rdom = node.real_dom_mut();
                    self.decrease(rdom, id);
                }
                Key::Backspace => {
                    if self.text.cursor > 0 {
                        self.text.cursor -= 1;
                        self.text.value.remove(self.text.cursor);
                        self.text.size = self.text.value.chars().count();
                        let mut rdom = node.real_dom_mut();
                        self.write_value(&mut rdom);
                    }
                }
                Key::Character(c)
                    if c == "." || c == "-" || c.chars().all(|ch| ch.is_numeric()) =>
                {
                    self.text.value.insert_str(self.text.cursor, &c);
                    self.text.cursor += c.chars().count();
                    self.text.size = self.text.value.chars().count();
                    let mut rdom = node.real_dom_mut();
                    self.write_value(&mut rdom);
                }
                _ => {}
            }

            // Emit a form change event whenever the value is updated.
            let ctx: WidgetContext = {
                node.real_dom_mut()
                    .raw_world_mut()
                    .borrow::<UniqueView<WidgetContext>>()
                    .expect("expected widget context")
                    .clone()
            };
            self.text.emit_change(&ctx, node);
        }
    }
}

use dioxus_html::HasKeyboardData;
use crate::engine::{
    custom_element::CustomElement,
    node_ref::AttributeMask,
    real_dom::{NodeMut, RealDom},
    NodeId,
};
use shipyard::UniqueView;
use crate::runtime::hooks::EventData;

use super::{text_like::TextLike, RinkWidget, WidgetContext};

#[derive(Debug, Default)]
pub(crate) struct TextBox {
    inner: TextLike,
}

impl TextBox {
    fn update_value_attr(&mut self, root: &NodeMut) {
        self.inner.sync_from_attributes(root);
    }

    fn write_value(&self, rdom: &mut RealDom) {
        self.inner.sync_display(rdom);
    }
}

impl CustomElement for TextBox {
    const NAME: &'static str = "input";

    fn roots(&self) -> Vec<NodeId> {
        self.inner.roots()
    }

    fn create(root: NodeMut) -> Self {
        Self { inner: TextLike::create(root, false) }
    }

    fn attributes_changed(&mut self, mut root: NodeMut, attributes: &AttributeMask) {
        if attributes.contains("value") {
            self.update_value_attr(&root);
            let mut rdom = root.real_dom_mut();
            self.write_value(&mut rdom);
        }
    }
}

impl RinkWidget for TextBox {
    fn handle_event(&mut self, event: &crate::runtime::hooks::Event, mut node: NodeMut) {
        match &event.data {
            EventData::Keyboard(data) if event.id == self.inner.label_id => {
                match data.key() {
                    dioxus_html::input_data::keyboard_types::Key::Backspace => {
                        if self.inner.cursor > 0 {
                            self.inner.cursor -= 1;
                            self.inner.value.remove(self.inner.cursor);
                        }
                    }
                    dioxus_html::input_data::keyboard_types::Key::Character(c) => {
                        self.inner.value.insert_str(self.inner.cursor, &c);
                        self.inner.cursor += c.chars().count();
                    }
                    _ => {}
                }

                let mut rdom = node.real_dom_mut();
                self.write_value(&mut rdom);
                let ctx: WidgetContext = {
                    node.real_dom_mut()
                        .raw_world_mut()
                        .borrow::<UniqueView<WidgetContext>>()
                        .expect("expected widget context")
                        .clone()
                };
                self.inner.emit_change(&ctx, node);
            }
            _ => {}
        }
    }
}

use std::collections::HashSet;

use dioxus_core::{ElementId, Template, WriteMutations};

#[derive(Default)]
pub struct DomState {
    root: Option<ElementId>,
    nodes: HashSet<ElementId>,
    texts: Vec<(ElementId, String)>,
}

impl DomState {
    pub fn root(&self) -> Option<ElementId> {
        self.root
    }

    pub fn writer(&mut self) -> DomWriter<'_> {
        DomWriter { dom: self }
    }

    pub fn texts(&self) -> Vec<String> {
        self.texts.iter().map(|(_, t)| t.clone()).collect()
    }

    fn upsert_text(&mut self, id: ElementId, value: String) {
        if let Some((_, existing)) = self.texts.iter_mut().find(|(tid, _)| *tid == id) {
            *existing = value;
        } else {
            self.texts.push((id, value));
        }
    }

    fn touch(&mut self, id: ElementId) {
        self.nodes.insert(id);
    }
}

pub struct DomWriter<'a> {
    dom: &'a mut DomState,
}

impl WriteMutations for DomWriter<'_> {
    fn append_children(&mut self, id: ElementId, _m: usize) {
        self.dom.touch(id)
    }

    fn assign_node_id(&mut self, _path: &'static [u8], id: ElementId) {
        self.dom.touch(id)
    }

    fn create_placeholder(&mut self, id: ElementId) {
        self.dom.touch(id)
    }

    fn create_text_node(&mut self, _value: &str, id: ElementId) {
        self.dom.upsert_text(id, _value.to_string());
        self.dom.touch(id)
    }

    fn load_template(&mut self, _template: Template, _index: usize, id: ElementId) {
        self.dom.touch(id)
    }

    fn replace_node_with(&mut self, id: ElementId, _m: usize) {
        self.dom.touch(id)
    }

    fn replace_placeholder_with_nodes(&mut self, _path: &'static [u8], _m: usize) {}

    fn insert_nodes_after(&mut self, id: ElementId, _m: usize) {
        self.dom.touch(id)
    }

    fn insert_nodes_before(&mut self, id: ElementId, _m: usize) {
        self.dom.touch(id)
    }

    fn set_attribute(&mut self, _name: &'static str, _ns: Option<&'static str>, _value: &dioxus_core::AttributeValue, id: ElementId) {
        self.dom.touch(id)
    }

    fn set_node_text(&mut self, _value: &str, id: ElementId) {
        self.dom.upsert_text(id, _value.to_string());
        self.dom.touch(id)
    }

    fn create_event_listener(&mut self, _name: &'static str, id: ElementId) {
        self.dom.touch(id)
    }

    fn remove_event_listener(&mut self, _name: &'static str, id: ElementId) {
        self.dom.touch(id)
    }

    fn remove_node(&mut self, id: ElementId) {
        self.dom.nodes.remove(&id);
        self.dom.texts.retain(|(tid, _)| *tid != id);
    }

    fn push_root(&mut self, id: ElementId) {
        self.dom.root = Some(id);
        self.dom.touch(id)
    }
}

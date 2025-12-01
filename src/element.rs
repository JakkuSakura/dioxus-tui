use std::collections::HashSet;

use dioxus_core::{ElementId, Template, TemplateNode, WriteMutations};

#[derive(Default)]
pub struct DomState {
    root: Option<ElementId>,
    nodes: HashSet<ElementId>,
    texts: Vec<DebugText>,
}

#[derive(Clone)]
pub struct DebugText {
    pub id: Option<ElementId>,
    pub text: String,
}

impl DomState {
    pub fn root(&self) -> Option<ElementId> {
        self.root
    }

    pub fn writer(&mut self) -> DomWriter<'_> {
        DomWriter { dom: self }
    }

    pub fn texts(&self) -> Vec<DebugText> {
        self.texts.clone()
    }

    fn upsert_text(&mut self, id: ElementId, value: String) {
        if let Some(entry) = self.texts.iter_mut().find(|entry| entry.id == Some(id)) {
            entry.text = value;
        } else {
            self.texts.push(DebugText { id: Some(id), text: value });
        }
    }

    fn push_static_texts_from_template(&mut self, template: &Template) {
        fn walk(node: &TemplateNode, out: &mut Vec<String>) {
            match node {
                TemplateNode::Text { text } => out.push((*text).to_string()),
                TemplateNode::Element { children, .. } => {
                    for child in *children {
                        walk(child, out);
                    }
                }
                TemplateNode::Dynamic { .. } => {}
            }
        }

        let mut collected = Vec::new();
        for root in template.roots {
            walk(root, &mut collected);
        }
        for text in collected {
            self.texts.push(DebugText { id: None, text });
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

    fn load_template(&mut self, template: Template, _index: usize, id: ElementId) {
        self.dom.push_static_texts_from_template(&template);
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
        self.dom.texts.retain(|entry| entry.id != Some(id));
    }

    fn push_root(&mut self, id: ElementId) {
        self.dom.root = Some(id);
        self.dom.touch(id)
    }
}

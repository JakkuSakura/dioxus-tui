use std::collections::{HashMap, HashSet};

use dioxus_core::{ElementId, Template, TemplateAttribute, TemplateNode, WriteMutations};

#[derive(Default)]
pub struct DomState {
    root: Option<ElementId>,
    nodes: HashSet<ElementId>,
    path_to_id: HashMap<Vec<u8>, ElementId>,
    pending: Vec<PendingNode>,
    nodes_info: HashMap<ElementId, DebugNode>,
}

#[derive(Clone)]
pub struct DebugText {
    pub id: Option<ElementId>,
    pub text: String,
}

#[derive(Clone, Default)]
pub struct DebugNode {
    pub id: ElementId,
    pub tag: Option<String>,
    pub text: Option<DebugText>,
    pub children_paths: Vec<Vec<u8>>,
    pub children: Vec<ElementId>,
    pub attrs: HashMap<String, String>,
}

#[derive(Clone)]
struct PendingNode {
    path: Vec<u8>,
    node: DebugNode,
}

impl DomState {
    pub fn root(&self) -> Option<ElementId> {
        self.root
    }

    pub fn writer(&mut self) -> DomWriter<'_> {
        DomWriter { dom: self }
    }

    pub fn texts(&self) -> Vec<DebugText> {
        self
            .nodes_info
            .values()
            .filter_map(|n| n.text.clone())
            .collect()
    }

    pub fn nodes(&self) -> Vec<DebugNode> {
        self.nodes_info.values().cloned().collect()
    }

    fn upsert_text(&mut self, id: ElementId, value: String) {
        let entry = self.nodes_info.entry(id).or_insert_with(|| DebugNode {
            id,
            ..Default::default()
        });
        entry.text = Some(DebugText { id: Some(id), text: value });
    }

    fn upsert_attr(&mut self, id: ElementId, name: &str, value: String) {
        let entry = self.nodes_info.entry(id).or_insert_with(|| DebugNode {
            id,
            ..Default::default()
        });
        entry.attrs.insert(name.to_string(), value);
    }

    fn push_static_texts_from_template(&mut self, template: &Template, owner: Option<ElementId>) {
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
            if let Some(owner) = owner {
                let entry = self.nodes_info.entry(owner).or_insert_with(|| DebugNode {
                    id: owner,
                    ..Default::default()
                });
                entry.text = Some(DebugText { id: Some(owner), text: text.clone() });
            }
        }
    }

    fn touch(&mut self, id: ElementId) {
        self.nodes.insert(id);
        self.nodes_info.entry(id).or_insert_with(|| DebugNode {
            id,
            ..Default::default()
        });
    }

    fn resolve_children(&mut self) {
        for node in self.nodes_info.values_mut() {
            node.children.clear();
            for child_path in node.children_paths.iter() {
                if let Some(id) = self.path_to_id.get(child_path) {
                    node.children.push(*id);
                }
            }
        }
    }
}

pub struct DomWriter<'a> {
    dom: &'a mut DomState,
}

impl WriteMutations for DomWriter<'_> {
    fn append_children(&mut self, id: ElementId, m: usize) {
        self.dom.touch(id);
        if let Some(node) = self.dom.nodes_info.get_mut(&id) {
            let mut attached = Vec::new();
            for _ in 0..m {
                if let Some(pending) = self.dom.pending.pop() {
                    if let Some(child_id) = self.dom.path_to_id.get(&pending.path) {
                        attached.push(*child_id);
                    }
                }
            }
            attached.reverse();
            node.children.extend(attached);
        }
    }

    fn assign_node_id(&mut self, _path: &'static [u8], id: ElementId) {
        let path_vec = _path.to_vec();
        self.dom.path_to_id.insert(path_vec.clone(), id);
        self.dom.touch(id);
        if let Some(idx) = self.dom.pending.iter().position(|p| p.path == path_vec) {
            let pending = self.dom.pending.remove(idx);
            let mut node = pending.node;
            node.id = id;
            self.dom.nodes_info.insert(id, node);
        }
        if self.dom.root.is_none() {
            self.dom.root = Some(id);
        }
        self.dom.resolve_children();
    }

    fn create_placeholder(&mut self, id: ElementId) {
        self.dom.touch(id)
    }

    fn create_text_node(&mut self, _value: &str, id: ElementId) {
        self.dom.upsert_text(id, _value.to_string());
        self.dom.touch(id);
        self.dom.resolve_children();
    }

    fn load_template(&mut self, template: Template, _index: usize, id: ElementId) {
        self.dom.push_static_texts_from_template(&template, Some(id));
        self.dom.touch(id);

        fn collect(node: &TemplateNode, path: &mut Vec<u8>, out: &mut Vec<PendingNode>) {
            match node {
                TemplateNode::Text { text } => {
                    out.push(PendingNode {
                        path: path.clone(),
                        node: DebugNode {
                            id: ElementId(0),
                            tag: None,
                            text: Some(DebugText { id: None, text: text.to_string() }),
                            children_paths: vec![],
                            children: vec![],
                            attrs: HashMap::new(),
                        },
                    });
                }
                TemplateNode::Element { tag, children, attrs, .. } => {
                    let mut children_paths = Vec::new();
                    for (idx, child) in children.iter().enumerate() {
                        let mut child_path = path.clone();
                        child_path.push(idx as u8);
                        children_paths.push(child_path.clone());
                        collect(child, &mut child_path, out);
                    }
                    let mut attr_map = HashMap::new();
                    for TemplateAttribute { name, value, .. } in *attrs {
                        let val_str = match value {
                            dioxus_core::internal::TemplateAttrValue::Static(s) => s.to_string(),
                            dioxus_core::internal::TemplateAttrValue::Dynamic { .. } => "{dynamic}".to_string(),
                        };
                        attr_map.insert(name.to_string(), val_str);
                    }
                    out.push(PendingNode {
                        path: path.clone(),
                        node: DebugNode {
                            id: ElementId(0),
                            tag: Some((*tag).to_string()),
                            text: None,
                            children_paths,
                            children: vec![],
                            attrs: attr_map,
                        },
                    });
                }
                TemplateNode::Dynamic { .. } => {}
            }
        }

        let mut path = vec![0];
        let mut collected = Vec::new();
        for (idx, root) in template.roots.iter().enumerate() {
            path[0] = idx as u8;
            collect(root, &mut path, &mut collected);
        }
        self.dom.pending.extend(collected);
        self.dom.resolve_children();
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
        self.dom.touch(id);
        // Best-effort stringification of attribute values for debugging/layout hints
        let val = format!("{:?}", _value);
        self.dom.upsert_attr(id, _name, val);
    }

    fn set_node_text(&mut self, _value: &str, id: ElementId) {
        self.dom.upsert_text(id, _value.to_string());
        self.dom.touch(id);
        self.dom.resolve_children();
    }

    fn create_event_listener(&mut self, _name: &'static str, id: ElementId) {
        self.dom.touch(id)
    }

    fn remove_event_listener(&mut self, _name: &'static str, id: ElementId) {
        self.dom.touch(id)
    }

    fn remove_node(&mut self, id: ElementId) {
        self.dom.nodes.remove(&id);
        self.dom.nodes_info.remove(&id);
    }

    fn push_root(&mut self, id: ElementId) {
        self.dom.root = Some(id);
        self.dom.touch(id)
    }
}

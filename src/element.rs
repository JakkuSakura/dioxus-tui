use std::collections::{HashMap, HashSet};

use dioxus_core::{ElementId, Template, TemplateAttribute, TemplateNode, WriteMutations};

#[derive(Default)]
pub struct DomState {
    root: Option<ElementId>,
    nodes: HashSet<ElementId>,
    path_to_id: HashMap<Vec<u8>, ElementId>,
    pending: Vec<PendingNode>,
    recent: Vec<ElementId>,
    next_synthetic: usize,
    nodes_info: HashMap<ElementId, DebugNode>,
}

#[derive(Clone)]
pub struct DebugText {
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

    pub fn nodes(&self) -> Vec<DebugNode> {
        self.nodes_info.values().cloned().collect()
    }

    fn upsert_text(&mut self, id: ElementId, value: String) {
        let entry = self.nodes_info.entry(id).or_insert_with(|| DebugNode {
            id,
            ..Default::default()
        });
        entry.text = Some(DebugText { text: value });
    }

    fn upsert_attr(&mut self, id: ElementId, name: &str, value: String) {
        let entry = self.nodes_info.entry(id).or_insert_with(|| DebugNode {
            id,
            ..Default::default()
        });
        entry.attrs.insert(name.to_string(), value);
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
        eprintln!("append_children id={id:?} m={m}");
        self.dom.touch(id);
        if let Some(node) = self.dom.nodes_info.get_mut(&id) {
            let mut attached = Vec::new();
            for _ in 0..m {
                if let Some(child_id) = self.dom.recent.pop() {
                    attached.push(child_id);
                }
            }
            attached.reverse();
            node.children.extend(attached);
        }
        self.dom.resolve_children();
    }

    fn assign_node_id(&mut self, _path: &'static [u8], id: ElementId) {
        eprintln!("assign_node_id id={id:?} path={:?}", _path);
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
        self.dom.recent.push(id);
        self.dom.resolve_children();
    }

    fn create_placeholder(&mut self, id: ElementId) {
        eprintln!("create_placeholder id={id:?}");
        self.dom.touch(id);
        self.dom.recent.push(id);
    }

    fn create_text_node(&mut self, _value: &str, id: ElementId) {
        eprintln!("create_text_node id={id:?} value={_value}");
        self.dom.upsert_text(id, _value.to_string());
        self.dom.touch(id);
        self.dom.recent.push(id);
        self.dom.resolve_children();
    }

    fn load_template(&mut self, template: Template, _index: usize, id: ElementId) {
        eprintln!(
            "load_template id={id:?} roots={} attr_paths={} index={_index}",
            template.roots.len(),
            template.attr_paths.len()
        );
        self.dom.touch(id);
        if self.dom.root.is_none() {
            self.dom.root = Some(id);
        }

        fn dump(node: &TemplateNode, depth: usize) {
            let indent = "  ".repeat(depth);
            match node {
                TemplateNode::Text { text } => {
                    eprintln!("{indent}text {text}");
                }
                TemplateNode::Element { tag, children, .. } => {
                    eprintln!("{indent}elem {tag}");
                    for child in *children {
                        dump(child, depth + 1);
                    }
                }
                TemplateNode::Dynamic { .. } => {
                    eprintln!("{indent}dyn");
                }
            }
        }

        fn walk(dom: &mut DomState, node: &TemplateNode, id: ElementId) -> DebugNode {
            match node {
                TemplateNode::Text { text } => DebugNode {
                    id,
                    tag: None,
                    text: Some(DebugText {
                        text: text.to_string(),
                    }),
                    children_paths: vec![],
                    children: vec![],
                    attrs: HashMap::new(),
                },
                TemplateNode::Element {
                    tag,
                    children,
                    attrs,
                    ..
                } => {
                    let mut attr_map = HashMap::new();
                    for attr in *attrs {
                        if let TemplateAttribute::Static { name, value, .. } = attr {
                            attr_map.insert(name.to_string(), value.to_string());
                        }
                    }

                    let mut child_ids = Vec::new();
                    let mut child_nodes = Vec::new();
                    for child in *children {
                        let new_id = ElementId(dom.next_synthetic);
                        dom.next_synthetic += 1;
                        child_ids.push(new_id);
                        child_nodes.push(walk(dom, child, new_id));
                    }

                    let mut dbg = DebugNode {
                        id,
                        tag: Some((*tag).to_string()),
                        text: None,
                        children_paths: Vec::new(),
                        children: child_ids,
                        attrs: attr_map,
                    };
                    for child in child_nodes {
                        dom.nodes_info.insert(child.id, child);
                    }
                    dbg
                }
                TemplateNode::Dynamic { .. } => DebugNode {
                    id,
                    tag: None,
                    text: None,
                    children_paths: Vec::new(),
                    children: Vec::new(),
                    attrs: HashMap::new(),
                },
            }
        }

        if let Some(root) = template.roots.get(_index) {
            let dbg_root = walk(&mut self.dom, root, id);
            self.dom.recent.push(id);
            self.dom.nodes_info.insert(id, dbg_root);
            self.dom.resolve_children();
        }
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

    fn set_attribute(
        &mut self,
        _name: &'static str,
        _ns: Option<&'static str>,
        _value: &dioxus_core::AttributeValue,
        id: ElementId,
    ) {
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

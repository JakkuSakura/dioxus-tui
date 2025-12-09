use std::collections::HashMap;

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use dioxus_core::{ElementId, Template, TemplateAttribute, TemplateNode, WriteMutations};

#[derive(Default)]
pub struct DomState {
    root: Option<ElementId>,
    path_to_id: HashMap<Vec<u8>, ElementId>,
    templates: HashMap<Vec<u8>, TemplateNodeEntry>,
    id_to_entity: HashMap<ElementId, Entity>,
    world: World,
    next_id: usize,
}

#[derive(Clone)]
pub struct NodeText {
    pub text: String,
}

/// Single node representation used for storage and view snapshots.
#[derive(Component, Clone, Default)]
pub struct DomNode {
    pub id: ElementId,
    pub tag: Option<String>,
    pub text: Option<NodeText>,
    pub children_paths: Vec<Vec<u8>>, // helper for resolving template children
    pub children: Vec<ElementId>,
    pub attrs: HashMap<String, String>,
}

#[derive(Clone, Default)]
struct TemplateNodeEntry {
    tag: Option<String>,
    text: Option<String>,
    attrs: HashMap<String, String>,
    child_paths: Vec<Vec<u8>>,
}

impl DomState {
    pub fn root(&self) -> Option<ElementId> {
        self.root
    }

    pub fn writer(&mut self) -> DomWriter<'_> {
        DomWriter { dom: self }
    }

    pub fn nodes(&mut self) -> Vec<DomNode> {
        let mut out = Vec::new();
        let mut query = self.world.query::<&DomNode>();
        for n in query.iter(&self.world) {
            out.push(n.clone());
        }
        out
    }

    fn ensure_entity(&mut self, id: ElementId) -> Entity {
        self.next_id = self.next_id.max(id.0.saturating_add(1));
        if let Some(entity) = self.id_to_entity.get(&id) {
            return *entity;
        }
        let entity = self
            .world
            .spawn(DomNode {
                id,
                ..Default::default()
            })
            .id();
        self.id_to_entity.insert(id, entity);
        entity
    }

    fn with_node_mut<F: FnOnce(&mut DomNode)>(&mut self, id: ElementId, f: F) {
        let entity = self.ensure_entity(id);
        if let Some(mut node) = self.world.get_mut::<DomNode>(entity) {
            f(&mut node);
        }
    }

    fn upsert_text(&mut self, id: ElementId, value: String) {
        self.with_node_mut(id, |node| {
            node.text = Some(NodeText { text: value });
        });
        self.next_id = self.next_id.max(id.0.saturating_add(1));
    }

    fn upsert_attr(&mut self, id: ElementId, name: &str, value: String) {
        self.with_node_mut(id, |node| {
            node.attrs.insert(name.to_string(), value);
        });
    }

    fn resolve_children(&mut self) {
        let mapping = self.path_to_id.clone();
        let entities: Vec<Entity> = self.id_to_entity.values().copied().collect();
        for entity in entities {
            if let Some(mut node) = self.world.get_mut::<DomNode>(entity) {
                let child_paths = node.children_paths.clone();
                node.children.clear();
                for child_path in child_paths.iter() {
                    if let Some(id) = mapping.get(child_path) {
                        node.children.push(*id);
                    }
                }
            }
        }
    }
}

pub struct DomWriter<'a> {
    dom: &'a mut DomState,
}

impl WriteMutations for DomWriter<'_> {
    fn append_children(&mut self, id: ElementId, _m: usize) {
        self.dom.ensure_entity(id);
        self.dom.resolve_children();
    }

    fn assign_node_id(&mut self, path: &'static [u8], id: ElementId) {
        let path_vec = path.to_vec();
        let previous = self.dom.path_to_id.insert(path_vec.clone(), id);

        if let Some(prev_id) = previous.filter(|old| *old != id) {
            if let Some(prev_entity) = self.dom.id_to_entity.remove(&prev_id) {
                let cloned = self
                    .dom
                    .world
                    .get::<DomNode>(prev_entity)
                    .map(|c| c.clone());
                if let Some(mut cloned) = cloned {
                    cloned.id = id;
                    self.dom.with_node_mut(id, |node| {
                        *node = cloned.clone();
                    });
                }
                let _ = self.dom.world.despawn(prev_entity);
            }
        }

        self.dom.ensure_entity(id);
        if self.dom.root.is_none() {
            self.dom.root = Some(id);
        }

        if let Some(template_entry) = self.dom.templates.get(&path_vec) {
            let child_paths = template_entry.child_paths.clone();
            let attrs = template_entry.attrs.clone();
            let tag = template_entry.tag.clone();
            let text = template_entry.text.clone().map(|t| NodeText { text: t });
            self.dom.with_node_mut(id, |node| {
                node.tag = tag.clone();
                node.attrs = attrs.clone();
                node.children_paths = child_paths.clone();
                node.text = text.clone();
            });
        }

        self.dom.next_id = self.dom.next_id.max(id.0.saturating_add(1));

        self.dom.resolve_children();
    }

    fn create_placeholder(&mut self, id: ElementId) {
        self.dom.ensure_entity(id);
    }

    fn create_text_node(&mut self, value: &str, id: ElementId) {
        self.dom.upsert_text(id, value.to_string());
        self.dom.ensure_entity(id);
        self.dom.resolve_children();
    }

    fn load_template(&mut self, template: Template, _index: usize, _id: ElementId) {
        let mut path = vec![0];

        fn collect(
            node: &TemplateNode,
            path: &mut Vec<u8>,
            out: &mut HashMap<Vec<u8>, TemplateNodeEntry>,
        ) {
            match node {
                TemplateNode::Text { text } => {
                    out.insert(
                        path.clone(),
                        TemplateNodeEntry {
                            text: Some(text.to_string()),
                            ..Default::default()
                        },
                    );
                }
                TemplateNode::Element {
                    tag,
                    children,
                    attrs,
                    ..
                } => {
                    let mut child_paths = Vec::new();
                    for (idx, child) in children.iter().enumerate() {
                        let mut child_path = path.clone();
                        child_path.push(idx as u8);
                        child_paths.push(child_path.clone());
                        collect(child, &mut child_path, out);
                    }

                    let mut attr_map = HashMap::new();
                    for attr in *attrs {
                        if let TemplateAttribute::Static { name, value, .. } = attr {
                            attr_map.insert(name.to_string(), value.to_string());
                        }
                    }

                    out.insert(
                        path.clone(),
                        TemplateNodeEntry {
                            tag: Some((*tag).to_string()),
                            attrs: attr_map,
                            child_paths,
                            ..Default::default()
                        },
                    );
                }
                TemplateNode::Dynamic { .. } => {}
            }
        }

        let mut collected = HashMap::new();
        for (idx, root) in template.roots.iter().enumerate() {
            path[0] = idx as u8;
            collect(root, &mut path, &mut collected);
        }
        self.dom.templates.extend(collected.clone());

        // Materialize template tree with generated IDs for rendering/debug.
        fn materialize(dom: &mut DomState, node: &TemplateNode, path: &mut Vec<u8>, id: ElementId) {
            dom.path_to_id.insert(path.clone(), id);
            match node {
                TemplateNode::Text { text } => {
                    dom.with_node_mut(id, |n| {
                        n.id = id;
                        n.tag = None;
                        n.text = Some(NodeText {
                            text: text.to_string(),
                        });
                        n.children_paths.clear();
                        n.children.clear();
                        n.attrs.clear();
                    });
                }
                TemplateNode::Element {
                    tag,
                    children,
                    attrs,
                    ..
                } => {
                    let mut child_paths = Vec::new();
                    let mut child_ids = Vec::new();
                    for (cidx, child) in children.iter().enumerate() {
                        let mut cpath = path.clone();
                        cpath.push(cidx as u8);
                        child_paths.push(cpath.clone());
                        let child_id = ElementId(dom.next_id);
                        dom.next_id += 1;
                        child_ids.push(child_id);
                        materialize(dom, child, &mut cpath, child_id);
                    }

                    let mut attr_map = HashMap::new();
                    for attr in *attrs {
                        if let TemplateAttribute::Static { name, value, .. } = attr {
                            attr_map.insert(name.to_string(), value.to_string());
                        }
                    }

                    dom.with_node_mut(id, |n| {
                        n.id = id;
                        n.tag = Some((*tag).to_string());
                        n.text = None;
                        n.children_paths = child_paths;
                        n.children = child_ids;
                        n.attrs = attr_map;
                    });
                }
                TemplateNode::Dynamic { .. } => {}
            }
        }

        if let Some(root_node) = template.roots.get(_index) {
            let mut root_path = vec![_index as u8];
            self.dom.next_id = self.dom.next_id.max(_id.0.saturating_add(1));
            materialize(&mut self.dom, root_node, &mut root_path, _id);
            if self.dom.root.is_none() {
                self.dom.root = Some(_id);
            }
        }

        self.dom.resolve_children();
    }

    fn replace_node_with(&mut self, id: ElementId, _m: usize) {
        self.dom.ensure_entity(id);
    }

    fn replace_placeholder_with_nodes(&mut self, _path: &'static [u8], _m: usize) {}

    fn insert_nodes_after(&mut self, id: ElementId, _m: usize) {
        self.dom.ensure_entity(id);
    }

    fn insert_nodes_before(&mut self, id: ElementId, _m: usize) {
        self.dom.ensure_entity(id);
    }

    fn set_attribute(
        &mut self,
        name: &'static str,
        _ns: Option<&'static str>,
        value: &dioxus_core::AttributeValue,
        id: ElementId,
    ) {
        self.dom.ensure_entity(id);
        let val = format!("{:?}", value);
        self.dom.upsert_attr(id, name, val);
    }

    fn set_node_text(&mut self, value: &str, id: ElementId) {
        self.dom.upsert_text(id, value.to_string());
        self.dom.ensure_entity(id);
        self.dom.resolve_children();
    }

    fn create_event_listener(&mut self, _name: &'static str, id: ElementId) {
        self.dom.ensure_entity(id);
    }

    fn remove_event_listener(&mut self, _name: &'static str, id: ElementId) {
        self.dom.ensure_entity(id);
    }

    fn remove_node(&mut self, id: ElementId) {
        if let Some(entity) = self.dom.id_to_entity.remove(&id) {
            let _ = self.dom.world.despawn(entity);
        }
    }

    fn push_root(&mut self, id: ElementId) {
        self.dom.root = Some(id);
        self.dom.ensure_entity(id);
    }
}

/// Coordinate space of incoming display list items.
#[derive(Debug, Clone, Copy)]
pub enum DisplaySpace {
    /// Logical pixels (CSS px) from Servo.
    LogicalPx,
}

/// Rounding behavior when mapping display list coordinates to cells.
#[derive(Debug, Clone, Copy)]
pub enum Rounding {
    /// Floor to the nearest cell boundary.
    Floor,
    /// Round to the nearest cell boundary.
    Nearest,
    /// Ceil to the nearest cell boundary.
    Ceil,
}

/// Contract describing how to interpret Servo display items for terminal rendering.
#[derive(Debug, Clone, Copy)]
pub struct DisplayListContract {
    pub space: DisplaySpace,
    pub rounding: Rounding,
    /// Whether the display list provides invalidation regions for diffing.
    pub has_invalidation_regions: bool,
}

impl Default for DisplayListContract {
    fn default() -> Self {
        Self {
            space: DisplaySpace::LogicalPx,
            rounding: Rounding::Nearest,
            has_invalidation_regions: false,
        }
    }
}

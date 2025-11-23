use dioxus_html::{
    input_data::keyboard_types::{Code, Modifiers},
    HasKeyboardData, ModifiersInteraction,
};
use dioxus_native_core::{
    custom_element::CustomElement,
    node::OwnedAttributeDiscription,
    node_ref::AttributeMask,
    prelude::{ElementNode, NodeType},
    real_dom::{NodeImmutable, NodeMut, NodeTypeMut, RealDom},
    NodeId,
};
use shipyard::UniqueView;

use crate::runtime::hooks::{FormData, Event, EventData};

use super::{RinkWidget, WidgetContext};

const ACTIVE_BG: &str = "rgb(30, 41, 59)";
const ACTIVE_BORDER: &str = "rgb(99, 102, 241)";
const INACTIVE_BORDER: &str = "rgb(51, 65, 85)";
const ACTIVE_TEXT: &str = "rgb(226, 232, 240)";
const MUTED_TEXT: &str = "rgb(148, 163, 184)";
const ACTIVE_ATTR: &str = "active-index";

#[derive(Default)]
pub(crate) struct TabView {
    container_id: NodeId,
    tab_bar_id: NodeId,
    content_slot_id: NodeId,
    tabs: Vec<TabEntry>,
    active_index: usize,
}

#[derive(Clone)]
struct TabEntry {
    header_id: NodeId,
    panel_id: NodeId,
    label: String,
    original_display: Option<String>,
}

impl CustomElement for TabView {
    const NAME: &'static str = "tabview";

    fn roots(&self) -> Vec<NodeId> {
        vec![self.container_id]
    }

    fn slot(&self) -> Option<NodeId> {
        Some(self.content_slot_id)
    }

    fn create(root: NodeMut) -> Self {
        let mut tab_view = Self::default();
        tab_view.initialize(root);
        tab_view
    }

    fn attributes_changed(&mut self, mut root: NodeMut, attributes: &AttributeMask) {
        let should_update = match attributes {
            AttributeMask::All => true,
            AttributeMask::Some(attrs) => attrs.contains(ACTIVE_ATTR),
        };

        if should_update {
            self.sync_active_from_attr(&root);
            if !self.tabs.is_empty() {
                let rdom = root.real_dom_mut();
                self.apply_tab_styles(rdom);
            }
        }
    }
}

impl TabView {
    fn initialize(&mut self, mut root: NodeMut) {
        let panel_ids = root.child_ids();

        {
            let rdom = root.real_dom_mut();
            let (container_id, tab_bar_id, content_slot_id) = Self::build_scaffold(rdom);
            self.container_id = container_id;
            self.tab_bar_id = tab_bar_id;
            self.content_slot_id = content_slot_id;
        }

        {
            let rdom = root.real_dom_mut();
            self.collect_tabs(rdom, &panel_ids);
        }

        self.sync_active_from_attr(&root);

        {
            let rdom = root.real_dom_mut();
            self.apply_tab_styles(rdom);
        }
    }

    fn build_scaffold(rdom: &mut RealDom) -> (NodeId, NodeId, NodeId) {
        let container_id = {
            let container = rdom.create_node(NodeType::Element(ElementNode {
                tag: "div".to_string(),
                attributes: [
                    (
                        OwnedAttributeDiscription { name: "display".to_string(), namespace: Some("style".to_string()) },
                        "flex".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "flex-direction".to_string(), namespace: Some("style".to_string()) },
                        "column".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "width".to_string(), namespace: Some("style".to_string()) },
                        "100%".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "height".to_string(), namespace: Some("style".to_string()) },
                        "100%".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "border-width".to_string(), namespace: Some("style".to_string()) },
                        "1px".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "border-color".to_string(), namespace: Some("style".to_string()) },
                        "rgb(71, 85, 105)".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "background-color".to_string(), namespace: Some("style".to_string()) },
                        "rgb(2, 6, 23)".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "padding".to_string(), namespace: Some("style".to_string()) },
                        "24px".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "gap".to_string(), namespace: Some("style".to_string()) },
                        "16px".to_string().into(),
                    ),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            }));
            container.id()
        };

        let tab_bar_id = {
            let mut tab_bar = rdom.create_node(NodeType::Element(ElementNode {
                tag: "div".to_string(),
                attributes: [
                    (
                        OwnedAttributeDiscription { name: "display".to_string(), namespace: Some("style".to_string()) },
                        "flex".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "flex-direction".to_string(), namespace: Some("style".to_string()) },
                        "row".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "border-bottom-width".to_string(), namespace: Some("style".to_string()) },
                        "1px".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "border-color".to_string(), namespace: Some("style".to_string()) },
                        INACTIVE_BORDER.to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "tabindex".to_string(), namespace: None },
                        "0".to_string().into(),
                    ),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            }));
            tab_bar.add_event_listener("keydown");
            tab_bar.id()
        };
        if let Some(mut container_node) = rdom.get_mut(container_id) {
            container_node.add_child(tab_bar_id);
        }

        let content_slot_id = {
            let content_slot = rdom.create_node(NodeType::Element(ElementNode {
                tag: "div".to_string(),
                attributes: [
                    (
                        OwnedAttributeDiscription { name: "display".to_string(), namespace: Some("style".to_string()) },
                        "flex".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "flex-direction".to_string(), namespace: Some("style".to_string()) },
                        "column".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "flex-grow".to_string(), namespace: Some("style".to_string()) },
                        "1".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "background-color".to_string(), namespace: Some("style".to_string()) },
                        "rgb(10, 15, 25)".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "border-width".to_string(), namespace: Some("style".to_string()) },
                        "1px".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "border-color".to_string(), namespace: Some("style".to_string()) },
                        "rgb(30, 41, 59)".to_string().into(),
                    ),
                    (
                        OwnedAttributeDiscription { name: "padding".to_string(), namespace: Some("style".to_string()) },
                        "16px".to_string().into(),
                    ),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            }));
            content_slot.id()
        };
        if let Some(mut container_node) = rdom.get_mut(container_id) {
            container_node.add_child(content_slot_id);
        }

        (container_id, tab_bar_id, content_slot_id)
    }

    fn collect_tabs(&mut self, rdom: &mut RealDom, panel_ids: &[NodeId]) {
        for (index, panel_id) in panel_ids.iter().enumerate() {
            let (label, original_display) = match rdom.get_mut(*panel_id) {
                Some(panel) => Self::panel_metadata(panel, index),
                None => continue,
            };

            let header_id = Self::create_header(rdom, &label);
            if let Some(mut bar) = rdom.get_mut(self.tab_bar_id) {
                bar.add_child(header_id);
            }

            self.tabs.push(TabEntry {
                header_id,
                panel_id: *panel_id,
                label,
                original_display,
            });
        }
    }

    fn panel_metadata(panel: NodeMut, index: usize) -> (String, Option<String>) {
        let default_label = format!("Tab {}", index + 1);
        let mut label = default_label.clone();
        let mut display = None;

        {
            let node_type = panel.node_type();
            if let NodeType::Element(el) = &*node_type {
                label = Self::read_text_attribute(
                    &el,
                    "label",
                    || Self::read_text_attribute(&el, "title", || default_label),
                );
                display = el
                    .attributes
                    .get(&OwnedAttributeDiscription {
                        name: "display".to_string(),
                        namespace: Some("style".to_string()),
                    })
                    .and_then(|value| value.as_text().map(|v| v.to_string()));
            }
        }

        (label, display)
    }

    fn read_text_attribute(
        el: &ElementNode,
        name: &str,
        fallback: impl FnOnce() -> String,
    ) -> String {
        el.attributes
            .get(&OwnedAttributeDiscription { name: name.to_string(), namespace: None })
            .and_then(|value| value.as_text().map(|v| v.to_string()))
            .unwrap_or_else(fallback)
    }

    fn create_header(rdom: &mut RealDom, label: &str) -> NodeId {
        let text_id = {
            let text_node = rdom.create_node(label.to_string());
            text_node.id()
        };
        let mut header = rdom.create_node(NodeType::Element(ElementNode {
            tag: "div".to_string(),
            attributes: [
                (
                    OwnedAttributeDiscription { name: "padding-left".to_string(), namespace: Some("style".to_string()) },
                    "24px".to_string().into(),
                ),
                (
                    OwnedAttributeDiscription { name: "padding-right".to_string(), namespace: Some("style".to_string()) },
                    "24px".to_string().into(),
                ),
                (
                    OwnedAttributeDiscription { name: "padding-top".to_string(), namespace: Some("style".to_string()) },
                    "12px".to_string().into(),
                ),
                (
                    OwnedAttributeDiscription { name: "padding-bottom".to_string(), namespace: Some("style".to_string()) },
                    "12px".to_string().into(),
                ),
                (
                    OwnedAttributeDiscription { name: "margin-right".to_string(), namespace: Some("style".to_string()) },
                    "16px".to_string().into(),
                ),
                (
                    OwnedAttributeDiscription { name: "border-bottom-width".to_string(), namespace: Some("style".to_string()) },
                    "1px".to_string().into(),
                ),
                (
                    OwnedAttributeDiscription { name: "border-color".to_string(), namespace: Some("style".to_string()) },
                    INACTIVE_BORDER.to_string().into(),
                ),
                (
                    OwnedAttributeDiscription { name: "color".to_string(), namespace: Some("style".to_string()) },
                    MUTED_TEXT.to_string().into(),
                ),
                (
                    OwnedAttributeDiscription { name: "cursor".to_string(), namespace: Some("style".to_string()) },
                    "pointer".to_string().into(),
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        }));
        header.add_event_listener("click");
        header.add_event_listener("keydown");
        let header_id = header.id();
        header.add_child(text_id);

        header_id
    }

    fn sync_active_from_attr(&mut self, root: &NodeMut) {
        if self.tabs.is_empty() {
            self.active_index = 0;
            return;
        }

        let mut target_index = 0usize;
        let node_type = root.node_type();
        if let NodeType::Element(el) = &*node_type {
            if let Some(value) = el
                .attributes
                .get(&OwnedAttributeDiscription { name: ACTIVE_ATTR.to_string(), namespace: None })
                .and_then(|value| value.as_text())
            {
                if let Ok(idx) = value.parse::<usize>() {
                    target_index = idx.min(self.tabs.len().saturating_sub(1));
                }
            }
        }

        self.active_index = target_index;
    }

    fn apply_tab_styles(&self, rdom: &mut RealDom) {
        if self.tabs.is_empty() {
            return;
        }
        for (idx, tab) in self.tabs.iter().enumerate() {
            let is_active = idx == self.active_index;
            if let Some(mut header) = rdom.get_mut(tab.header_id) {
                if let NodeTypeMut::Element(mut el) = header.node_type_mut() {
                    el.set_attribute(
                        OwnedAttributeDiscription { name: "background-color".to_string(), namespace: Some("style".to_string()) },
                        if is_active { ACTIVE_BG.to_string() } else { "transparent".to_string() },
                    );
                    el.set_attribute(
                        OwnedAttributeDiscription { name: "font-weight".to_string(), namespace: Some("style".to_string()) },
                        if is_active { "bold".to_string() } else { "normal".to_string() },
                    );
                    el.set_attribute(
                        OwnedAttributeDiscription { name: "color".to_string(), namespace: Some("style".to_string()) },
                        if is_active { ACTIVE_TEXT.to_string() } else { MUTED_TEXT.to_string() },
                    );
                    el.set_attribute(
                        OwnedAttributeDiscription { name: "border-bottom-width".to_string(), namespace: Some("style".to_string()) },
                        if is_active { "2px" } else { "1px" }.to_string(),
                    );
                    el.set_attribute(
                        OwnedAttributeDiscription { name: "border-color".to_string(), namespace: Some("style".to_string()) },
                        if is_active { ACTIVE_BORDER.to_string() } else { INACTIVE_BORDER.to_string() },
                    );
                }
            }

            if let Some(mut panel) = rdom.get_mut(tab.panel_id) {
                if let NodeTypeMut::Element(mut el) = panel.node_type_mut() {
                    let display_name = OwnedAttributeDiscription {
                        name: "display".to_string(),
                        namespace: Some("style".to_string()),
                    };
                    if is_active {
                        if let Some(value) = &tab.original_display {
                            el.set_attribute(display_name.clone(), value.clone());
                        } else {
                            el.remove_attribute(&display_name);
                        }
                    } else {
                        el.set_attribute(display_name, "none".to_string());
                    }
                }
            }
        }
    }

    fn set_active(&mut self, index: usize, node: &mut NodeMut) {
        if self.tabs.is_empty() || index >= self.tabs.len() || index == self.active_index {
            return;
        }
        self.active_index = index;
        {
            let rdom = node.real_dom_mut();
            self.apply_tab_styles(rdom);
        }
        self.emit_change(node);
    }

    fn emit_change(&self, node: &mut NodeMut) {
        if self.tabs.is_empty() {
            return;
        }
        let ctx: WidgetContext = {
            node.real_dom_mut()
                .raw_world_mut()
                .borrow::<UniqueView<WidgetContext>>()
                .expect("expected widget context")
                .clone()
        };

        let data = FormData {
            value: self.tabs[self.active_index].label.clone(),
            values: Vec::new(),
            valid: true,
        };
        ctx.send(Event {
            id: node.id(),
            name: "input",
            data: EventData::Form(data),
            bubbles: true,
        });
    }
}

impl RinkWidget for TabView {
    fn handle_event(&mut self, event: &Event, mut node: NodeMut) {
        match event.name {
            "click" => {
                if let Some(idx) = self.tabs.iter().position(|tab| tab.header_id == event.id) {
                    self.set_active(idx, &mut node);
                }
            }
            "keydown" => {
                if let EventData::Keyboard(data) = &event.data {
                    if event.id == self.tab_bar_id {
                        let is_shifted = data.modifiers().contains(Modifiers::SHIFT);
                        match data.code() {
                            Code::ArrowRight | Code::Tab if !is_shifted => {
                                if !self.tabs.is_empty() {
                                    let next = (self.active_index + 1) % self.tabs.len();
                                    self.set_active(next, &mut node);
                                }
                            }
                            Code::ArrowLeft | Code::Tab if is_shifted => {
                                if !self.tabs.is_empty() {
                                    let len = self.tabs.len();
                                    let next = (self.active_index + len - 1) % len;
                                    self.set_active(next, &mut node);
                                }
                            }
                            _ => {}
                        }
                    } else if matches!(data.code(), Code::Space | Code::Enter | Code::Tab) {
                        if let Some(idx) = self.tabs.iter().position(|tab| tab.header_id == event.id)
                        {
                            self.set_active(idx, &mut node);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}


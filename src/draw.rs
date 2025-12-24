use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};

use dioxus_core::{AttributeValue, ElementId, IntoAttributeValue, WriteMutations};
use dioxus_core::nodes::AnyValue;
use dioxus_native_dom::mutation_writer::MutationWriter;
use futures_util::{pin_mut, FutureExt};
use futures_util::task::noop_waker;

use crate::config::{ColorMode, PaletteRoles};
use crate::geometry::Rect;
use crate::surface::Surface;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomDrawMode {
    Html,
    Native,
}

pub struct DrawContext<'a> {
    pub surface: &'a mut Surface,
    pub rect: Rect,
    pub color_mode: ColorMode,
    pub truecolor: bool,
    pub palette_roles: PaletteRoles,
}

#[derive(Clone)]
pub struct OnDraw {
    pub(crate) cb: Arc<dyn Fn(&mut DrawContext) + Send + Sync>,
}

impl OnDraw {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut DrawContext) + Send + Sync + 'static,
    {
        Self { cb: Arc::new(f) }
    }
}

impl AnyValue for OnDraw {
    fn any_cmp(&self, other: &dyn AnyValue) -> bool {
        other
            .as_any()
            .downcast_ref::<OnDraw>()
            .map(|o| Arc::ptr_eq(&self.cb, &o.cb))
            .unwrap_or(false)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IntoAttributeValue for OnDraw {
    fn into_value(self) -> AttributeValue {
        AttributeValue::Any(Rc::new(self))
    }
}

pub fn on_draw<F>(f: F) -> OnDraw
where
    F: Fn(&mut DrawContext) + Send + Sync + 'static,
{
    OnDraw::new(f)
}

#[derive(Default)]
pub struct DrawState {
    callbacks: HashMap<usize, Arc<dyn Fn(&mut DrawContext) + Send + Sync>>,
}

impl DrawState {
    pub fn set(&mut self, node_id: usize, cb: Arc<dyn Fn(&mut DrawContext) + Send + Sync>) {
        self.callbacks.insert(node_id, cb);
    }

    pub fn remove(&mut self, node_id: usize) {
        self.callbacks.remove(&node_id);
    }

    pub fn get(&self, node_id: usize) -> Option<&Arc<dyn Fn(&mut DrawContext) + Send + Sync>> {
        self.callbacks.get(&node_id)
    }
}

pub struct OnDrawWriter<'a> {
    inner: MutationWriter<'a>,
    draw_state: &'a mut DrawState,
}

impl<'a> OnDrawWriter<'a> {
    pub fn new(inner: MutationWriter<'a>, draw_state: &'a mut DrawState) -> Self {
        Self { inner, draw_state }
    }
}

impl WriteMutations for OnDrawWriter<'_> {
    fn append_children(&mut self, id: ElementId, m: usize) {
        self.inner.append_children(id, m);
    }

    fn assign_node_id(&mut self, path: &'static [u8], id: ElementId) {
        self.inner.assign_node_id(path, id);
    }

    fn create_placeholder(&mut self, id: ElementId) {
        self.inner.create_placeholder(id);
    }

    fn create_text_node(&mut self, value: &str, id: ElementId) {
        self.inner.create_text_node(value, id);
    }

    fn load_template(&mut self, template: dioxus_core::Template, index: usize, id: ElementId) {
        self.inner.load_template(template, index, id);
    }

    fn replace_node_with(&mut self, id: ElementId, m: usize) {
        self.inner.replace_node_with(id, m);
    }

    fn replace_placeholder_with_nodes(&mut self, path: &'static [u8], m: usize) {
        self.inner.replace_placeholder_with_nodes(path, m);
    }

    fn insert_nodes_after(&mut self, id: ElementId, m: usize) {
        self.inner.insert_nodes_after(id, m);
    }

    fn insert_nodes_before(&mut self, id: ElementId, m: usize) {
        self.inner.insert_nodes_before(id, m);
    }

    fn set_attribute(
        &mut self,
        name: &'static str,
        ns: Option<&'static str>,
        value: &AttributeValue,
        id: ElementId,
    ) {
        if name == "on_draw" {
            let node_id = self.inner.state.element_to_node_id(id);
            match value {
                AttributeValue::Any(any) => {
                    if let Some(draw) = any.as_any().downcast_ref::<OnDraw>() {
                        self.draw_state.set(node_id, draw.cb.clone());
                        return;
                    }
                }
                AttributeValue::None => {
                    self.draw_state.remove(node_id);
                    return;
                }
                _ => {}
            }
        }

        self.inner.set_attribute(name, ns, value, id);
    }

    fn set_node_text(&mut self, value: &str, id: ElementId) {
        self.inner.set_node_text(value, id);
    }

    fn create_event_listener(&mut self, name: &'static str, id: ElementId) {
        self.inner.create_event_listener(name, id);
    }

    fn remove_event_listener(&mut self, name: &'static str, id: ElementId) {
        self.inner.remove_event_listener(name, id);
    }

    fn remove_node(&mut self, id: ElementId) {
        let node_id = self.inner.state.element_to_node_id(id);
        self.draw_state.remove(node_id);
        self.inner.remove_node(id);
    }

    fn push_root(&mut self, id: ElementId) {
        self.inner.push_root(id);
    }
}

pub fn poll_vdom_with_on_draw(
    vdom: &mut dioxus_core::VirtualDom,
    doc: &mut blitz_dom::BaseDocument,
    vdom_state: &mut dioxus_native_dom::mutation_writer::DioxusState,
    draw_state: &mut DrawState,
) -> bool {
    let fut = vdom.wait_for_work();
    pin_mut!(fut);

    static NOOP_WAKER: std::sync::LazyLock<std::task::Waker> =
        std::sync::LazyLock::new(noop_waker);
    let mut cx = TaskContext::from_waker(&NOOP_WAKER);
    if matches!(fut.poll_unpin(&mut cx), Poll::Pending) {
        return false;
    }

    let mut writer = MutationWriter::new(doc, vdom_state);
    let mut writer = OnDrawWriter::new(writer, draw_state);
    vdom.render_immediate(&mut writer);
    true
}

pub fn rgb_to_attr(
    r: u8,
    g: u8,
    b: u8,
    color_mode: ColorMode,
    truecolor: bool,
) -> termwiz::color::ColorAttribute {
    use termwiz::color::{ColorAttribute, SrgbaTuple};

    let srgb = SrgbaTuple::from((r, g, b));
    let palette_idx_256 = 16 + 36 * (r as u16 / 51) as u8 + 6 * (g as u16 / 51) as u8 + (b as u16 / 51) as u8;
    let base_idx = (if r >= 128 { 1 } else { 0 }) | (if g >= 128 { 2 } else { 0 }) | (if b >= 128 { 4 } else { 0 });

    match color_mode {
        ColorMode::BaseColors => ColorAttribute::PaletteIndex(base_idx),
        ColorMode::Ansi => ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256),
        ColorMode::Rgb => {
            if truecolor {
                ColorAttribute::TrueColorWithDefaultFallback(srgb)
            } else {
                ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256)
            }
        }
    }
}

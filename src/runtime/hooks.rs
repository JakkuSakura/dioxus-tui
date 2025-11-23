use crossterm::event::{
    Event as TermEvent, KeyCode as TermKeyCode, KeyModifiers, ModifierKeyCode, MouseButton,
    MouseEventKind,
};
use dioxus_core_types::event_bubbles;
use dioxus_html::{
    geometry::{ClientPoint, Coordinates, ElementPoint, PagePoint, ScreenPoint},
    input_data::keyboard_types::{Code, Key, Location, Modifiers},
    input_data::{MouseButton as DioxusMouseButton, MouseButtonSet as DioxusMouseButtons},
    point_interaction::SerializedPointInteraction,
    FileData, FormValue, HasFileData, HasFormData, HasKeyboardData, HasWheelData,
    InteractionElementOffset, InteractionLocation, ModifiersInteraction, PointerInteraction,
    SerializedFocusData, SerializedKeyboardData, SerializedMouseData, SerializedWheelData,
};
use dioxus_native_core::prelude::*;
use dioxus_native_core::real_dom::NodeImmutable;
use rustc_hash::{FxHashMap, FxHashSet};
use std::any::Any;
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    time::{Duration, Instant},
};
use taffy::geometry::{Point, Size};
use taffy::{prelude::Layout, Taffy};

use crate::runtime::focus::{Focus, Focused, FocusState};
use crate::runtime::layout::TaffyLayout;
use crate::runtime::get_abs_layout;
use crate::runtime::layout_to_screen_space;

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id: NodeId,
    pub name: &'static str,
    pub data: EventData,
    pub bubbles: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventData {
    Mouse(SerializedMouseData),
    Keyboard(SerializedKeyboardData),
    Focus(SerializedFocusData),
    Wheel(SerializedWheelData),
    Form(FormData),
}

impl EventData {
    pub fn into_any(self) -> Rc<dyn Any> {
        match self {
            EventData::Mouse(m) => Rc::new(m),
            EventData::Keyboard(k) => Rc::new(k),
            EventData::Focus(f) => Rc::new(f),
            EventData::Wheel(w) => Rc::new(w),
            EventData::Form(f) => Rc::new(f),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormData {
    pub(crate) value: String,
    pub values: Vec<(String, FormValue)>,
    pub(crate) valid: bool,
}

impl HasFormData for FormData {
    fn value(&self) -> String {
        self.value.clone()
    }

    fn valid(&self) -> bool {
        self.valid
    }

    fn values(&self) -> Vec<(String, FormValue)> {
        self.values.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl HasFileData for FormData {
    fn files(&self) -> Vec<FileData> {
        self.values
            .iter()
            .filter_map(|(_, value)| {
                if let FormValue::File(Some(file)) = value {
                    Some(file.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

type EventCore = (&'static str, EventData);

const MAX_REPEAT_TIME: Duration = Duration::from_millis(100);

pub struct InnerInputState {
    mouse: Option<SerializedMouseData>,
    wheel: Option<SerializedWheelData>,
    last_key_pressed: Option<(SerializedKeyboardData, Instant)>,
    pub(crate) focus_state: FocusState,
}

impl InnerInputState {
    pub fn create(rdom: &mut RealDom) -> Self {
        Self {
            mouse: None,
            wheel: None,
            last_key_pressed: None,
            focus_state: FocusState::create(rdom),
        }
    }

    fn apply_event(&mut self, evt: &mut EventCore) {
        match evt.1 {
            EventData::Mouse(ref mut m) => {
                // updated by resolve_mouse_events
                self.mouse = Some(m.clone());
            }
            EventData::Form(_) => {}
            EventData::Focus(_) => {}
            EventData::Wheel(ref mut w) => {
                self.wheel = Some(w.clone());
            }
            EventData::Keyboard(ref mut k) => {
                let is_repeating = self
                    .last_key_pressed
                    .as_ref()
                    .filter(|(last_data, last_instant)| {
                        last_data.key() == k.key() && last_instant.elapsed() < MAX_REPEAT_TIME
                    })
                    .is_some();

                if is_repeating {
                    *k = SerializedKeyboardData::new(
                        k.key(),
                        k.code(),
                        k.location(),
                        is_repeating,
                        k.modifiers(),
                        k.is_composing(),
                    );
                }

                self.last_key_pressed = Some((k.clone(), Instant::now()));
            }
        }
    }

    pub fn update(
        &mut self,
        evts: &mut Vec<EventCore>,
        resolved_events: &mut Vec<Event>,
        layout: &Taffy,
        dom: &mut RealDom,
    ) {
        let previous_mouse = self.mouse.clone();

        self.wheel = None;

        let old_focus = self.focus_state.last_focused_id;

        evts.retain(|e| match &e.1 {
            EventData::Keyboard(k) => match k.code() {
                Code::Tab => !self
                    .focus_state
                    .progress(dom, !k.modifiers().contains(Modifiers::SHIFT)),
                _ => true,
            },
            _ => true,
        });

        for e in evts.iter_mut() {
            self.apply_event(e);
        }

        self.resolve_mouse_events(previous_mouse, resolved_events, layout, dom);

        if old_focus != self.focus_state.last_focused_id {
            if let Some(id) = self.focus_state.last_focused_id {
                resolved_events.push(Event {
                    name: "focus",
                    id,
                    data: EventData::Focus(SerializedFocusData::default()),
                    bubbles: event_bubbles("focus"),
                });
                resolved_events.push(Event {
                    name: "focusin",
                    id,
                    data: EventData::Focus(SerializedFocusData::default()),
                    bubbles: event_bubbles("focusin"),
                });
            }
            if let Some(id) = old_focus {
                resolved_events.push(Event {
                    name: "focusout",
                    id,
                    data: EventData::Focus(SerializedFocusData::default()),
                    bubbles: event_bubbles("focusout"),
                });
            }
        }
    }

    fn resolve_mouse_events(
        &mut self,
        previous_mouse: Option<SerializedMouseData>,
        resolved_events: &mut Vec<Event>,
        layout: &Taffy,
        dom: &mut RealDom,
    ) {
        let Some(mouse) = self.mouse.clone() else {
            return;
        };

        let mut last_over = FxHashSet::default();
        let mut current_over = FxHashSet::default();

        if let Some(prev_mouse) = previous_mouse {
            last_over = self.get_mouse_over(prev_mouse, layout, dom);
        }

        current_over = self.get_mouse_over(mouse.clone(), layout, dom);

        let entered: FxHashSet<_> = current_over.difference(&last_over).copied().collect();
        let exited: FxHashSet<_> = last_over.difference(&current_over).copied().collect();

        for id in &entered {
            resolved_events.push(Event {
                name: "mouseenter",
                id: *id,
                data: EventData::Mouse(mouse.clone()),
                bubbles: event_bubbles("mouseenter"),
            });
            resolved_events.push(Event {
                name: "mouseover",
                id: *id,
                data: EventData::Mouse(mouse.clone()),
                bubbles: event_bubbles("mouseover"),
            });
        }

        for id in &exited {
            resolved_events.push(Event {
                name: "mouseleave",
                id: *id,
                data: EventData::Mouse(mouse.clone()),
                bubbles: event_bubbles("mouseleave"),
            });
            resolved_events.push(Event {
                name: "mouseout",
                id: *id,
                data: EventData::Mouse(mouse.clone()),
                bubbles: event_bubbles("mouseout"),
            });
        }
    }

    fn get_mouse_over(
        &self,
        mouse: SerializedMouseData,
        layout: &Taffy,
        dom: &mut RealDom,
    ) -> FxHashSet<NodeId> {
        let mut over = FxHashSet::default();

        dom.traverse_depth_first(|n| {
            if let Some(layout_state) = n.get::<TaffyLayout>() {
                let node_layout = layout
                    .layout(layout_state.node.unwrap())
                    .expect("layout should exist");

                let x = node_layout.location.x;
                let y = node_layout.location.y;
                let w = node_layout.size.width;
                let h = node_layout.size.height;

                let mx = mouse.coordinates().client().x as f32;
                let my = mouse.coordinates().client().y as f32;

                if mx >= x && mx <= x + w && my >= y && my <= y + h {
                    over.insert(n.id());
                }
            }
        });

        over
    }
}

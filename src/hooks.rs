use std::any::Any;

use crossterm::event::{Event as TermEvent, KeyCode as TermKeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use dioxus_core::ElementId;
use dioxus_html::geometry::{ClientPoint, Coordinates, ElementPoint, PagePoint, ScreenPoint};
use dioxus_html::input_data::keyboard_types::{Code, Key, Location, Modifiers};
use dioxus_html::input_data::{MouseButton, MouseButtonSet};
use dioxus_html::point_interaction::SerializedPointInteraction;
use dioxus_html::{SerializedKeyboardData, SerializedMouseData, SerializedWheelData};

#[derive(Debug, Clone, PartialEq)]
pub enum EventData {
    Mouse(SerializedMouseData),
    Keyboard(SerializedKeyboardData),
    Wheel(SerializedWheelData),
}

impl EventData {
    pub fn into_platform_event(self, _bubbles: bool) -> Box<dyn Any> {
        Box::new(self)
    }
}

fn map_modifiers(mods: KeyModifiers) -> Modifiers {
    let mut m = Modifiers::empty();
    if mods.contains(KeyModifiers::SHIFT) {
        m.insert(Modifiers::SHIFT);
    }
    if mods.contains(KeyModifiers::CONTROL) {
        m.insert(Modifiers::CONTROL);
    }
    if mods.contains(KeyModifiers::ALT) {
        m.insert(Modifiers::ALT);
    }
    if mods.contains(KeyModifiers::META) {
        m.insert(Modifiers::META);
    }
    m
}

fn map_code(key: &KeyEvent) -> (Key, Code) {
    match key.code {
        TermKeyCode::Char(c) => (Key::Character(c.to_string()), Code::Unidentified),
        TermKeyCode::Tab => (Key::Tab, Code::Tab),
        TermKeyCode::Enter => (Key::Enter, Code::Enter),
        TermKeyCode::Backspace => (Key::Backspace, Code::Backspace),
        TermKeyCode::Left => (Key::ArrowLeft, Code::ArrowLeft),
        TermKeyCode::Right => (Key::ArrowRight, Code::ArrowRight),
        TermKeyCode::Up => (Key::ArrowUp, Code::ArrowUp),
        TermKeyCode::Down => (Key::ArrowDown, Code::ArrowDown),
        _ => (Key::Unidentified, Code::Unidentified),
    }
}

fn to_button_set(btn: Option<MouseButton>) -> MouseButtonSet {
    let mut set = MouseButtonSet::empty();
    if let Some(b) = btn {
        set.insert(b);
    }
    set
}

pub fn event_from_crossterm(evt: TermEvent, target: ElementId) -> Vec<(ElementId, &'static str, EventData, bool)> {
    match evt {
        TermEvent::Key(key) => {
            let (key_val, code) = map_code(&key);
            let mods = map_modifiers(key.modifiers);
            let data = SerializedKeyboardData::new(
                key_val,
                code,
                Location::Standard,
                false,
                mods,
                false,
            );
            vec![(target, "keydown", EventData::Keyboard(data), true)]
        }
        TermEvent::Mouse(MouseEvent { kind, column, row, modifiers }) => {
            let modifiers = map_modifiers(modifiers);
            let screen = ScreenPoint::new(column.into(), row.into());
            let client = ClientPoint::new(column.into(), row.into());
            let element = ElementPoint::new(column.into(), row.into());
            let page = PagePoint::new(column.into(), row.into());
            let coords = Coordinates::new(screen, client, element, page);

            match kind {
                MouseEventKind::Down(button) => {
                    let btn = map_button(button);
                    let data = SerializedMouseData::new(Some(btn), to_button_set(Some(btn)), coords, modifiers);
                    vec![(target, "mousedown", EventData::Mouse(data), true)]
                }
                MouseEventKind::Up(button) => {
                    let btn = map_button(button);
                    let data = SerializedMouseData::new(Some(btn), MouseButtonSet::empty(), coords, modifiers);
                    let mut evts = vec![(target, "mouseup", EventData::Mouse(data.clone()), true)];
                    let name = if btn == MouseButton::Primary { "click" } else { "contextmenu" };
                    evts.push((target, name, EventData::Mouse(data), true));
                    evts
                }
                MouseEventKind::Moved => {
                    let data = SerializedMouseData::new(None, MouseButtonSet::empty(), coords, modifiers);
                    vec![
                        (target, "mousemove", EventData::Mouse(data.clone()), true),
                        (target, "mouseenter", EventData::Mouse(data), true),
                    ]
                }
                MouseEventKind::ScrollDown => {
                    let point = SerializedPointInteraction::new(None, MouseButtonSet::empty(), coords, modifiers);
                    let data = SerializedWheelData {
                        mouse: point,
                        delta_mode: 1,
                        delta_x: 0.0,
                        delta_y: 1.0,
                        delta_z: 0.0,
                    };
                    vec![(target, "wheel", EventData::Wheel(data), true)]
                }
                MouseEventKind::ScrollUp => {
                    let point = SerializedPointInteraction::new(None, MouseButtonSet::empty(), coords, modifiers);
                    let data = SerializedWheelData {
                        mouse: point,
                        delta_mode: 1,
                        delta_x: 0.0,
                        delta_y: -1.0,
                        delta_z: 0.0,
                    };
                    vec![(target, "wheel", EventData::Wheel(data), true)]
                }
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}

fn map_button(button: crossterm::event::MouseButton) -> MouseButton {
    match button {
        crossterm::event::MouseButton::Left => MouseButton::Primary,
        crossterm::event::MouseButton::Right => MouseButton::Secondary,
        crossterm::event::MouseButton::Middle => MouseButton::Auxiliary,
    }
}

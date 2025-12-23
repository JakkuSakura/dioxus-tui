use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use dioxus::prelude::{use_context, use_hook, use_signal, Signal, WritableExt};
use dioxus_html::geometry::{ClientPoint, Coordinates, ElementPoint, PagePoint, ScreenPoint};
use crate::geometry::Rect;
use dioxus_html::input_data::keyboard_types::{Code, Key, Location, Modifiers};
use dioxus_html::input_data::{MouseButton, MouseButtonSet};
use dioxus_html::point_interaction::SerializedPointInteraction;
use dioxus_html::{SerializedFocusData, SerializedKeyboardData, SerializedMouseData, SerializedWheelData};
use termwiz::input::{InputEvent, KeyCode as TermKeyCode, KeyEvent, Modifiers as TermModifiers, MouseButtons, MouseEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum EventData {
    Mouse(SerializedMouseData),
    Keyboard(SerializedKeyboardData),
    Wheel(SerializedWheelData),
    Focus(SerializedFocusData),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawInputEvent {
    pub name: &'static str,
    pub data: EventData,
    pub bubbles: bool,
}

#[derive(Clone, Default)]
pub struct TuiInputBus {
    listeners: Rc<RefCell<Vec<Option<Rc<dyn Fn(RawInputEvent)>>>>>,
}

impl TuiInputBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn subscribe(&self, listener: Rc<dyn Fn(RawInputEvent)>) -> InputSubscription {
        let mut listeners = self.listeners.borrow_mut();
        let id = listeners.len();
        listeners.push(Some(listener));
        Rc::new(InputSubscriptionInner {
            id,
            listeners: Rc::downgrade(&self.listeners),
        })
    }

    pub fn publish(&self, event: RawInputEvent) {
        for listener in self.listeners.borrow().iter().flatten() {
            listener(event.clone());
        }
    }
}

pub(crate) struct InputSubscriptionInner {
    id: usize,
    listeners: Weak<RefCell<Vec<Option<Rc<dyn Fn(RawInputEvent)>>>>>,
}

impl Drop for InputSubscriptionInner {
    fn drop(&mut self) {
        if let Some(listeners) = self.listeners.upgrade() {
            if let Some(slot) = listeners.borrow_mut().get_mut(self.id) {
                *slot = None;
            }
        }
    }
}

pub type InputSubscription = Rc<InputSubscriptionInner>;

impl EventData {
    pub fn into_platform_event(self, _bubbles: bool) -> Box<dyn Any> {
        // Dioxus HTML expects the underlying `Serialized*Data` as the platform event payload.
        // Wrapping our enum would prevent the event converter from downcasting correctly.
        match self {
            EventData::Mouse(data) => Box::new(data),
            EventData::Keyboard(data) => Box::new(data),
            EventData::Wheel(data) => Box::new(data),
            EventData::Focus(data) => Box::new(data),
        }
    }
}

pub(crate) fn map_modifiers(mods: TermModifiers) -> Modifiers {
    let mut m = Modifiers::empty();
    if mods.contains(TermModifiers::SHIFT) {
        m.insert(Modifiers::SHIFT);
    }
    if mods.contains(TermModifiers::CTRL) {
        m.insert(Modifiers::CONTROL);
    }
    if mods.contains(TermModifiers::ALT) {
        m.insert(Modifiers::ALT);
    }
    if mods.contains(TermModifiers::SUPER) {
        m.insert(Modifiers::META);
    }
    m
}

pub(crate) fn map_code(key: &KeyEvent) -> (Key, Code) {
    match key.key {
        TermKeyCode::Char(c) => (Key::Character(c.to_string()), code_from_char(c)),
        TermKeyCode::Tab => (Key::Tab, Code::Tab),
        TermKeyCode::Enter => (Key::Enter, Code::Enter),
        TermKeyCode::Backspace => (Key::Backspace, Code::Backspace),
        TermKeyCode::LeftArrow => (Key::ArrowLeft, Code::ArrowLeft),
        TermKeyCode::RightArrow => (Key::ArrowRight, Code::ArrowRight),
        TermKeyCode::UpArrow => (Key::ArrowUp, Code::ArrowUp),
        TermKeyCode::DownArrow => (Key::ArrowDown, Code::ArrowDown),
        _ => (Key::Unidentified, Code::Unidentified),
    }
}

fn code_from_char(c: char) -> Code {
    // Termwiz only provides a character and modifiers, not the physical key.
    // Map common ASCII characters to the closest `Code` so apps can match on it.
    // This is necessarily layout-dependent; for non-US layouts, `Key::Character` is more reliable.
    match c {
        ' ' => Code::Space,
        '0' => Code::Digit0,
        '1' => Code::Digit1,
        '2' => Code::Digit2,
        '3' => Code::Digit3,
        '4' => Code::Digit4,
        '5' => Code::Digit5,
        '6' => Code::Digit6,
        '7' => Code::Digit7,
        '8' => Code::Digit8,
        '9' => Code::Digit9,

        'a' | 'A' => Code::KeyA,
        'b' | 'B' => Code::KeyB,
        'c' | 'C' => Code::KeyC,
        'd' | 'D' => Code::KeyD,
        'e' | 'E' => Code::KeyE,
        'f' | 'F' => Code::KeyF,
        'g' | 'G' => Code::KeyG,
        'h' | 'H' => Code::KeyH,
        'i' | 'I' => Code::KeyI,
        'j' | 'J' => Code::KeyJ,
        'k' | 'K' => Code::KeyK,
        'l' | 'L' => Code::KeyL,
        'm' | 'M' => Code::KeyM,
        'n' | 'N' => Code::KeyN,
        'o' | 'O' => Code::KeyO,
        'p' | 'P' => Code::KeyP,
        'q' | 'Q' => Code::KeyQ,
        'r' | 'R' => Code::KeyR,
        's' | 'S' => Code::KeyS,
        't' | 'T' => Code::KeyT,
        'u' | 'U' => Code::KeyU,
        'v' | 'V' => Code::KeyV,
        'w' | 'W' => Code::KeyW,
        'x' | 'X' => Code::KeyX,
        'y' | 'Y' => Code::KeyY,
        'z' | 'Z' => Code::KeyZ,

        '-' => Code::Minus,
        '=' => Code::Equal,
        '[' => Code::BracketLeft,
        ']' => Code::BracketRight,
        '\\' => Code::Backslash,
        ';' => Code::Semicolon,
        '\'' => Code::Quote,
        ',' => Code::Comma,
        '.' => Code::Period,
        '/' => Code::Slash,

        _ => Code::Unidentified,
    }
}

#[cfg(test)]
mod tests {
    use super::code_from_char;
    use dioxus_html::input_data::keyboard_types::Code;

    #[test]
    fn code_from_char_maps_common_ascii() {
        assert_eq!(code_from_char('q'), Code::KeyQ);
        assert_eq!(code_from_char('Q'), Code::KeyQ);
        assert_eq!(code_from_char(' '), Code::Space);
        assert_eq!(code_from_char('7'), Code::Digit7);
        assert_eq!(code_from_char('['), Code::BracketLeft);
    }
}

fn to_button_set(btn: Option<MouseButton>) -> MouseButtonSet {
    let mut set = MouseButtonSet::empty();
    if let Some(b) = btn {
        set.insert(b);
    }
    set
}

pub fn raw_input_from_termwiz(
    evt: &InputEvent,
    viewport: Rect,
    pixel_viewport: Option<Rect>,
) -> Vec<RawInputEvent> {
    match evt {
        InputEvent::Key(key) => {
            let (key_val, code) = map_code(key);
            let mods = map_modifiers(key.modifiers);
            let data =
                SerializedKeyboardData::new(key_val, code, Location::Standard, false, mods, false);
            vec![RawInputEvent {
                name: "keydown",
                data: EventData::Keyboard(data),
                bubbles: true,
            }]
        }
        InputEvent::Mouse(mouse_evt) => map_mouse_input(mouse_evt, viewport),
        InputEvent::PixelMouse(mouse_evt) => {
            let viewport = pixel_viewport.unwrap_or(viewport);
            map_pixel_mouse_input(mouse_evt, viewport)
        }
        _ => Vec::new(),
    }
}

fn map_mouse_input(evt: &MouseEvent, viewport: Rect) -> Vec<RawInputEvent> {
    if evt.mouse_buttons.contains(MouseButtons::VERT_WHEEL)
        || evt.mouse_buttons.contains(MouseButtons::HORZ_WHEEL)
    {
        let (delta_x, delta_y) = wheel_delta(evt.mouse_buttons.clone());
        let (_, _, coords) = build_coords(evt.x, evt.y, viewport);
        let modifiers = map_modifiers(evt.modifiers);
        let point = SerializedPointInteraction::new(None, MouseButtonSet::empty(), coords, modifiers);
        let data = SerializedWheelData {
            mouse: point,
            delta_mode: 1,
            delta_x,
            delta_y,
            delta_z: 0.0,
        };
        return vec![RawInputEvent {
            name: "wheel",
            data: EventData::Wheel(data),
            bubbles: true,
        }];
    }

    let btn = button_from_mask(evt.mouse_buttons.clone());
    let (pressed, button) = match btn {
        Some(b) => (true, Some(b)),
        None => (false, None),
    };

    let (_, _, coords) = build_coords(evt.x, evt.y, viewport);
    let modifiers = map_modifiers(evt.modifiers);
    let data = SerializedMouseData::new(button, to_button_set(button), coords, modifiers);

    if pressed {
        vec![RawInputEvent {
            name: "mousedown",
            data: EventData::Mouse(data),
            bubbles: true,
        }]
    } else {
        vec![
            RawInputEvent {
                name: "mousemove",
                data: EventData::Mouse(data.clone()),
                bubbles: true,
            },
            RawInputEvent {
                name: "mouseenter",
                data: EventData::Mouse(data),
                bubbles: true,
            },
        ]
    }
}

fn map_pixel_mouse_input(evt: &termwiz::input::PixelMouseEvent, viewport: Rect) -> Vec<RawInputEvent> {
    if evt.mouse_buttons.contains(MouseButtons::VERT_WHEEL)
        || evt.mouse_buttons.contains(MouseButtons::HORZ_WHEEL)
    {
        let (delta_x, delta_y) = wheel_delta(evt.mouse_buttons.clone());
        let (_, _, coords) = build_coords(evt.x_pixels, evt.y_pixels, viewport);
        let modifiers = map_modifiers(evt.modifiers);
        let point = SerializedPointInteraction::new(None, MouseButtonSet::empty(), coords, modifiers);
        let data = SerializedWheelData {
            mouse: point,
            delta_mode: 0,
            delta_x,
            delta_y,
            delta_z: 0.0,
        };
        return vec![RawInputEvent {
            name: "wheel",
            data: EventData::Wheel(data),
            bubbles: true,
        }];
    }

    let btn = button_from_mask(evt.mouse_buttons.clone());
    let (pressed, button) = match btn {
        Some(b) => (true, Some(b)),
        None => (false, None),
    };

    let (_, _, coords) = build_coords(evt.x_pixels, evt.y_pixels, viewport);
    let modifiers = map_modifiers(evt.modifiers);
    let data = SerializedMouseData::new(button, to_button_set(button), coords, modifiers);

    if pressed {
        vec![RawInputEvent {
            name: "mousedown",
            data: EventData::Mouse(data),
            bubbles: true,
        }]
    } else {
        vec![
            RawInputEvent {
                name: "mousemove",
                data: EventData::Mouse(data.clone()),
                bubbles: true,
            },
            RawInputEvent {
                name: "mouseenter",
                data: EventData::Mouse(data),
                bubbles: true,
            },
        ]
    }
}

pub fn use_raw_input() -> Signal<Option<RawInputEvent>> {
    let bus = use_context::<TuiInputBus>();
    let signal = use_signal(|| None);
    let _subscription = use_hook(|| {
        let signal = signal.clone();
        bus.subscribe(Rc::new(move |event| {
            *signal.write_unchecked() = Some(event);
        }))
    });
    signal
}

pub fn use_keyboard_input() -> Signal<Option<SerializedKeyboardData>> {
    let bus = use_context::<TuiInputBus>();
    let signal = use_signal(|| None);
    let _subscription = use_hook(|| {
        let signal = signal.clone();
        bus.subscribe(Rc::new(move |event| {
            if let EventData::Keyboard(data) = event.data {
                *signal.write_unchecked() = Some(data);
            }
        }))
    });
    signal
}

pub fn use_mouse_input() -> Signal<Option<SerializedMouseData>> {
    let bus = use_context::<TuiInputBus>();
    let signal = use_signal(|| None);
    let _subscription = use_hook(|| {
        let signal = signal.clone();
        bus.subscribe(Rc::new(move |event| {
            if let EventData::Mouse(data) = event.data {
                *signal.write_unchecked() = Some(data);
            }
        }))
    });
    signal
}

pub fn use_wheel_input() -> Signal<Option<SerializedWheelData>> {
    let bus = use_context::<TuiInputBus>();
    let signal = use_signal(|| None);
    let _subscription = use_hook(|| {
        let signal = signal.clone();
        bus.subscribe(Rc::new(move |event| {
            if let EventData::Wheel(data) = event.data {
                *signal.write_unchecked() = Some(data);
            }
        }))
    });
    signal
}

fn wheel_delta(buttons: MouseButtons) -> (f64, f64) {
    let sign = if buttons.contains(MouseButtons::WHEEL_POSITIVE) {
        1.0
    } else {
        -1.0
    };
    if buttons.contains(MouseButtons::VERT_WHEEL) {
        (0.0, sign)
    } else if buttons.contains(MouseButtons::HORZ_WHEEL) {
        (sign, 0.0)
    } else {
        (0.0, 0.0)
    }
}

fn button_from_mask(mask: MouseButtons) -> Option<MouseButton> {
    if mask.contains(MouseButtons::LEFT) {
        Some(MouseButton::Primary)
    } else if mask.contains(MouseButtons::RIGHT) {
        Some(MouseButton::Secondary)
    } else if mask.contains(MouseButtons::MIDDLE) {
        Some(MouseButton::Auxiliary)
    } else {
        None
    }
}

fn build_coords(x: u16, y: u16, viewport: Rect) -> (f64, f64, Coordinates) {
    let clamped_x = (x as i64)
        .min(viewport.width as i64 - 1)
        .max(0) as f64;
    let clamped_y = (y as i64)
        .min(viewport.height as i64 - 1)
        .max(0) as f64;
    let screen = ScreenPoint::new(clamped_x, clamped_y);
    let client = ClientPoint::new(clamped_x, clamped_y);
    let element = ElementPoint::new(clamped_x, clamped_y);
    let page = PagePoint::new(clamped_x, clamped_y);
    (clamped_x, clamped_y, Coordinates::new(screen, client, element, page))
}

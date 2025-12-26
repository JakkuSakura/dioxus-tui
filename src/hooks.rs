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

#[derive(Clone, Default)]
pub struct ViewportBus {
    listeners: Rc<RefCell<Vec<Option<Rc<dyn Fn(Rect)>>>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Underline,
    Beam,
    Crosshair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorUnit {
    Cell,
    Pixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMode {
    FollowMouse,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorCommand {
    Show,
    Hide,
    SetStyle(CursorStyle),
    FollowMouse,
    SetCellPosition(f32, f32),
    SetPixelPosition(f32, f32),
}

#[derive(Clone)]
pub(crate) struct CursorState {
    pub(crate) visible: bool,
    pub(crate) style: CursorStyle,
    pub(crate) mode: CursorMode,
    pub(crate) unit: CursorUnit,
    pub(crate) position: Option<(f32, f32)>,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            visible: false,
            style: CursorStyle::Block,
            mode: CursorMode::FollowMouse,
            unit: CursorUnit::Cell,
            position: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct CursorBus {
    listeners: Rc<RefCell<Vec<Option<Rc<dyn Fn(CursorCommand)>>>>>,
}

impl CursorBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn subscribe(&self, listener: Rc<dyn Fn(CursorCommand)>) -> CursorSubscription {
        let mut listeners = self.listeners.borrow_mut();
        let id = listeners.len();
        listeners.push(Some(listener));
        Rc::new(CursorSubscriptionInner {
            id,
            listeners: Rc::downgrade(&self.listeners),
        })
    }

    pub fn publish(&self, event: CursorCommand) {
        for listener in self.listeners.borrow().iter().flatten() {
            listener(event);
        }
    }
}

pub(crate) type CursorSubscription = Rc<CursorSubscriptionInner>;

pub(crate) struct CursorSubscriptionInner {
    id: usize,
    listeners: Weak<RefCell<Vec<Option<Rc<dyn Fn(CursorCommand)>>>>>,
}

impl Drop for CursorSubscriptionInner {
    fn drop(&mut self) {
        if let Some(listeners) = self.listeners.upgrade() {
            if let Some(slot) = listeners.borrow_mut().get_mut(self.id) {
                *slot = None;
            }
        }
    }
}

impl ViewportBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn subscribe(&self, listener: Rc<dyn Fn(Rect)>) -> ViewportSubscription {
        let mut listeners = self.listeners.borrow_mut();
        let id = listeners.len();
        listeners.push(Some(listener));
        Rc::new(ViewportSubscriptionInner {
            id,
            listeners: Rc::downgrade(&self.listeners),
        })
    }

    pub fn publish(&self, rect: Rect) {
        for listener in self.listeners.borrow().iter().flatten() {
            listener(rect);
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

pub(crate) struct ViewportSubscriptionInner {
    id: usize,
    listeners: Weak<RefCell<Vec<Option<Rc<dyn Fn(Rect)>>>>>,
}

impl Drop for ViewportSubscriptionInner {
    fn drop(&mut self) {
        if let Some(listeners) = self.listeners.upgrade() {
            if let Some(slot) = listeners.borrow_mut().get_mut(self.id) {
                *slot = None;
            }
        }
    }
}

pub type InputSubscription = Rc<InputSubscriptionInner>;
pub type ViewportSubscription = Rc<ViewportSubscriptionInner>;

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
        TermKeyCode::Char('\n' | '\r') => (Key::Enter, Code::Enter),
        TermKeyCode::Char(c) => (Key::Character(c.to_string()), code_from_char(c)),
        TermKeyCode::Tab => (Key::Tab, Code::Tab),
        TermKeyCode::Escape => (Key::Escape, Code::Escape),
        TermKeyCode::Enter => (Key::Enter, Code::Enter),
        TermKeyCode::KeyPadHome => (Key::Home, Code::Numpad7),
        TermKeyCode::KeyPadEnd => (Key::End, Code::Numpad1),
        TermKeyCode::KeyPadPageUp => (Key::PageUp, Code::Numpad9),
        TermKeyCode::KeyPadPageDown => (Key::PageDown, Code::Numpad3),
        TermKeyCode::KeyPadBegin => (Key::Clear, Code::Numpad5),
        TermKeyCode::Backspace => (Key::Backspace, Code::Backspace),
        TermKeyCode::Insert => (Key::Insert, Code::Insert),
        TermKeyCode::Delete => (Key::Delete, Code::Delete),
        TermKeyCode::Home => (Key::Home, Code::Home),
        TermKeyCode::End => (Key::End, Code::End),
        TermKeyCode::PageUp => (Key::PageUp, Code::PageUp),
        TermKeyCode::PageDown => (Key::PageDown, Code::PageDown),
        TermKeyCode::LeftArrow => (Key::ArrowLeft, Code::ArrowLeft),
        TermKeyCode::RightArrow => (Key::ArrowRight, Code::ArrowRight),
        TermKeyCode::UpArrow => (Key::ArrowUp, Code::ArrowUp),
        TermKeyCode::DownArrow => (Key::ArrowDown, Code::ArrowDown),
        TermKeyCode::NumLock => (Key::NumLock, Code::NumLock),
        TermKeyCode::ScrollLock => (Key::ScrollLock, Code::ScrollLock),
        TermKeyCode::Copy => (Key::Copy, Code::Copy),
        TermKeyCode::Cut => (Key::Cut, Code::Cut),
        TermKeyCode::Paste => (Key::Paste, Code::Paste),
        TermKeyCode::BrowserBack => (Key::BrowserBack, Code::BrowserBack),
        TermKeyCode::BrowserForward => (Key::BrowserForward, Code::BrowserForward),
        TermKeyCode::BrowserRefresh => (Key::BrowserRefresh, Code::BrowserRefresh),
        TermKeyCode::BrowserStop => (Key::BrowserStop, Code::BrowserStop),
        TermKeyCode::BrowserSearch => (Key::BrowserSearch, Code::BrowserSearch),
        TermKeyCode::BrowserFavorites => (Key::BrowserFavorites, Code::BrowserFavorites),
        TermKeyCode::BrowserHome => (Key::BrowserHome, Code::BrowserHome),
        TermKeyCode::VolumeMute => (Key::AudioVolumeMute, Code::AudioVolumeMute),
        TermKeyCode::VolumeDown => (Key::AudioVolumeDown, Code::AudioVolumeDown),
        TermKeyCode::VolumeUp => (Key::AudioVolumeUp, Code::AudioVolumeUp),
        TermKeyCode::MediaNextTrack => (Key::MediaTrackNext, Code::MediaTrackNext),
        TermKeyCode::MediaPrevTrack => (Key::MediaTrackPrevious, Code::MediaTrackPrevious),
        TermKeyCode::MediaStop => (Key::MediaStop, Code::MediaStop),
        TermKeyCode::MediaPlayPause => (Key::MediaPlayPause, Code::MediaPlayPause),
        TermKeyCode::Numpad0 => (Key::Character("0".to_string()), Code::Numpad0),
        TermKeyCode::Numpad1 => (Key::Character("1".to_string()), Code::Numpad1),
        TermKeyCode::Numpad2 => (Key::Character("2".to_string()), Code::Numpad2),
        TermKeyCode::Numpad3 => (Key::Character("3".to_string()), Code::Numpad3),
        TermKeyCode::Numpad4 => (Key::Character("4".to_string()), Code::Numpad4),
        TermKeyCode::Numpad5 => (Key::Character("5".to_string()), Code::Numpad5),
        TermKeyCode::Numpad6 => (Key::Character("6".to_string()), Code::Numpad6),
        TermKeyCode::Numpad7 => (Key::Character("7".to_string()), Code::Numpad7),
        TermKeyCode::Numpad8 => (Key::Character("8".to_string()), Code::Numpad8),
        TermKeyCode::Numpad9 => (Key::Character("9".to_string()), Code::Numpad9),
        TermKeyCode::Multiply => (Key::Character("*".to_string()), Code::NumpadMultiply),
        TermKeyCode::Add => (Key::Character("+".to_string()), Code::NumpadAdd),
        TermKeyCode::Separator => (Key::Character(",".to_string()), Code::NumpadComma),
        TermKeyCode::Subtract => (Key::Character("-".to_string()), Code::NumpadSubtract),
        TermKeyCode::Decimal => (Key::Character(".".to_string()), Code::NumpadDecimal),
        TermKeyCode::Divide => (Key::Character("/".to_string()), Code::NumpadDivide),
        TermKeyCode::Function(n) => (
            key_from_function(n).unwrap_or(Key::Unidentified),
            code_from_function(n).unwrap_or(Code::Unidentified),
        ),
        _ => (Key::Unidentified, Code::Unidentified),
    }
}

fn key_from_function(n: u8) -> Option<Key> {
    Some(match n {
        1 => Key::F1,
        2 => Key::F2,
        3 => Key::F3,
        4 => Key::F4,
        5 => Key::F5,
        6 => Key::F6,
        7 => Key::F7,
        8 => Key::F8,
        9 => Key::F9,
        10 => Key::F10,
        11 => Key::F11,
        12 => Key::F12,
        13 => Key::F13,
        14 => Key::F14,
        15 => Key::F15,
        16 => Key::F16,
        17 => Key::F17,
        18 => Key::F18,
        19 => Key::F19,
        20 => Key::F20,
        21 => Key::F21,
        22 => Key::F22,
        23 => Key::F23,
        24 => Key::F24,
        _ => return None,
    })
}

fn code_from_function(n: u8) -> Option<Code> {
    Some(match n {
        1 => Code::F1,
        2 => Code::F2,
        3 => Code::F3,
        4 => Code::F4,
        5 => Code::F5,
        6 => Code::F6,
        7 => Code::F7,
        8 => Code::F8,
        9 => Code::F9,
        10 => Code::F10,
        11 => Code::F11,
        12 => Code::F12,
        13 => Code::F13,
        14 => Code::F14,
        15 => Code::F15,
        16 => Code::F16,
        17 => Code::F17,
        18 => Code::F18,
        19 => Code::F19,
        20 => Code::F20,
        21 => Code::F21,
        22 => Code::F22,
        23 => Code::F23,
        24 => Code::F24,
        _ => return None,
    })
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
    use super::*;
    use dioxus_html::input_data::keyboard_types::Code;
    use dioxus_html::point_interaction::InteractionLocation;
    use termwiz::input::{InputEvent, Modifiers, MouseButtons, MouseEvent, PixelMouseEvent};

    #[test]
    fn code_from_char_maps_common_ascii() {
        assert_eq!(code_from_char('q'), Code::KeyQ);
        assert_eq!(code_from_char('Q'), Code::KeyQ);
        assert_eq!(code_from_char(' '), Code::Space);
        assert_eq!(code_from_char('7'), Code::Digit7);
        assert_eq!(code_from_char('['), Code::BracketLeft);
    }

    #[test]
    fn cursor_position_maps_to_rendered_location() {
        let viewport = Rect::new(0, 0, 80, 24);
        let pixel_viewport = Some(Rect::new(0, 0, 640, 480));

        let mut mouse_state = RawMouseState::default();
        let events = raw_input_from_termwiz(
            &InputEvent::Mouse(MouseEvent {
                x: 12,
                y: 7,
                mouse_buttons: MouseButtons::NONE,
                modifiers: Modifiers::NONE,
            }),
            viewport,
            pixel_viewport,
            &mut mouse_state,
        );

        let cell_event = events
            .iter()
            .find(|evt| evt.name == "mousemove")
            .expect("mousemove event");

        let EventData::Mouse(cell_mouse) = &cell_event.data else {
            panic!("expected mouse data");
        };
        let coords = cell_mouse.client_coordinates();
        assert_eq!(coords.x, 12.0);
        assert_eq!(coords.y, 7.0);
        let rendered_left = format!("{}ch", coords.x.floor());
        let rendered_top = format!("{}ch", coords.y.floor());
        assert_eq!(rendered_left, "12ch");
        assert_eq!(rendered_top, "7ch");

        let events = raw_input_from_termwiz(
            &InputEvent::PixelMouse(PixelMouseEvent {
                x_pixels: 33,
                y_pixels: 44,
                mouse_buttons: MouseButtons::NONE,
                modifiers: Modifiers::NONE,
            }),
            viewport,
            pixel_viewport,
            &mut mouse_state,
        );

        let pixel_event = events
            .iter()
            .find(|evt| evt.name == "pixelmousemove")
            .expect("pixelmousemove event");

        let EventData::Mouse(pixel_mouse) = &pixel_event.data else {
            panic!("expected pixel mouse data");
        };
        let coords = pixel_mouse.client_coordinates();
        assert_eq!(coords.x, 33.0);
        assert_eq!(coords.y, 44.0);
        let rendered_left = format!("{}px", coords.x);
        let rendered_top = format!("{}px", coords.y);
        assert_eq!(rendered_left, "33px");
        assert_eq!(rendered_top, "44px");
    }
}

fn to_button_set(btn: Option<MouseButton>) -> MouseButtonSet {
    let mut set = MouseButtonSet::empty();
    if let Some(b) = btn {
        set.insert(b);
    }
    set
}

#[derive(Default)]
pub struct RawMouseState {
    last_buttons: MouseButtons,
}

pub fn raw_input_from_termwiz(
    evt: &InputEvent,
    viewport: Rect,
    pixel_viewport: Option<Rect>,
    mouse_state: &mut RawMouseState,
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
        InputEvent::Mouse(mouse_evt) => map_mouse_input(mouse_evt, viewport, mouse_state),
        InputEvent::PixelMouse(mouse_evt) => {
            let viewport = pixel_viewport.unwrap_or(Rect::new(0, 0, 0, 0));
            map_pixel_mouse_input(mouse_evt, viewport, mouse_state)
        }
        _ => Vec::new(),
    }
}

fn map_mouse_input(
    evt: &MouseEvent,
    viewport: Rect,
    mouse_state: &mut RawMouseState,
) -> Vec<RawInputEvent> {
    if evt.mouse_buttons.contains(MouseButtons::VERT_WHEEL)
        || evt.mouse_buttons.contains(MouseButtons::HORZ_WHEEL)
    {
        let (delta_x, delta_y) = wheel_delta(evt.mouse_buttons.clone());
        let (x, y) = normalize_cell_coords(evt.x, evt.y);
        let (_, _, coords) = build_coords(x, y, viewport);
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

    let current_buttons = evt.mouse_buttons.clone();
    let previous_buttons = mouse_state.last_buttons.clone();
    let released = previous_buttons.clone() & !current_buttons.clone();
    let added = current_buttons.clone() & !previous_buttons;
    mouse_state.last_buttons = current_buttons;

    let mut events = Vec::new();
    let modifiers = map_modifiers(evt.modifiers);

    for button in buttons_from_mask(released.clone()) {
        let (x, y) = normalize_cell_coords(evt.x, evt.y);
        let (_, _, coords) = build_coords(x, y, viewport);
        let data = SerializedMouseData::new(
            Some(button),
            to_button_set(Some(button)),
            coords,
            modifiers,
        );
        events.push(RawInputEvent {
            name: "mouseup",
            data: EventData::Mouse(data),
            bubbles: true,
        });
    }

    for button in buttons_from_mask(added.clone()) {
        let (x, y) = normalize_cell_coords(evt.x, evt.y);
        let (_, _, coords) = build_coords(x, y, viewport);
        let data = SerializedMouseData::new(
            Some(button),
            to_button_set(Some(button)),
            coords,
            modifiers,
        );
        events.push(RawInputEvent {
            name: "mousedown",
            data: EventData::Mouse(data),
            bubbles: true,
        });
    }

    if !events.is_empty() {
        return events;
    }

    let (x, y) = normalize_cell_coords(evt.x, evt.y);
    let (_, _, coords) = build_coords(x, y, viewport);
    let data = SerializedMouseData::new(None, MouseButtonSet::empty(), coords, modifiers);
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

fn map_pixel_mouse_input(
    evt: &termwiz::input::PixelMouseEvent,
    viewport: Rect,
    mouse_state: &mut RawMouseState,
) -> Vec<RawInputEvent> {
    let fallback = Rect::new(
        0,
        0,
        evt.x_pixels.saturating_add(1).max(1),
        evt.y_pixels.saturating_add(1).max(1),
    );
    let viewport = if viewport.width == 0 || viewport.height == 0 {
        fallback
    } else {
        Rect::new(
            viewport.x,
            viewport.y,
            viewport.width.max(fallback.width),
            viewport.height.max(fallback.height),
        )
    };
    if evt.mouse_buttons.contains(MouseButtons::VERT_WHEEL)
        || evt.mouse_buttons.contains(MouseButtons::HORZ_WHEEL)
    {
        let (delta_x, delta_y) = wheel_delta(evt.mouse_buttons.clone());
        let (x, y) = normalize_pixel_coords(evt.x_pixels, evt.y_pixels);
        let (_, _, coords) = build_coords(x, y, viewport);
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
            name: "pixelwheel",
            data: EventData::Wheel(data),
            bubbles: true,
        }];
    }

    let current_buttons = evt.mouse_buttons.clone();
    let previous_buttons = mouse_state.last_buttons.clone();
    let released = previous_buttons.clone() & !current_buttons.clone();
    let added = current_buttons.clone() & !previous_buttons;
    mouse_state.last_buttons = current_buttons;

    let mut events = Vec::new();
    let modifiers = map_modifiers(evt.modifiers);

    for button in buttons_from_mask(released.clone()) {
        let (x, y) = normalize_pixel_coords(evt.x_pixels, evt.y_pixels);
        let (_, _, coords) = build_coords(x, y, viewport);
        let data = SerializedMouseData::new(
            Some(button),
            to_button_set(Some(button)),
            coords,
            modifiers,
        );
        events.push(RawInputEvent {
            name: "pixelmouseup",
            data: EventData::Mouse(data),
            bubbles: true,
        });
    }

    for button in buttons_from_mask(added.clone()) {
        let (x, y) = normalize_pixel_coords(evt.x_pixels, evt.y_pixels);
        let (_, _, coords) = build_coords(x, y, viewport);
        let data = SerializedMouseData::new(
            Some(button),
            to_button_set(Some(button)),
            coords,
            modifiers,
        );
        events.push(RawInputEvent {
            name: "pixelmousedown",
            data: EventData::Mouse(data),
            bubbles: true,
        });
    }

    if !events.is_empty() {
        return events;
    }

    let (x, y) = normalize_pixel_coords(evt.x_pixels, evt.y_pixels);
    let (_, _, coords) = build_coords(x, y, viewport);
    let data = SerializedMouseData::new(None, MouseButtonSet::empty(), coords, modifiers);
    vec![
        RawInputEvent {
            name: "pixelmousemove",
            data: EventData::Mouse(data.clone()),
            bubbles: true,
        },
        RawInputEvent {
            name: "pixelmouseenter",
            data: EventData::Mouse(data),
            bubbles: true,
        },
    ]
}

fn buttons_from_mask(mask: MouseButtons) -> Vec<MouseButton> {
    let mut buttons = Vec::new();
    if mask.contains(MouseButtons::LEFT) {
        buttons.push(MouseButton::Primary);
    }
    if mask.contains(MouseButtons::RIGHT) {
        buttons.push(MouseButton::Secondary);
    }
    if mask.contains(MouseButtons::MIDDLE) {
        buttons.push(MouseButton::Auxiliary);
    }
    buttons
}

fn normalize_cell_coords(x: u16, y: u16) -> (u16, u16) {
    (x.saturating_sub(1), y.saturating_sub(1))
}

fn normalize_pixel_coords(x: u16, y: u16) -> (u16, u16) {
    (x.saturating_sub(1), y.saturating_sub(1))
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

pub fn use_viewport() -> Signal<Rect> {
    let bus = use_context::<ViewportBus>();
    let signal = use_signal(|| Rect::new(0, 0, 0, 0));
    let _subscription = use_hook(|| {
        let signal = signal.clone();
        bus.subscribe(Rc::new(move |rect| {
            *signal.write_unchecked() = rect;
        }))
    });
    signal
}

#[derive(Clone)]
pub struct CursorHandle {
    bus: CursorBus,
}

impl CursorHandle {
    pub fn show(&self) {
        self.bus.publish(CursorCommand::Show);
    }

    pub fn hide(&self) {
        self.bus.publish(CursorCommand::Hide);
    }

    pub fn follow_mouse(&self) {
        self.bus.publish(CursorCommand::FollowMouse);
    }

    pub fn set_style(&self, style: CursorStyle) {
        self.bus.publish(CursorCommand::SetStyle(style));
    }

    pub fn set_cell_position(&self, x: f32, y: f32) {
        self.bus.publish(CursorCommand::SetCellPosition(x, y));
    }

    pub fn set_pixel_position(&self, x: f32, y: f32) {
        self.bus.publish(CursorCommand::SetPixelPosition(x, y));
    }
}

pub fn use_cursor() -> CursorHandle {
    let bus = use_context::<CursorBus>();
    CursorHandle { bus }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaretCommand {
    Show,
    Hide,
    SetPosition(u16, u16),
}

#[derive(Clone, Default)]
pub struct CaretBus {
    listeners: Rc<RefCell<Vec<Option<Rc<dyn Fn(CaretCommand)>>>>>,
}

impl CaretBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn subscribe(&self, listener: Rc<dyn Fn(CaretCommand)>) -> CaretSubscription {
        let mut listeners = self.listeners.borrow_mut();
        let id = listeners.len();
        listeners.push(Some(listener));
        Rc::new(CaretSubscriptionInner {
            id,
            listeners: Rc::downgrade(&self.listeners),
        })
    }

    pub fn publish(&self, event: CaretCommand) {
        for listener in self.listeners.borrow().iter().flatten() {
            listener(event);
        }
    }
}

pub(crate) type CaretSubscription = Rc<CaretSubscriptionInner>;

pub(crate) struct CaretSubscriptionInner {
    id: usize,
    listeners: Weak<RefCell<Vec<Option<Rc<dyn Fn(CaretCommand)>>>>>,
}

impl Drop for CaretSubscriptionInner {
    fn drop(&mut self) {
        if let Some(listeners) = self.listeners.upgrade() {
            if let Some(slot) = listeners.borrow_mut().get_mut(self.id) {
                *slot = None;
            }
        }
    }
}

#[derive(Clone)]
pub struct CaretHandle {
    bus: CaretBus,
}

impl CaretHandle {
    pub fn show(&self) {
        self.bus.publish(CaretCommand::Show);
    }

    pub fn hide(&self) {
        self.bus.publish(CaretCommand::Hide);
    }

    pub fn set_cell_position(&self, x: u16, y: u16) {
        self.bus.publish(CaretCommand::SetPosition(x, y));
    }
}

pub fn use_caret() -> CaretHandle {
    let bus = use_context::<CaretBus>();
    CaretHandle { bus }
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

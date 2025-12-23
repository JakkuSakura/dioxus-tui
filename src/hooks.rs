use std::any::Any;

use dioxus_core::ElementId;
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

fn map_modifiers(mods: TermModifiers) -> Modifiers {
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

fn map_code(key: &KeyEvent) -> (Key, Code) {
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

pub fn event_from_termwiz(
    evt: InputEvent,
    target: ElementId,
    viewport: Rect,
    pixel_viewport: Option<Rect>,
) -> Vec<(ElementId, &'static str, EventData, bool)> {
    match evt {
        InputEvent::Key(key) => {
            let (key_val, code) = map_code(&key);
            let mods = map_modifiers(key.modifiers);
            let data =
                SerializedKeyboardData::new(key_val, code, Location::Standard, false, mods, false);
            vec![(target, "keydown", EventData::Keyboard(data), true)]
        }
        InputEvent::Mouse(mouse_evt) => map_mouse(mouse_evt, target, viewport),
        InputEvent::PixelMouse(mouse_evt) => {
            let viewport = pixel_viewport.unwrap_or(viewport);
            map_pixel_mouse(mouse_evt, target, viewport)
        }
        _ => Vec::new(),
    }
}

fn map_mouse(
    evt: MouseEvent,
    target: ElementId,
    viewport: Rect,
) -> Vec<(ElementId, &'static str, EventData, bool)> {
    if evt.mouse_buttons.contains(MouseButtons::VERT_WHEEL)
        || evt.mouse_buttons.contains(MouseButtons::HORZ_WHEEL)
    {
        let (delta_x, delta_y) = wheel_delta(evt.mouse_buttons);
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
        return vec![(target, "wheel", EventData::Wheel(data), true)];
    }

    let btn = button_from_mask(evt.mouse_buttons);
    let (pressed, button) = match btn {
        Some(b) => (true, Some(b)),
        None => (false, None),
    };

    let (_, _, coords) = build_coords(evt.x, evt.y, viewport);
    let modifiers = map_modifiers(evt.modifiers);
    let data = SerializedMouseData::new(button, to_button_set(button), coords, modifiers);

    if pressed {
        vec![(target, "mousedown", EventData::Mouse(data), true)]
    } else {
        vec![
            (target, "mousemove", EventData::Mouse(data.clone()), true),
            (target, "mouseenter", EventData::Mouse(data), true),
        ]
    }
}

fn map_pixel_mouse(
    evt: termwiz::input::PixelMouseEvent,
    target: ElementId,
    viewport: Rect,
) -> Vec<(ElementId, &'static str, EventData, bool)> {
    if evt.mouse_buttons.contains(MouseButtons::VERT_WHEEL)
        || evt.mouse_buttons.contains(MouseButtons::HORZ_WHEEL)
    {
        let (delta_x, delta_y) = wheel_delta(evt.mouse_buttons);
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
        return vec![(target, "wheel", EventData::Wheel(data), true)];
    }

    let btn = button_from_mask(evt.mouse_buttons);
    let (pressed, button) = match btn {
        Some(b) => (true, Some(b)),
        None => (false, None),
    };

    let (_, _, coords) = build_coords(evt.x_pixels, evt.y_pixels, viewport);
    let modifiers = map_modifiers(evt.modifiers);
    let data = SerializedMouseData::new(button, to_button_set(button), coords, modifiers);

    if pressed {
        vec![(target, "mousedown", EventData::Mouse(data), true)]
    } else {
        vec![
            (target, "mousemove", EventData::Mouse(data.clone()), true),
            (target, "mouseenter", EventData::Mouse(data), true),
        ]
    }
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

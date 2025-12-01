use dioxus_html::*;

use crate::hooks::EventData;

fn downcast(event: &PlatformEventData) -> EventData {
    event
        .downcast::<EventData>()
        .expect("event should be of type EventData")
        .clone()
}

pub(crate) struct SerializedHtmlEventConverter;

impl HtmlEventConverter for SerializedHtmlEventConverter {
    fn convert_animation_data(&self, _: &PlatformEventData) -> AnimationData {
        panic!("animation events not supported")
    }

    fn convert_cancel_data(&self, _: &PlatformEventData) -> CancelData {
        panic!("cancel events not supported")
    }

    fn convert_clipboard_data(&self, _: &PlatformEventData) -> ClipboardData {
        panic!("clipboard events not supported")
    }

    fn convert_composition_data(&self, _: &PlatformEventData) -> CompositionData {
        panic!("composition events not supported")
    }

    fn convert_drag_data(&self, _: &PlatformEventData) -> DragData {
        panic!("drag events not supported")
    }

    fn convert_focus_data(&self, _: &PlatformEventData) -> FocusData {
        panic!("focus events not supported")
    }

    fn convert_form_data(&self, _: &PlatformEventData) -> FormData {
        panic!("form events not supported")
    }

    fn convert_image_data(&self, _: &PlatformEventData) -> ImageData {
        panic!("image events not supported")
    }

    fn convert_keyboard_data(&self, event: &PlatformEventData) -> KeyboardData {
        if let EventData::Keyboard(event) = downcast(event) {
            KeyboardData::new(event)
        } else {
            panic!("event should be of type Keyboard")
        }
    }

    fn convert_media_data(&self, _: &PlatformEventData) -> MediaData {
        panic!("media events not supported")
    }

    fn convert_mounted_data(&self, _: &PlatformEventData) -> MountedData {
        panic!("mounted events not supported")
    }

    fn convert_mouse_data(&self, event: &PlatformEventData) -> MouseData {
        if let EventData::Mouse(event) = downcast(event) {
            MouseData::new(event)
        } else {
            panic!("event should be of type Mouse")
        }
    }

    fn convert_pointer_data(&self, _: &PlatformEventData) -> PointerData {
        panic!("pointer events not supported")
    }

    fn convert_resize_data(&self, _: &PlatformEventData) -> ResizeData {
        panic!("resize events not supported")
    }

    fn convert_scroll_data(&self, _: &PlatformEventData) -> ScrollData {
        panic!("scroll events not supported")
    }

    fn convert_selection_data(&self, _: &PlatformEventData) -> SelectionData {
        panic!("selection events not supported")
    }

    fn convert_toggle_data(&self, _: &PlatformEventData) -> ToggleData {
        panic!("toggle events not supported")
    }

    fn convert_touch_data(&self, _: &PlatformEventData) -> TouchData {
        panic!("touch events not supported")
    }

    fn convert_transition_data(&self, _: &PlatformEventData) -> TransitionData {
        panic!("transition events not supported")
    }

    fn convert_wheel_data(&self, event: &PlatformEventData) -> WheelData {
        if let EventData::Wheel(event) = downcast(event) {
            WheelData::new(event)
        } else {
            panic!("event should be of type Wheel")
        }
    }

    fn convert_visible_data(&self, _: &PlatformEventData) -> VisibleData {
        panic!("visible events not supported")
    }
}

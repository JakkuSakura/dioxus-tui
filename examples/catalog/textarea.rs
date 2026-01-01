use dioxus::prelude::*;
use dioxus::prelude::HasKeyboardData;
use dioxus_html::input_data::keyboard_types::Key;
use dioxus_tui::{CaretMode, TuiContext, use_keyboard_input, use_layout_rect};
use dioxus_tui_components::{TextareaAction, TextareaView, TextBuffer, use_textarea_view_model};

use crate::catalog::ExampleFrame;

#[derive(Clone)]
struct CaretDebugInfo {
    layout: dioxus_tui::Rect,
    has_layout: bool,
    caret: (u16, u16),
    has_caret: bool,
    row: usize,
    col: usize,
    mode: CaretMode,
}

impl Default for CaretDebugInfo {
    fn default() -> Self {
        Self {
            layout: dioxus_tui::Rect::new(0, 0, 0, 0),
            has_layout: false,
            caret: (0, 0),
            has_caret: false,
            row: 0,
            col: 0,
            mode: CaretMode::Physical,
        }
    }
}

fn caret_position(layout: dioxus_tui::Rect, state: &TextBuffer, padding: u16) -> (u16, u16) {
    let max_x = layout.x.saturating_add(layout.width.saturating_sub(1));
    let max_y = layout.y.saturating_add(layout.height.saturating_sub(1));
    let caret_x = layout
        .x
        .saturating_add(padding)
        .saturating_add(state.col as u16)
        .min(max_x);
    let caret_y = layout
        .y
        .saturating_add(padding)
        .saturating_add(state.row as u16)
        .min(max_y);
    (caret_x, caret_y)
}

#[component]
fn TextareaCaretDebug(
    buffer: Signal<TextBuffer>,
    caret_mode: Signal<CaretMode>,
    debug_info: Signal<CaretDebugInfo>,
) -> Element {
    let layout_rect = use_layout_rect();
    let _layout_subscription = layout_rect.read().clone();
    let mut debug_update = debug_info.clone();

    use_effect(move || {
        let state = buffer.read().clone();
        let layout = layout_rect.read().clone();
        let padding = 1u16;
        let (caret, has_caret) = if let Some(layout) = layout {
            let caret = caret_position(layout, &state, padding);
            (caret, true)
        } else {
            ((0, 0), false)
        };
        let (layout, has_layout) = layout.map_or((dioxus_tui::Rect::new(0, 0, 0, 0), false), |rect| {
            (rect, true)
        });
        debug_update.set(CaretDebugInfo {
            layout,
            has_layout,
            caret,
            has_caret,
            row: state.row,
            col: state.col,
            mode: *caret_mode.read(),
        });
    });

    rsx! {
        TextareaView {
            buffer,
            caret_mode: *caret_mode.read(),
            padding: 1,
        }
    }
}

pub fn app() -> Element {
    let tui: TuiContext = consume_context();
    let key_input = use_keyboard_input();
    let vm = use_textarea_view_model();
    let vm_update = vm.clone();
    let caret_mode = use_signal(|| CaretMode::Physical);
    let mut caret_mode_update = caret_mode.clone();
    let debug_info = use_signal(CaretDebugInfo::default);

    use_effect(move || {
        let Some(data) = key_input.read().clone() else {
            return;
        };
        if data.key() == Key::F2 {
            caret_mode_update.with_mut(|mode| {
                *mode = match *mode {
                    CaretMode::Physical => CaretMode::Soft,
                    CaretMode::Soft => CaretMode::Physical,
                };
            });
            return;
        }
        if vm_update.handle_key(&data.key()) == TextareaAction::Quit {
            tui.quit();
        }
    });

    let debug = debug_info.read().clone();
    let caret_mode_label = match debug.mode {
        CaretMode::Physical => "physical",
        CaretMode::Soft => "soft",
    };

    rsx! {
        ExampleFrame {
            title: "Textarea",
            help: &[
                "Type to insert text. Enter makes a new line.",
                "Use arrow keys, Backspace, Delete. Esc to quit.",
                "F2 toggles caret mode (soft/physical).",
            ],

            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                align_items: "center",
                justify_content: "center",

                TextareaCaretDebug { buffer: vm.buffer(), caret_mode, debug_info }

                div {
                    margin_top: "1ch",
                    width: "80%",
                    color: "#a9b1d6",
                    "Caret row/col: {debug.row}, {debug.col}"
                }
                div {
                    width: "80%",
                    color: "#a9b1d6",
                    "Caret mode: {caret_mode_label}"
                }
                if debug.has_layout {
                    div {
                        width: "80%",
                        color: "#a9b1d6",
                        "Layout rect: {debug.layout.x},{debug.layout.y} {debug.layout.width}x{debug.layout.height}"
                    }
                }
                if debug.has_caret {
                    div {
                        width: "80%",
                        color: "#a9b1d6",
                        "Caret cell: {debug.caret.0},{debug.caret.1}"
                    }
                }
            }
        }
    }
}

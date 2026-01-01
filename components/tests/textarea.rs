use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Key;
use dioxus_tui::{CaretMode, Config, RawVirtualDom, Rect, Surface};
use dioxus_tui::test_utils::{CaretSnapshot, palette_entry_to_attr, render_once_with_caret};
use dioxus_tui_components::{TextBuffer, TextareaView};

#[test]
fn text_buffer_enter_key_preserves_second_line_content() {
    let mut buffer = TextBuffer::default();

    for ch in ["H", "e", "l", "l", "o"] {
        buffer.handle_key(&Key::Character(ch.to_string()));
    }
    buffer.handle_key(&Key::Enter);
    for ch in ["W", "o", "r", "l", "d"] {
        buffer.handle_key(&Key::Character(ch.to_string()));
    }

    assert_eq!(buffer.lines, vec!["Hello", "World"]);
    assert_eq!(buffer.row, 1);
    assert_eq!(buffer.col, 5);
}

#[derive(Clone, Props, PartialEq)]
struct TextareaRenderProbeProps {
    caret_mode: CaretMode,
}

#[component]
fn TextareaRenderProbe(props: TextareaRenderProbeProps) -> Element {
    let buffer = use_signal(|| {
        let mut buf = TextBuffer::default();
        buf.lines = vec!["Hello".to_string()];
        buf
    });

    rsx! {
        TextareaView {
            buffer,
            caret_mode: props.caret_mode,
            padding: 1,
        }
    }
}

fn render_textarea(caret_mode: CaretMode) -> (Surface, CaretSnapshot) {
    let area = Rect::new(0, 0, 20, 6);
    let cfg = Config::default();
    render_once_with_caret(
        cfg,
        RawVirtualDom::with_props(TextareaRenderProbe, TextareaRenderProbeProps { caret_mode }),
        area,
    )
    .expect("render once")
}

#[test]
fn textarea_view_renders_text_cells() {
    let (surface, _snapshot) = render_textarea(CaretMode::Physical);
    let width = surface.width() as usize;
    let mut found = false;
    for row in surface.content.chunks(width) {
        let line: String = row.iter().map(|cell| cell.ch).collect();
        if line.contains("Hello") {
            found = true;
            break;
        }
    }
    assert!(found, "expected text not found in surface");
}

#[test]
fn caret_overlay_renders_in_surface_cells() {
    let (surface_physical, physical_snapshot) = render_textarea(CaretMode::Physical);
    let (surface_soft, soft_snapshot) = render_textarea(CaretMode::Soft);

    assert_eq!(physical_snapshot.mode, CaretMode::Physical);
    assert_eq!(soft_snapshot.mode, CaretMode::Soft);
    let caret_pos = soft_snapshot.position.expect("caret position");
    assert_eq!(soft_snapshot.position, physical_snapshot.position);
    assert!(soft_snapshot.visible, "caret should be visible");
    assert!(physical_snapshot.visible, "caret should be visible");

    let palette_roles = dioxus_tui::PaletteRoles::default();
    let color_mode = dioxus_tui::ColorMode::Rgb;
    let default_fg = palette_entry_to_attr(palette_roles.fg_primary, color_mode, false);
    let default_bg = palette_entry_to_attr(palette_roles.bg_primary, color_mode, false);

    let width = surface_soft.width() as usize;
    let height = surface_soft.height() as usize;
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let base_cell = &surface_physical.content[idx];
            let soft_cell = &surface_soft.content[idx];
            let expected_ch = base_cell.ch;
            assert_eq!(soft_cell.ch, expected_ch, "unexpected cell at ({x}, {y})");

            if (x as u16, y as u16) == caret_pos {
                let base_fg = base_cell.fg.unwrap_or(default_fg);
                let base_bg = base_cell.bg.unwrap_or(default_bg);
                assert_eq!(soft_cell.fg, Some(base_bg));
                assert_eq!(soft_cell.bg, Some(base_fg));
            } else {
                assert_eq!(soft_cell.fg, base_cell.fg);
                assert_eq!(soft_cell.bg, base_cell.bg);
            }
        }
    }
}

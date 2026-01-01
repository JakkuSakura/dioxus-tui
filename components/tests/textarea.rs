use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus::prelude::*;
use dioxus_core::VirtualDom;
use dioxus_html::input_data::keyboard_types::Key;
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use dioxus_tui::capabilities::InlineImageProtocol;
use dioxus_tui::layout::node_rect;
use dioxus_tui::render::{apply_caret_overlay_at, caret_changes};
use dioxus_tui::{CaretBus, CellMetrics, Config, LayoutBus, RawVirtualDom, Rect, TerminalCapabilities, render};
use dioxus_tui_components::{TextBuffer, TextareaView};
use termwiz::color::{ColorAttribute, SrgbaTuple};

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

#[component]
fn TextareaRenderProbe() -> Element {
    let buffer = use_signal(|| {
        let mut buf = TextBuffer::default();
        buf.lines = vec!["Hello".to_string()];
        buf
    });

    rsx! {
        TextareaView {
            buffer,
            caret_mode: dioxus_tui::CaretMode::Soft,
            padding: 1,
        }
    }
}

#[test]
fn textarea_view_renders_text_cells() {
    let area = Rect::new(0, 0, 20, 6);
    let raw = RawVirtualDom::new(TextareaRenderProbe);
    let surface = render::render_once(Config::default(), raw, area).expect("render once");
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
    let area = Rect::new(0, 0, 20, 6);
    let raw = RawVirtualDom::new(TextareaRenderProbe);
    let mut surface = render::render_once(Config::default(), raw, area).expect("render once");
    let (doc, root) = build_doc_with_layout(TextareaRenderProbe, area.width, area.height);
    let metrics = CellMetrics {
        cell_w_px: 8.0,
        cell_h_px: 16.0,
    };
    let rect = node_rect(
        &doc.inner,
        doc.inner.get_node(root).expect("root node"),
        area,
        metrics,
    );
    let caret_pos = caret_position(rect, &TextBuffer::default(), 1);
    let baseline: Vec<char> = surface.content.iter().map(|cell| cell.ch).collect();
    let baseline_fg: Vec<Option<termwiz::color::ColorAttribute>> = surface
        .content
        .iter()
        .map(|cell| cell.fg)
        .collect();
    let baseline_bg: Vec<Option<termwiz::color::ColorAttribute>> = surface
        .content
        .iter()
        .map(|cell| cell.bg)
        .collect();

    let capabilities = TerminalCapabilities {
        truecolor: false,
        inline_images: false,
        inline_protocol: InlineImageProtocol::None,
    };
    let metrics = CellMetrics {
        cell_w_px: 8.0,
        cell_h_px: 16.0,
    };
    let cfg = Config::default();
    let palette_roles = dioxus_tui::PaletteRoles::default();
    let color_mode = dioxus_tui::ColorMode::Rgb;
    let default_fg = palette_entry_to_attr(
        palette_roles.fg_primary,
        color_mode,
        capabilities.truecolor,
    );
    let default_bg = palette_entry_to_attr(
        palette_roles.bg_primary,
        color_mode,
        capabilities.truecolor,
    );
    apply_caret_overlay_at(
        &mut surface,
        caret_pos,
        cfg,
        &capabilities,
        metrics,
    );

    let width = surface.width() as usize;
    let height = surface.height() as usize;
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let actual = surface.content[idx].ch;
            let expected_ch = baseline[idx];
            assert_eq!(actual, expected_ch, "unexpected cell at ({x}, {y})");

            if (x as u16, y as u16) == caret_pos {
                let base_fg = baseline_fg[idx].unwrap_or(default_fg);
                let base_bg = baseline_bg[idx].unwrap_or(default_bg);
                assert_eq!(surface.content[idx].fg, Some(base_bg));
                assert_eq!(surface.content[idx].bg, Some(base_fg));
            } else {
                assert_eq!(surface.content[idx].fg, baseline_fg[idx]);
                assert_eq!(surface.content[idx].bg, baseline_bg[idx]);
            }
        }
    }
}

#[test]
fn caret_changes_include_visibility_and_position() {
    let changes = caret_changes(true, Some((2, 0)));

    assert_eq!(changes.len(), 2);
    assert!(matches!(changes[0], termwiz::surface::Change::CursorVisibility(_)));
    assert!(matches!(
        changes[1],
        termwiz::surface::Change::CursorPosition { x, y }
        if x == termwiz::surface::Position::Absolute(2)
            && y == termwiz::surface::Position::Absolute(0)
    ));
}

fn caret_position(layout: Rect, state: &TextBuffer, padding: u16) -> (u16, u16) {
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

fn build_doc_with_layout(
    app: fn() -> Element,
    width: u16,
    height: u16,
) -> (DioxusDocument, usize) {
    let metrics = CellMetrics {
        cell_w_px: 8.0,
        cell_h_px: 16.0,
    };
    let vdom = VirtualDom::new(app)
        .with_root_context(CaretBus::new())
        .with_root_context(LayoutBus::new());
    let viewport = Viewport::new(
        (width as f32 * metrics.cell_w_px).ceil().max(1.0) as u32,
        (height as f32 * metrics.cell_h_px).ceil().max(1.0) as u32,
        1.0,
        ColorScheme::Light,
    );
    let mut doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            viewport: Some(viewport),
            ..Default::default()
        },
    );
    doc.initial_build();
    let root = dioxus_tui::layout::resolve_document(
        &mut doc,
        Rect::new(0, 0, width, height),
        metrics,
    )
    .expect("root layout");
    (doc, root)
}

fn palette_entry_to_attr(
    entry: dioxus_tui::PaletteEntry,
    color_mode: dioxus_tui::ColorMode,
    truecolor: bool,
) -> ColorAttribute {
    match entry {
        dioxus_tui::PaletteEntry::Ansi(idx) | dioxus_tui::PaletteEntry::Palette256(idx) => {
            ColorAttribute::PaletteIndex(idx)
        }
        dioxus_tui::PaletteEntry::Rgb(r, g, b) => {
            let srgb = SrgbaTuple::from((r, g, b));
            let palette_idx_256 =
                16 + 36 * (r as u16 / 51) as u8 + 6 * (g as u16 / 51) as u8 + (b as u16 / 51) as u8;
            let base_idx = (if r >= 128 { 1 } else { 0 })
                | (if g >= 128 { 2 } else { 0 })
                | (if b >= 128 { 4 } else { 0 });
            match color_mode {
                dioxus_tui::ColorMode::BaseColors => ColorAttribute::PaletteIndex(base_idx),
                dioxus_tui::ColorMode::Ansi => {
                    ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256)
                }
                dioxus_tui::ColorMode::Rgb => {
                    if truecolor {
                        ColorAttribute::TrueColorWithDefaultFallback(srgb)
                    } else {
                        ColorAttribute::TrueColorWithPaletteFallback(srgb, palette_idx_256)
                    }
                }
            }
        }
    }
}

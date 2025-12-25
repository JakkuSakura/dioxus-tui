use std::any::Any;

use dioxus::prelude::*;
use dioxus_tui::{ColorMode, Config, CustomDrawMode, RawVirtualDom, Rect, Surface};
use dioxus_tui_btop::app;

const SNAPSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/btop_120x50.txt"
);

fn render_app(app: fn() -> Element, width: u16, height: u16) -> Surface {
    let cfg = Config::default()
        .with_color_mode(ColorMode::Rgb)
        .with_custom_draw_mode(CustomDrawMode::Native);
    let area = Rect::new(0, 0, width, height);

    let contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>> = Vec::new();
    let raw = RawVirtualDom::with_contexts(move |_| app(), (), contexts);

    dioxus_tui::render_surface_raw(raw, cfg, area).expect("render")
}

fn fixture_dims(contents: &str) -> (u16, u16) {
    let lines: Vec<&str> = contents.trim_end_matches('\n').lines().collect();
    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as u16;
    let height = lines.len() as u16;
    (width, height)
}

fn normalize_expected(contents: &str, width: u16, height: u16) -> String {
    let mut lines: Vec<String> = contents
        .trim_end_matches('\n')
        .lines()
        .map(|line| {
            let len = line.chars().count() as u16;
            if len >= width {
                line.to_string()
            } else {
                format!("{line}{}", " ".repeat((width - len) as usize))
            }
        })
        .collect();

    while lines.len() < height as usize {
        lines.push(" ".repeat(width as usize));
    }

    lines.join("\n")
}

fn assert_lines_eq(actual: &str, expected: &str) {
    let actual_lines: Vec<&str> = actual.lines().collect();
    let expected_lines: Vec<&str> = expected.lines().collect();
    let max = actual_lines.len().max(expected_lines.len());
    for idx in 0..max {
        let actual_line = actual_lines.get(idx).copied().unwrap_or("");
        let expected_line = expected_lines.get(idx).copied().unwrap_or("");
        if actual_line != expected_line {
            panic!(
                "line {idx} mismatch\nactual:   {actual_line}\nexpected: {expected_line}"
            );
        }
    }
}

#[test]
fn btop_renders_full_screen() {
    let snapshot = std::fs::read_to_string(SNAPSHOT_PATH)
        .expect("read snapshot")
        .trim_end_matches('\n')
        .to_string();
    let (width, height) = fixture_dims(&snapshot);

    let expected = normalize_expected(&snapshot, width, height);
    let composed = app::render_screen_text();
    assert_eq!(composed, expected, "composed screen mismatch");

    let surface = render_app(app::app, width, height);
    let actual = surface.lines().join("\n");

    assert_lines_eq(&actual, &expected);
}

#[test]
fn btop_fills_taller_screen() {
    let width = 120;
    let height = 40;
    let surface = render_app(app::app, width, height);
    for (idx, line) in surface.lines().iter().enumerate() {
        let is_blank = line.chars().all(|ch| ch == ' ');
        assert!(!is_blank, "blank row at {idx}");
    }
}

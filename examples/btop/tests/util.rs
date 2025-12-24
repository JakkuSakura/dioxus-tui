use std::any::Any;

use dioxus::prelude::*;
use dioxus_tui::{ColorMode, Config, RawVirtualDom, Rect, Surface};

pub fn fixture_dims(contents: &str) -> (u16, u16) {
    let lines: Vec<&str> = contents.trim_end_matches('\n').lines().collect();
    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as u16;
    let height = lines.len() as u16;
    (width, height)
}

pub fn normalize_expected(contents: &str, width: u16, height: u16) -> String {
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

pub fn render_text(text: String, width: u16, height: u16) -> Surface {
    let cfg = Config::default().with_color_mode(ColorMode::Rgb);
    let area = Rect::new(0, 0, width, height);
    let style = format!(
        "white-space: pre; font-family: monospace; line-height: 1em; width: {width}ch; height: {height}ch; position: absolute; left: 0; top: 0;",
    );

    let contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>> = Vec::new();
    let raw = RawVirtualDom::with_contexts(
        move |_| {
            rsx! {
                pre {
                    style: "{style}",
                    "data-pre": "true",
                    "{text}"
                }
            }
        },
        (),
        contexts,
    );

    dioxus_tui::render_surface_raw(raw, cfg, area).expect("render")
}

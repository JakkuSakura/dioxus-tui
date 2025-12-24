use std::any::Any;

use dioxus::prelude::*;
use dioxus_tui::{ColorMode, Config, RawVirtualDom, Rect, Surface};
use dioxus_tui_btop::btop;

const SNAPSHOT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/btop_120x50.txt"
);

fn render_app(app: fn() -> Element, width: u16, height: u16) -> Surface {
    let cfg = Config::default().with_color_mode(ColorMode::Rgb);
    let area = Rect::new(0, 0, width, height);

    let contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>> = Vec::new();
    let raw = RawVirtualDom::with_contexts(move |_| app(), (), contexts);

    dioxus_tui::render_surface_raw(raw, cfg, area).expect("render")
}

#[test]
fn btop_renders_core_sections() {
    let surface = render_app(btop::app, 120, 50);
    let lines = surface.lines();
    let actual = lines.join("\n");

    if std::env::var("UPDATE_SNAPSHOT").is_ok() {
        if let Some(parent) = std::path::Path::new(SNAPSHOT_PATH).parent() {
            std::fs::create_dir_all(parent).expect("create snapshot directory");
        }
        std::fs::write(SNAPSHOT_PATH, &actual).expect("write snapshot");
        return;
    }

    let expected = std::fs::read_to_string(SNAPSHOT_PATH)
        .expect("read snapshot")
        .trim_end_matches('\n')
        .to_string();

    assert_eq!(actual, expected, "btop snapshot mismatch");
}

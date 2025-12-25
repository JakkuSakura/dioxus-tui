mod util;

use dioxus_tui_btop::components::disk_panel;
use dioxus_tui_btop::data::MOCK_DATA;

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/disk_panel.txt"
);

#[test]
fn disk_panel_renders_snapshot() {
    let snapshot = std::fs::read_to_string(FIXTURE_PATH)
        .expect("read fixture")
        .trim_end_matches('\n')
        .to_string();

    let (width, height) = util::fixture_dims(&snapshot);
    let rect = disk_panel::render_with_size(&MOCK_DATA.disk, width as usize, height as usize);
    let actual = rect.to_lines().join("\n");

    assert_eq!(actual, snapshot, "disk panel snapshot mismatch");
}

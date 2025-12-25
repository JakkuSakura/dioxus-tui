mod util;

use dioxus_tui_btop::components::proc_panel_bottom;
use dioxus_tui_btop::data::MOCK_DATA;

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/proc_panel_bottom.txt"
);

#[test]
fn proc_panel_bottom_renders_snapshot() {
    let snapshot = std::fs::read_to_string(FIXTURE_PATH)
        .expect("read fixture")
        .trim_end_matches('\n')
        .to_string();

    let (width, height) = util::fixture_dims(&snapshot);
    let rect = proc_panel_bottom::render_with_size(&MOCK_DATA.proc, width as usize, height as usize);
    let actual = rect.to_lines().join("\n");

    assert_eq!(actual, snapshot, "proc panel bottom snapshot mismatch");
}

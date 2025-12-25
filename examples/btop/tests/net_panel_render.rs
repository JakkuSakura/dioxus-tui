mod util;

use dioxus_tui_btop::components::net_panel;
use dioxus_tui_btop::data::MOCK_DATA;

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/net_panel.txt"
);

#[test]
fn net_panel_renders_snapshot() {
    let snapshot = std::fs::read_to_string(FIXTURE_PATH)
        .expect("read fixture")
        .trim_end_matches('\n')
        .to_string();

    let (width, height) = util::fixture_dims(&snapshot);
    let rect = net_panel::render_with_size(&MOCK_DATA.net, width as usize, height as usize);
    let actual = rect.to_lines().join("\n");

    assert_eq!(actual, snapshot, "net panel snapshot mismatch");
}

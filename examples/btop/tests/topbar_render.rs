mod util;

use dioxus_tui_btop::components::topbar;
use dioxus_tui_btop::data::MOCK_DATA;

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/topbar.txt"
);

#[test]
fn topbar_renders_snapshot() {
    let snapshot = std::fs::read_to_string(FIXTURE_PATH)
        .expect("read fixture")
        .trim_end_matches('\n')
        .to_string();

    let (width, _height) = util::fixture_dims(&snapshot);
    let rect = topbar::render_with_width(&MOCK_DATA.topbar, width as usize);
    let actual = rect.to_lines().join("\n");

    assert_eq!(actual, snapshot, "topbar snapshot mismatch");
}

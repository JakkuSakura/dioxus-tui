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

    let block = topbar::render(&MOCK_DATA.topbar);
    let actual = block.lines.join("\n");

    assert_eq!(actual, snapshot, "topbar snapshot mismatch");
}

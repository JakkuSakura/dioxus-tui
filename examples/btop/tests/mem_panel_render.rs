use dioxus_tui_btop::components::mem_panel;
use dioxus_tui_btop::data::MOCK_DATA;

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/mem_panel.txt"
);

#[test]
fn mem_panel_renders_snapshot() {
    let snapshot = std::fs::read_to_string(FIXTURE_PATH)
        .expect("read fixture")
        .trim_end_matches('\n')
        .to_string();

    let block = mem_panel::render(&MOCK_DATA.mem);
    let actual = block.lines().join("\n");

    assert_eq!(actual, snapshot, "mem panel snapshot mismatch");
}

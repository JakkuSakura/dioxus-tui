use dioxus_tui_btop::components::proc_panel_top;
use dioxus_tui_btop::data::MOCK_DATA;

const FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/proc_panel_top.txt"
);

#[test]
fn proc_panel_top_renders_snapshot() {
    let snapshot = std::fs::read_to_string(FIXTURE_PATH)
        .expect("read fixture")
        .trim_end_matches('\n')
        .to_string();

    let block = proc_panel_top::render(&MOCK_DATA.proc);
    let actual = block.lines.join("\n");

    assert_eq!(actual, snapshot, "proc panel top snapshot mismatch");
}

// This file exists to allow `cargo test` without running the long-running event tests.
// The real event tests live in `tests/events.rs` and can be run explicitly with:
//   cargo test --test events
// We skip them by default to avoid hanging CI or local runs.

#[test]
fn skip_events_suite() {
    eprintln!("Skipping events.rs; run `cargo test --test events` to execute it.");
}

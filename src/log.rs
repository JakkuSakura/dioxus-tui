use std::fs::OpenOptions;

use tracing_subscriber::fmt::writer::MakeWriterExt;

/// Initialize a tracing subscriber that writes to `debug.log`.
/// Safe to call multiple times; initialization runs once.
pub fn log_to_file() {
    let make_writer = || {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .expect("open debug.log for tracing")
    };

    let subscriber = tracing_subscriber::fmt()
        .with_writer(make_writer.with_max_level(tracing::Level::DEBUG))
        .with_ansi(false)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

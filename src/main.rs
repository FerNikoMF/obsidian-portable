// Hide the console window in release builds, but keep it for `cargo test` so
// the test harness can actually print its results on Windows.
#![cfg_attr(not(test), windows_subsystem = "windows")]

mod app;
mod github;
mod i18n;
mod launcher;
mod shortcuts;
mod updater;
mod utils;

fn main() {
    // Initialize file logging
    let log_dir = crate::utils::exe_dir().join("Data");
    let _ = std::fs::create_dir_all(&log_dir);
    let file_appender = tracing_appender::rolling::never(&log_dir, "debug.log");
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_ansi(false)
        .init();

    tracing::info!("Obsidian Portable starting");

    shortcuts::create_if_needed();

    if std::env::args().skip(1).any(|a| a == "--updater") {
        let _ = app::run();
    } else {
        launcher::launch();
    }
}

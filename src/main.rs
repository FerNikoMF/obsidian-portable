#![windows_subsystem = "windows"]

mod app;
mod github;
mod i18n;
mod launcher;
mod shortcuts;
mod updater;
mod utils;

fn main() {
    shortcuts::create_if_needed();

    if std::env::args().skip(1).any(|a| a == "--updater") {
        let _ = app::run();
    } else {
        launcher::launch();
    }
}

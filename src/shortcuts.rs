use std::env;
use std::fs;

use mslnk::ShellLink;

use crate::utils::exe_dir;

const GITHUB_URL: &str = "https://github.com/FerNikoMF/Obsidian-Portable";

/// Runs only once — the presence of `Obsidian Updater.lnk` is the marker.
/// No extra `.initialized` file is created.
pub fn create_if_needed() {
    // If the shortcut already exists, nothing to do
    if exe_dir().join("Obsidian Updater.lnk").exists() {
        return;
    }
    create_updater_shortcut();
    create_github_url();
}

fn create_updater_shortcut() {
    let Ok(exe_path) = env::current_exe() else { return };

    let lnk      = exe_dir().join("Obsidian Updater.lnk");
    let icon_src = exe_dir().join("assets").join("obsidian-icon.ico");

    let mut sl = ShellLink::new(exe_path.to_string_lossy().as_ref())
        .expect("Failed to create ShellLink");

    sl.set_arguments(Some("--updater".to_string()));

    if icon_src.exists() {
        sl.set_icon_location(Some(icon_src.to_string_lossy().into_owned()));
    }

    let _ = sl.create_lnk(lnk);
}

fn create_github_url() {
    let content = format!("[InternetShortcut]\r\nURL={GITHUB_URL}\r\n");
    let _ = fs::write(exe_dir().join("GitHub.url"), content);
}

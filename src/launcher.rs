use std::fs;
use std::process::Command;
use std::os::windows::process::CommandExt;

use crate::utils::{obsidian_data_dir, obsidian_exe, user_data_arg, DETACHED_PROCESS};

/// Launches `App/Obsidian.exe` in fully portable mode.
/// If Obsidian is not yet installed, opens the Updater GUI instead.
pub fn launch() {
    let exe = obsidian_exe();

    if !exe.exists() {
        let _ = crate::app::run();
        return;
    }

    let _ = fs::create_dir_all(obsidian_data_dir());

    // Spawn detached so the launcher can exit immediately
    let _ = Command::new(exe)
        .arg(user_data_arg())
        .creation_flags(DETACHED_PROCESS)
        .spawn();
}

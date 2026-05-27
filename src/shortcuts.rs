use std::env;
use std::fs;
use std::process::Command;
use std::os::windows::process::CommandExt;

use crate::utils::{exe_dir, CREATE_NO_WINDOW};

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

    let exe      = exe_path.to_string_lossy();
    let lnk      = exe_dir().join("Obsidian Updater.lnk");
    let icon_src = exe_dir().join("assets").join("obsidian-icon.ico");

    let mut vbs = format!(
        "Set sh = CreateObject(\"WScript.Shell\")\r\n\
         Set lnk = sh.CreateShortcut(\"{lnk}\")\r\n\
         lnk.TargetPath = \"{exe}\"\r\n\
         lnk.Arguments  = \"--updater\"\r\n",
        lnk = lnk.to_string_lossy().replace('"', "\"\""),
        exe = exe.replace('"', "\"\""),
    );
    if icon_src.exists() {
        vbs.push_str(&format!(
            "lnk.IconLocation = \"{}\"\r\n",
            icon_src.to_string_lossy().replace('"', "\"\"")
        ));
    }
    vbs.push_str("lnk.Save\r\n");

    let vbs_path = exe_dir().join("_tmp.vbs");
    if fs::write(&vbs_path, vbs).is_ok() {
        let _ = Command::new("cscript")
            .args(["//nologo", &vbs_path.to_string_lossy()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let _ = fs::remove_file(&vbs_path);
    }
}

fn create_github_url() {
    let content = format!("[InternetShortcut]\r\nURL={GITHUB_URL}\r\n");
    let _ = fs::write(exe_dir().join("GitHub.url"), content);
}

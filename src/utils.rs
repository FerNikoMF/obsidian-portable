use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::os::windows::process::CommandExt;

// ── Directories ───────────────────────────────────────────────────────────────

pub const APP_DIR:     &str = "App";

// ── WinAPI flags ──────────────────────────────────────────────────────────────

pub const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const DETACHED_PROCESS: u32 = 0x00000008;

// ── Paths ─────────────────────────────────────────────────────────────────────

/// Directory containing the launcher executable.
pub fn exe_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| env::current_dir().unwrap())
}

/// `<exe_dir>/App/Obsidian.exe`
pub fn obsidian_exe() -> PathBuf {
    exe_dir().join(APP_DIR).join("Obsidian.exe")
}

/// `<exe_dir>/Data/ObsidianAppData` — matches the official PortableApps.com convention.
/// Passed as `--user-data-dir=<this>` so Obsidian never touches %AppData%.
pub fn obsidian_data_dir() -> PathBuf {
    exe_dir().join("Data").join("ObsidianAppData")
}

/// Builds the `--user-data-dir=<path>` argument for Obsidian.
/// Must be a single combined arg — Electron ignores two separate args.
pub fn user_data_arg() -> String {
    format!("--user-data-dir={}", obsidian_data_dir().to_string_lossy())
}

// ── Version ───────────────────────────────────────────────────────────────────

/// Reads `ProductVersion` from `App/Obsidian.exe` via PowerShell.
pub fn installed_version() -> String {
    let exe = obsidian_exe();
    if !exe.exists() {
        return "Не установлена".to_owned();
    }

    let script = format!(
        "(Get-Item '{}').VersionInfo.ProductVersion",
        exe.to_string_lossy()
    );
    Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_owned();
            if v.is_empty() { None } else { Some(v) }
        })
        .unwrap_or_else(|| "Неизвестная версия".to_owned())
}

// ── File search ───────────────────────────────────────────────────────────────

/// Recursively finds the first file with `filename` inside `dir`.
pub fn find_file(dir: &Path, filename: &str) -> Option<PathBuf> {
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file(&path, filename) {
                return Some(found);
            }
        } else if path.file_name().is_some_and(|n| n.to_string_lossy() == filename) {
            return Some(path);
        }
    }
    None
}

// ── Version normalization ─────────────────────────────────────────────────────

/// Strips "v" prefix and trailing ".0" segments so comparison works correctly.
/// "v1.12.7"  → "1.12.7"
/// "1.12.7.0" → "1.12.7"
pub fn normalize_version(v: &str) -> String {
    let v = v.trim_start_matches('v');
    let parts: Vec<&str> = v.split('.').collect();
    let mut end = parts.len();
    while end > 1 && parts[end - 1] == "0" {
        end -= 1;
    }
    parts[..end].join(".")
}

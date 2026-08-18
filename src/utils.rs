use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// ── Directories ───────────────────────────────────────────────────────────────

pub const APP_DIR:     &str = "App";

// ── WinAPI flags ──────────────────────────────────────────────────────────────

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

/// Reads `ProductVersion` from `App/Obsidian.exe` using the PE version resource.
/// Returns `None` if Obsidian is not installed or the version cannot be read.
pub fn installed_version() -> Option<String> {
    let exe = obsidian_exe();
    if !exe.exists() {
        return None;
    }

    let data = fs::read(&exe).ok()?;
    let pe = pelite::PeFile::from_bytes(&data).ok()?;

    use pelite::resources::FindError;
    let resources = pe.resources().ok()?;
    let version_info = match resources.version_info() {
        Ok(vi) => vi,
        Err(FindError::NotFound) => return None,
        Err(_) => return None,
    };

    // Try to get ProductVersion from the StringFileInfo
    let fixed = version_info.fixed()?;
    let ver = format!(
        "{}.{}.{}",
        fixed.dwProductVersion.Major,
        fixed.dwProductVersion.Minor,
        fixed.dwProductVersion.Patch,
    );
    Some(ver)
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

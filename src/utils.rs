use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// ── Directories ───────────────────────────────────────────────────────────────

pub const APP_DIR:     &str = "App";

// ── Project links ─────────────────────────────────────────────────────────────

pub const GITHUB_URL: &str = "https://github.com/FerNikoMF/Obsidian-Portable";

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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_v_prefix() {
        assert_eq!(normalize_version("v1.12.7"), "1.12.7");
    }

    #[test]
    fn normalize_strips_trailing_zero_segments() {
        assert_eq!(normalize_version("1.12.7.0"), "1.12.7");
        assert_eq!(normalize_version("1.12.0.0"), "1.12");
        assert_eq!(normalize_version("1.0.0.0"), "1");
    }

    #[test]
    fn normalize_keeps_meaningful_zeros() {
        assert_eq!(normalize_version("1.0.7"), "1.0.7");
        assert_eq!(normalize_version("1.10.0"), "1.10");
    }

    #[test]
    fn normalize_handles_v_and_trailing_zeros_together() {
        assert_eq!(normalize_version("v1.12.7.0"), "1.12.7");
    }

    #[test]
    fn normalize_matching_versions_are_equal() {
        assert_eq!(normalize_version("v1.12.7"), normalize_version("1.12.7.0"));
    }

    #[test]
    fn normalize_single_component() {
        assert_eq!(normalize_version("v2"), "2");
        assert_eq!(normalize_version("0"), "0");
    }

    #[test]
    fn find_file_locates_nested_file() {
        let dir = std::env::temp_dir().join(format!(
            "obsidian_portable_test_{}",
            std::process::id()
        ));
        let nested = dir.join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        let target = nested.join("app-64.7z");
        fs::write(&target, b"data").unwrap();

        let found = find_file(&dir, "app-64.7z");
        assert_eq!(found, Some(target));

        let missing = find_file(&dir, "does-not-exist.7z");
        assert_eq!(missing, None);

        let _ = fs::remove_dir_all(&dir);
    }
}

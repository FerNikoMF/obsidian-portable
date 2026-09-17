use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::utils::exe_dir;

// ── Language selection ────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Ru,
}

/// Name of the file (next to the exe) that stores the user's saved language
/// preference. Absence means "not chosen yet — fall back to OS locale".
const LANG_PREF_FILE: &str = "lang.pref";

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        match code.trim().to_ascii_lowercase().as_str() {
            "en" => Some(Lang::En),
            "ru" => Some(Lang::Ru),
            _ => None,
        }
    }

    /// Toggles between the two supported languages.
    pub fn toggled(self) -> Self {
        match self {
            Lang::En => Lang::Ru,
            Lang::Ru => Lang::En,
        }
    }

    fn pref_path() -> PathBuf {
        exe_dir().join(LANG_PREF_FILE)
    }

    /// Persists the chosen language so it's remembered across runs.
    pub fn save(self) {
        let _ = std::fs::write(Self::pref_path(), self.code());
    }

    /// Picks the active language: a previously saved preference takes
    /// priority; otherwise falls back to the Windows UI locale, defaulting
    /// to English if it can't be determined.
    pub fn load() -> Self {
        if let Ok(saved) = std::fs::read_to_string(Self::pref_path()) {
            if let Some(lang) = Self::from_code(&saved) {
                return lang;
            }
        }
        Self::from_system_locale().unwrap_or_default()
    }

    /// Reads the current user's Windows UI language via `GetUserDefaultLocaleName`.
    #[cfg(windows)]
    fn from_system_locale() -> Option<Self> {
        use std::os::windows::ffi::OsStringExt;

        #[link(name = "kernel32")]
        extern "system" {
            fn GetUserDefaultLocaleName(lp_locale_name: *mut u16, cch_locale_name: i32) -> i32;
        }

        const LOCALE_NAME_MAX_LENGTH: usize = 85;
        let mut buf = [0u16; LOCALE_NAME_MAX_LENGTH];
        let len = unsafe {
            GetUserDefaultLocaleName(buf.as_mut_ptr(), LOCALE_NAME_MAX_LENGTH as i32)
        };
        if len <= 0 {
            return None;
        }
        let name = std::ffi::OsString::from_wide(&buf[..(len as usize - 1)])
            .to_string_lossy()
            .into_owned();
        if name.to_ascii_lowercase().starts_with("ru") {
            Some(Lang::Ru)
        } else {
            Some(Lang::En)
        }
    }

    #[cfg(not(windows))]
    fn from_system_locale() -> Option<Self> {
        None
    }
}

// ── All UI strings (deserialised from JSON) ───────────────────────────────────
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct T {
    pub app_name:        String,
    pub app_subtitle:    String,
    // Version card labels
    pub installed:       String,
    pub available:       String,
    // States
    pub checking:        String,
    pub up_to_date:      String,
    pub update_ready:    String,
    pub not_installed:   String,
    // Buttons
    pub btn_install:     String,
    pub btn_update:      String, // {ver} placeholder
    pub btn_reinstall:   String,
    pub btn_launch:      String,
    pub btn_retry:       String,
    pub btn_cancel:      String,
    pub step_prepare:    String,
    pub step_download:   String, // {name} placeholder
    pub step_unpack1:    String,
    pub step_find_core:  String,
    pub step_unpack2:    String,
    pub step_cleanup:    String,
    pub step_done:       String,
    // Errors
    pub err_fetch:       String,
    pub err_no_release:  String,
    pub err_too_small:   String,
    pub err_no_asset:    String,
    pub err_no_core:     String,
    pub err_hash_mismatch: String,
    pub err_label:       String,
    pub done_label:      String,
    // Footer
    pub footer_link:     String,
    // Speed unit
    pub mib_s:           String,
}

// ── Embedded defaults (compiled in; used when the external file is missing) ───
const EN_JSON: &str = include_str!("../lang/en.json");
const RU_JSON: &str = include_str!("../lang/ru.json");

fn embedded(lang: Lang) -> &'static T {
    static EN: OnceLock<T> = OnceLock::new();
    static RU: OnceLock<T> = OnceLock::new();
    let cell = match lang {
        Lang::En => &EN,
        Lang::Ru => &RU,
    };
    cell.get_or_init(|| {
        let json = match lang {
            Lang::En => EN_JSON,
            Lang::Ru => RU_JSON,
        };
        serde_json::from_str(json).expect("invalid embedded lang json")
    })
}

/// Path to `<exe_dir>/lang` — external translations live next to the binary.
fn lang_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join("lang"))
}

/// Loads the language file from disk; falls back to the embedded default.
fn load_external(lang: Lang) -> Option<T> {
    let path = lang_dir()?.join(format!("{}.json", lang.code()));
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn get(lang: Lang) -> T {
    load_external(lang).unwrap_or_else(|| embedded(lang).clone())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_code_parses_known_codes() {
        assert_eq!(Lang::from_code("en"), Some(Lang::En));
        assert_eq!(Lang::from_code("RU"), Some(Lang::Ru));
        assert_eq!(Lang::from_code(" ru \n"), Some(Lang::Ru));
        assert_eq!(Lang::from_code("de"), None);
        assert_eq!(Lang::from_code(""), None);
    }

    #[test]
    fn toggled_switches_between_languages() {
        assert_eq!(Lang::En.toggled(), Lang::Ru);
        assert_eq!(Lang::Ru.toggled(), Lang::En);
    }

    #[test]
    fn embedded_translations_parse() {
        let en = embedded(Lang::En);
        assert!(!en.btn_install.is_empty());
        assert!(!en.checking.is_empty());

        let ru = embedded(Lang::Ru);
        assert!(!ru.btn_install.is_empty());
        assert!(!ru.checking.is_empty());
    }

    #[test]
    fn embedded_translations_have_matching_fields() {
        // Both language files must define the same non-empty string set,
        // otherwise switching languages would silently show blank labels.
        let en: serde_json::Value = serde_json::from_str(EN_JSON).unwrap();
        let ru: serde_json::Value = serde_json::from_str(RU_JSON).unwrap();
        let en_map = en.as_object().unwrap();
        let ru_map = ru.as_object().unwrap();

        for key in en_map.keys() {
            assert!(ru_map.contains_key(key), "ru.json is missing key: {key}");
        }
        for (key, value) in en_map.iter().chain(ru_map.iter()) {
            let s = value.as_str().unwrap_or("");
            assert!(!s.is_empty(), "translation is empty: {key}");
        }
    }

    /// Validates the raw `GetUserDefaultLocaleName` FFI declaration.
    #[cfg(windows)]
    #[test]
    fn system_locale_is_detected() {
        assert!(Lang::from_system_locale().is_some());
    }

    /// `save` + `load` must round-trip, and `load` must prefer the saved file.
    #[test]
    fn saved_preference_round_trips() {
        let original = std::fs::read_to_string(Lang::pref_path()).ok();

        Lang::Ru.save();
        assert_eq!(Lang::load(), Lang::Ru);
        Lang::En.save();
        assert_eq!(Lang::load(), Lang::En);

        // Restore whatever was there before the test ran.
        match original {
            Some(text) => { let _ = std::fs::write(Lang::pref_path(), text); }
            None       => { let _ = std::fs::remove_file(Lang::pref_path()); }
        }
    }
}

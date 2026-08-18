use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

// ── Language selection ────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum Lang {
    #[default]
    En,
    Ru,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ru => "ru",
        }
    }

    /// Picks the active language. Currently fixed to English; extend later with
    /// the OS locale or a saved user preference.
    pub fn load() -> Self {
        Lang::En
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
    // Progress steps
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
    pub err_label:       String,
    // Result
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

use std::fs;
use crate::utils::exe_dir;

const LANG_FILE: &str = "lang.txt";

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Lang { #[default] Ru, En }

impl Lang {
    pub fn load() -> Self {
        exe_dir().join(LANG_FILE)
            .pipe(|p| fs::read_to_string(p).unwrap_or_default())
            .trim()
            .pipe(|s| if s == "en" { Lang::En } else { Lang::Ru })
    }

    pub fn save(self) {
        let _ = fs::write(exe_dir().join(LANG_FILE), self.code());
    }

    pub fn toggle(self) -> Self {
        match self { Lang::Ru => Lang::En, Lang::En => Lang::Ru }
    }

    pub fn code(self) -> &'static str {
        match self { Lang::Ru => "ru", Lang::En => "en" }
    }

    pub fn label(self) -> &'static str {
        match self { Lang::Ru => "RU", Lang::En => "EN" }
    }
}

// ── Extension trait so we can .pipe() without intermediate bindings ───────────
trait Pipe: Sized {
    fn pipe<F: FnOnce(Self) -> R, R>(self, f: F) -> R { f(self) }
}
impl<T> Pipe for T {}

// ── All UI strings ────────────────────────────────────────────────────────────
pub struct T {
    // Header
    pub app_name:        &'static str,
    pub app_subtitle:    &'static str,
    // Version card labels
    pub installed:       &'static str,
    pub available:       &'static str,
    // States
    pub checking:        &'static str,
    pub up_to_date:      &'static str,
    pub update_ready:    &'static str,
    pub not_installed:   &'static str,
    // Buttons
    pub btn_install:     &'static str,
    pub btn_update:      &'static str, // {ver} placeholder
    pub btn_reinstall:   &'static str,
    pub btn_launch:      &'static str,
    pub btn_retry:       &'static str,
    // Progress steps
    pub step_prepare:    &'static str,
    pub step_archiver:   &'static str,
    pub step_download:   &'static str, // {name} placeholder
    pub step_unpack1:    &'static str,
    pub step_find_core:  &'static str,
    pub step_unpack2:    &'static str,
    pub step_cleanup:    &'static str,
    pub step_done:       &'static str,
    // Errors
    pub err_fetch:       &'static str,
    pub err_no_release:  &'static str,
    pub err_too_small:   &'static str,
    pub err_no_asset:    &'static str,
    pub err_no_core:     &'static str,
    pub err_label:       &'static str,
    // Result
    pub done_label:      &'static str,
    // Footer
    pub footer_link:     &'static str,
    // Speed unit
    pub mib_s:           &'static str,
}

pub const RU: T = T {
    app_name:        "Obsidian Portable",
    app_subtitle:    "Автообновление  ·  Портабельный режим",
    installed:       "Установлена",
    available:       "Доступна",
    checking:        "Получение информации о версии…",
    up_to_date:      "Актуальная версия",
    update_ready:    "Доступно обновление",
    not_installed:   "Не установлена",
    btn_install:     "⬇   Установить Obsidian Portable",
    btn_update:      "↑   Обновить до",
    btn_reinstall:   "↺   Переустановить текущую версию",
    btn_launch:      "▶   Запустить Obsidian",
    btn_retry:       "↺   Попробовать снова",
    step_prepare:    "Подготовка…",
    step_archiver:   "Подготовка архиватора…",
    step_download:   "Скачивание",
    step_unpack1:    "Распаковка инсталлятора (1/2)…",
    step_find_core:  "Поиск ядра приложения…",
    step_unpack2:    "Извлечение в App/ (2/2)…",
    step_cleanup:    "Очистка…",
    step_done:       "Готово!",
    err_fetch:       "Не удалось проверить обновления",
    err_no_release:  "Данные релиза недоступны",
    err_too_small:   "Файл слишком мал — загрузка прервана",
    err_no_asset:    "x64-инсталлятор не найден в релизе GitHub",
    err_no_core:     "app-64.7z не найден — структура установщика изменилась?",
    err_label:       "Ошибка",
    done_label:      "Установлено",
    footer_link:     "FerNikoMF / Obsidian-Portable",
    mib_s:           "МиБ/с",
};

pub const EN: T = T {
    app_name:        "Obsidian Portable",
    app_subtitle:    "Auto-update  ·  Portable mode",
    installed:       "Installed",
    available:       "Available",
    checking:        "Fetching version info…",
    up_to_date:      "Up to date",
    update_ready:    "Update available",
    not_installed:   "Not installed",
    btn_install:     "⬇   Install Obsidian Portable",
    btn_update:      "↑   Update to",
    btn_reinstall:   "↺   Reinstall current version",
    btn_launch:      "▶   Launch Obsidian",
    btn_retry:       "↺   Try again",
    step_prepare:    "Preparing…",
    step_archiver:   "Extracting archiver…",
    step_download:   "Downloading",
    step_unpack1:    "Unpacking installer (1/2)…",
    step_find_core:  "Locating app core…",
    step_unpack2:    "Extracting to App/ (2/2)…",
    step_cleanup:    "Cleaning up…",
    step_done:       "Done!",
    err_fetch:       "Failed to check for updates",
    err_no_release:  "Release data unavailable",
    err_too_small:   "File too small — download interrupted",
    err_no_asset:    "x64 installer not found in GitHub release",
    err_no_core:     "app-64.7z not found — installer structure changed?",
    err_label:       "Error",
    done_label:      "Installed",
    footer_link:     "FerNikoMF / Obsidian-Portable",
    mib_s:           "MiB/s",
};

pub fn get(lang: Lang) -> &'static T {
    match lang { Lang::Ru => &RU, Lang::En => &EN }
}

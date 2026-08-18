<div align="center">

<img src="assets/obsidian-icon.ico" width="72" alt="Obsidian Portable icon" />

# Obsidian Portable

**Портабельный лаунчер и авто-апдейтер для [Obsidian](https://obsidian.md)**  
Никаких следов в системе — всё хранится рядом с `.exe`

[![Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)    [![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blueviolet)](LICENSE)   [![Vibe Coded](https://img.shields.io/badge/vibe-coded-purple)](https://github.com/FerNikoMF/Obsidian-Portable)    [![Platform](https://img.shields.io/badge/platform-Windows-blue?logo=windows)](https://github.com/FerNikoMF/Obsidian-Portable/releases)

[🇬🇧 English version](README.md)

</div>

---

## Что это

Obsidian при обычной установке пишет данные в `%AppData%\obsidian` — это не портабельно.  
Этот лаунчер решает проблему: запускает Obsidian с флагом `--user-data-dir`, перенаправляя все данные в локальную папку `Data\ObsidianAppData`, никакого AppData.

Также умеет сам скачать и установить актуальную версию Obsidian — прямо с GitHub Releases, без браузера.

## Возможности

- ✦ **Полная портабельность** — всё в одной папке, можно носить на флешке
- ✦ **Авто-обновление** — сравнивает установленную и доступную версию, скачивает и распаковывает
- ✦ **Распаковка на чистом Rust** — NSIS-инсталлятор парсится и распаковывается внутри процесса, без внешнего 7-Zip
- ✦ **Ярлык при первом запуске** — `Obsidian Updater.lnk` для обновление
- ✦ **Чистый апдейтер** — окно с прогресс-баром, скоростью загрузки и состоянием установки

## Структура папки

```
ObsidianPortable/
├── obsidian-portable.exe ← запускает Obsidian (или updater если не установлен)
├── Obsidian Updater.lnk  ← ярлык для апдейтера (создаётся автоматически)
├── GitHub.url            ← ссылка на этот репозиторий
├── App/
│   └── Obsidian.exe      ← ядро Obsidian (распаковывается сюда)
└── Data/
    └── ObsidianAppData/  ← данные приложения
```

## Использование

| Действие | Способ |
|---|---|
| Запустить Obsidian | `obsidian-portable.exe` |
| Открыть апдейтер | `obsidian-portable.exe --updater` или `Obsidian Updater.lnk` |
| Первая установка | `obsidian-portable.exe` сам откроет апдейтер |

## Сборка из исходников

```bash
# Требуется Rust: https://rustup.rs

git clone https://github.com/FerNikoMF/Obsidian-Portable
cd Obsidian-Portable

cargo build --release
# Бинарник: target/release/launcher.exe
```

Внешние бинарники не нужны — всё необходимое собирается из crates.

### Зависимости

| Крейт | Назначение |
|---|---|
| `eframe` / `egui` | GUI |
| `reqwest` | HTTP-загрузка |
| `serde` | парсинг JSON (GitHub API) |
| `sevenz-rust` | распаковка `app-64.7z` в `App/` |
| `nsis` | парсинг и распаковка NSIS-инсталлятора |
| `pelite` | чтение версии из PE-ресурса `Obsidian.exe` |
| `mslnk` | создание ярлыка `Obsidian Updater.lnk` |

## Вайб-кодинг

Этот проект сделан в стиле **vibe coding** — быстро, итерационно, с помощью AI.  
Если хочешь помочь или предложить идею — открывай [issue](https://github.com/FerNikoMF/Obsidian-Portable/issues) или PR.

## Лицензия

Проект распространяется под лицензией **GPL-3.0**.
Подробнее: [LICENSE](LICENSE) или [gnu.org/licenses/gpl-3.0](https://www.gnu.org/licenses/gpl-3.0.html)

---

<div align="center">
  <sub>Сделано с 💜 · Не связан с командой Obsidian</sub>
</div>

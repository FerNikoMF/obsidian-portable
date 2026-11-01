<img align="right" src="assets/png.jpg" width="320" alt="Obsidian Portable icon" />

<h1 align="left">Obsidian Portable</h1>

<p align="left"><b>A portable launcher and auto-updater for <a href="https://obsidian.md">Obsidian</a></b><br>
No system traces — everything stays in the app folder</p>

<p align="left">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/built_with-Rust-orange?logo=rust&logoColor=white" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blueviolet" alt="License: GPL-3.0"></a>
  <a href="https://github.com/FerNikoMF/Obsidian-Portable"><img src="https://img.shields.io/badge/vibe-coded-purple" alt="Vibe Coded"></a>
  <a href="https://github.com/FerNikoMF/Obsidian-Portable/releases"><img src="https://img.shields.io/badge/platform-Windows-blue?logo=windows" alt="Platform"></a>
</p>

<p align="left">
  <a href="https://github.com/FerNikoMF/Obsidian-Portable/releases"><b>⬇ Download</b></a> ·
  <a href="#usage">Usage</a> ·
  <a href="#building-from-source">Build</a> ·
  <a href="README.ru.md">🇷🇺 Русская версия</a>
</p>

<br clear="right">

## What is this?

Normally, Obsidian writes all its data to `%AppData%\obsidian`, which makes it impossible to carry around.
This launcher fixes that: it starts Obsidian with the `--user-data-dir` flag, redirecting everything into a local `Data\ObsidianAppData` folder. No AppData writes.

It also downloads and installs the latest Obsidian release automatically — straight from GitHub Releases, no browser needed.

## Features

- ✦ **Fully portable** — everything in one folder, works from a USB drive
- ✦ **Auto-update** — compares installed vs available version, downloads and unpacks automatically
- ✦ **Pure-Rust extraction** — the NSIS installer is parsed and unpacked in-process, no external 7-Zip binary
- ✦ **First-run shortcut** — `Obsidian Updater.lnk` is created automatically on first launch
- ✦ **Clean updater UI** — progress bar, download speed and install status
- ✦ **RU / EN interface** — language toggle built into the app

## Folder structure

```text
ObsidianPortable/
├── obsidian-portable.exe    # launches Obsidian (or the updater if not installed)
├── Obsidian Updater.lnk     # updater shortcut (created automatically)
├── GitHub.url               # link to this repository
├── lang.pref                # saved UI language (created on first toggle)
├── App/
│   └── Obsidian.exe         # Obsidian core (unpacked here)
└── Data/
    └── ObsidianAppData/     # all vaults, plugins, settings
```

## Usage

| Action | How |
|---|---|
| Launch Obsidian | `obsidian-portable.exe` |
| Open updater | `obsidian-portable.exe --updater` or `Obsidian Updater.lnk` |
| First install | `obsidian-portable.exe` opens the updater automatically |
| Switch language | Click the **EN / RU** button in the updater footer |

## Building from source

Requires Rust: <https://rustup.rs>

```bash
git clone https://github.com/FerNikoMF/Obsidian-Portable
cd Obsidian-Portable

cargo build --release
# Output: target/release/obsidian-portable.exe

cargo test
# Runs the unit tests (version normalisation, i18n, file search)
```

No external binaries are required — everything is pulled in from crates at build time.

### Dependencies

| Crate | Purpose |
|---|---|
| `eframe` / `egui` | GUI |
| `reqwest` | HTTP download |
| `serde` | JSON parsing (GitHub API) |
| `sevenz-rust2` | Unpack `app-64.7z` into `App/` |
| `nsis` | Parse and unpack the NSIS installer |
| `pelite` | Read Obsidian.exe version from its PE resource |
| `mslnk` | Create the `Obsidian Updater.lnk` shortcut |

## Vibe coding

This project was built in a **vibe coding** style — fast, iterative, AI-assisted.

Feel free to open an [issue](https://github.com/FerNikoMF/Obsidian-Portable/issues) or a PR if you want to contribute.

## License

Licensed under **GPL-3.0**.
See [LICENSE](LICENSE) or [gnu.org/licenses/gpl-3.0](https://www.gnu.org/licenses/gpl-3.0.html) for details.

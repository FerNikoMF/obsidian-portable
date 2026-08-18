<div align="center">

<img src="assets/obsidian-icon.ico" width="72" alt="Obsidian Portable icon" />

# Obsidian Portable

**A portable launcher and auto-updater for [Obsidian](https://obsidian.md)**  
No system traces — everything stays in the app folder

[![Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)    [![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blueviolet)](LICENSE)   [![Vibe Coded](https://img.shields.io/badge/vibe-coded-purple)](https://github.com/FerNikoMF/Obsidian-Portable)    [![Platform](https://img.shields.io/badge/platform-Windows-blue?logo=windows)](https://github.com/FerNikoMF/Obsidian-Portable/releases)

[🇷🇺 Русская версия](README.ru.md)

</div>

---

## What is this?

Normally, Obsidian writes all its data to `%AppData%\obsidian` — which makes it impossible to carry around.  
This launcher fixes that: it starts Obsidian with the `--user-data-dir` flag, redirecting everything into a local `Data\ObsidianAppData` folder. No AppData writes.

It also handles downloading and installing the latest Obsidian release automatically — straight from GitHub Releases, no browser needed.

## Features

- ✦ **Fully portable** — everything in one folder, works from a USB drive
- ✦ **Auto-update** — compares installed vs available version, downloads and unpacks automatically
- ✦ **Pure-Rust extraction** — the NSIS installer is parsed and unpacked in-process, no external 7-Zip binary
- ✦ **First-run shortcut** — `Obsidian Updater.lnk` is created automatically on first launch
- ✦ **Clean updater UI** — progress bar, download speed and install status
- ✦ **RU / EN interface** — language toggle built into the app

## Folder structure

```
ObsidianPortable/
├── obsidian-portable.exe ← launches Obsidian (or updater if not installed)
├── Obsidian Updater.lnk  ← updater shortcut (created automatically)
├── GitHub.url            ← link to this repository
├── App/
│   └── Obsidian.exe      ← Obsidian core (unpacked here)
└── Data/
    └── ObsidianAppData/  ← all vaults, plugins, settings
```

## Usage

| Action | How |
|---|---|
| Launch Obsidian | `obsidian-portable.exe` |
| Open updater | `obsidian-portable.exe --updater` or `Obsidian Updater.lnk` |
| First install | `obsidian-portable.exe` will open the updater automatically |

## Building from source

```bash
# Requires Rust: https://rustup.rs

git clone https://github.com/FerNikoMF/Obsidian-Portable
cd Obsidian-Portable

cargo build --release
# Output: target/release/obsidian-portable.exe
```

No external binaries are required — everything is pulled in from crates at build time.

### Dependencies

| Crate | Purpose |
|---|---|
| `eframe` / `egui` | GUI |
| `reqwest` | HTTP download |
| `serde` | JSON parsing (GitHub API) |
| `sevenz-rust` | unpack `app-64.7z` into `App/` |
| `nsis` | parse and unpack the NSIS installer |
| `pelite` | read Obsidian.exe version from its PE resource |
| `mslnk` | create the `Obsidian Updater.lnk` shortcut |

## Vibe coding

This project was built in a **vibe coding** style — fast, iterative, AI-assisted.  
Feel free to open an [issue](https://github.com/FerNikoMF/Obsidian-Portable/issues) or a PR if you want to contribute.

## License

Licensed under **GPL-3.0**.  
See [LICENSE](LICENSE) or [gnu.org/licenses/gpl-3.0](https://www.gnu.org/licenses/gpl-3.0.html) for details.

---

<div align="center">
  <sub>Made with 💜 · Not affiliated with the Obsidian team</sub>
</div>

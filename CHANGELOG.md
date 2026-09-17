# Changelog

## [2.1.0] - 2026-09-17

### Added
- **SHA-256 verification** — downloaded installer is now verified against `.sha256` checksum from GitHub release (if available)
- **Graceful cancellation** — "Cancel" button stops download/installation mid-process via `AtomicBool`
- **Retry with exponential backoff** — GitHub API requests retry up to 3 times (2s → 4s) on transient network errors
- **Auto-launch after install** — Obsidian launches automatically after successful installation/update
- **File logging** — all operations logged to `Data/debug.log` via `tracing`
- **Smart x64 asset selection** — if the latest release has no x64 installer, previous releases are searched automatically (up to 30)
- **Fallback 7z search** — if `app-64.7z` is not found inside NSIS installer, any `.7z` file is used as fallback
- **New i18n strings** — `btn_cancel`, `err_hash_mismatch` added to EN/RU locales

### Changed
- **eframe 0.31 → 0.36.2** — migrated to `glow` (OpenGL) backend, eliminating `wgpu-hal` dependency conflicts
- **sevenz-rust → sevenz-rust2 0.22.2** — switched to actively maintained fork
- **nsis 0.3 → 0.4.1** — updated to latest version
- **sha2 0.10 → 0.11.0** — updated hash library
- **reqwest 0.13 → 0.13.5** — minor update
- All other dependencies bumped to latest versions

### Fixed
- Version check no longer shows "update available" for releases without x64 installer
- egui 0.36 API migration (`update` → `ui`, `set_style` → `set_theme`)
- sha2 0.11 hex formatting compatibility

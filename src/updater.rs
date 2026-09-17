use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use eframe::egui;
use nsis::NsisInstaller;
use sha2::{Digest, Sha256};
use tracing::{error, info, warn};

use crate::app::AppState;
use crate::i18n::{self, Lang};
use crate::utils::{exe_dir, installed_version, APP_DIR};

pub fn run(
    state: &Arc<Mutex<AppState>>,
    ctx:   &egui::Context,
    cancel: &Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {

    // Read lang once at start — doesn't change during install
    let t = i18n::get(Lang::load());

    let set = |msg: &str, prog: f32, speed: &str| {
        if cancel.load(Ordering::Relaxed) { return; }
        let mut s  = state.lock().unwrap();
        s.status   = msg.to_owned();
        s.progress = prog;
        s.speed    = speed.to_owned();
        ctx.request_repaint();
    };

    let release = state.lock().unwrap()
        .release.clone()
        .ok_or(t.err_no_release)?;

    let base = exe_dir();
    let tmp  = base.join(".tmp_update");
    let _    = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp)?;

    // ── Smart asset selection with release fallback ───────────────────────
    // Try current release first; if no x64 .exe found, search previous releases
    let (asset, used_release) = find_x64_asset(&release)?;

    // If we had to fall back to an older release, update state so the UI shows correct version
    let release = if used_release.tag_name != release.tag_name {
        warn!(from = %release.tag_name, fallback = %used_release.tag_name, "x64 not in latest, using older release");
        set(&format!("{} ({})", t.step_prepare, used_release.tag_name), 0.10, "");
        used_release.clone()
    } else {
        release
    };

    info!(asset = %asset.name, version = %release.tag_name, "Selected installer asset");

    let installer = tmp.join(&asset.name);

    // ── Download ──────────────────────────────────────────────────────────
    {
        let dl_msg = format!("{}  {}…", t.step_download, asset.name);
        set(&dl_msg, 0.15, &format!("0 {}", t.mib_s));

        let mut resp  = reqwest::blocking::Client::builder()
            .user_agent("obsidian-portable-launcher")
            .build()?
            .get(&asset.browser_download_url)
            .send()?.error_for_status()?;

        let total     = resp.content_length().unwrap_or(1) as f32;
        let mut file  = File::create(&installer)?;
        let mut buf   = [0u8; 65536];
        let mut done  = 0u64;
        let mut timer = Instant::now();
        let mut chunk = 0usize;
        let mut hasher = Sha256::new();

        loop {
            if cancel.load(Ordering::Relaxed) {
                info!("Download cancelled by user");
                let _ = fs::remove_dir_all(&tmp);
                return Err("Cancelled".into());
            }

            let n = resp.read(&mut buf)?;
            if n == 0 { break; }
            file.write_all(&buf[..n])?;
            hasher.update(&buf[..n]);
            done  += n as u64;
            chunk += n;
            if timer.elapsed() >= Duration::from_millis(250) {
                let speed = format!("{:.1} {}", (chunk as f32 / 1_048_576.0) / timer.elapsed().as_secs_f32(), t.mib_s);
                set(&dl_msg, 0.15 + (done as f32 / total) * 0.50, &speed);
                chunk = 0;
                timer = Instant::now();
            }
        }

        // ── SHA-256 verification ──────────────────────────────────────
        let hash_bytes = hasher.finalize();
        let computed_hash: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        info!(hash = %computed_hash, size = done, "Download complete");

        // Try to fetch .sha256 checksum file from the same release
        if let Some(checksum) = fetch_sha256_checksum(&release, &asset.name) {
            if checksum != computed_hash {
                error!(expected = %checksum, got = %computed_hash, "SHA-256 mismatch");
                let _ = fs::remove_dir_all(&tmp);
                return Err(t.err_hash_mismatch.into());
            }
            info!("SHA-256 verified successfully");
        } else {
            warn!("No .sha256 checksum file found in release — skipping hash verification");
        }
    }

    if cancel.load(Ordering::Relaxed) {
        let _ = fs::remove_dir_all(&tmp);
        return Err("Cancelled".into());
    }

    if fs::metadata(&installer)?.len() < 10_000_000 {
        let _ = fs::remove_dir_all(&tmp);
        error!("Downloaded file too small");
        return Err(t.err_too_small.into());
    }

    // ── Extract NSIS installer → find app-64.7z ───────────────────────────
    set(&t.step_unpack1, 0.70, "");

    let extracted_dir = tmp.join("extracted");
    fs::create_dir_all(&extracted_dir)?;

    extract_nsis_files(&installer, &extracted_dir)
        .map_err(|e| {
            error!(error = %e, "NSIS extraction failed");
            let _ = fs::remove_dir_all(&tmp);
            e
        })?;

    // ── Find and extract app-64.7z ────────────────────────────────────────
    set(&t.step_find_core, 0.80, "");
    // Try app-64.7z first, then fall back to any .7z in the extracted dir
    let core = crate::utils::find_file(&extracted_dir, "app-64.7z")
        .or_else(|| find_any_7z(&extracted_dir))
        .ok_or(t.err_no_core)?;
    info!(core = %core.display(), "Found core archive");

    set(&t.step_unpack2, 0.85, "");
    let app_dir = base.join(APP_DIR);
    fs::create_dir_all(&app_dir)?;

    sevenz_rust2::decompress_file(&core, &app_dir)
        .map_err(|e| {
            error!(error = %e, "7z decompression failed");
            let _ = fs::remove_dir_all(&tmp);
            format!("7z: {e}")
        })?;

    // ── Cleanup ───────────────────────────────────────────────────────────
    set(&t.step_cleanup, 0.99, "");
    let _ = fs::remove_dir_all(&tmp);

    let mut s   = state.lock().unwrap();
    s.status    = t.step_done.to_owned();
    s.installed = installed_version();
    s.progress  = 1.0;
    s.success   = Some(true);
    s.running   = false;
    ctx.request_repaint();

    info!(version = ?s.installed, "Installation completed successfully");

    Ok(())
}

/// Attempts to download the `.sha256` checksum file for a given asset from the release.
/// Returns the expected hex digest, or None if no checksum file exists.
fn fetch_sha256_checksum(release: &crate::github::Release, asset_name: &str) -> Option<String> {
    // GitHub releases sometimes include a `.sha256` or `.SHA256` file
    let checksum_name = format!("{}.sha256", asset_name);
    let checksum_asset = release.assets.iter()
        .find(|a| a.name.eq_ignore_ascii_case(&checksum_name))?;

    let resp = reqwest::blocking::Client::builder()
        .user_agent("obsidian-portable-launcher")
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?
        .get(&checksum_asset.browser_download_url)
        .send()
        .ok()?
        .error_for_status()
        .ok()?;

    let text = resp.text().ok()?;
    // Checksum files are typically: "<hash>  <filename>" or just "<hash>"
    let hash = text.split_whitespace().next()?.to_lowercase();
    if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hash)
    } else {
        warn!(content = %text, "Checksum file has unexpected format");
        None
    }
}

/// Extracts embedded files from an NSIS installer into `out_dir`.
/// Uses the `nsis` crate to parse the NSIS structure and decompress payloads.
fn extract_nsis_files(
    installer: &Path,
    out_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(installer)?;
    let inst = NsisInstaller::from_bytes(&data)
        .map_err(|e| format!("nsis parse: {e}"))?;

    for file in inst.files() {
        let file = file.map_err(|e| format!("nsis file entry: {e}"))?;
        let raw = file.name().map_err(|e| format!("nsis file name: {e}"))?;
        let name = raw.to_string();
        let base = Path::new(&name)
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| name.clone());

        // We only need the bundled 7z archive; skip the rest.
        if !base.ends_with(".7z") {
            continue;
        }

        let content = file
            .decompress()
            .map_err(|e| format!("nsis decompress {base}: {e}"))?;
        fs::write(out_dir.join(&base), &content)?;
        info!(file = %base, "Extracted from NSIS");
    }

    Ok(())
}

/// Recursively finds the first `.7z` file in a directory (fallback for renamed archives).
fn find_any_7z(dir: &Path) -> Option<std::path::PathBuf> {
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_any_7z(&path) {
                return Some(found);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("7z") {
            return Some(path);
        }
    }
    None
}

/// Public wrapper for find_x64_asset — used by app.rs during version check.
pub fn find_x64_asset_public(
    latest: &crate::github::Release,
) -> Result<(crate::github::Asset, crate::github::Release), Box<dyn std::error::Error>> {
    find_x64_asset(latest)
}

/// Tries to find an x64 .exe asset in the given release.
/// If not found, fetches previous releases from GitHub and searches them one by one.
/// Returns (asset, release) — the release may be older than the latest.
fn find_x64_asset(
    latest: &crate::github::Release,
) -> Result<(crate::github::Asset, crate::github::Release), Box<dyn std::error::Error>> {
    // Try the latest release first
    if let Some(asset) = pick_x64_from(latest) {
        return Ok((asset, latest.clone()));
    }

    warn!(tag = %latest.tag_name, "No x64 installer in latest release, searching older releases…");

    // Fetch previous releases
    let releases = crate::github::fetch_releases()?;

    for rel in &releases {
        // Skip the latest — we already tried it
        if rel.tag_name == latest.tag_name {
            continue;
        }
        if let Some(asset) = pick_x64_from(rel) {
            info!(tag = %rel.tag_name, asset = %asset.name, "Found x64 installer in older release");
            return Ok((asset, rel.clone()));
        }
    }

    // Nothing found anywhere — build a helpful error
    let all_names: Vec<&str> = releases.iter()
        .flat_map(|r| r.assets.iter().map(|a| a.name.as_str()))
        .collect();
    error!("No x64 installer found in any of the last {} releases", releases.len());
    Err(format!(
        "x64 installer not found in any recent release.\nAll assets: {}",
        all_names.join(", ")
    ).into())
}

/// Picks the best x64 .exe from a single release's assets.
fn pick_x64_from(release: &crate::github::Release) -> Option<crate::github::Asset> {
    let exe_assets: Vec<_> = release.assets.iter()
        .filter(|a| a.name.starts_with("Obsidian-") && a.name.ends_with(".exe"))
        .collect();

    exe_assets.iter()
        .find(|a| !a.name.contains("arm64") && (a.name.contains("x64") || a.name.contains("64")))
        .or_else(|| exe_assets.iter().find(|a| !a.name.contains("arm64")))
        .or_else(|| exe_assets.first())
        .map(|a| (*a).clone())
}

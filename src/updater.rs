use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use eframe::egui;
use nsis::NsisInstaller;

use crate::app::AppState;
use crate::i18n::{self, Lang};
use crate::utils::{exe_dir, installed_version, APP_DIR};

pub fn run(
    state: &Arc<Mutex<AppState>>,
    ctx:   &egui::Context,
) -> Result<(), Box<dyn std::error::Error>> {

    // Read lang once at start — doesn't change during install
    let t = i18n::get(Lang::load());

    let set = |msg: &str, prog: f32, speed: &str| {
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

    let asset = release.assets.iter()
        .find(|a| a.name.starts_with("Obsidian-") && a.name.ends_with(".exe") && !a.name.contains("arm64"))
        .ok_or(t.err_no_asset)?;

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

        loop {
            let n = resp.read(&mut buf)?;
            if n == 0 { break; }
            file.write_all(&buf[..n])?;
            done  += n as u64;
            chunk += n;
            if timer.elapsed() >= Duration::from_millis(250) {
                let speed = format!("{:.1} {}", (chunk as f32 / 1_048_576.0) / timer.elapsed().as_secs_f32(), t.mib_s);
                set(&dl_msg, 0.15 + (done as f32 / total) * 0.55, &speed);
                chunk = 0;
                timer = Instant::now();
            }
        }
    }

    if fs::metadata(&installer)?.len() < 10_000_000 {
        let _ = fs::remove_dir_all(&tmp);
        return Err(t.err_too_small.into());
    }

    // ── Extract NSIS installer → find app-64.7z ───────────────────────────
    set(&t.step_unpack1, 0.75, "");

    let extracted_dir = tmp.join("extracted");
    fs::create_dir_all(&extracted_dir)?;

    extract_nsis_files(&installer, &extracted_dir)
        .map_err(|e| { let _ = fs::remove_dir_all(&tmp); e })?;

    // ── Find and extract app-64.7z ────────────────────────────────────────
    set(&t.step_find_core, 0.85, "");
    let core = crate::utils::find_file(&extracted_dir, "app-64.7z")
        .ok_or(t.err_no_core)?;

    set(&t.step_unpack2, 0.90, "");
    let app_dir = base.join(APP_DIR);
    fs::create_dir_all(&app_dir)?;

    sevenz_rust::decompress_file(&core, &app_dir)
        .map_err(|e| { let _ = fs::remove_dir_all(&tmp); format!("7z: {e}") })?;

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

    Ok(())
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
    }

    Ok(())
}

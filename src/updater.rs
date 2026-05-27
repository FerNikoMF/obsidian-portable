use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::os::windows::process::CommandExt;
use eframe::egui;

use crate::app::AppState;
use crate::i18n::{self, Lang};
use crate::utils::{exe_dir, find_file, installed_version, APP_DIR, CREATE_NO_WINDOW};

const EMBEDDED_7Z_EXE: &[u8] = include_bytes!("../7z.exe");
const EMBEDDED_7Z_DLL: &[u8] = include_bytes!("../7z.dll");

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

    set(t.step_archiver, 0.0, "");
    File::create(tmp.join("7z.exe"))?.write_all(EMBEDDED_7Z_EXE)?;
    File::create(tmp.join("7z.dll"))?.write_all(EMBEDDED_7Z_DLL)?;
    let z = tmp.join("7z.exe");

    let asset = release.assets.iter()
        .find(|a| a.name.starts_with("Obsidian-") && a.name.ends_with(".exe") && !a.name.contains("arm64"))
        .ok_or(t.err_no_asset)?;

    let installer = tmp.join(&asset.name);

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
                set(&dl_msg, 0.15 + (done as f32 / total) * 0.65, &speed);
                chunk = 0;
                timer = Instant::now();
            }
        }
    }

    if fs::metadata(&installer)?.len() < 10_000_000 {
        let _ = fs::remove_dir_all(&tmp);
        return Err(t.err_too_small.into());
    }

    set(t.step_unpack1, 0.85, "");
    let extracted = tmp.join("extracted");
    run_7z(&z, &["x", installer.to_str().unwrap(), &format!("-o{}", extracted.to_str().unwrap()), "-y"])
        .map_err(|e| { let _ = fs::remove_dir_all(&tmp); e })?;

    set(t.step_find_core, 0.92, "");
    let core = find_file(&extracted, "app-64.7z")
        .ok_or(t.err_no_core)?;

    set(t.step_unpack2, 0.95, "");
    let app_dir = base.join(APP_DIR);
    fs::create_dir_all(&app_dir)?;
    run_7z(&z, &["x", core.to_str().unwrap(), &format!("-o{}", app_dir.to_str().unwrap()), "-y"])
        .map_err(|e| { let _ = fs::remove_dir_all(&tmp); e })?;

    set(t.step_cleanup, 0.99, "");
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

fn run_7z(z: &std::path::Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let out = Command::new(z).args(args).creation_flags(CREATE_NO_WINDOW).output()?;
    if out.status.success() { Ok(()) }
    else {
        Err(format!("7-Zip: {}", encoding_rs::IBM866.decode(&out.stderr).0).into())
    }
}

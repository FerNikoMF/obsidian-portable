use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::os::windows::process::CommandExt;
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Stroke, StrokeKind, Vec2};

use crate::github::Release;
use crate::i18n::{self, Lang};
use crate::utils::{
    installed_version, normalize_version,
    obsidian_data_dir, obsidian_exe, user_data_arg, DETACHED_PROCESS,
};
use crate::updater;

// ── Palette ───────────────────────────────────────────────────────────────────
const BG:          Color32 = Color32::from_rgb(0x14, 0x13, 0x1b);
const SURFACE:     Color32 = Color32::from_rgb(0x1d, 0x1c, 0x26);
const SURFACE2:    Color32 = Color32::from_rgb(0x25, 0x24, 0x30);
const BORDER:      Color32 = Color32::from_rgb(0x34, 0x33, 0x46);
const PURPLE:      Color32 = Color32::from_rgb(0x7c, 0x3a, 0xed);
const TEXT:        Color32 = Color32::from_rgb(0xe9, 0xe9, 0xf2);
const MUTED:       Color32 = Color32::from_rgb(0x9a, 0x99, 0xad);
const GREEN:       Color32 = Color32::from_rgb(0x9c, 0xce, 0x9c);
const RED:         Color32 = Color32::from_rgb(0xf0, 0x8a, 0xa6);
const YELLOW:      Color32 = Color32::from_rgb(0xf2, 0xd9, 0x8c);

const GITHUB_URL: &str = "https://github.com/FerNikoMF/Obsidian-Portable";

const PURPLE_HOVER:  Color32 = Color32::from_rgb(0x8b, 0x5c, 0xf6);
const SURFACE2_HOVER: Color32 = Color32::from_rgb(0x32, 0x31, 0x40);

// ── Unicode icons ─────────────────────────────────────────────────────────────

fn icon_ui(ui: &mut egui::Ui, icon: &str, color: Color32, size: f32) {
    ui.label(RichText::new(icon).color(color).size(size));
}

// ── Shared state ──────────────────────────────────────────────────────────────

pub struct AppState {
    pub running:   bool,
    pub status:    String,
    pub progress:  f32,
    pub speed:     String,
    pub success:   Option<bool>,
    pub installed: Option<String>,
    pub available: String,
    pub release:   Option<Release>,
}

// ── View enum — derived each frame ───────────────────────────────────────────

enum View<'a> {
    Loading,
    NotInstalled { available: &'a str },
    UpToDate     { version:   &'a str },
    UpdateReady  { installed: &'a str, available: &'a str },
    Installing,
    Done         { version:   &'a str },
    Failed       (&'a str),
}

fn view_of(s: &AppState) -> View<'_> {
    if s.running                    { return View::Installing; }
    if s.success == Some(false)     { return View::Failed(&s.status); }
    if let Some(true) = s.success {
        let ver = s.installed.as_deref().unwrap_or("?");
        return View::Done { version: ver };
    }
    if s.release.is_none()          { return View::Loading; }

    match &s.installed {
        None => View::NotInstalled { available: &s.available },
        Some(inst) => {
            if normalize_version(inst) == normalize_version(&s.available) {
                View::UpToDate { version: inst }
            } else {
                View::UpdateReady { installed: inst, available: &s.available }
            }
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct UpdaterApp {
    state: Arc<Mutex<AppState>>,
    lang:  Lang,
}

impl UpdaterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);

        let state = Arc::new(Mutex::new(AppState {
            running:   false,
            status:    i18n::get(Lang::En).checking.to_owned(),
            progress:  0.0,
            speed:     String::new(),
            success:   None,
            installed: installed_version(),
            available: String::new(),
            release:   None,
        }));

        let sc  = Arc::clone(&state);
        let ctx = cc.egui_ctx.clone();
        thread::spawn(move || {
            match crate::github::fetch_latest() {
                Ok(r) => {
                    let mut s = sc.lock().unwrap();
                    s.available = r.tag_name.clone();
                    s.release   = Some(r);
                }
                Err(e) => {
                    let mut s = sc.lock().unwrap();
                    s.available = "–".to_owned();
                    s.status    = format!("{}: {e}", i18n::get(Lang::En).err_fetch);
                    s.success   = Some(false);
                }
            }
            ctx.request_repaint();
        });

        Self { state, lang: Lang::load() }
    }
}

impl eframe::App for UpdaterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state.lock().unwrap().running { ctx.request_repaint(); }

        let t = i18n::get(self.lang);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(0.0))
            .show(ctx, |ui| {
                        draw_header(ui, &t);

                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(18, 16))
                    .show(ui, |ui| {
                        let mut s = self.state.lock().unwrap();
                        match view_of(&s) {

                            View::Loading => {
                                info_card(ui, SURFACE, BORDER, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label(muted(&t.checking));
                                    });
                                });
                            }

                            View::NotInstalled { available } => {
                                let avail = available.to_owned();
                                info_card(ui, SURFACE, BORDER, |ui| {
                                    ver_row(ui, &t.installed, &t.not_installed, RED);
                                    ui.add_space(4.0);
                                    ver_row(ui, &t.available, &avail, GREEN);
                                });
                                ui.add_space(10.0);
                                if action_btn(ui, ctx, "⬇", &t.btn_install, PURPLE, PURPLE_HOVER) {
                                    start_install(&mut s, &self.state, ctx, &t.step_prepare);
                                }
                            }

                            View::UpToDate { version } => {
                                let ver = version.to_owned();
                                info_card(ui, SURFACE, GREEN.gamma_multiply(0.25), |ui| {
                                    ui.horizontal(|ui| {
                                        icon_ui(ui, "✔", GREEN, 16.0);
                                        ui.add_space(6.0);
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(&t.up_to_date).color(TEXT).size(13.0).strong());
                                            ui.label(muted(&ver));
                                        });
                                    });
                                });
                                ui.add_space(10.0);
                                if action_btn(ui, ctx, "↻", &t.btn_reinstall, SURFACE2, SURFACE2_HOVER) {
                                    start_install(&mut s, &self.state, ctx, &t.step_prepare);
                                }
                                ui.add_space(6.0);
                                launch_btn(ui, ctx, "▶", &t.btn_launch);
                            }

                            View::UpdateReady { installed, available } => {
                                let (inst, avail) = (installed.to_owned(), available.to_owned());
                                info_card(ui, SURFACE, YELLOW.gamma_multiply(0.3), |ui| {
                                    ui.horizontal(|ui| {
                                        icon_ui(ui, "▲", YELLOW, 16.0);
                                        ui.add_space(6.0);
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(&t.update_ready).color(TEXT).size(13.0).strong());
                                            ui.horizontal(|ui| {
                                                ui.label(muted(&inst));
                                                ui.label(muted("→"));
                                                ui.label(RichText::new(&avail).color(YELLOW).size(12.0));
                                            });
                                        });
                                    });
                                });
                                ui.add_space(10.0);
                                if action_btn(ui, ctx, "⬆", &format!("{} {}", t.btn_update, avail), PURPLE, PURPLE_HOVER) {
                                    start_install(&mut s, &self.state, ctx, &t.step_prepare);
                                }
                            }

                            View::Installing => {
                                progress_button(ui, ctx, s.progress, &s.status, &s.speed, &t.mib_s);
                            }

                            View::Done { version } => {
                                let ver = version.to_owned();
                                info_card(ui, SURFACE, GREEN.gamma_multiply(0.25), |ui| {
                                    ui.horizontal(|ui| {
                                        icon_ui(ui, "✔", GREEN, 16.0);
                                        ui.add_space(6.0);
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(&t.done_label).color(GREEN).size(13.0).strong());
                                            ui.label(muted(&ver));
                                        });
                                    });
                                });
                                ui.add_space(10.0);
                                launch_btn(ui, ctx, "▶", &t.btn_launch);
                            }

                            View::Failed(msg) => {
                                let msg = msg.to_owned();
                                info_card(ui, SURFACE, RED.gamma_multiply(0.3), |ui| {
                                    ui.horizontal(|ui| {
                                        icon_ui(ui, "✖", RED, 16.0);
                                        ui.add_space(6.0);
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(&t.err_label).color(RED).size(13.0).strong());
                                            ui.label(muted(&msg));
                                        });
                                    });
                                });
                                ui.add_space(10.0);
                                if action_btn(ui, ctx, "↻", &t.btn_retry, SURFACE2, SURFACE2_HOVER) {
                                    s.success  = None;
                                    s.status.clear();
                                    s.progress = 0.0;
                                }
                            }
                        }
                    });

                let remaining = ui.available_height();
                if remaining > 32.0 { ui.add_space(remaining - 32.0); }
                draw_footer(ui, &t.footer_link);
            });
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn start_install(s: &mut AppState, state: &Arc<Mutex<AppState>>, ctx: &egui::Context, step: &str) {
    s.running  = true;
    s.success  = None;
    s.progress = 0.0;
    s.speed.clear();
    s.status   = step.to_owned();

    let sc  = Arc::clone(state);
    let ctx = ctx.clone();
    thread::spawn(move || {
        if let Err(e) = updater::run(&sc, &ctx) {
            let mut s = sc.lock().unwrap();
            s.status  = e.to_string();
            s.success = Some(false);
            s.running = false;
            ctx.request_repaint();
        }
    });
}

fn launch_obsidian(ctx: &egui::Context) {
    let _ = fs::create_dir_all(obsidian_data_dir());
    let _ = Command::new(obsidian_exe())
        .arg(user_data_arg())
        .creation_flags(DETACHED_PROCESS)
        .spawn();
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
}

// ── Widgets ───────────────────────────────────────────────────────────────────

fn draw_header(ui: &mut egui::Ui, t: &i18n::T) {
    egui::Frame::new()
        .fill(SURFACE)
        .inner_margin(egui::Margin::symmetric(18, 14))
        .stroke(Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                draw_logo(ui);
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&t.app_name)
                            .font(FontId::proportional(15.5))
                            .color(TEXT)
                            .strong(),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(&t.app_subtitle)
                            .font(FontId::proportional(10.5))
                            .color(MUTED),
                    );
                });
            });
        });
}

/// Draws the Obsidian diamond logo as a vector shape.
fn draw_logo(ui: &mut egui::Ui) {
    let size = 28.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(size, size), egui::Sense::hover());
    let p = ui.painter();
    let c = rect.center();
    let s = size * 0.44;

    let outer = vec![
        egui::pos2(c.x, c.y - s),
        egui::pos2(c.x + s * 0.62, c.y),
        egui::pos2(c.x, c.y + s),
        egui::pos2(c.x - s * 0.62, c.y),
    ];
    p.add(egui::Shape::convex_polygon(outer, PURPLE, Stroke::NONE));

    let inner_s = s * 0.48;
    let inner = vec![
        egui::pos2(c.x, c.y - inner_s),
        egui::pos2(c.x + inner_s * 0.45, c.y),
        egui::pos2(c.x, c.y + inner_s * 0.55),
        egui::pos2(c.x - inner_s * 0.45, c.y),
    ];
    p.add(egui::Shape::convex_polygon(
        inner,
        Color32::from_rgb(0x9b, 0x6b, 0xf7),
        Stroke::NONE,
    ));
}

fn draw_footer(ui: &mut egui::Ui, link_label: &str) {
    egui::Frame::new()
        .fill(SURFACE)
        .inner_margin(egui::Margin::symmetric(18, 9))
        .stroke(Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::new(7.0, 7.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 3.0, PURPLE);
                ui.add_space(6.0);
                ui.hyperlink_to(
                    RichText::new(link_label).color(MUTED).size(11.0),
                    GITHUB_URL,
                );
            });
        });
}

fn info_card(ui: &mut egui::Ui, fill: Color32, border: Color32, f: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(16, 14))
        .corner_radius(CornerRadius::same(12))
        .stroke(Stroke::new(1.0_f32, border))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            f(ui);
        });
}

fn ver_row(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(muted(label));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).color(color).size(13.0));
        });
    });
}

fn action_btn(ui: &mut egui::Ui, _ctx: &egui::Context, icon: &str, label: &str, fill: Color32, hover: Color32) -> bool {
    icon_button(ui, icon, label, fill, hover)
}

fn launch_btn(ui: &mut egui::Ui, ctx: &egui::Context, icon: &str, label: &str) {
    if icon_button(ui, icon, label, SURFACE2, SURFACE2_HOVER) {
        launch_obsidian(ctx);
    }
}

fn icon_button(ui: &mut egui::Ui, icon: &str, label: &str, fill: Color32, hover: Color32) -> bool {
    let h = 38.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), egui::Sense::click());
    let p = ui.painter();
    p.rect_filled(rect, 10.0, if resp.hovered() { hover } else { fill });
    p.text(
        egui::pos2(rect.min.x + 20.0, rect.center().y),
        egui::Align2::CENTER_CENTER,
        icon,
        FontId::proportional(15.0),
        Color32::WHITE,
    );
    p.text(rect.center(), egui::Align2::CENTER_CENTER, label,
        FontId::proportional(13.0), Color32::WHITE);
    resp.clicked()
}

fn progress_button(ui: &mut egui::Ui, ctx: &egui::Context, progress: f32, status: &str, speed: &str, mib_s: &str) {
    let size = Vec2::new(ui.available_width(), 38.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if !ui.is_rect_visible(rect) { return; }

    let p = ui.painter();
    p.rect_filled(rect, 10.0, SURFACE2);

    if progress > 0.001 {
        let fw        = (rect.width() * progress).min(rect.width());
        let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fw, rect.height()));
        p.with_clip_rect(fill_rect).rect_filled(rect, 10.0, PURPLE);

        let t  = ctx.input(|i| i.time) as f32;
        let sx = rect.left() + (t * 0.55 % 1.0_f32) * fw;
        let sw = (fw * 0.20).max(28.0);
        let shimmer = egui::Rect::from_x_y_ranges(sx..=(sx + sw), rect.y_range());
        p.with_clip_rect(fill_rect).rect_filled(
            shimmer, 0.0,
            Color32::from_rgba_premultiplied(255, 255, 255, 10),
        );
    }

    p.rect_stroke(rect, 10.0, Stroke::new(1.0_f32, BORDER), StrokeKind::Outside);

    let text = {
        let pct = format!("{:.0}%", progress * 100.0);
        match (status.is_empty(), speed.is_empty()) {
            (false, false) => format!("{status}  ·  {speed} {mib_s}  ·  {pct}"),
            (false, true)  => format!("{status}  ·  {pct}"),
            _              => pct,
        }
    };
    p.text(rect.center(), egui::Align2::CENTER_CENTER, text,
        FontId::proportional(12.0), Color32::WHITE);
}

fn muted(s: &str) -> RichText { RichText::new(s).color(MUTED).size(12.0) }

// ── Theme ─────────────────────────────────────────────────────────────────────

fn apply_theme(ctx: &egui::Context) {
    let mut vis = egui::Visuals::dark();
    vis.panel_fill                       = BG;
    vis.window_fill                      = SURFACE;
    vis.override_text_color              = Some(TEXT);
    vis.widgets.noninteractive.bg_fill   = SURFACE;
    vis.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    vis.widgets.inactive.bg_fill         = PURPLE;
    vis.widgets.inactive.bg_stroke       = Stroke::NONE;
    vis.widgets.hovered.bg_fill          = Color32::from_rgb(0x8b, 0x5c, 0xf6);
    vis.widgets.hovered.bg_stroke        = Stroke::NONE;
    vis.widgets.active.bg_fill           = PURPLE;
    vis.widgets.active.bg_stroke         = Stroke::NONE;
    vis.selection.bg_fill                = PURPLE;
    vis.window_corner_radius             = CornerRadius::same(14);
    vis.window_shadow                    = egui::Shadow {
        color:  Color32::from_rgba_premultiplied(0, 0, 0, 50),
        blur:   16,
        spread: 0,
        offset: [0, 6],
    };
    ctx.set_visuals(vis);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing  = Vec2::new(10.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(0);
    ctx.set_style(style);
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run() -> eframe::Result<()> {
    eframe::run_native(
        "Obsidian Portable",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([420.0_f32, 340.0_f32])
                .with_resizable(false),
            persist_window: false,
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(UpdaterApp::new(cc)))),
    )
}

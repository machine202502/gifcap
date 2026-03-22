#![cfg_attr(windows, windows_subsystem = "windows")]
#![cfg_attr(not(windows), allow(dead_code, unused_imports))]

mod icon_bitmap;
mod theme;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::sync::Arc;

#[cfg(windows)]
use eframe::egui;
#[cfg(windows)]
use gifcap_core::{
    ensure_dir, export_session, instance_session_dir, log_action, log_error, log_info, log_warn,
    output_dir, output_filename, ExportFormat, Session,
};
#[cfg(windows)]
use image::RgbaImage;
#[cfg(windows)]
use uuid::Uuid;
#[cfg(windows)]
use raw_window_handle::HasWindowHandle;
#[cfg(windows)]
use gifcap_windows::PhysicalRect;

#[cfg(not(windows))]
fn main() {
    eprintln!("gifcap currently runs only on Windows (capture + GUI are wired for Win32).");
    std::process::exit(1);
}

/// Minimum capture viewport edge in **physical** pixels (region below toolbar).
const MIN_CAPTURE_PX: u32 = 32;
/// Minimum inner width in **physical pixels** (UI may clip; capture still needs a larger area to record).
const MIN_INNER_WIDTH_PX: f32 = 48.0;

/// Per-format encoder quality defaults (1–100), remembered when switching format.
const DEFAULT_QUALITY_GIF: u8 = 8;
const DEFAULT_QUALITY_WEBP: u8 = 50;
/// MP4: 100 reproduces previous libx264 CRF 17 / h264_mf quality ~90 mapping.
const DEFAULT_QUALITY_MP4: u8 = 100;
/// Minimum inner height in **points** (toolbar rows + capture strip).
const MIN_INNER_HEIGHT_PT: f32 = 140.0;

const STATUS_SAVED_PREFIX: &str = "Saved ";

const GITHUB_REPO_URL: &str = "https://github.com/machine202502/gifcap";

#[cfg(windows)]
fn main() -> eframe::Result<()> {
    gifcap_windows::init_dpi_awareness();

    let session_id = Uuid::new_v4();
    let session_id_str = session_id.to_string();
    log_action(
        Some(session_id_str.as_str()),
        "application started; session workspace ready",
    );

    let session_dir = instance_session_dir(&session_id_str).map_err(|e| {
        eframe::Error::AppCreation(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("session dir: {e}"),
        )))
    })?;
    ensure_dir(&session_dir).map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("gifcap")
            .with_inner_size([600.0, 500.0])
            // Width: permissive until first frame applies `MIN_INNER_WIDTH_PX` via pixels_per_point.
            .with_min_inner_size(egui::vec2(1.0, MIN_INNER_HEIGHT_PT))
            .with_clamp_size_to_monitor_size(true)
            .with_always_on_top()
            .with_icon(Arc::new(theme::app_icon())),
        ..Default::default()
    };

    eframe::run_native(
        "gifcap",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(GifcapApp::new(session_id)))
        }),
    )
}

#[cfg(windows)]
struct GifcapApp {
    session_id: Uuid,
    fps: f32,
    output_format: ExportFormat,
    quality_gif: u8,
    #[cfg(not(feature = "slim"))]
    quality_webp: u8,
    #[cfg(not(feature = "slim"))]
    quality_mp4: u8,
    recording: bool,
    paused: bool,
    session: Option<Session>,
    last_tick: Instant,
    /// Toolbar height in **physical client pixels** (drives `SetWindowRgn` + capture rect).
    toolbar_h_px: i32,
    /// While recording: frozen toolbar height used for Rgn + capture so layout changes (e.g. “frames: N”) do not change capture size vs [`Session`].
    recording_toolbar_h: Option<i32>,
    status: String,
    /// After copying saved path to clipboard; show brief «Скопировано».
    copy_feedback_until: Option<Instant>,
    export_busy: bool,
    /// Format of the export in flight (spinner copy for WebP vs GIF/MP4).
    export_busy_format: Option<ExportFormat>,
    export_rx: Option<Receiver<Result<PathBuf, String>>>,
    /// WebP transcode only: set to `true` from UI to request [`CoreError::Cancelled`].
    export_cancel: Option<Arc<AtomicBool>>,
}

#[cfg(windows)]
impl GifcapApp {
    #[inline]
    fn log_sid(&self) -> String {
        self.session_id.to_string()
    }

    fn log_a(&self, msg: &str) {
        let id = self.log_sid();
        log_action(Some(&id), msg);
    }

    fn log_i(&self, msg: &str) {
        let id = self.log_sid();
        log_info(Some(&id), msg);
    }

    fn log_w(&self, msg: &str) {
        let id = self.log_sid();
        log_warn(Some(&id), msg);
    }

    fn log_e(&self, msg: &str) {
        let id = self.log_sid();
        log_error(Some(&id), msg);
    }

    fn new(session_id: Uuid) -> Self {
        Self {
            session_id,
            fps: 10.0,
            output_format: ExportFormat::Gif,
            quality_gif: DEFAULT_QUALITY_GIF,
            #[cfg(not(feature = "slim"))]
            quality_webp: DEFAULT_QUALITY_WEBP,
            #[cfg(not(feature = "slim"))]
            quality_mp4: DEFAULT_QUALITY_MP4,
            recording: false,
            paused: false,
            session: None,
            last_tick: Instant::now(),
            toolbar_h_px: 0,
            recording_toolbar_h: None,
            status: String::new(),
            copy_feedback_until: None,
            export_busy: false,
            export_busy_format: None,
            export_rx: None,
            export_cancel: None,
        }
    }
}

#[cfg(windows)]
impl eframe::App for GifcapApp {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.cleanup_on_exit();
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.fps = self.fps.clamp(4.0, 60.0);
        let text_main = egui::Color32::from_rgb(15, 65, 85);

        if !self.status.starts_with(STATUS_SAVED_PREFIX) {
            self.copy_feedback_until = None;
        }
        if self
            .copy_feedback_until
            .is_some_and(|t| Instant::now() < t)
        {
            ctx.request_repaint_after(Duration::from_millis(200));
        }

        if let Some(rx) = &self.export_rx {
            match rx.try_recv() {
                Ok(Ok(path)) => {
                    let p = path.display().to_string();
                    self.log_i(&format!("UI: export saved to {p}"));
                    self.status = format!("{}{}", STATUS_SAVED_PREFIX, p);
                    self.copy_feedback_until = None;
                    self.export_busy = false;
                    self.export_busy_format = None;
                    self.export_rx = None;
                    self.export_cancel = None;
                }
                Ok(Err(e)) => {
                    if e.contains("cancelled") {
                        self.log_w(&format!("UI: export cancelled: {e}"));
                    } else {
                        self.log_e(&format!("UI: export failed: {e}"));
                    }
                    self.copy_feedback_until = None;
                    self.status = e;
                    self.export_busy = false;
                    self.export_busy_format = None;
                    self.export_rx = None;
                    self.export_cancel = None;
                }
                Err(TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(32));
                }
                Err(TryRecvError::Disconnected) => {
                    self.log_w("export channel disconnected before result");
                    self.export_busy = false;
                    self.export_busy_format = None;
                    self.export_rx = None;
                    self.export_cancel = None;
                }
            }
        }

        let ui_enabled = !self.export_busy;

        let panel = egui::TopBottomPanel::top("bar")
            .frame(theme::toolbar_frame(ctx))
            .show(ctx, |ui| {
                if self.export_busy {
                    let busy_label = {
                        #[cfg(feature = "slim")]
                        {
                            "Saving file…"
                        }
                        #[cfg(not(feature = "slim"))]
                        {
                            match self.export_busy_format {
                                Some(ExportFormat::Webp) => "Converting to WebP…",
                                _ => "Saving file…",
                            }
                        }
                    };
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                egui::RichText::new(busy_label)
                                    .small()
                                    .color(text_main),
                            );
                            #[cfg(not(feature = "slim"))]
                            if self.export_busy_format == Some(ExportFormat::Webp) {
                                if ui.add(theme::secondary_button("Cancel")).clicked() {
                                    if let Some(c) = &self.export_cancel {
                                        c.store(true, Ordering::Relaxed);
                                    }
                                }
                            }
                        });
                    });
                    return;
                }

                ui.add_enabled_ui(ui_enabled, |ui| {
                    theme::apply_toolbar_spacing(ui);
                    ui.vertical(|ui| {
                        // Row 1: FPS, Quality (per format), Format. Статус — строка 3.
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("FPS")
                                    .small()
                                    .color(text_main)
                                    .strong(),
                            );
                            ui.add_sized(
                                [100.0, 18.0],
                                egui::Slider::new(&mut self.fps, 4.0..=60.0)
                                    .integer()
                                    .text_color(text_main)
                                    .handle_shape(egui::style::HandleShape::Rect {
                                        aspect_ratio: 0.45,
                                    })
                                    .trailing_fill(true),
                            );
                            ui.separator();

                            ui.add_enabled_ui(!self.recording, |ui| {
                                #[cfg(feature = "slim")]
                                {
                                    let q_ref: &mut u8 = &mut self.quality_gif;
                                    let mut q = i32::from(*q_ref);
                                    ui.label(
                                        egui::RichText::new("Quality")
                                            .small()
                                            .color(text_main),
                                    );
                                    if ui
                                        .add_sized(
                                            [120.0, 18.0],
                                            egui::Slider::new(&mut q, 1..=100)
                                                .integer()
                                                .suffix("%")
                                                .text_color(text_main)
                                                .handle_shape(egui::style::HandleShape::Rect {
                                                    aspect_ratio: 0.45,
                                                })
                                                .trailing_fill(true),
                                        )
                                        .changed()
                                    {
                                        *q_ref = q.clamp(1, 100) as u8;
                                    }
                                }
                                #[cfg(not(feature = "slim"))]
                                {
                                    let q_ref: &mut u8 = match self.output_format {
                                        ExportFormat::Gif => &mut self.quality_gif,
                                        ExportFormat::Webp => &mut self.quality_webp,
                                        ExportFormat::Mp4 => &mut self.quality_mp4,
                                    };
                                    let mut q = i32::from(*q_ref);
                                    ui.label(
                                        egui::RichText::new("Quality")
                                            .small()
                                            .color(text_main),
                                    );
                                    if ui
                                        .add_sized(
                                            [120.0, 18.0],
                                            egui::Slider::new(&mut q, 1..=100)
                                                .integer()
                                                .suffix("%")
                                                .text_color(text_main)
                                                .handle_shape(egui::style::HandleShape::Rect {
                                                    aspect_ratio: 0.45,
                                                })
                                                .trailing_fill(true),
                                        )
                                        .changed()
                                    {
                                        *q_ref = q.clamp(1, 100) as u8;
                                    }
                                    ui.separator();
                                    ui.label(
                                        egui::RichText::new("Format")
                                            .small()
                                            .color(text_main),
                                    );
                                    ui.radio_value(&mut self.output_format, ExportFormat::Gif, "GIF");
                                    ui.radio_value(&mut self.output_format, ExportFormat::Mp4, "MP4");
                                    ui.radio_value(&mut self.output_format, ExportFormat::Webp, "WebP");
                                }
                            });
                        });

                        // Row 2: Record + Screen, или Save / Pause|Resume / Discard / frames / Screen.
                        ui.horizontal(|ui| {
                            if !self.recording {
                                if ui.add(theme::primary_button("Record")).clicked() {
                                    match self.start_recording(ctx, frame) {
                                        Ok(()) => self.status.clear(),
                                        Err(e) => {
                                            self.log_e(&format!("Record failed: {e}"));
                                            self.status = e;
                                        }
                                    }
                                }
                                if ui.add(theme::secondary_button("Screen")).clicked() {
                                    self.save_screenshot(ctx, frame);
                                }
                            } else {
                                if ui.add(theme::primary_button("Save")).clicked() {
                                    self.stop_and_save_async(ctx);
                                }
                                if self.paused {
                                    if ui.add(theme::secondary_button("Resume")).clicked() {
                                        self.log_a("Resume");
                                        self.paused = false;
                                    }
                                } else if ui.add(theme::secondary_button("Pause")).clicked() {
                                    self.log_a("Pause");
                                    self.paused = true;
                                }
                                if ui.add(theme::danger_button("Discard")).clicked() {
                                    self.discard_recording(ctx);
                                }
                                let n = self
                                    .session
                                    .as_ref()
                                    .map(|s| s.frame_count())
                                    .unwrap_or(0);
                                ui.separator();
                                ui.label(
                                    egui::RichText::new(format!("frames: {n}"))
                                        .small()
                                        .color(text_main),
                                );
                                ui.separator();
                                if ui.add(theme::secondary_button("Screen")).clicked() {
                                    self.save_screenshot(ctx, frame);
                                }
                            }
                        });

                        // Row 3: одна линия — [статус…][GitHub] (`horizontal` не переносит ряд, в отличие от `horizontal_wrapped`).
                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let gh = ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("GitHub").small(),
                                            )
                                            .small(),
                                        )
                                        .on_hover_text(format!(
                                            "{GITHUB_REPO_URL} — открыть в браузере"
                                        ));
                                    if gh.clicked() {
                                        ctx.open_url(egui::OpenUrl {
                                            url: GITHUB_REPO_URL.to_string(),
                                            new_tab: true,
                                        });
                                    }

                                    ui.add_space(6.0);

                                    let row_h = ui.text_style_height(&egui::TextStyle::Small);
                                    let text_w = ui.available_width().max(0.0);
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(text_w, row_h),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            if self.status.is_empty() {
                                                return;
                                            }
                                            if let Some(path) =
                                                self.status.strip_prefix(STATUS_SAVED_PREFIX)
                                            {
                                                let copied = self
                                                    .copy_feedback_until
                                                    .is_some_and(|t| Instant::now() < t);
                                                ui.label(
                                                    egui::RichText::new("Сохранено:")
                                                        .small()
                                                        .color(text_main),
                                                );
                                                let path_response = ui
                                                    .add(
                                                        egui::Label::new(
                                                            egui::RichText::new(path).small(),
                                                        )
                                                        .sense(egui::Sense::click())
                                                        .truncate(),
                                                    )
                                                    .on_hover_text(
                                                        "Скопировать полный путь в буфер обмена",
                                                    );
                                                if path_response.clicked() {
                                                    ctx.copy_text(path.to_string());
                                                    self.copy_feedback_until = Some(
                                                        Instant::now()
                                                            + Duration::from_secs(2),
                                                    );
                                                }
                                                ui.label(
                                                    egui::RichText::new("· нажмите путь")
                                                        .small()
                                                        .weak(),
                                                );
                                                if copied {
                                                    ui.label(
                                                        egui::RichText::new("— скопировано")
                                                            .small()
                                                            .color(
                                                                egui::Color32::from_rgb(
                                                                    40, 140, 80,
                                                                ),
                                                            ),
                                                    );
                                                }
                                            } else {
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new(&self.status)
                                                            .small()
                                                            .color(
                                                                ctx.style()
                                                                    .visuals
                                                                    .error_fg_color,
                                                            ),
                                                    )
                                                    .truncate(),
                                                );
                                            }
                                        },
                                    );
                                },
                            );
                        });
                    });
                });
            });

        if self.export_busy {
            let busy_label = {
                #[cfg(feature = "slim")]
                {
                    "Saving file…"
                }
                #[cfg(not(feature = "slim"))]
                {
                    match self.export_busy_format {
                        Some(ExportFormat::Webp) => "Converting to WebP…",
                        _ => "Saving file…",
                    }
                }
            };
            egui::CentralPanel::default()
                .frame(
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgba_unmultiplied(245, 250, 252, 242))
                        .inner_margin(0.0)
                        .outer_margin(0.0),
                )
                .show(ctx, |ui| {
                    ui.expand_to_include_rect(ui.max_rect());
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.28);
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(busy_label)
                                .color(text_main),
                        );
                        #[cfg(not(feature = "slim"))]
                        if self.export_busy_format == Some(ExportFormat::Webp) {
                            ui.add_space(16.0);
                            if ui.add(theme::secondary_button("Cancel")).clicked() {
                                if let Some(c) = &self.export_cancel {
                                    c.store(true, Ordering::Relaxed);
                                }
                            }
                        }
                    });
                });
            ctx.request_repaint_after(Duration::from_millis(32));
        }

        let ppp = ctx.pixels_per_point();
        self.toolbar_h_px = (panel.response.rect.height() * ppp).ceil() as i32;

        let th_px = self.toolbar_h_px.max(48);
        let min_h_pt = ((th_px as f32 + MIN_CAPTURE_PX as f32) / ppp).max(MIN_INNER_HEIGHT_PT);
        let min_w_pt = (MIN_INNER_WIDTH_PX / ppp).max(1.0);
        if self.recording {
            // Lock native window size: resizing breaks fixed capture dimensions + session.
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::EnableButtons {
                close: true,
                minimized: true,
                maximize: false,
            });
            if let Some(ir) = ctx.input(|i| i.viewport().inner_rect) {
                let sz = ir.size();
                if sz.x >= 1.0 && sz.y >= 1.0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(sz));
                    ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(sz));
                }
            }
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::EnableButtons {
                close: true,
                minimized: true,
                maximize: true,
            });
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(
                min_w_pt, min_h_pt,
            )));
            if let Some(ms) = ctx.input(|i| i.viewport().monitor_size) {
                if ms.x > 1.0 && ms.y > 1.0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(ms));
                }
            }
        }

        if let Ok(h) = frame.window_handle() {
            let raw = h.as_raw();
            let _ = gifcap_windows::try_style_nonclient_sea_breeze(raw);
            let th = self
                .recording_toolbar_h
                .unwrap_or(self.toolbar_h_px);
            let _ = gifcap_windows::apply_toolbar_window_region(raw, th);
        }

        if self.recording && !self.paused && !self.export_busy {
            self.tick_capture(ctx, frame);
            ctx.request_repaint_after(Duration::from_millis(1));
        }
    }
}

#[cfg(windows)]
impl GifcapApp {
    fn start_recording(&mut self, _ctx: &egui::Context, frame: &eframe::Frame) -> Result<(), String> {
        let raw = frame
            .window_handle()
            .map_err(|e| format!("window handle: {e}"))?
            .as_raw();
        if self.toolbar_h_px <= 0 {
            return Err("toolbar layout not ready; wait one frame".into());
        }
        let th = self.toolbar_h_px;
        self.recording_toolbar_h = Some(th);
        let pr = match gifcap_windows::physical_viewport_rect(raw, th) {
            Ok(p) => p,
            Err(e) => {
                self.recording_toolbar_h = None;
                return Err(e.to_string());
            }
        };
        let cap_w = pr.width & !1;
        let cap_h = pr.height & !1;
        if cap_w < MIN_CAPTURE_PX || cap_h < MIN_CAPTURE_PX {
            self.recording_toolbar_h = None;
            return Err(format!(
                "capture area must be at least {MIN_CAPTURE_PX}×{MIN_CAPTURE_PX} px (resize the window)"
            ));
        }
        let dir = instance_session_dir(&self.session_id.to_string()).map_err(|e| e.to_string())?;
        let enc_q = {
            #[cfg(feature = "slim")]
            {
                self.quality_gif
            }
            #[cfg(not(feature = "slim"))]
            {
                match self.output_format {
                    ExportFormat::Gif => self.quality_gif,
                    ExportFormat::Webp => self.quality_webp,
                    ExportFormat::Mp4 => self.quality_mp4,
                }
            }
        };
        let session = match Session::create_in_dir(
            dir,
            cap_w,
            cap_h,
            f64::from(self.fps),
            self.output_format,
            enc_q,
        ) {
            Ok(s) => s,
            Err(e) => {
                self.recording_toolbar_h = None;
                return Err(e.to_string());
            }
        };
        self.session = Some(session);
        self.recording = true;
        self.paused = false;
        self.last_tick = Instant::now() - Duration::from_secs(60);
        self.log_a(&format!(
            "Record started {}×{} px (viewport {}×{}, even for encoder) @ {:.1} fps, toolbar_h_px={th}, export format {:?}, quality={}%",
            cap_w,
            cap_h,
            pr.width,
            pr.height,
            self.fps,
            self.output_format,
            enc_q
        ));
        Ok(())
    }

    fn save_screenshot(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let Ok(h) = frame.window_handle() else {
            self.status = "window handle unavailable".into();
            return;
        };
        let raw = h.as_raw();
        if self.toolbar_h_px <= 0 {
            self.status = "toolbar layout not ready".into();
            return;
        }
        let th = self.recording_toolbar_h.unwrap_or(self.toolbar_h_px);
        let pr_raw = match gifcap_windows::physical_viewport_rect(raw, th) {
            Ok(p) => p,
            Err(e) => {
                self.status = e.to_string();
                return;
            }
        };
        let cap_w = pr_raw.width & !1;
        let cap_h = pr_raw.height & !1;
        if cap_w < MIN_CAPTURE_PX || cap_h < MIN_CAPTURE_PX {
            self.status = format!(
                "capture area must be at least {MIN_CAPTURE_PX}×{MIN_CAPTURE_PX} px"
            );
            return;
        }
        let pr = PhysicalRect {
            x: pr_raw.x,
            y: pr_raw.y,
            width: cap_w,
            height: cap_h,
        };
        let pixels = match gifcap_windows::capture_bgra(pr) {
            Ok(p) => p,
            Err(e) => {
                self.log_e(&format!("Screenshot capture failed: {e}"));
                self.status = format!("Screenshot failed: {e}");
                return;
            }
        };
        let out_dir = match output_dir() {
            Ok(d) => d,
            Err(e) => {
                self.status = e.to_string();
                return;
            }
        };
        if let Err(e) = ensure_dir(&out_dir) {
            self.status = format!("output dir: {e}");
            return;
        }
        let path = out_dir.join(output_filename("png"));
        if let Err(e) = save_bgra_as_png(&path, cap_w, cap_h, &pixels) {
            self.log_e(&format!("Screenshot save failed: {e}"));
            self.status = e;
            return;
        }
        self.log_a(&format!("Screen saved {}", path.display()));
        self.copy_feedback_until = None;
        self.status = format!("{}{}", STATUS_SAVED_PREFIX, path.display());
        ctx.request_repaint();
    }

    /// On app shutdown: wait for export worker if needed, stop recording, remove instance session dir.
    fn cleanup_on_exit(&mut self) {
        self.log_a("application exiting; cleaning session workspace");

        if let Some(rx) = self.export_rx.take() {
            // Export may be reading `snapshot.dir` (MP4/WebP transcode); wait before removing files.
            let _ = rx.recv();
        }
        self.export_busy = false;
        self.export_busy_format = None;
        self.export_cancel = None;

        if let Some(sess) = self.session.take() {
            let dir = sess.dir.clone();
            drop(sess);
            if let Err(e) = std::fs::remove_dir_all(&dir) {
                self.log_w(&format!(
                    "exit: could not remove session dir {}: {e}",
                    dir.display()
                ));
            }
        }
        self.recording = false;
        self.paused = false;

        if let Ok(dir) = instance_session_dir(&self.session_id.to_string()) {
            if dir.exists() {
                if let Err(e) = std::fs::remove_dir_all(&dir) {
                    self.log_w(&format!(
                        "exit: could not remove instance dir {}: {e}",
                        dir.display()
                    ));
                }
            }
        }
    }

    /// Drop the in-progress capture without exporting; delete the session folder on disk.
    fn discard_recording(&mut self, ctx: &egui::Context) {
        let frames = self
            .session
            .as_ref()
            .map(|s| s.frame_count())
            .unwrap_or(0);
        self.log_a(&format!("Discard recording, frames={frames}"));
        if let Some(sess) = self.session.take() {
            let dir = sess.dir.clone();
            drop(sess);
            if let Err(e) = std::fs::remove_dir_all(&dir) {
                self.log_w(&format!(
                    "could not remove session dir {}: {e}",
                    dir.display()
                ));
            }
        }
        self.recording = false;
        self.paused = false;
        self.recording_toolbar_h = None;
        self.status.clear();
        ctx.request_repaint();
    }

    fn stop_and_save_async(&mut self, ctx: &egui::Context) {
        let frames = self
            .session
            .as_ref()
            .map(|s| s.frame_count())
            .unwrap_or(0);
        self.recording = false;
        self.paused = false;
        self.recording_toolbar_h = None;
        let Some(sess) = self.session.take() else {
            self.log_w("Save with no active session");
            return;
        };
        let snapshot = match sess.finish() {
            Ok(s) => s,
            Err(e) => {
                self.status = e.to_string();
                return;
            }
        };
        self.log_a(&format!(
            "Save pressed, format {:?}, frames={frames}",
            snapshot.format
        ));
        let cancel_flag: Option<Arc<AtomicBool>> = {
            #[cfg(feature = "slim")]
            {
                None
            }
            #[cfg(not(feature = "slim"))]
            {
                match snapshot.format {
                    ExportFormat::Webp => Some(Arc::new(AtomicBool::new(false))),
                    _ => None,
                }
            }
        };
        self.export_cancel = cancel_flag.as_ref().map(Arc::clone);
        let (tx, rx) = mpsc::channel();
        self.export_rx = Some(rx);
        self.export_busy = true;
        self.export_busy_format = Some(snapshot.format);
        self.status.clear();
        let export_log_id = snapshot.log_instance_id.clone();
        std::thread::spawn(move || {
            let cancel = cancel_flag.as_deref();
            let lid = Some(export_log_id.as_str());
            let result = export_session(&snapshot, cancel).map_err(|e| e.to_string());
            match &result {
                Ok(path) => {
                    log_info(
                        lid,
                        &format!(
                            "export OK {:?} → {}",
                            snapshot.format,
                            path.display()
                        ),
                    );
                }
                Err(e) => {
                    if e.contains("cancelled") {
                        log_warn(lid, &format!("export CANCELLED {:?}: {e}", snapshot.format));
                    } else {
                        log_error(lid, &format!("export FAILED {:?}: {e}", snapshot.format));
                    }
                }
            }
            if result.is_ok() {
                let _ = std::fs::remove_dir_all(&snapshot.dir);
            }
            let _ = tx.send(result);
        });
        ctx.request_repaint();
    }

    fn tick_capture(&mut self, _ctx: &egui::Context, frame: &mut eframe::Frame) {
        let interval = Duration::from_secs_f64(1.0 / f64::from(self.fps.clamp(4.0, 60.0)));
        if self.last_tick.elapsed() < interval {
            return;
        }
        let Ok(h) = frame.window_handle() else {
            return;
        };
        let raw = h.as_raw();
        if self.toolbar_h_px <= 0 {
            return;
        }
        let th = self
            .recording_toolbar_h
            .unwrap_or(self.toolbar_h_px);
        let Ok(pr_raw) = gifcap_windows::physical_viewport_rect(raw, th) else {
            return;
        };
        let cap_w = pr_raw.width & !1;
        let cap_h = pr_raw.height & !1;

        let Some(ref mut s) = self.session else {
            return;
        };
        if cap_w != s.width || cap_h != s.height {
            let msg = "Recording: window/viewport size changed; stop and restart recording.";
            if self.status != msg {
                self.log_w(msg);
                self.status = msg.to_string();
            }
            return;
        }

        let pr = PhysicalRect {
            x: pr_raw.x,
            y: pr_raw.y,
            width: cap_w,
            height: cap_h,
        };

        self.last_tick = Instant::now();

        match gifcap_windows::capture_bgra(pr) {
            Ok(pixels) => {
                let n = s.frame_count();
                if let Err(e) = s.push_frame(&pixels) {
                    let msg = format!("Write failed: {e}");
                    self.log_e(&format!(
                        "{msg} (after frame {n}, {} BGRA bytes)",
                        pixels.len()
                    ));
                    self.status = msg;
                }
            }
            Err(e) => {
                let msg = format!("Capture failed: {e}");
                self.log_e(&msg);
                self.status = msg;
            }
        }
    }
}

#[cfg(windows)]
fn save_bgra_as_png(path: &Path, width: u32, height: u32, bgra: &[u8]) -> Result<(), String> {
    let expected = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    if bgra.len() != expected {
        return Err(format!(
            "BGRA size mismatch: got {} expected {}",
            bgra.len(),
            expected
        ));
    }
    let mut rgba = Vec::with_capacity(expected);
    for px in bgra.chunks_exact(4) {
        rgba.push(px[2]);
        rgba.push(px[1]);
        rgba.push(px[0]);
        rgba.push(px[3]);
    }
    let img = RgbaImage::from_raw(width, height, rgba).ok_or_else(|| "PNG buffer".to_string())?;
    img.save(path).map_err(|e| e.to_string())?;
    Ok(())
}

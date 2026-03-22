use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::export_format::ExportFormat;
use crate::ffmpeg;
use crate::file_log;
use crate::paths::ensure_dir;
use crate::CoreError;

const META: &str = "meta.txt";

fn log_id_from_dir(dir: &Path) -> String {
    dir.file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

/// BGRA frames in flight (sync channel). Kept small so RAM stays ~`FRAME_IN_FLIGHT_CAP` frames,
/// not hundreds — the writer returns each `Vec` for reuse (`buffer_pool`).
const FRAME_IN_FLIGHT_CAP: usize = 6;
/// Cap on pooled BGRA buffers (writer may outpace capture briefly).
const POOL_MAX_BUFFERS: usize = FRAME_IN_FLIGHT_CAP + 4;

/// Finished recording: session directory + dimensions + chosen format (writer has joined).
#[derive(Debug, Clone)]
pub struct SessionSnapshot {
    pub dir: PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub format: ExportFormat,
    /// Encoder / export quality (1–100): GIF output scale + scaler; MP4 (CRF/MF/etc.); WebP transcode.
    pub encode_quality: u8,
    /// Same as session folder name (instance UUID); used for log prefix `{id} :: …`.
    pub log_instance_id: String,
}

/// Disk-backed session: FFmpeg muxes BGRA into `recording.gif` (slim) or `recording.gif` / `recording.mp4` (full; WebP mode records MP4 until export).
pub struct Session {
    pub dir: PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    format: ExportFormat,
    encode_quality: u8,
    log_instance_id: String,
    tx: Option<SyncSender<Vec<u8>>>,
    /// Buffers returned by the writer thread (`Vec` capacity reused for the next frame).
    return_rx: Receiver<Vec<u8>>,
    buffer_pool: Vec<Vec<u8>>,
    join: Option<JoinHandle<Result<(), CoreError>>>,
    frames_captured: Arc<AtomicU32>,
    /// Set by the writer thread on error so `push_frame` can report the real cause after the channel disconnects.
    writer_error: Arc<Mutex<Option<String>>>,
}

impl Session {
    /// Creates or replaces `dir` and starts the FFmpeg writer thread (`recording.*` + `meta.txt`).
    pub fn create_in_dir(
        dir: PathBuf,
        width: u32,
        height: u32,
        fps: f64,
        format: ExportFormat,
        encode_quality: u8,
    ) -> Result<Self, CoreError> {
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        ensure_dir(&dir)?;
        #[cfg(feature = "slim")]
        if !matches!(format, ExportFormat::Gif) {
            return Err(CoreError::Gif(
                "slim build supports only GIF recording (MP4/WebP disabled)".into(),
            ));
        }
        let log_instance_id = log_id_from_dir(&dir);
        let frames_captured = Arc::new(AtomicU32::new(0));
        let writer_error = Arc::new(Mutex::new(None));
        let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(FRAME_IN_FLIGHT_CAP);
        let (return_tx, return_rx) = std::sync::mpsc::channel::<Vec<u8>>();
        let err_slot = Arc::clone(&writer_error);
        let out_path = dir.join(format.recording_file_name());
        let wid = log_instance_id.clone();
        let join = std::thread::spawn(move || {
            let lid = Some(wid.as_str());
            let r = ffmpeg::writer_loop(
                rx,
                return_tx,
                out_path,
                width,
                height,
                fps,
                format,
                encode_quality,
            )
                .map_err(|e| {
                    file_log::log_error(
                        lid,
                        &format!("FFmpeg writer stopped with error: {e}"),
                    );
                    e
                });
            if let Err(ref e) = r {
                if let Ok(mut g) = err_slot.lock() {
                    *g = Some(e.to_string());
                }
            } else {
                file_log::log_info(lid, "capture writer: FFmpeg session completed OK");
            }
            r
        });

        let s = Session {
            dir,
            width,
            height,
            fps,
            format,
            encode_quality,
            log_instance_id,
            tx: Some(tx),
            return_rx,
            buffer_pool: Vec::new(),
            join: Some(join),
            frames_captured,
            writer_error,
        };
        s.write_meta()?;
        Ok(s)
    }

    fn write_meta(&self) -> Result<(), CoreError> {
        let path = self.dir.join(META);
        let mut f = fs::File::create(path)?;
        writeln!(f, "{}", self.width)?;
        writeln!(f, "{}", self.height)?;
        writeln!(f, "{}", self.fps)?;
        writeln!(
            f,
            "{}",
            match self.format {
                ExportFormat::Gif => "gif",
                ExportFormat::Webp => "webp",
                ExportFormat::Mp4 => "mp4",
            }
        )?;
        Ok(())
    }

    /// Queue one raw BGRA frame (`width * height * 4` bytes). Reuses a pooled `Vec` when possible.
    pub fn push_frame(&mut self, bgra: &[u8]) -> Result<(), CoreError> {
        let expected = (self.width as usize)
            .checked_mul(self.height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or(CoreError::InvalidMeta)?;
        if bgra.len() != expected {
            let msg = format!(
                "frame size mismatch: got {} expected {}",
                bgra.len(),
                expected
            );
            file_log::log_error(Some(&self.log_instance_id), &msg);
            return Err(CoreError::Gif(msg));
        }
        while let Ok(v) = self.return_rx.try_recv() {
            if self.buffer_pool.len() < POOL_MAX_BUFFERS {
                self.buffer_pool.push(v);
            }
        }
        let mut buf = self.buffer_pool.pop().unwrap_or_else(|| Vec::with_capacity(expected));
        if buf.capacity() < expected {
            buf = Vec::with_capacity(expected);
        }
        buf.clear();
        buf.extend_from_slice(bgra);

        let tx = self
            .tx
            .as_ref()
            .ok_or_else(|| CoreError::Gif("session is not recording".into()))?;
        tx.send(buf).map_err(|_| {
            let detail = self
                .writer_error
                .lock()
                .ok()
                .and_then(|g| g.clone())
                .unwrap_or_else(|| "writer thread ended (see disk space / antivirus)".into());
            let full = format!("frame writer stopped: {detail}");
            file_log::log_error(Some(&self.log_instance_id), &full);
            CoreError::Gif(full)
        })?;
        self.frames_captured.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn frame_count(&self) -> u32 {
        self.frames_captured.load(Ordering::Relaxed)
    }

    /// Stop the writer, flush remaining frames, and return a snapshot for export.
    pub fn finish(mut self) -> Result<SessionSnapshot, CoreError> {
        let dir = self.dir.clone();
        let width = self.width;
        let height = self.height;
        let fps = self.fps;
        let format = self.format;
        let encode_quality = self.encode_quality;
        let log_instance_id = self.log_instance_id.clone();
        drop(
            self.tx
                .take()
                .ok_or_else(|| CoreError::Gif("session already finished".into()))?,
        );
        let join = self
            .join
            .take()
            .ok_or_else(|| CoreError::Gif("session already finished".into()))?;
        match join.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                file_log::log_error(
                    Some(&log_instance_id),
                    &format!("session writer finished with error: {e}"),
                );
                return Err(e);
            }
            Err(_) => {
                file_log::log_error(
                    Some(&log_instance_id),
                    "frame writer thread panicked on join",
                );
                return Err(CoreError::Gif("frame writer thread panicked".into()));
            }
        }
        Ok(SessionSnapshot {
            dir,
            width,
            height,
            fps,
            format,
            encode_quality,
            log_instance_id,
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        drop(self.tx.take());
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

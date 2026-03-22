//! FFmpeg-backed mux: GIF always; MP4 during capture and WebP on save only in full builds.
mod common;
mod gif;
#[cfg(not(feature = "slim"))]
mod mp4;
#[cfg(not(feature = "slim"))]
mod webp;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use crate::export_format::ExportFormat;
use crate::CoreError;

#[cfg(not(feature = "slim"))]
pub(crate) use webp::convert_mp4_to_webp;

pub(crate) fn writer_loop(
    rx: Receiver<Vec<u8>>,
    return_tx: Sender<Vec<u8>>,
    output_path: PathBuf,
    width: u32,
    height: u32,
    fps: f64,
    fmt: ExportFormat,
    quality: u8,
) -> Result<(), CoreError> {
    common::ensure_ffmpeg_init();
    match fmt {
        ExportFormat::Gif => {
            gif::writer_loop(rx, return_tx, output_path, width, height, fps, quality)
        }
        #[cfg(not(feature = "slim"))]
        ExportFormat::Mp4 | ExportFormat::Webp => {
            mp4::writer_loop(rx, return_tx, output_path, width, height, fps, quality)
        }
        #[cfg(feature = "slim")]
        ExportFormat::Mp4 | ExportFormat::Webp => Err(CoreError::Export(
            "MP4/WebP recording is disabled in slim builds (GIF only).".into(),
        )),
    }
}

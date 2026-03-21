//! FFmpeg-backed mux: GIF and MP4 during capture; WebP is produced on save via in-process transcode (see [`webp::convert_mp4_to_webp`]).
mod common;
mod gif;
mod mp4;
mod webp;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use crate::export_format::ExportFormat;
use crate::CoreError;

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
        ExportFormat::Mp4 | ExportFormat::Webp => {
            mp4::writer_loop(rx, return_tx, output_path, width, height, fps, quality)
        }
    }
}

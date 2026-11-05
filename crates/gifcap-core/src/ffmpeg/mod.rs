//! FFmpeg muxes the recording directly to GIF.
mod common;
mod gif;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use crate::export_format::ExportFormat;
use crate::CoreError;

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
    }
}

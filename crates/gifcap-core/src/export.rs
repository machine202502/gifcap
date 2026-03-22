use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

#[cfg(not(feature = "slim"))]
use crate::ffmpeg::convert_mp4_to_webp;
use crate::paths::{ensure_dir, output_dir, output_filename};
use crate::session::SessionSnapshot;
use crate::CoreError;

pub use crate::export_format::ExportFormat;

fn rename_or_copy_recording(src: &std::path::Path, ext: &str) -> Result<PathBuf, CoreError> {
    let out_dir = output_dir()?;
    ensure_dir(&out_dir)?;
    let out_path = out_dir.join(output_filename(ext));
    std::fs::rename(src, &out_path).or_else(|_| {
        std::fs::copy(src, &out_path)?;
        std::fs::remove_file(src)?;
        Ok::<_, std::io::Error>(())
    })?;
    Ok(out_path)
}

/// Moves the finished recording from the active session dir into [`output_dir`](crate::paths::output_dir).
/// WebP runs an async-friendly **MP4 → WebP** transcode via linked FFmpeg (caller should run off the UI thread).
///
/// For WebP only: `cancel` is polled between frames; when `true`, returns [`CoreError::Cancelled`].
pub fn export_session(
    snapshot: &SessionSnapshot,
    cancel: Option<&AtomicBool>,
) -> Result<PathBuf, CoreError> {
    let src = snapshot.dir.join(snapshot.format.recording_file_name());
    if !src.exists() {
        return Err(CoreError::NoFrames);
    }
    let len = std::fs::metadata(&src).map(|m| m.len()).unwrap_or(0);
    if len == 0 {
        return Err(CoreError::NoFrames);
    }

    #[cfg(feature = "slim")]
    {
        match snapshot.format {
            ExportFormat::Gif => rename_or_copy_recording(&src, "gif"),
            ExportFormat::Webp | ExportFormat::Mp4 => Err(CoreError::Export(
                "MP4/WebP export is disabled in slim builds.".into(),
            )),
        }
    }

    #[cfg(not(feature = "slim"))]
    {
        match snapshot.format {
            ExportFormat::Webp => {
                let out_dir = output_dir()?;
                ensure_dir(&out_dir)?;
                let out_path = out_dir.join(output_filename("webp"));
                convert_mp4_to_webp(&src, &out_path, snapshot.encode_quality, cancel)?;
                Ok(out_path)
            }
            ExportFormat::Gif => rename_or_copy_recording(&src, "gif"),
            ExportFormat::Mp4 => rename_or_copy_recording(&src, "mp4"),
        }
    }
}

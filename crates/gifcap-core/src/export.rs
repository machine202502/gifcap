use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use crate::paths::{ensure_dir, output_dir, output_filename};
use crate::session::SessionSnapshot;
use crate::CoreError;

pub use crate::export_format::ExportFormat;

fn rename_or_copy_recording(src: &std::path::Path) -> Result<PathBuf, CoreError> {
    let out_dir = output_dir()?;
    ensure_dir(&out_dir)?;
    let out_path = out_dir.join(output_filename("gif"));
    std::fs::rename(src, &out_path).or_else(|_| {
        std::fs::copy(src, &out_path)?;
        std::fs::remove_file(src)?;
        Ok::<_, std::io::Error>(())
    })?;
    Ok(out_path)
}

/// Moves the finished GIF from the active session directory into the Pictures folder.
pub fn export_session(
    snapshot: &SessionSnapshot,
    _cancel: Option<&AtomicBool>,
) -> Result<PathBuf, CoreError> {
    let src = snapshot.dir.join(snapshot.format.recording_file_name());
    if !src.exists() {
        return Err(CoreError::NoFrames);
    }
    let len = std::fs::metadata(&src).map(|m| m.len()).unwrap_or(0);
    if len == 0 {
        return Err(CoreError::NoFrames);
    }
    rename_or_copy_recording(&src)
}

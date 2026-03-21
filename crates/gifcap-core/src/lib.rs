//! Recording muxed incrementally with FFmpeg (GIF or MP4). WebP mode records to MP4, then runs the
//! `ffmpeg` CLI to build animated WebP on save. Capture feeds BGRA frames into a writer thread;
//! the container grows on disk with bounded memory.

mod error;
mod export;
mod export_format;
mod ffmpeg;
mod file_log;
mod paths;
mod session;

pub use error::CoreError;
pub use export::{export_session, ExportFormat};
pub use file_log::{log_action, log_error, log_info, log_warn};
pub use paths::{
    ensure_dir, gifcap_root, instance_session_dir, logs_dir, output_dir, output_filename,
    timestamp_filename,
};
pub use session::{Session, SessionSnapshot};

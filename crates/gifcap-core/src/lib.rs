//! Recording muxed incrementally with FFmpeg (GIF; full builds also MP4 / WebP via MP4 + transcode).
//! Capture feeds BGRA frames into a writer thread; the container grows on disk with bounded memory.
//! Feature **`slim`**: GIF only; screenshots stay PNG via the `image` crate in the UI crate.

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

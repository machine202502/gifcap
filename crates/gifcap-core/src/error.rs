use std::io;

#[derive(Debug)]
pub enum CoreError {
    Io(io::Error),
    NoHomeDir,
    InvalidMeta,
    NoFrames,
    Gif(String),
    /// FFmpeg mux / encode failures.
    Export(String),
    /// User cancelled a long-running export (e.g. WebP transcode).
    Cancelled,
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::Io(e) => write!(f, "{e}"),
            CoreError::NoHomeDir => write!(f, "could not resolve user home directory"),
            CoreError::InvalidMeta => write!(f, "invalid session metadata"),
            CoreError::NoFrames => write!(f, "no frames captured"),
            CoreError::Gif(s) => write!(f, "{s}"),
            CoreError::Export(s) => write!(f, "{s}"),
            CoreError::Cancelled => write!(f, "WebP conversion cancelled"),
        }
    }
}

impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CoreError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for CoreError {
    fn from(e: io::Error) -> Self {
        CoreError::Io(e)
    }
}

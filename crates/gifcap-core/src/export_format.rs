#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Gif,
    Webp,
    Mp4,
}

impl ExportFormat {
    pub(crate) fn recording_file_name(self) -> &'static str {
        match self {
            ExportFormat::Gif => "recording.gif",
            // WebP is produced on export via `ffmpeg` from this intermediate file.
            ExportFormat::Webp => "recording.mp4",
            ExportFormat::Mp4 => "recording.mp4",
        }
    }

    pub(crate) fn output_extension(self) -> &'static str {
        match self {
            ExportFormat::Gif => "gif",
            ExportFormat::Webp => "webp",
            ExportFormat::Mp4 => "mp4",
        }
    }
}

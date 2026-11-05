#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Gif,
}

impl ExportFormat {
    pub(crate) fn recording_file_name(self) -> &'static str {
        "recording.gif"
    }

    pub(crate) fn output_extension(self) -> &'static str {
        "gif"
    }
}

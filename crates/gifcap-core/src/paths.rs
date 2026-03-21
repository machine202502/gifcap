use std::path::{Path, PathBuf};

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

/// `~/.gifcap` — working area (per-instance session dirs, logs).
pub fn gifcap_root() -> Result<PathBuf, crate::CoreError> {
    let mut p = home_dir().ok_or(crate::CoreError::NoHomeDir)?;
    p.push(".gifcap");
    Ok(p)
}

/// `~/.gifcap/logs` — rotating file log (`gifcap.log`).
pub fn logs_dir() -> Result<PathBuf, crate::CoreError> {
    let mut p = gifcap_root()?;
    p.push("logs");
    Ok(p)
}

/// Per-process working folder: `~/.gifcap/<instance_id>/` (`instance_id` is typically a UUID).
/// Lets multiple gifcap instances record without clashing on a single shared `active` dir.
pub fn instance_session_dir(instance_id: &str) -> Result<PathBuf, crate::CoreError> {
    let mut p = gifcap_root()?;
    p.push(instance_id);
    Ok(p)
}

/// Final GIFs: on Windows `%USERPROFILE%\Pictures\gifcap\`, elsewhere `~/images/gifcap/`.
pub fn output_dir() -> Result<PathBuf, crate::CoreError> {
    let mut p = home_dir().ok_or(crate::CoreError::NoHomeDir)?;
    #[cfg(windows)]
    {
        p.push("Pictures");
    }
    #[cfg(not(windows))]
    {
        p.push("images");
    }
    p.push("gifcap");
    Ok(p)
}

/// Local date-time file name: `dd.mm.yyyy HH-MM-SS.gif` (colons replaced with `-` — invalid on Windows).
pub fn timestamp_filename() -> String {
    output_filename("gif")
}

/// `dd.mm.yyyy HH-MM-SS.<ext>`
pub fn output_filename(ext: &str) -> String {
    format!(
        "{}.{}",
        chrono::Local::now().format("%d.%m.%Y %H-%M-%S"),
        ext.trim_start_matches('.')
    )
}

pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

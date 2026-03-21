//! Append-only log under `~/.gifcap/logs/gifcap.log` with size-based rotation (`gifcap.log.1` …).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Local;

/// Current log file size soft cap; when exceeded, rotate before appending.
const MAX_LOG_BYTES: u64 = 512 * 1024;
/// Keep current `gifcap.log` plus `gifcap.log.1` … up to this suffix (4 rotated backups).
const ROTATE_MAX_SUFFIX: u32 = 4;

static LOG_MUTEX: Mutex<()> = Mutex::new(());

fn one_line(msg: &str) -> String {
    msg.chars()
        .map(|c| match c {
            '\n' | '\r' => ' ',
            _ => c,
        })
        .collect()
}

/// `Some(id)` → `{id} :: message`; `None` → message only.
pub fn prefixed_line(instance_id: Option<&str>, message: &str) -> String {
    let m = one_line(message);
    match instance_id {
        Some(id) if !id.is_empty() => format!("{id} :: {m}"),
        _ => m,
    }
}

fn rotated_path(base: &Path, idx: u32) -> PathBuf {
    if idx == 0 {
        base.to_path_buf()
    } else {
        PathBuf::from(format!("{}.{}", base.display(), idx))
    }
}

fn rotate_if_needed(active: &Path, incoming_len: u64) -> std::io::Result<()> {
    let cur = fs::metadata(active).map(|m| m.len()).unwrap_or(0);
    if cur + incoming_len <= MAX_LOG_BYTES {
        return Ok(());
    }
    let _ = fs::remove_file(rotated_path(active, ROTATE_MAX_SUFFIX));
    for i in (1..ROTATE_MAX_SUFFIX).rev() {
        let from = rotated_path(active, i);
        let to = rotated_path(active, i + 1);
        if from.exists() {
            let _ = fs::rename(&from, &to);
        }
    }
    if active.exists() {
        fs::rename(active, rotated_path(active, 1))?;
    }
    Ok(())
}

fn append_line(level: &str, instance_id: Option<&str>, message: &str) {
    let _guard = LOG_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let Ok(dir) = crate::paths::logs_dir() else {
        return;
    };
    let body = prefixed_line(instance_id, message);
    if let Err(e) = append_line_inner(&dir, level, &body) {
        eprintln!("gifcap: could not write ~/.gifcap/logs: {e}");
    }
}

fn append_line_inner(dir: &Path, level: &str, body: &str) -> std::io::Result<()> {
    crate::paths::ensure_dir(dir)?;
    let path = dir.join("gifcap.log");
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let line = format!("{ts} [{level}] {body}");
    let incoming = line.as_bytes().len() as u64 + 1;
    rotate_if_needed(&path, incoming)?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(f, "{line}")?;
    f.flush()?;
    Ok(())
}

/// User-visible actions (Record, Stop, format change, etc.).
pub fn log_action(instance_id: Option<&str>, message: &str) {
    append_line("ACTION", instance_id, message);
}

/// Failures and detailed diagnostics.
pub fn log_error(instance_id: Option<&str>, message: &str) {
    append_line("ERROR", instance_id, message);
}

/// Non-fatal issues.
pub fn log_warn(instance_id: Option<&str>, message: &str) {
    append_line("WARN", instance_id, message);
}

/// General information (export path, startup).
pub fn log_info(instance_id: Option<&str>, message: &str) {
    append_line("INFO", instance_id, message);
}

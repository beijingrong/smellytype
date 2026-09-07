//! Local microphone time for Doubao dictation, including cancelled sessions.
//! This is not provider billing and never stores speech or transcripts.
use serde::{Deserialize, Serialize};
use std::{
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct Usage {
    version: u32,
    total_ms: u64,
    sessions: u64,
    since: String,
}

fn path() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".local/state"))
        .join("smellytype/dictation-usage.json")
}

fn accumulate(path: &Path, duration: Duration) -> io::Result<()> {
    let mut usage: Usage = match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Usage::default(),
        Err(e) => return Err(e),
    };
    usage.version = 1;
    if usage.since.is_empty() {
        usage.since = chrono::Utc::now().to_rfc3339();
    }
    usage.total_ms = usage
        .total_ms
        .saturating_add(duration.as_millis().min(u64::MAX as u128) as u64);
    usage.sessions = usage.sessions.saturating_add(1);
    std::fs::create_dir_all(path.parent().unwrap())?;
    let temporary = path.with_extension("tmp");
    std::fs::write(
        &temporary,
        serde_json::to_vec(&usage).map_err(io::Error::other)?,
    )?;
    std::fs::rename(temporary, path)
}

pub fn begin() -> Instant {
    let value = serde_json::json!({"started_ms": chrono::Utc::now().timestamp_millis(), "pid": std::process::id()});
    let path = crate::config::Config::runtime_dir().join("dictation-active.json");
    if let Err(e) = std::fs::write(path, value.to_string()) {
        tracing::warn!("Cannot publish local recording timer: {e}");
    }
    Instant::now()
}

pub fn finish(started: &mut Option<Instant>) {
    if let Some(start) = started.take() {
        let _ = std::fs::remove_file(
            crate::config::Config::runtime_dir().join("dictation-active.json"),
        );
        if let Err(e) = accumulate(&path(), start.elapsed()) {
            tracing::warn!("Cannot save local dictation duration: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adds_sessions_and_preserves_start_date() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("usage.json");
        accumulate(&p, Duration::from_millis(1200)).unwrap();
        let first: Usage = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        accumulate(&p, Duration::from_millis(2300)).unwrap();
        let next: Usage = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        assert_eq!((next.sessions, next.total_ms), (2, 3500));
        assert_eq!(first.since, next.since);
    }
    #[test]
    fn corrupt_history_is_not_silently_reset() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("usage.json");
        std::fs::write(&p, b"broken").unwrap();
        assert!(accumulate(&p, Duration::from_secs(2)).is_err());
        assert_eq!(std::fs::read(&p).unwrap(), b"broken");
    }
}

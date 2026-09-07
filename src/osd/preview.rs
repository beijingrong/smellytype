//! Ephemeral cumulative transcript for the Quickshell OSD.
use crate::config::Config;
use std::io::Write;

pub fn publish(text: &str) {
    let dir = Config::runtime_dir();
    let target = dir.join("preview.json");
    let result = (|| -> std::io::Result<()> {
        let mut file = tempfile::NamedTempFile::new_in(&dir)?;
        file.write_all(serde_json::json!({"text": text}).to_string().as_bytes())?;
        file.persist(&target).map_err(|e| e.error)?;
        Ok(())
    })();
    if let Err(err) = result {
        tracing::warn!("Cannot publish transcript preview: {err}");
    }
}

pub fn clear() {
    let _ = std::fs::remove_file(Config::runtime_dir().join("preview.json"));
}

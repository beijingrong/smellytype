//! Ephemeral cumulative transcript for the Quickshell OSD.
use crate::config::Config;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;

pub fn publish(text: &str) {
    let dir = Config::runtime_dir();
    let target = dir.join("preview.json");
    let temp = dir.join("preview.json.tmp");
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&temp)?;
        file.write_all(serde_json::json!({"text": text}).to_string().as_bytes())?;
        std::fs::rename(&temp, &target)
    })();
    if let Err(err) = result {
        tracing::warn!("Cannot publish transcript preview: {err}");
    }
}

pub fn clear() {
    let _ = std::fs::remove_file(Config::runtime_dir().join("preview.json"));
}

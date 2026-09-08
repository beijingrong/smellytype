//! Doubao / Volcengine streaming ASR 2.0 configuration.
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DoubaoConfig {
    /// Prefer DOUBAO_API_KEY over storing credentials in TOML.
    pub api_key: Option<String>,
    /// Streaming 2.0 hourly or concurrent resource identifier.
    pub resource_id: String,
    /// Maximum wait after audio ends, including second-pass recognition.
    pub final_timeout_secs: u64,
    /// One vocabulary hint per line; sent to Doubao with each recording.
    pub hotwords: String,
    /// Remove spoken disfluencies using the provider's semantic smoothing.
    pub enable_ddc: bool,
}

impl Default for DoubaoConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            resource_id: "volc.seedasr.sauc.duration".into(),
            final_timeout_secs: 60,
            hotwords: String::new(),
            enable_ddc: false,
        }
    }
}

impl std::fmt::Debug for DoubaoConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoubaoConfig")
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .field("resource_id", &self.resource_id)
            .field("final_timeout_secs", &self.final_timeout_secs)
            .field("enable_ddc", &self.enable_ddc)
            .field("hotwords", &"[REDACTED]")
            .finish()
    }
}

//! dotool-based text output
//!
//! Uses dotool to simulate keyboard input with keyboard-layout-aware key lookup.
//! Unlike ydotool, direct `dotool` invocations can respect keyboard layouts
//! and variants via XKB environment variables when converting text to key
//! events.
//!
//! SmellyType uses direct `dotool` processes. It deliberately does not discover
//! the shared `/tmp/dotool-pipe`: an unrelated local account can own its reader.
//! Text containing control characters is rejected before any output, allowing
//! the output chain to fall back to a text-safe backend such as the clipboard.
//!
//! Requires dotool and uinput access. Configured XKB layout/variant hints apply
//! to each process; the active compositor layout must match.

use super::TextOutput;
use crate::error::OutputError;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DotoolInvocation {
    binary: &'static str,
    pipe: Option<PathBuf>,
    set_layout_env: bool,
    skipped_daemon_for_layout: bool,
}

/// dotool-based text output with keyboard layout support.
pub struct DotoolOutput {
    /// Delay between keypresses in milliseconds
    type_delay_ms: u32,
    /// Delay before typing starts in milliseconds
    pre_type_delay_ms: u32,
    /// Whether to send Enter key after output
    auto_submit: bool,
    /// Text to append after transcription (before auto_submit)
    append_text: Option<String>,
    /// Keyboard layout (e.g., "de" for German, "fr" for French)
    xkb_layout: Option<String>,
    /// Keyboard layout variant (e.g., "nodeadkeys")
    xkb_variant: Option<String>,
}

impl DotoolOutput {
    /// Create a new dotool output
    pub fn new(
        type_delay_ms: u32,
        pre_type_delay_ms: u32,
        auto_submit: bool,
        append_text: Option<String>,
        xkb_layout: Option<String>,
        xkb_variant: Option<String>,
    ) -> Self {
        if let Some(ref layout) = xkb_layout {
            tracing::debug!("dotool: using keyboard layout '{}'", layout);
        }
        Self {
            type_delay_ms,
            pre_type_delay_ms,
            auto_submit,
            append_text,
            xkb_layout,
            xkb_variant,
        }
    }

    /// Shared FIFO discovery is disabled: no authenticated peer is available.
    /// Streaming backspace callers must also use direct dotool.
    pub fn live_daemon_pipe_path() -> Option<PathBuf> {
        None
    }

    fn daemon_pipe_path() -> Option<PathBuf> {
        None
    }

    fn build_commands(&self, text: &str) -> Result<String, OutputError> {
        // A line-oriented command transport cannot safely represent arbitrary
        // control characters. Fail before spawning or writing any partial text.
        if text.chars().any(char::is_control)
            || self
                .append_text
                .as_deref()
                .is_some_and(|s| s.chars().any(char::is_control))
        {
            return Err(OutputError::InjectionFailed(
                "dotool cannot safely type control characters; use another output backend".into(),
            ));
        }
        let mut commands = String::new();

        // Set delays if configured
        if self.type_delay_ms > 0 {
            commands.push_str(&format!("typedelay {}\n", self.type_delay_ms));
            commands.push_str(&format!("typehold {}\n", self.type_delay_ms));
        }

        // Type the text
        // Note: dotool's type command takes text on the same line
        commands.push_str(&format!("type {}\n", text));

        // Append text if configured (e.g., a space to separate sentences)
        if let Some(ref append) = self.append_text {
            commands.push_str(&format!("type {}\n", append));
        }

        // Send Enter key if auto_submit is enabled
        if self.auto_submit {
            commands.push_str("key enter\n");
        }

        Ok(commands)
    }

    fn has_xkb_override(&self) -> bool {
        self.xkb_layout.is_some() || self.xkb_variant.is_some()
    }

    fn choose_invocation(&self, daemon_pipe: Option<PathBuf>) -> DotoolInvocation {
        if self.has_xkb_override() {
            return DotoolInvocation {
                binary: "dotool",
                pipe: None,
                set_layout_env: true,
                skipped_daemon_for_layout: daemon_pipe.is_some(),
            };
        }

        match daemon_pipe {
            Some(pipe) => DotoolInvocation {
                binary: "dotoolc",
                pipe: Some(pipe),
                set_layout_env: false,
                skipped_daemon_for_layout: false,
            },
            None => DotoolInvocation {
                binary: "dotool",
                pipe: None,
                set_layout_env: true,
                skipped_daemon_for_layout: false,
            },
        }
    }
}

#[async_trait::async_trait]
impl TextOutput for DotoolOutput {
    async fn output(&self, text: &str) -> Result<(), OutputError> {
        if text.is_empty() {
            return Ok(());
        }

        // Pre-typing delay if configured
        if self.pre_type_delay_ms > 0 {
            tracing::debug!(
                "dotool: sleeping {}ms before typing",
                self.pre_type_delay_ms
            );
            tokio::time::sleep(Duration::from_millis(self.pre_type_delay_ms as u64)).await;
        }

        let commands = self.build_commands(text)?;
        let invocation = self.choose_invocation(Self::daemon_pipe_path());
        if invocation.skipped_daemon_for_layout {
            tracing::debug!(
                "dotool: using direct dotool instead of dotoolc so the XKB layout/variant hint is honored"
            );
        }
        let mut cmd = Command::new(invocation.binary);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        if let Some(ref pipe) = invocation.pipe {
            cmd.env("DOTOOL_PIPE", pipe);
        }
        if invocation.set_layout_env {
            if let Some(ref layout) = self.xkb_layout {
                cmd.env("DOTOOL_XKB_LAYOUT", layout);
                cmd.env("XKB_DEFAULT_LAYOUT", layout);
            }
            if let Some(ref variant) = self.xkb_variant {
                cmd.env("DOTOOL_XKB_VARIANT", variant);
                cmd.env("XKB_DEFAULT_VARIANT", variant);
            }
        }

        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                OutputError::DotoolNotFound
            } else {
                OutputError::InjectionFailed(format!(
                    "Failed to spawn {}: {}",
                    invocation.binary, e
                ))
            }
        })?;

        // Write commands to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(commands.as_bytes()).await.map_err(|e| {
                OutputError::InjectionFailed(format!(
                    "Failed to write to {} stdin: {}",
                    invocation.binary, e
                ))
            })?;
            // Close stdin to signal end of input
            drop(stdin);
        }

        // Wait for dotool to complete
        let output = child.wait_with_output().await.map_err(|e| {
            OutputError::InjectionFailed(format!("Failed to wait for {}: {}", invocation.binary, e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Check for common errors
            if stderr.contains("uinput") || stderr.contains("permission") {
                return Err(OutputError::InjectionFailed(format!(
                    "{}: uinput permission denied. Is user in 'input' group?",
                    invocation.binary
                )));
            }
            return Err(OutputError::InjectionFailed(format!(
                "{} exited with error: {}",
                invocation.binary, stderr
            )));
        }

        tracing::info!(
            "Text typed via {} ({} chars)",
            invocation.binary,
            text.chars().count()
        );
        Ok(())
    }

    async fn is_available(&self) -> bool {
        // Check if dotool exists in PATH
        Command::new("which")
            .arg("dotool")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn name(&self) -> &'static str {
        "dotool"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_protocol_injection_before_output() {
        let output = DotoolOutput::new(0, 0, false, None, None, None);
        for text in [
            "hello\nkey enter",
            "hello\rkey ctrl+a",
            "hello\0world",
            "hello\x1bworld",
            "hello\tworld",
        ] {
            assert!(output.build_commands(text).is_err());
        }
        let append = DotoolOutput::new(0, 0, false, Some("\nkey enter".into()), None, None);
        assert!(append.build_commands("你好").is_err());
        assert_eq!(
            output.build_commands("你好 key enter").unwrap(),
            "type 你好 key enter\n"
        );
    }

    #[test]
    fn shared_daemon_discovery_is_disabled_for_all_output_paths() {
        assert!(DotoolOutput::daemon_pipe_path().is_none());
        assert!(DotoolOutput::live_daemon_pipe_path().is_none());
    }

    #[test]
    fn test_new() {
        let output = DotoolOutput::new(10, 0, false, None, Some("de".to_string()), None);
        assert_eq!(output.type_delay_ms, 10);
        assert_eq!(output.pre_type_delay_ms, 0);
        assert!(!output.auto_submit);
        assert_eq!(output.xkb_layout, Some("de".to_string()));
    }

    #[test]
    fn build_commands_basic() {
        let output = DotoolOutput::new(0, 0, false, None, None, None);
        let cmds = output.build_commands("Hello world").unwrap();
        assert_eq!(cmds, "type Hello world\n");
    }

    #[test]
    fn build_commands_with_delay() {
        let output = DotoolOutput::new(17, 0, false, None, None, None);
        let cmds = output.build_commands("Test").unwrap();
        assert!(cmds.contains("typedelay 17"));
        assert!(cmds.contains("typehold 17"));
        assert!(cmds.contains("type Test"));
    }

    #[test]
    fn build_commands_auto_submit_appends_enter() {
        let output = DotoolOutput::new(0, 0, true, None, None, None);
        let cmds = output.build_commands("hi").unwrap();
        assert!(cmds.contains("key enter"));
    }

    #[test]
    fn build_commands_appends_text_before_enter() {
        let output = DotoolOutput::new(0, 0, true, Some(".".to_string()), None, None);
        let cmds = output.build_commands("hi").unwrap();
        let dot_pos = cmds.find("type .\n").unwrap();
        let enter_pos = cmds.find("key enter\n").unwrap();
        assert!(dot_pos < enter_pos);
    }

    #[test]
    fn choose_invocation_uses_dotoolc_when_daemon_available_without_xkb_override() {
        let output = DotoolOutput::new(0, 0, false, None, None, None);
        let invocation = output.choose_invocation(Some(PathBuf::from("/tmp/dotool-pipe")));

        assert_eq!(invocation.binary, "dotoolc");
        assert_eq!(invocation.pipe, Some(PathBuf::from("/tmp/dotool-pipe")));
        assert!(!invocation.set_layout_env);
        assert!(!invocation.skipped_daemon_for_layout);
    }

    #[test]
    fn choose_invocation_bypasses_daemon_when_layout_override_is_set() {
        let output = DotoolOutput::new(0, 0, false, None, Some("ru".to_string()), None);
        let invocation = output.choose_invocation(Some(PathBuf::from("/tmp/dotool-pipe")));

        assert_eq!(invocation.binary, "dotool");
        assert_eq!(invocation.pipe, None);
        assert!(invocation.set_layout_env);
        assert!(invocation.skipped_daemon_for_layout);
    }

    #[test]
    fn choose_invocation_bypasses_daemon_when_variant_override_is_set() {
        let output = DotoolOutput::new(0, 0, false, None, None, Some("phonetic".to_string()));
        let invocation = output.choose_invocation(Some(PathBuf::from("/tmp/dotool-pipe")));

        assert_eq!(invocation.binary, "dotool");
        assert_eq!(invocation.pipe, None);
        assert!(invocation.set_layout_env);
        assert!(invocation.skipped_daemon_for_layout);
    }

    /// Serialize tests that mutate `DOTOOL_PIPE` — Rust's default
    /// parallel test runner would otherwise see one test's env change
    /// from another. RAII guard restores the prior value on drop so a
    /// panicking test doesn't pollute the rest of the run.
    static DOTOOL_PIPE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct DotoolPipeEnvGuard {
        prior: Option<String>,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl DotoolPipeEnvGuard {
        fn set(value: &str) -> Self {
            // Allow re-entry on a poisoned lock; one panicking test
            // shouldn't break every subsequent test in the same file.
            let lock = DOTOOL_PIPE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let prior = std::env::var("DOTOOL_PIPE").ok();
            std::env::set_var("DOTOOL_PIPE", value);
            Self { prior, _lock: lock }
        }
    }

    impl Drop for DotoolPipeEnvGuard {
        fn drop(&mut self) {
            match self.prior.take() {
                Some(v) => std::env::set_var("DOTOOL_PIPE", v),
                None => std::env::remove_var("DOTOOL_PIPE"),
            }
        }
    }

    #[test]
    fn daemon_pipe_detection_respects_env_var() {
        let _guard = DotoolPipeEnvGuard::set("/nonexistent/dotool-pipe-test");
        assert!(DotoolOutput::daemon_pipe_path().is_none());
    }
}

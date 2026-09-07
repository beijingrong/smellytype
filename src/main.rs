//! SmellyType - Push-to-talk voice-to-text for Linux
//!
//! Run with `smellytype` or `smellytype daemon` to start the daemon.
//! Use `smellytype setup` to check dependencies and download models.
//! Use `smellytype transcribe <file>` to transcribe an audio file.
//!
//! The binary entry point is intentionally thin: install the SIGILL handler
//! before any other code, reset SIGPIPE, parse CLI, set up logging, load
//! config, then hand off to `app::run`. Every long handler (status, meeting,
//! record, …) lives under `src/app/`.

mod app;

use app::sigpipe;
use clap::Parser;
use smellytype::{config, cpu, Cli, Commands};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install SIGILL handler early to catch illegal instruction crashes
    // and provide a helpful error message instead of core dumping
    cpu::install_sigill_handler();

    // Reset SIGPIPE to default behavior (terminate silently) to avoid panics
    // when output is piped through commands like `head` that close the pipe early
    sigpipe::reset_sigpipe();

    let mut cli = Cli::parse();
    if matches!(cli.command, Some(Commands::Configure { .. })) {
        let status = std::process::Command::new("omarchy-shell")
            .args(["beijingrong.smellytype", "open"])
            .status()?;
        anyhow::ensure!(
            status.success(),
            "Cannot open SmellyType settings; enable the Omarchy plugin"
        );
        return Ok(());
    }
    match &mut cli.command {
        Some(
            Commands::Setup { .. }
            | Commands::Meeting { .. }
            | Commands::CheckUpdate
            | Commands::TranscribeWorker { .. },
        ) => {
            anyhow::bail!("This command is not part of SmellyType. See README.md for cloud installation and configuration.");
        }
        Some(Commands::Transcribe { engine, .. }) => *engine = Some("doubao".to_string()),
        _ => {}
    }
    // Cloud-first product policy; inherited local-model code is retained only
    // during the initial extraction and is not an installation prerequisite.
    cli.engine = Some("doubao".to_string());

    // Check if this is the worker command (needs stderr-only logging)
    let is_worker = matches!(cli.command, Some(Commands::TranscribeWorker { .. }));

    // Initialize logging
    let log_level = if cli.quiet {
        "error"
    } else {
        match cli.verbose {
            0 => "info",
            1 => "debug",
            _ => "trace",
        }
    };

    if is_worker {
        // Worker uses stderr for logging (stdout is reserved for IPC protocol)
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new(format!("smellytype={},warn", log_level))),
            )
            .with_target(false)
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new(format!("smellytype={},warn", log_level))),
            )
            .with_target(false)
            .init();
    }

    // Load configuration. config_path tracks the file we actually loaded (or
    // would load), so subprocess transcribers can reuse the same source.
    let config_path = cli
        .config
        .clone()
        .or_else(config::Config::resolve_existing_path)
        .or_else(config::Config::default_path);
    let config = config::load_config(cli.config.as_deref())?;

    app::run(cli, config_path, config).await
}

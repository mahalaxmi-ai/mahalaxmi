// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Logging setup
//
// Demonstrates how to initialize the tracing subscriber using Mahalaxmi's
// logging configuration, then emit structured log events at different levels.
// The log level is read from the general config, falling back to "info" if unset.

use mahalaxmi_core::{
    config::loader::{default_config_path, load_config},
    i18n::{locale::SupportedLocale, I18nService},
    logging,
};
use tracing::{debug, error, info, warn};

fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);

    let config = load_config(Some(&default_config_path()), &i18n).unwrap_or_default();

    // Initialize the global tracing subscriber from the logging config.
    // After this call, all `tracing::info!` / `debug!` / `warn!` / `error!`
    // macros emit structured log events.
    // The returned guard must be kept alive for the duration of the program.
    let _guard = logging::init_logging(&config.general, &i18n)
        .expect("logging init failed — check that the log level string in config is valid");

    info!(version = "100.0.0", "Mahalaxmi started");

    debug!(
        log_level = %config.general.log_level,
        max_workers = config.orchestration.max_concurrent_workers,
        "Configuration loaded"
    );

    warn!("This is a warning — would appear at warn level and above");

    // Structured fields are emitted as key=value pairs in JSON mode.
    info!(
        provider = "claude-code",
        task_id = "task-001",
        "Worker task dispatched"
    );

    error!("This is an error — check stderr in production");

    info!("Logging example complete");
}

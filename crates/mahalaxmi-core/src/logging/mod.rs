//! Structured logging infrastructure for Mahalaxmi.
//!
//! Provides dual-output logging: human-readable console output for development
//! and JSON-formatted file output for production analysis and debugging.
//!
//! # Usage
//!
//! ```
//! let dir = tempfile::tempdir().unwrap();
//! let config = mahalaxmi_core::config::GeneralConfig {
//!     data_directory: dir.path().to_path_buf(),
//!     ..Default::default()
//! };
//! let i18n = mahalaxmi_core::i18n::I18nService::new(
//!     mahalaxmi_core::i18n::SupportedLocale::EnUs,
//! );
//! let _guard = mahalaxmi_core::logging::init_logging(&config, &i18n).unwrap();
//! ```

pub mod spans;

use crate::config::GeneralConfig;
use crate::error::MahalaxmiError;
use crate::i18n::messages::keys;
use crate::i18n::I18nService;
use std::fs;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

/// Application version from Cargo.toml, embedded at compile time.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Guard that must be held for the lifetime of the application.
///
/// When dropped, flushes any buffered log entries to the file output.
/// Store this in your `main()` function or equivalent top-level scope.
/// If file logging was unavailable, only console logging was active.
pub struct LoggingGuard {
    _file_guard: Option<WorkerGuard>,
}

/// Initialize the logging system.
///
/// Sets up two output layers:
/// 1. **Console**: Human-readable, colored output to stdout.
/// 2. **File**: JSON-formatted structured logs written to disk with daily rotation.
///
/// Log file location: `{data_directory}/logs/mahalaxmi.YYYY-MM-DD`
/// Rotation: Daily, keeps 7 days of history.
///
/// If the `RUST_LOG` environment variable is set, it takes precedence over the
/// configured `log_level` and a translated message is logged noting the override.
///
/// If the log directory cannot be created, the system falls back to console-only
/// logging and logs a translated warning.
///
/// All structured log output includes `app_version`, `locale`, and `pid` fields.
///
/// # Returns
///
/// A [`LoggingGuard`] that must be held for the lifetime of the application.
/// Dropping the guard flushes buffered log entries to the file output.
///
/// # Errors
///
/// Returns `MahalaxmiError::Config` if the log level string is invalid or if the
/// global subscriber has already been set.
pub fn init_logging(
    config: &GeneralConfig,
    i18n: &I18nService,
) -> Result<LoggingGuard, MahalaxmiError> {
    let log_dir = config.data_directory.join("logs");
    let rust_log_active = std::env::var("RUST_LOG").is_ok();
    let pid = std::process::id();

    let filter = match EnvFilter::try_from_default_env() {
        Ok(env_filter) => env_filter,
        Err(_) => EnvFilter::try_new(&config.log_level).map_err(|e| {
            MahalaxmiError::config(
                i18n,
                keys::error::LOG_INIT_FAILED,
                &[("reason", &e.to_string())],
            )
        })?,
    };

    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_thread_names(true)
        .with_target(true)
        .with_level(true);

    let mut fallback_path: Option<String> = None;
    let (non_blocking, file_guard) = match prepare_file_writer(&log_dir) {
        Ok((nb, guard)) => (Some(nb), Some(guard)),
        Err(path) => {
            fallback_path = Some(path);
            (None, None)
        }
    };

    let file_layer = non_blocking.map(|writer| {
        tracing_subscriber::fmt::layer()
            .json()
            .with_writer(writer)
            .with_thread_names(true)
            .with_target(true)
            .with_level(true)
    });

    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber).map_err(|e| {
        MahalaxmiError::config(
            i18n,
            keys::error::LOG_INIT_FAILED,
            &[("reason", &e.to_string())],
        )
    })?;

    if let Some(ref path) = fallback_path {
        tracing::warn!(
            app_version = APP_VERSION,
            locale = %config.locale,
            pid = pid,
            "{}",
            i18n.translate(
                keys::logging::DIR_CREATE_FAILED_FALLBACK,
                &[("path", path.as_str())],
            )
        );
    }

    if rust_log_active {
        tracing::info!(
            app_version = APP_VERSION,
            locale = %config.locale,
            pid = pid,
            "{}",
            i18n.t(keys::logging::RUST_LOG_OVERRIDE)
        );
    }

    tracing::info!(
        app_version = APP_VERSION,
        locale = %config.locale,
        pid = pid,
        "{}",
        i18n.translate(keys::logging::INITIALIZED, &[("level", &config.log_level)])
    );

    tracing::info!(
        app_version = APP_VERSION,
        locale = %config.locale,
        pid = pid,
        "{}",
        i18n.translate(
            keys::logging::FILE_PATH,
            &[("path", &log_dir.display().to_string())],
        )
    );

    Ok(LoggingGuard {
        _file_guard: file_guard,
    })
}

/// Prepare the non-blocking file writer and guard for log file output.
///
/// Creates the log directory if needed. Returns the non-blocking writer and
/// its guard on success, or the directory path string on failure.
fn prepare_file_writer(
    log_dir: &std::path::Path,
) -> Result<(tracing_appender::non_blocking::NonBlocking, WorkerGuard), String> {
    if !log_dir.exists() {
        fs::create_dir_all(log_dir).map_err(|_| log_dir.display().to_string())?;
    }

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("mahalaxmi")
        .max_log_files(7)
        .build(log_dir)
        .map_err(|_| log_dir.display().to_string())?;

    Ok(tracing_appender::non_blocking(file_appender))
}

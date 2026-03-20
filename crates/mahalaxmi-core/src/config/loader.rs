//! Configuration loader with TOML parsing, environment variable overrides, and validation.
//!
//! All log messages and validation errors go through i18n. No hardcoded English strings.

use crate::config::{MahalaxmiConfig, RawMahalaxmiConfig};
use crate::error::MahalaxmiError;
use crate::i18n::locale::SupportedLocale;
use crate::i18n::messages::keys;
use crate::i18n::I18nService;
use crate::MahalaxmiResult;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Valid log levels for configuration validation.
const VALID_LOG_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error"];

/// Valid consensus strategies for configuration validation.
const VALID_CONSENSUS_STRATEGIES: &[&str] = &[
    "union",
    "intersection",
    "weighted_voting",
    "complexity_weighted",
];

/// Valid UI themes for configuration validation.
const VALID_THEMES: &[&str] = &["dark", "light", "system"];

/// Minimum allowed value for `max_concurrent_workers`.
const MIN_WORKERS: u32 = 1;

/// Maximum allowed value for `max_concurrent_workers`.
const MAX_WORKERS: u32 = 100;

/// Minimum allowed value for timeout fields in seconds.
const MIN_TIMEOUT_SECONDS: u64 = 10;

/// Minimum allowed value for `offline_grace_days`.
const MIN_OFFLINE_GRACE_DAYS: u32 = 1;

/// Minimum allowed value for `terminal_font_size`.
const MIN_FONT_SIZE: u32 = 8;

/// Maximum allowed value for `terminal_font_size`.
const MAX_FONT_SIZE: u32 = 72;

/// Returns the default config file path: `~/.mahalaxmi/config.toml`.
pub fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".mahalaxmi")
        .join("config.toml")
}

/// Load configuration from the given path, or the default path.
///
/// Behavior:
/// 1. If `path` is `Some`, load from that path. If file doesn't exist, return defaults.
/// 2. If `path` is `None`, try `default_config_path()`. If file doesn't exist, return defaults.
/// 3. If file exists but fails to parse, return a `MahalaxmiError::Config`.
/// 4. After loading, apply environment variable overrides.
/// 5. Log translated messages for each action (loaded, using defaults, env override).
pub fn load_config(
    path: Option<&Path>,
    i18n: &I18nService,
) -> Result<MahalaxmiConfig, MahalaxmiError> {
    let config_path = path.map(PathBuf::from).unwrap_or_else(default_config_path);

    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let raw: RawMahalaxmiConfig = toml::from_str(&content).map_err(|e| {
            MahalaxmiError::config(
                i18n,
                keys::error::CONFIG_PARSE_FAILED,
                &[("reason", &e.to_string())],
            )
        })?;

        let path_display = config_path.display().to_string();
        info!(
            "{}",
            i18n.translate(keys::config::LOADED, &[("path", &path_display)])
        );

        MahalaxmiConfig::from_raw(raw, None)
    } else {
        info!("{}", i18n.t(keys::config::USING_DEFAULTS));
        MahalaxmiConfig::default()
    };

    apply_env_overrides(&mut config, i18n);

    Ok(config)
}

/// Generates a default configuration file at the specified path.
///
/// Creates parent directories if they don't exist.
/// Returns an error if the file already exists (to prevent accidental overwrites).
pub fn generate_default_config(path: &Path, i18n: &I18nService) -> MahalaxmiResult<()> {
    if path.exists() {
        return Err(MahalaxmiError::config(
            i18n,
            keys::config::ALREADY_EXISTS,
            &[("path", &path.display().to_string())],
        ));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let config = MahalaxmiConfig::default();
    let toml_content = toml::to_string_pretty(&config).map_err(|e| {
        MahalaxmiError::config(
            i18n,
            keys::error::CONFIG_PARSE_FAILED,
            &[("reason", &e.to_string())],
        )
    })?;

    std::fs::write(path, toml_content)?;

    let path_display = path.display().to_string();
    info!(
        "{}",
        i18n.translate(
            keys::config::GENERATED_SUCCESSFULLY,
            &[("path", &path_display)]
        )
    );

    Ok(())
}

/// Apply environment variable overrides to a loaded config.
///
/// Supported environment variables:
/// - `MAHALAXMI_LOCALE` -> `general.locale`
/// - `MAHALAXMI_LOG_LEVEL` -> `general.log_level`
/// - `MAHALAXMI_DATA_DIR` -> `general.data_directory`
/// - `MAHALAXMI_MAX_WORKERS` -> `orchestration.max_concurrent_workers`
/// - `MAHALAXMI_CONSENSUS_STRATEGY` -> `orchestration.consensus_strategy`
/// - `MAHALAXMI_MANAGER_TIMEOUT` -> `orchestration.manager_timeout_seconds`
/// - `MAHALAXMI_WORKER_TIMEOUT` -> `orchestration.worker_timeout_seconds`
/// - `MAHALAXMI_DEFAULT_PROVIDER` -> `providers.default_provider`
/// - `MAHALAXMI_THEME` -> `ui.theme`
fn apply_env_overrides(config: &mut MahalaxmiConfig, i18n: &I18nService) {
    if let Ok(val) = std::env::var("MAHALAXMI_LOCALE") {
        if let Some(locale) = SupportedLocale::from_code(&val) {
            config.general.locale = locale.to_string();
            info!(
                "{}",
                i18n.translate(keys::config::ENV_OVERRIDE, &[("var", "MAHALAXMI_LOCALE")])
            );
        }
    }

    if let Ok(val) = std::env::var("MAHALAXMI_LOG_LEVEL") {
        config.general.log_level = val;
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_LOG_LEVEL")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_DATA_DIR") {
        config.general.data_directory = PathBuf::from(val);
        info!(
            "{}",
            i18n.translate(keys::config::ENV_OVERRIDE, &[("var", "MAHALAXMI_DATA_DIR")])
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_MAX_WORKERS") {
        match val.parse::<u32>() {
            Ok(n) => {
                config.orchestration.max_concurrent_workers = n;
                info!(
                    "{}",
                    i18n.translate(
                        keys::config::ENV_OVERRIDE,
                        &[("var", "MAHALAXMI_MAX_WORKERS")]
                    )
                );
            }
            Err(_) => {
                warn!(
                    "{}",
                    i18n.translate(
                        keys::config::ENV_OVERRIDE_INVALID,
                        &[("var", "MAHALAXMI_MAX_WORKERS"), ("value", &val)]
                    )
                );
            }
        }
    }

    if let Ok(val) = std::env::var("MAHALAXMI_CONSENSUS_STRATEGY") {
        config.orchestration.consensus_strategy = val;
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_CONSENSUS_STRATEGY")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_MANAGER_TIMEOUT") {
        match val.parse::<u64>() {
            Ok(n) => {
                config.orchestration.manager_timeout_seconds = n;
                info!(
                    "{}",
                    i18n.translate(
                        keys::config::ENV_OVERRIDE,
                        &[("var", "MAHALAXMI_MANAGER_TIMEOUT")]
                    )
                );
            }
            Err(_) => {
                warn!(
                    "{}",
                    i18n.translate(
                        keys::config::ENV_OVERRIDE_INVALID,
                        &[("var", "MAHALAXMI_MANAGER_TIMEOUT"), ("value", &val)]
                    )
                );
            }
        }
    }

    if let Ok(val) = std::env::var("MAHALAXMI_WORKER_TIMEOUT") {
        match val.parse::<u64>() {
            Ok(n) => {
                config.orchestration.worker_timeout_seconds = n;
                info!(
                    "{}",
                    i18n.translate(
                        keys::config::ENV_OVERRIDE,
                        &[("var", "MAHALAXMI_WORKER_TIMEOUT")]
                    )
                );
            }
            Err(_) => {
                warn!(
                    "{}",
                    i18n.translate(
                        keys::config::ENV_OVERRIDE_INVALID,
                        &[("var", "MAHALAXMI_WORKER_TIMEOUT"), ("value", &val)]
                    )
                );
            }
        }
    }

    if let Ok(val) = std::env::var("MAHALAXMI_DEFAULT_PROVIDER") {
        config.providers.default_provider = val;
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_DEFAULT_PROVIDER")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_THEME") {
        config.ui.theme = val;
        info!(
            "{}",
            i18n.translate(keys::config::ENV_OVERRIDE, &[("var", "MAHALAXMI_THEME")])
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_TERMINAL_VERBOSE_LOGGING") {
        config.general.terminal_verbose_logging = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_TERMINAL_VERBOSE_LOGGING")]
            )
        );
    }

    // Per-area logging flag overrides
    if let Ok(val) = std::env::var("MAHALAXMI_LOG_SESSION") {
        config.logging.session = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_LOG_SESSION")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_LOG_ORCHESTRATION") {
        config.logging.orchestration_phases = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_LOG_ORCHESTRATION")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_LOG_MANAGER") {
        config.logging.manager = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_LOG_MANAGER")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_LOG_WORKER") {
        config.logging.worker = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_LOG_WORKER")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_LOG_CONSENSUS") {
        config.logging.consensus = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_LOG_CONSENSUS")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_LOG_PROVIDER_ROUTING") {
        config.logging.provider_routing = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_LOG_PROVIDER_ROUTING")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_LOG_PTY") {
        config.logging.pty = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(keys::config::ENV_OVERRIDE, &[("var", "MAHALAXMI_LOG_PTY")])
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_LOG_DETECTION") {
        config.logging.detection = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(
                keys::config::ENV_OVERRIDE,
                &[("var", "MAHALAXMI_LOG_DETECTION")]
            )
        );
    }

    if let Ok(val) = std::env::var("MAHALAXMI_LOG_GIT") {
        config.logging.git = val == "true" || val == "1";
        info!(
            "{}",
            i18n.translate(keys::config::ENV_OVERRIDE, &[("var", "MAHALAXMI_LOG_GIT")])
        );
    }
}

/// Validate the configuration, returning a list of translated error messages.
///
/// Validation rules:
/// - `general.log_level` must be one of: trace, debug, info, warn, error
/// - `general.data_directory` must not contain null bytes
/// - `orchestration.max_concurrent_workers` must be 0 (auto-scale) or 1..=100
/// - `orchestration.consensus_strategy` must be a valid strategy
/// - `orchestration.manager_timeout_seconds` must be >= 10
/// - `orchestration.worker_timeout_seconds` must be >= 10
/// - `providers.default_provider` must not be empty
/// - `ui.theme` must be one of: dark, light, system
/// - `ui.terminal_font_size` must be 8..=72
/// - `licensing.offline_grace_days` must be >= 1
pub fn validate_config(config: &MahalaxmiConfig, i18n: &I18nService) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if !VALID_LOG_LEVELS.contains(&config.general.log_level.as_str()) {
        errors.push(i18n.translate(
            keys::validation::INVALID_LOG_LEVEL,
            &[
                ("level", &config.general.log_level),
                ("valid", &VALID_LOG_LEVELS.join(", ")),
            ],
        ));
    }

    if config
        .general
        .data_directory
        .to_string_lossy()
        .contains('\0')
    {
        errors.push(i18n.t(keys::validation::INVALID_DATA_DIRECTORY));
    }

    // 0 is a valid sentinel meaning "auto-scale to task count after consensus" (P1).
    if config.orchestration.max_concurrent_workers != 0
        && (config.orchestration.max_concurrent_workers < MIN_WORKERS
            || config.orchestration.max_concurrent_workers > MAX_WORKERS)
    {
        errors.push(i18n.translate(
            keys::validation::WORKERS_OUT_OF_RANGE,
            &[
                (
                    "value",
                    &config.orchestration.max_concurrent_workers.to_string(),
                ),
                ("min", &MIN_WORKERS.to_string()),
                ("max", &MAX_WORKERS.to_string()),
            ],
        ));
    }

    if !VALID_CONSENSUS_STRATEGIES.contains(&config.orchestration.consensus_strategy.as_str()) {
        errors.push(i18n.translate(
            keys::validation::INVALID_CONSENSUS_STRATEGY,
            &[
                ("value", &config.orchestration.consensus_strategy),
                ("valid", &VALID_CONSENSUS_STRATEGIES.join(", ")),
            ],
        ));
    }

    if config.orchestration.manager_timeout_seconds < MIN_TIMEOUT_SECONDS {
        errors.push(i18n.translate(
            keys::validation::MANAGER_TIMEOUT_TOO_LOW,
            &[
                (
                    "value",
                    &config.orchestration.manager_timeout_seconds.to_string(),
                ),
                ("min", &MIN_TIMEOUT_SECONDS.to_string()),
            ],
        ));
    }

    if config.orchestration.worker_timeout_seconds < MIN_TIMEOUT_SECONDS {
        errors.push(i18n.translate(
            keys::validation::WORKER_TIMEOUT_TOO_LOW,
            &[
                (
                    "value",
                    &config.orchestration.worker_timeout_seconds.to_string(),
                ),
                ("min", &MIN_TIMEOUT_SECONDS.to_string()),
            ],
        ));
    }

    if config.providers.default_provider.is_empty() {
        errors.push(i18n.t(keys::validation::EMPTY_DEFAULT_PROVIDER));
    }

    if !VALID_THEMES.contains(&config.ui.theme.as_str()) {
        errors.push(i18n.translate(
            keys::validation::INVALID_THEME,
            &[
                ("value", &config.ui.theme),
                ("valid", &VALID_THEMES.join(", ")),
            ],
        ));
    }

    if config.ui.terminal_font_size < MIN_FONT_SIZE || config.ui.terminal_font_size > MAX_FONT_SIZE
    {
        errors.push(i18n.translate(
            keys::validation::FONT_SIZE_OUT_OF_RANGE,
            &[
                ("value", &config.ui.terminal_font_size.to_string()),
                ("min", &MIN_FONT_SIZE.to_string()),
                ("max", &MAX_FONT_SIZE.to_string()),
            ],
        ));
    }

    if config.licensing.offline_grace_days < MIN_OFFLINE_GRACE_DAYS {
        errors.push(i18n.translate(
            keys::validation::OFFLINE_GRACE_TOO_LOW,
            &[
                ("value", &config.licensing.offline_grace_days.to_string()),
                ("min", &MIN_OFFLINE_GRACE_DAYS.to_string()),
            ],
        ));
    }

    if config.orchestration.max_batch_retries == 0 {
        errors.push(i18n.translate(
            keys::validation::INVALID_MAX_BATCH_RETRIES,
            &[("value", &config.orchestration.max_batch_retries.to_string())],
        ));
    }

    if config.orchestration.max_total_batches < 2 {
        errors.push(i18n.translate(
            keys::validation::INVALID_MAX_TOTAL_BATCHES,
            &[("value", &config.orchestration.max_total_batches.to_string())],
        ));
    }

    if config.orchestration.stall_detection_threshold < 2 {
        errors.push(i18n.translate(
            keys::validation::INVALID_STALL_DETECTION_THRESHOLD,
            &[(
                "value",
                &config.orchestration.stall_detection_threshold.to_string(),
            )],
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

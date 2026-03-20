use mahalaxmi_core::config::loader::{
    default_config_path, generate_default_config, load_config, validate_config,
};
use mahalaxmi_core::config::MahalaxmiConfig;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use std::io::Write;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn clear_mahalaxmi_env_vars() {
    std::env::remove_var("MAHALAXMI_LOCALE");
    std::env::remove_var("MAHALAXMI_LOG_LEVEL");
    std::env::remove_var("MAHALAXMI_DATA_DIR");
    std::env::remove_var("MAHALAXMI_MAX_WORKERS");
    std::env::remove_var("MAHALAXMI_DEFAULT_PROVIDER");
    std::env::remove_var("MAHALAXMI_CONSENSUS_STRATEGY");
    std::env::remove_var("MAHALAXMI_MANAGER_TIMEOUT");
    std::env::remove_var("MAHALAXMI_WORKER_TIMEOUT");
    std::env::remove_var("MAHALAXMI_THEME");
}

#[test]
fn default_config_has_valid_values() {
    let config = MahalaxmiConfig::default();
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(
        result.is_ok(),
        "Default config should be valid: {:?}",
        result
    );
}

#[test]
fn load_valid_toml_file() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    let mut file = std::fs::File::create(&config_path).unwrap();
    write!(
        file,
        r#"
[general]
locale = "es-es"
log_level = "debug"

[orchestration]
max_concurrent_workers = 10
"#
    )
    .unwrap();

    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(Some(config_path.as_path()), &i18n).unwrap();
    assert_eq!(config.general.locale, "es-ES");
    assert_eq!(config.general.log_level, "debug");
    assert_eq!(config.orchestration.max_concurrent_workers, 10);
}

#[test]
fn load_toml_with_missing_sections_uses_defaults() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    let mut file = std::fs::File::create(&config_path).unwrap();
    write!(
        file,
        r#"
[general]
log_level = "warn"
"#
    )
    .unwrap();

    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(Some(config_path.as_path()), &i18n).unwrap();
    assert_eq!(config.orchestration.max_concurrent_workers, 0); // 0 = auto-scale (P1)
    assert_eq!(config.ui.theme, "dark");
}

#[test]
fn missing_file_returns_defaults() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new("/nonexistent/path/config.toml")),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.general.locale, "en-US");
    assert_eq!(config.general.log_level, "info");
}

#[test]
fn env_override_locale() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_LOCALE", "es-ES");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new("/nonexistent/env_test_locale.toml")),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.general.locale, "es-ES");
    std::env::remove_var("MAHALAXMI_LOCALE");
}

#[test]
fn env_override_log_level() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_LOG_LEVEL", "debug");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new("/nonexistent/env_test_log.toml")),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.general.log_level, "debug");
    std::env::remove_var("MAHALAXMI_LOG_LEVEL");
}

#[test]
fn env_override_data_dir() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_DATA_DIR", "/tmp/mahalaxmi-test");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new("/nonexistent/env_test_data.toml")),
        &i18n,
    )
    .unwrap();
    assert_eq!(
        config.general.data_directory,
        std::path::PathBuf::from("/tmp/mahalaxmi-test")
    );
    std::env::remove_var("MAHALAXMI_DATA_DIR");
}

#[test]
fn env_override_consensus_strategy() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_CONSENSUS_STRATEGY", "union");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new("/nonexistent/env_test_consensus.toml")),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.orchestration.consensus_strategy, "union");
    std::env::remove_var("MAHALAXMI_CONSENSUS_STRATEGY");
}

#[test]
fn env_override_manager_timeout() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_MANAGER_TIMEOUT", "120");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new(
            "/nonexistent/env_test_mgr_timeout.toml",
        )),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.orchestration.manager_timeout_seconds, 120);
    std::env::remove_var("MAHALAXMI_MANAGER_TIMEOUT");
}

#[test]
fn env_override_worker_timeout() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_WORKER_TIMEOUT", "900");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new(
            "/nonexistent/env_test_wrk_timeout.toml",
        )),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.orchestration.worker_timeout_seconds, 900);
    std::env::remove_var("MAHALAXMI_WORKER_TIMEOUT");
}

#[test]
fn env_override_default_provider() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_DEFAULT_PROVIDER", "openai-foundry");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new("/nonexistent/env_test_provider.toml")),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.providers.default_provider, "openai-foundry");
    std::env::remove_var("MAHALAXMI_DEFAULT_PROVIDER");
}

#[test]
fn env_override_theme() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_THEME", "light");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new("/nonexistent/env_test_theme.toml")),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.ui.theme, "light");
    std::env::remove_var("MAHALAXMI_THEME");
}

#[test]
fn env_override_invalid_max_workers_keeps_configured_value() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_MAX_WORKERS", "not-a-number");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new(
            "/nonexistent/env_test_bad_workers.toml",
        )),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.orchestration.max_concurrent_workers, 0); // falls back to default (0 = auto)
    std::env::remove_var("MAHALAXMI_MAX_WORKERS");
}

#[test]
fn env_override_invalid_manager_timeout_keeps_configured_value() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_MANAGER_TIMEOUT", "abc");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new(
            "/nonexistent/env_test_bad_mgr_timeout.toml",
        )),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.orchestration.manager_timeout_seconds, 300);
    std::env::remove_var("MAHALAXMI_MANAGER_TIMEOUT");
}

#[test]
fn env_override_invalid_worker_timeout_keeps_configured_value() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    std::env::set_var("MAHALAXMI_WORKER_TIMEOUT", "xyz");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new(
            "/nonexistent/env_test_bad_wrk_timeout.toml",
        )),
        &i18n,
    )
    .unwrap();
    assert_eq!(config.orchestration.worker_timeout_seconds, 600);
    std::env::remove_var("MAHALAXMI_WORKER_TIMEOUT");
}

#[test]
fn validate_rejects_invalid_log_level() {
    let mut config = MahalaxmiConfig::default();
    config.general.log_level = "invalid".to_string();
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
}

#[test]
fn validate_accepts_zero_workers_as_auto_scale() {
    // P1: 0 means "auto-scale after consensus" — must be valid, not rejected.
    let mut config = MahalaxmiConfig::default();
    config.orchestration.max_concurrent_workers = 0;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(
        result.is_ok(),
        "0 (auto-scale) must be accepted by the validator"
    );
}

#[test]
fn validate_rejects_excessive_workers() {
    let mut config = MahalaxmiConfig::default();
    config.orchestration.max_concurrent_workers = 200;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
}

#[test]
fn validate_rejects_low_manager_timeout() {
    let mut config = MahalaxmiConfig::default();
    config.orchestration.manager_timeout_seconds = 5;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
}

#[test]
fn validate_rejects_low_worker_timeout() {
    let mut config = MahalaxmiConfig::default();
    config.orchestration.worker_timeout_seconds = 5;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
}

#[test]
fn validate_rejects_zero_offline_grace_days() {
    let mut config = MahalaxmiConfig::default();
    config.licensing.offline_grace_days = 0;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
}

#[test]
fn validate_rejects_invalid_consensus_strategy() {
    let mut config = MahalaxmiConfig::default();
    config.orchestration.consensus_strategy = "random".to_string();
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("random")));
}

#[test]
fn validate_accepts_all_valid_consensus_strategies() {
    let strategies = [
        "union",
        "intersection",
        "weighted_voting",
        "complexity_weighted",
    ];
    let i18n = I18nService::new(SupportedLocale::EnUs);
    for strategy in &strategies {
        let mut config = MahalaxmiConfig::default();
        config.orchestration.consensus_strategy = strategy.to_string();
        assert!(
            validate_config(&config, &i18n).is_ok(),
            "Strategy '{strategy}' should be valid"
        );
    }
}

#[test]
fn validate_rejects_invalid_theme() {
    let mut config = MahalaxmiConfig::default();
    config.ui.theme = "neon".to_string();
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("neon")));
}

#[test]
fn validate_accepts_all_valid_themes() {
    let themes = ["dark", "light", "system"];
    let i18n = I18nService::new(SupportedLocale::EnUs);
    for theme in &themes {
        let mut config = MahalaxmiConfig::default();
        config.ui.theme = theme.to_string();
        assert!(
            validate_config(&config, &i18n).is_ok(),
            "Theme '{theme}' should be valid"
        );
    }
}

#[test]
fn validate_rejects_font_size_too_small() {
    let mut config = MahalaxmiConfig::default();
    config.ui.terminal_font_size = 4;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
}

#[test]
fn validate_rejects_font_size_too_large() {
    let mut config = MahalaxmiConfig::default();
    config.ui.terminal_font_size = 100;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
}

#[test]
fn validate_accepts_font_size_boundary_min() {
    let mut config = MahalaxmiConfig::default();
    config.ui.terminal_font_size = 8;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    assert!(validate_config(&config, &i18n).is_ok());
}

#[test]
fn validate_accepts_font_size_boundary_max() {
    let mut config = MahalaxmiConfig::default();
    config.ui.terminal_font_size = 72;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    assert!(validate_config(&config, &i18n).is_ok());
}

#[test]
fn validate_rejects_empty_default_provider() {
    let mut config = MahalaxmiConfig::default();
    config.providers.default_provider = String::new();
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
}

#[test]
fn validate_rejects_data_directory_with_null_bytes() {
    let mut config = MahalaxmiConfig::default();
    config.general.data_directory = std::path::PathBuf::from("/tmp/bad\0path");
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = validate_config(&config, &i18n);
    assert!(result.is_err());
}

#[test]
fn config_roundtrip_serialization() {
    let config = MahalaxmiConfig::default();
    let toml_str = toml::to_string(&config).expect("Serialize failed");
    let parsed: MahalaxmiConfig = toml::from_str(&toml_str).expect("Deserialize failed");
    assert_eq!(config.general.locale, parsed.general.locale);
    assert_eq!(config.general.log_level, parsed.general.log_level);
    assert_eq!(
        config.orchestration.max_concurrent_workers,
        parsed.orchestration.max_concurrent_workers
    );
}

#[test]
fn default_config_path_is_in_home_directory() {
    let path = default_config_path();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains(".mahalaxmi"),
        "Path should contain .mahalaxmi: {path_str}"
    );
    assert!(
        path_str.contains("config.toml"),
        "Path should contain config.toml: {path_str}"
    );
}

#[test]
fn validate_accepts_boundary_workers_min() {
    let mut config = MahalaxmiConfig::default();
    config.orchestration.max_concurrent_workers = 1;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    assert!(validate_config(&config, &i18n).is_ok());
}

#[test]
fn validate_accepts_boundary_workers_max() {
    let mut config = MahalaxmiConfig::default();
    config.orchestration.max_concurrent_workers = 100;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    assert!(validate_config(&config, &i18n).is_ok());
}

#[test]
fn validate_accepts_boundary_timeout_min() {
    let mut config = MahalaxmiConfig::default();
    config.orchestration.manager_timeout_seconds = 10;
    config.orchestration.worker_timeout_seconds = 10;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    assert!(validate_config(&config, &i18n).is_ok());
}

#[test]
fn validate_collects_multiple_errors() {
    let mut config = MahalaxmiConfig::default();
    config.general.log_level = "invalid".to_string();
    config.orchestration.max_concurrent_workers = 200; // out-of-range (valid range: 1..=100 or 0)
    config.orchestration.manager_timeout_seconds = 1;
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let errors = validate_config(&config, &i18n).unwrap_err();
    assert!(
        errors.len() >= 3,
        "Should collect multiple errors, got: {}",
        errors.len()
    );
}

#[test]
fn validation_errors_are_translated_to_spanish() {
    let mut config = MahalaxmiConfig::default();
    config.general.log_level = "invalid".to_string();

    let en_i18n = I18nService::new(SupportedLocale::EnUs);
    let es_i18n = I18nService::new(SupportedLocale::EsEs);

    let en_errors = validate_config(&config, &en_i18n).unwrap_err();
    let es_errors = validate_config(&config, &es_i18n).unwrap_err();

    assert!(!en_errors.is_empty());
    assert!(!es_errors.is_empty());
    assert_ne!(
        en_errors[0], es_errors[0],
        "Validation messages should differ between locales"
    );
}

#[test]
fn load_malformed_toml_returns_error() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("bad_config.toml");
    std::fs::write(&config_path, "this is not valid toml [[[").unwrap();

    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = load_config(Some(config_path.as_path()), &i18n);
    assert!(result.is_err(), "Malformed TOML should produce an error");
}

#[test]
fn default_config_field_values() {
    let config = MahalaxmiConfig::default();
    assert_eq!(config.general.locale, "en-US");
    assert_eq!(config.general.log_level, "info");
    assert_eq!(config.orchestration.max_concurrent_workers, 0); // 0 = auto-scale (P1)
    assert_eq!(config.orchestration.consensus_strategy, "weighted_voting");
    assert_eq!(config.orchestration.manager_timeout_seconds, 300);
    assert_eq!(config.orchestration.worker_timeout_seconds, 600);
    assert_eq!(config.verification.max_retries, 3);
    assert_eq!(config.providers.default_provider, "claude-code");
    assert_eq!(config.ui.theme, "dark");
    assert_eq!(config.ui.terminal_font_size, 14);
    assert_eq!(config.licensing.offline_grace_days, 7);
}

#[test]
fn all_valid_log_levels_pass_validation() {
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    let i18n = I18nService::new(SupportedLocale::EnUs);
    for level in &valid_levels {
        let mut config = MahalaxmiConfig::default();
        config.general.log_level = level.to_string();
        assert!(
            validate_config(&config, &i18n).is_ok(),
            "Log level '{level}' should be valid"
        );
    }
}

#[test]
fn generate_default_config_creates_valid_toml() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("generated_config.toml");
    let i18n = I18nService::new(SupportedLocale::EnUs);

    generate_default_config(&config_path, &i18n).unwrap();

    assert!(config_path.exists());
    let content = std::fs::read_to_string(&config_path).unwrap();
    let parsed: MahalaxmiConfig = toml::from_str(&content).unwrap();
    assert_eq!(parsed.general.log_level, "info");
}

#[test]
fn generate_default_config_fails_if_file_exists() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("existing_config.toml");
    std::fs::write(&config_path, "existing content").unwrap();

    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = generate_default_config(&config_path, &i18n);
    assert!(result.is_err());
}

#[test]
fn generate_default_config_creates_parent_directories() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("deep").join("nested").join("config.toml");
    let i18n = I18nService::new(SupportedLocale::EnUs);

    generate_default_config(&config_path, &i18n).unwrap();

    assert!(config_path.exists());
}

#[test]
fn generated_config_can_be_loaded_and_passes_validation() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_mahalaxmi_env_vars();

    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("roundtrip_config.toml");
    let i18n = I18nService::new(SupportedLocale::EnUs);

    generate_default_config(&config_path, &i18n).unwrap();

    let loaded = load_config(Some(config_path.as_path()), &i18n).unwrap();
    let validation_result = validate_config(&loaded, &i18n);
    assert!(
        validation_result.is_ok(),
        "Generated config should pass validation: {:?}",
        validation_result
    );
}

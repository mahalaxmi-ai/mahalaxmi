//! Integration tests for the Mahalaxmi foundation (Step 1).
//!
//! Verifies that all crates link correctly and the
//! config -> i18n -> logging -> error pipeline works end-to-end.

use mahalaxmi_core::config::loader::{load_config, validate_config};
use mahalaxmi_core::config::MahalaxmiConfig;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn clear_env() {
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
fn full_initialization_pipeline() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_env();

    // 1. Create I18nService with English
    let mut i18n = I18nService::new(SupportedLocale::EnUs);

    // 2. Load config (defaults, no file)
    let config = load_config(
        Some(std::path::Path::new("/nonexistent/foundation_test.toml")),
        &i18n,
    )
    .unwrap();

    // 3. Verify config defaults
    assert_eq!(config.general.locale, "en-US");
    assert_eq!(config.general.log_level, "info");

    // 4. Validate config
    assert!(validate_config(&config, &i18n).is_ok());

    // 5. Create a config error with i18n -> verify English message
    let err = MahalaxmiError::config(
        &i18n,
        keys::error::CONFIG_FILE_NOT_FOUND,
        &[("path", "/test")],
    );
    let en_msg = err.to_string();
    assert!(en_msg.contains("/test"));

    // 6. Switch locale to Spanish
    i18n.set_locale(SupportedLocale::EsEs);

    // 7. Create same error -> verify Spanish message
    let err = MahalaxmiError::config(
        &i18n,
        keys::error::CONFIG_FILE_NOT_FOUND,
        &[("path", "/test")],
    );
    let es_msg = err.to_string();
    assert!(es_msg.contains("/test"));
    assert_ne!(en_msg, es_msg, "English and Spanish messages should differ");
}

#[test]
fn config_locale_drives_error_messages() {
    let i18n = I18nService::new(SupportedLocale::EsEs);
    let err = MahalaxmiError::config(
        &i18n,
        keys::error::CONFIG_PARSE_FAILED,
        &[("reason", "bad toml")],
    );
    let msg = err.to_string();
    assert!(msg.contains("bad toml"));
    assert_eq!(err.i18n_key(), Some(keys::error::CONFIG_PARSE_FAILED));
}

#[test]
fn all_crates_are_importable() {
    // Compile-time checks: if any crate fails to link, this won't compile.
    let _locale = SupportedLocale::EnUs;
    let _config = MahalaxmiConfig::default();
    use mahalaxmi_core as _;
}

#[test]
fn validation_errors_are_translated() {
    let mut config = MahalaxmiConfig::default();
    config.general.log_level = "invalid_level".to_string();

    let en_i18n = I18nService::new(SupportedLocale::EnUs);
    let en_errors = validate_config(&config, &en_i18n).unwrap_err();
    assert!(!en_errors.is_empty());

    let es_i18n = I18nService::new(SupportedLocale::EsEs);
    let es_errors = validate_config(&config, &es_i18n).unwrap_err();
    assert!(!es_errors.is_empty());

    assert_ne!(
        en_errors[0], es_errors[0],
        "Validation error messages should differ between locales"
    );
}

#[test]
fn error_hierarchy_preserves_keys_across_locales() {
    let en_i18n = I18nService::new(SupportedLocale::EnUs);
    let es_i18n = I18nService::new(SupportedLocale::EsEs);

    let en_err = MahalaxmiError::config(
        &en_i18n,
        keys::error::CONFIG_VALIDATION_FAILED,
        &[("details", "test")],
    );
    let es_err = MahalaxmiError::config(
        &es_i18n,
        keys::error::CONFIG_VALIDATION_FAILED,
        &[("details", "test")],
    );

    // i18n keys should be the same regardless of locale
    assert_eq!(en_err.i18n_key(), es_err.i18n_key());
    // But display messages should differ
    assert_ne!(en_err.to_string(), es_err.to_string());
}

#[test]
fn i18n_service_round_trip_with_config() {
    let _lock = ENV_LOCK.lock().unwrap();
    clear_env();

    // Start with English, load config, switch to Spanish per config
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = load_config(
        Some(std::path::Path::new("/nonexistent/roundtrip_test.toml")),
        &i18n,
    )
    .unwrap();

    // Default config should have EnUs
    assert_eq!(config.general.locale, "en-US");

    // Verify we can create errors in both locales from the same pipeline
    let en_err = MahalaxmiError::config(
        &I18nService::new(
            mahalaxmi_core::i18n::locale::SupportedLocale::from_code(&config.general.locale)
                .unwrap_or_default(),
        ),
        keys::error::LOG_INIT_FAILED,
        &[("reason", "test failure")],
    );
    let es_err = MahalaxmiError::config(
        &I18nService::new(SupportedLocale::EsEs),
        keys::error::LOG_INIT_FAILED,
        &[("reason", "test failure")],
    );

    assert!(en_err.to_string().contains("test failure"));
    assert!(es_err.to_string().contains("test failure"));
    assert_ne!(en_err.to_string(), es_err.to_string());
}

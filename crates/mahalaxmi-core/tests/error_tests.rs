use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;

#[test]
fn config_error_contains_translated_message() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::config(
        &i18n,
        keys::error::CONFIG_FILE_NOT_FOUND,
        &[("path", "/test")],
    );
    let msg = err.to_string();
    assert!(
        msg.contains("/test"),
        "Error message should contain path: {msg}"
    );
    assert!(!msg.is_empty());
}

#[test]
fn config_error_message_changes_with_locale() {
    let en_i18n = I18nService::new(SupportedLocale::EnUs);
    let es_i18n = I18nService::new(SupportedLocale::EsEs);

    let en_err = MahalaxmiError::config(
        &en_i18n,
        keys::error::CONFIG_FILE_NOT_FOUND,
        &[("path", "/x")],
    );
    let es_err = MahalaxmiError::config(
        &es_i18n,
        keys::error::CONFIG_FILE_NOT_FOUND,
        &[("path", "/x")],
    );

    assert_ne!(
        en_err.to_string(),
        es_err.to_string(),
        "English and Spanish should differ"
    );
}

#[test]
fn error_i18n_key_returns_correct_key() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::config(
        &i18n,
        keys::error::CONFIG_PARSE_FAILED,
        &[("reason", "test")],
    );
    assert_eq!(err.i18n_key(), Some(keys::error::CONFIG_PARSE_FAILED));
}

#[test]
fn io_error_i18n_key_returns_none() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
    let err: MahalaxmiError = io_err.into();
    assert_eq!(err.i18n_key(), None);
}

#[test]
fn io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: MahalaxmiError = io_err.into();
    assert!(matches!(err, MahalaxmiError::Io(_)));
    assert!(err.to_string().contains("file not found"));
}

#[test]
fn toml_error_conversion() {
    let bad_toml = "this is not valid toml [[[";
    let result: Result<toml::Value, _> = toml::from_str(bad_toml);
    let toml_err = result.unwrap_err();
    let err: MahalaxmiError = toml_err.into();
    assert!(matches!(err, MahalaxmiError::TomlParse(_)));
}

#[test]
fn all_error_variants_implement_display() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let errors = [
        MahalaxmiError::config(&i18n, "test-key", &[]),
        MahalaxmiError::pty(&i18n, "test-key", &[]),
        MahalaxmiError::provider(&i18n, "test-key", &[]),
        MahalaxmiError::orchestration(&i18n, "test-key", &[]),
        MahalaxmiError::template(&i18n, "test-key", &[]),
        MahalaxmiError::license(&i18n, "test-key", &[]),
        MahalaxmiError::platform(&i18n, "test-key", &[]),
        MahalaxmiError::i18n(&i18n, "test-key", &[]),
    ];
    for err in &errors {
        let display = err.to_string();
        assert!(
            !display.is_empty(),
            "Display should not be empty for {:?}",
            err
        );
    }
}

#[test]
fn all_error_variants_implement_debug() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::config(&i18n, "test-key", &[]);
    let debug = format!("{:?}", err);
    assert!(!debug.is_empty());
}

#[test]
fn error_interpolation_includes_arguments() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::config(
        &i18n,
        keys::error::CONFIG_FILE_NOT_FOUND,
        &[("path", "/my/custom/path")],
    );
    assert!(err.to_string().contains("/my/custom/path"));
}

#[test]
fn pty_error_has_correct_key() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::pty(&i18n, "test-pty-key", &[]);
    assert_eq!(err.i18n_key(), Some("test-pty-key"));
}

#[test]
fn provider_error_has_correct_key() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::provider(&i18n, "test-provider-key", &[]);
    assert_eq!(err.i18n_key(), Some("test-provider-key"));
}

#[test]
fn orchestration_error_has_correct_key() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::orchestration(&i18n, "test-orchestration-key", &[]);
    assert_eq!(err.i18n_key(), Some("test-orchestration-key"));
}

#[test]
fn template_error_has_correct_key() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::template(&i18n, "test-template-key", &[]);
    assert_eq!(err.i18n_key(), Some("test-template-key"));
}

#[test]
fn license_error_has_correct_key() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::license(&i18n, "test-license-key", &[]);
    assert_eq!(err.i18n_key(), Some("test-license-key"));
}

#[test]
fn platform_error_has_correct_key() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::platform(&i18n, "test-platform-key", &[]);
    assert_eq!(err.i18n_key(), Some("test-platform-key"));
}

#[test]
fn i18n_error_has_correct_key() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::i18n(&i18n, "test-i18n-key", &[]);
    assert_eq!(err.i18n_key(), Some("test-i18n-key"));
}

#[test]
fn config_error_spanish_contains_translated_text() {
    let i18n = I18nService::new(SupportedLocale::EsEs);
    let err = MahalaxmiError::config(
        &i18n,
        keys::error::CONFIG_FILE_NOT_FOUND,
        &[("path", "/test/path")],
    );
    let msg = err.to_string();
    assert!(
        msg.contains("/test/path"),
        "Should contain interpolated path"
    );
    assert!(
        msg.contains("no encontrado") || msg.contains("configuración"),
        "Should contain Spanish text: {msg}"
    );
}

#[test]
fn error_with_unknown_key_uses_key_as_message() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::config(&i18n, "completely-unknown-key", &[]);
    assert_eq!(err.to_string(), "completely-unknown-key");
    assert_eq!(err.i18n_key(), Some("completely-unknown-key"));
}

#[test]
fn io_error_preserves_original_message() {
    let io_err = std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "access denied to /secret",
    );
    let err: MahalaxmiError = io_err.into();
    assert!(err.to_string().contains("access denied to /secret"));
}

#[test]
fn error_with_multiline_interpolation() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::config(
        &i18n,
        keys::error::CONFIG_PARSE_FAILED,
        &[("reason", "unexpected token at line 5\nexpected '='")],
    );
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected token"),
        "Should contain reason: {msg}"
    );
}

#[test]
fn is_config_returns_true_for_config_variant() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::config(&i18n, "test-key", &[]);
    assert!(err.is_config());
}

#[test]
fn is_config_returns_false_for_other_variants() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let variants = [
        MahalaxmiError::pty(&i18n, "test-key", &[]),
        MahalaxmiError::provider(&i18n, "test-key", &[]),
        MahalaxmiError::orchestration(&i18n, "test-key", &[]),
        MahalaxmiError::template(&i18n, "test-key", &[]),
        MahalaxmiError::license(&i18n, "test-key", &[]),
        MahalaxmiError::platform(&i18n, "test-key", &[]),
        MahalaxmiError::i18n(&i18n, "test-key", &[]),
    ];
    for err in &variants {
        assert!(
            !err.is_config(),
            "is_config() should be false for {:?}",
            err
        );
    }

    let io_err: MahalaxmiError = std::io::Error::new(std::io::ErrorKind::NotFound, "test").into();
    assert!(!io_err.is_config());
}

#[test]
fn is_io_returns_true_for_io_variant() {
    let io_err: MahalaxmiError = std::io::Error::new(std::io::ErrorKind::NotFound, "test").into();
    assert!(io_err.is_io());
}

#[test]
fn is_io_returns_false_for_other_variants() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let err = MahalaxmiError::config(&i18n, "test-key", &[]);
    assert!(!err.is_io());

    let err = MahalaxmiError::pty(&i18n, "test-key", &[]);
    assert!(!err.is_io());

    let bad_toml = "this is not valid toml [[[";
    let result: Result<toml::Value, _> = toml::from_str(bad_toml);
    let toml_err: MahalaxmiError = result.unwrap_err().into();
    assert!(!toml_err.is_io());
}

#[test]
fn category_returns_correct_string_for_each_variant() {
    let i18n = I18nService::new(SupportedLocale::EnUs);

    assert_eq!(MahalaxmiError::config(&i18n, "k", &[]).category(), "config");
    assert_eq!(MahalaxmiError::pty(&i18n, "k", &[]).category(), "pty");
    assert_eq!(
        MahalaxmiError::provider(&i18n, "k", &[]).category(),
        "provider"
    );
    assert_eq!(
        MahalaxmiError::orchestration(&i18n, "k", &[]).category(),
        "orchestration"
    );
    assert_eq!(
        MahalaxmiError::template(&i18n, "k", &[]).category(),
        "template"
    );
    assert_eq!(
        MahalaxmiError::license(&i18n, "k", &[]).category(),
        "license"
    );
    assert_eq!(
        MahalaxmiError::platform(&i18n, "k", &[]).category(),
        "platform"
    );
    assert_eq!(MahalaxmiError::i18n(&i18n, "k", &[]).category(), "i18n");

    let io_err: MahalaxmiError = std::io::Error::new(std::io::ErrorKind::NotFound, "test").into();
    assert_eq!(io_err.category(), "io");

    let bad_toml = "this is not valid toml [[[";
    let result: Result<toml::Value, _> = toml::from_str(bad_toml);
    let toml_err: MahalaxmiError = result.unwrap_err().into();
    assert_eq!(toml_err.category(), "toml-parse");
}

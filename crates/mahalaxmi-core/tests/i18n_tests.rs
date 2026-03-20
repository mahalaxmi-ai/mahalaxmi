use mahalaxmi_core::i18n::locale::{SupportedLocale, ALL};
use mahalaxmi_core::i18n::I18nService;

// ============================
// SupportedLocale Tests
// ============================

#[test]
fn all_locales_have_valid_code() {
    for locale in ALL {
        let code = locale.as_code();
        assert!(!code.is_empty(), "Locale {:?} has empty code", locale);
        assert!(code.contains('-'), "Locale code {:?} missing hyphen", code);
    }
}

#[test]
fn from_code_roundtrip() {
    for locale in ALL {
        let code = locale.as_code();
        let parsed = SupportedLocale::from_code(code);
        assert_eq!(parsed, Some(*locale), "Roundtrip failed for {code}");
    }
}

#[test]
fn from_code_case_insensitive() {
    assert_eq!(
        SupportedLocale::from_code("EN-US"),
        Some(SupportedLocale::EnUs)
    );
    assert_eq!(
        SupportedLocale::from_code("en-us"),
        Some(SupportedLocale::EnUs)
    );
    assert_eq!(
        SupportedLocale::from_code("Es-eS"),
        Some(SupportedLocale::EsEs)
    );
}

#[test]
fn from_code_invalid_returns_none() {
    assert_eq!(SupportedLocale::from_code("xx-XX"), None);
    assert_eq!(SupportedLocale::from_code(""), None);
    assert_eq!(SupportedLocale::from_code("english"), None);
    assert_eq!(SupportedLocale::from_code("en"), None);
}

#[test]
fn display_names_are_not_empty() {
    for locale in ALL {
        assert!(
            !locale.display_name().is_empty(),
            "Display name empty for {:?}",
            locale
        );
        assert!(
            !locale.native_name().is_empty(),
            "Native name empty for {:?}",
            locale
        );
    }
}

#[test]
fn display_names_contain_expected_text() {
    assert!(SupportedLocale::EnUs.display_name().contains("English"));
    assert!(SupportedLocale::EsEs.display_name().contains("Spanish"));
    assert!(SupportedLocale::FrFr.display_name().contains("French"));
    assert!(SupportedLocale::DeDe.display_name().contains("German"));
    assert!(SupportedLocale::JaJp.display_name().contains("Japanese"));
}

#[test]
fn native_names_are_in_native_language() {
    assert_eq!(SupportedLocale::EsEs.native_name(), "Español");
    assert_eq!(SupportedLocale::FrFr.native_name(), "Français");
    assert_eq!(SupportedLocale::DeDe.native_name(), "Deutsch");
}

#[test]
fn only_arabic_is_rtl() {
    for locale in ALL {
        if *locale == SupportedLocale::ArSa {
            assert!(locale.is_rtl(), "Arabic should be RTL");
        } else {
            assert!(!locale.is_rtl(), "{:?} should not be RTL", locale);
        }
    }
}

#[test]
fn default_locale_is_english() {
    assert_eq!(SupportedLocale::default(), SupportedLocale::EnUs);
}

#[test]
fn all_constant_contains_all_variants() {
    assert_eq!(ALL.len(), 10);
}

#[test]
fn display_trait_shows_code() {
    assert_eq!(format!("{}", SupportedLocale::EnUs), "en-US");
    assert_eq!(format!("{}", SupportedLocale::EsEs), "es-ES");
    assert_eq!(format!("{}", SupportedLocale::ArSa), "ar-SA");
}

// ============================
// I18nService Tests
// ============================

#[test]
fn translate_simple_key_english() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = i18n.t("app-ready");
    assert!(!result.is_empty());
    assert_ne!(
        result, "app-ready",
        "Should return translated text, not key"
    );
}

#[test]
fn translate_simple_key_spanish() {
    let i18n = I18nService::new(SupportedLocale::EsEs);
    let result = i18n.t("app-ready");
    assert!(!result.is_empty());
    assert_ne!(result, "app-ready");
    let en = I18nService::new(SupportedLocale::EnUs).t("app-ready");
    assert_ne!(result, en);
}

#[test]
fn translate_with_interpolation() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = i18n.translate("error-config-file-not-found", &[("path", "/etc/foo")]);
    assert!(
        result.contains("/etc/foo"),
        "Interpolation failed: {result}"
    );
}

#[test]
fn translate_with_interpolation_spanish() {
    let i18n = I18nService::new(SupportedLocale::EsEs);
    let result = i18n.translate("error-config-file-not-found", &[("path", "/etc/foo")]);
    assert!(
        result.contains("/etc/foo"),
        "Interpolation failed in Spanish: {result}"
    );
    assert!(
        result.contains("no encontrado") || result.contains("configuración"),
        "Should be Spanish text: {result}"
    );
}

#[test]
fn translate_missing_key_returns_key() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let key = "this-key-does-not-exist-anywhere";
    let result = i18n.translate(key, &[]);
    assert_eq!(result, key);
}

#[test]
fn switch_locale_changes_output() {
    let mut i18n = I18nService::new(SupportedLocale::EnUs);
    let english = i18n.t("app-ready");

    i18n.set_locale(SupportedLocale::EsEs);
    let spanish = i18n.t("app-ready");

    assert_ne!(
        english, spanish,
        "Locale switch should produce different text"
    );
}

#[test]
fn current_locale_reflects_changes() {
    let mut i18n = I18nService::new(SupportedLocale::EnUs);
    assert_eq!(i18n.current_locale(), SupportedLocale::EnUs);

    i18n.set_locale(SupportedLocale::EsEs);
    assert_eq!(i18n.current_locale(), SupportedLocale::EsEs);
}

#[test]
fn available_locales_returns_all() {
    let locales = I18nService::available_locales();
    assert_eq!(locales.len(), 10);
}

#[test]
fn all_english_keys_have_spanish_translations() {
    let en = I18nService::new(SupportedLocale::EnUs);
    let es = I18nService::new(SupportedLocale::EsEs);

    let keys = [
        "error-config-file-not-found",
        "error-config-parse-failed",
        "error-config-validation-failed",
        "error-locale-not-supported",
        "error-log-init-failed",
        "config-loaded-successfully",
        "config-using-default",
        "config-env-override",
        "logging-initialized",
        "app-starting",
        "app-ready",
        "app-shutting-down",
    ];

    for key in &keys {
        let en_result = en.t(key);
        let es_result = es.t(key);
        assert_ne!(en_result, *key, "English missing key: {key}");
        assert_ne!(es_result, *key, "Spanish missing key: {key}");
    }
}

#[test]
fn translate_with_multiple_args() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let result = i18n.translate(
        "error-log-dir-create-failed",
        &[
            ("path", "/var/log/mahalaxmi"),
            ("reason", "permission denied"),
        ],
    );
    assert!(
        result.contains("/var/log/mahalaxmi"),
        "Missing path arg: {result}"
    );
    assert!(
        result.contains("permission denied"),
        "Missing reason arg: {result}"
    );
}

#[test]
fn locale_serde_roundtrip() {
    for locale in ALL {
        let json = serde_json::to_string(locale).expect("serialize failed");
        let parsed: SupportedLocale = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(*locale, parsed);
    }
}

#[test]
fn missing_key_falls_back_to_key_string() {
    let i18n = I18nService::new(SupportedLocale::KoKr);
    let result = i18n.translate("nonexistent-key-for-fallback-test", &[]);
    assert_eq!(
        result, "nonexistent-key-for-fallback-test",
        "Missing key should return key string"
    );
}

#[test]
fn korean_locale_translates_natively() {
    let i18n = I18nService::new(SupportedLocale::KoKr);
    let result = i18n.translate("error-config-file-not-found", &[("path", "/etc/foo")]);
    assert_ne!(
        result, "Configuration file not found at /etc/foo",
        "Korean should not return English text"
    );
    assert!(
        result.contains("/etc/foo"),
        "Korean translation should contain the interpolated path"
    );
}

#[test]
fn translate_all_validation_keys() {
    let en = I18nService::new(SupportedLocale::EnUs);
    let es = I18nService::new(SupportedLocale::EsEs);

    let validation_keys_with_args: Vec<(&str, Vec<(&str, &str)>)> = vec![
        (
            "validation-invalid-log-level",
            vec![
                ("level", "bad"),
                ("valid", "trace, debug, info, warn, error"),
            ],
        ),
        (
            "validation-workers-out-of-range",
            vec![("value", "0"), ("min", "1"), ("max", "100")],
        ),
        (
            "validation-manager-timeout-too-low",
            vec![("value", "5"), ("min", "10")],
        ),
        (
            "validation-worker-timeout-too-low",
            vec![("value", "5"), ("min", "10")],
        ),
        (
            "validation-offline-grace-too-low",
            vec![("value", "0"), ("min", "1")],
        ),
    ];

    for (key, args) in &validation_keys_with_args {
        let en_result = en.translate(key, args);
        let es_result = es.translate(key, args);
        assert_ne!(en_result, *key, "English missing validation key: {key}");
        assert_ne!(es_result, *key, "Spanish missing validation key: {key}");
        assert_ne!(
            en_result, es_result,
            "English and Spanish should differ for: {key}"
        );
    }
}

#[test]
fn english_and_spanish_translations_differ_for_all_keys() {
    let en = I18nService::new(SupportedLocale::EnUs);
    let es = I18nService::new(SupportedLocale::EsEs);

    let no_arg_keys = ["config-using-default", "app-ready", "app-shutting-down"];

    for key in &no_arg_keys {
        let en_result = en.t(key);
        let es_result = es.t(key);
        assert_ne!(
            en_result, es_result,
            "Translations should differ for key: {key}"
        );
    }
}

// ============================
// French (fr-FR) Locale Tests
// ============================

#[test]
fn translate_simple_key_french() {
    let i18n = I18nService::new(SupportedLocale::FrFr);
    let result = i18n.t("app-ready");
    assert!(!result.is_empty());
    assert_ne!(
        result, "app-ready",
        "Should return translated text, not key"
    );
    let en = I18nService::new(SupportedLocale::EnUs).t("app-ready");
    assert_ne!(result, en, "French should differ from English");
}

#[test]
fn translate_with_interpolation_french() {
    let i18n = I18nService::new(SupportedLocale::FrFr);
    let result = i18n.translate("error-config-file-not-found", &[("path", "/etc/foo")]);
    assert!(
        result.contains("/etc/foo"),
        "Interpolation failed in French: {result}"
    );
}

#[test]
fn all_english_keys_have_french_translations() {
    let fr = I18nService::new(SupportedLocale::FrFr);

    let keys = [
        "error-config-file-not-found",
        "error-config-parse-failed",
        "error-config-validation-failed",
        "error-locale-not-supported",
        "error-log-init-failed",
        "config-loaded-successfully",
        "config-using-default",
        "config-env-override",
        "logging-initialized",
        "app-starting",
        "app-ready",
        "app-shutting-down",
    ];

    for key in &keys {
        let fr_result = fr.t(key);
        assert_ne!(fr_result, *key, "French missing key: {key}");
    }
}

#[test]
fn translate_all_french_keys_with_args() {
    let fr = I18nService::new(SupportedLocale::FrFr);

    let keys_with_args: Vec<(&str, Vec<(&str, &str)>)> = vec![
        ("error-config-file-not-found", vec![("path", "/test")]),
        ("error-config-parse-failed", vec![("reason", "syntaxe")]),
        (
            "error-config-validation-failed",
            vec![("details", "champ manquant")],
        ),
        ("error-locale-not-supported", vec![("locale", "xx-XX")]),
        ("error-log-init-failed", vec![("reason", "erreur io")]),
        (
            "error-log-dir-create-failed",
            vec![("path", "/tmp"), ("reason", "plein")],
        ),
        ("config-loaded-successfully", vec![("path", "/etc/config")]),
        ("config-using-default", vec![]),
        ("config-env-override", vec![("var", "MAHALAXMI_PORT")]),
        ("logging-initialized", vec![("level", "debug")]),
        ("app-starting", vec![("version", "1.0.0")]),
        ("app-ready", vec![]),
        ("app-shutting-down", vec![]),
    ];

    for (key, args) in &keys_with_args {
        let result = fr.translate(key, args);
        assert_ne!(result, *key, "Key '{}' was not found in fr-FR locale", key);
    }
}

#[test]
fn french_and_english_translations_differ() {
    let en = I18nService::new(SupportedLocale::EnUs);
    let fr = I18nService::new(SupportedLocale::FrFr);

    let no_arg_keys = ["config-using-default", "app-ready", "app-shutting-down"];

    for key in &no_arg_keys {
        let en_result = en.t(key);
        let fr_result = fr.t(key);
        assert_ne!(
            en_result, fr_result,
            "French and English should differ for key: {key}"
        );
    }
}

#[test]
fn switch_locale_to_french() {
    let mut i18n = I18nService::new(SupportedLocale::EnUs);
    let english = i18n.t("app-ready");

    i18n.set_locale(SupportedLocale::FrFr);
    let french = i18n.t("app-ready");

    assert_ne!(
        english, french,
        "Switching to French should produce different text"
    );
}

#[test]
fn translate_all_french_validation_keys() {
    let fr = I18nService::new(SupportedLocale::FrFr);

    let validation_keys_with_args: Vec<(&str, Vec<(&str, &str)>)> = vec![
        (
            "validation-invalid-log-level",
            vec![
                ("level", "mauvais"),
                ("valid", "trace, debug, info, warn, error"),
            ],
        ),
        (
            "validation-workers-out-of-range",
            vec![("value", "0"), ("min", "1"), ("max", "100")],
        ),
        (
            "validation-manager-timeout-too-low",
            vec![("value", "5"), ("min", "10")],
        ),
        (
            "validation-worker-timeout-too-low",
            vec![("value", "5"), ("min", "10")],
        ),
        (
            "validation-offline-grace-too-low",
            vec![("value", "0"), ("min", "1")],
        ),
    ];

    for (key, args) in &validation_keys_with_args {
        let result = fr.translate(key, args);
        assert_ne!(result, *key, "French missing validation key: {key}");
    }
}

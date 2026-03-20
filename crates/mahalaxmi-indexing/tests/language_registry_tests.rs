//! Integration tests for the language registry.
//!
//! Validates that the default [`LanguageRegistry`] contains configurations for
//! all 8 supported languages, that the [`ExtractorFactory`] can create
//! extractors for each one, and that every configuration carries non-empty
//! query patterns.

use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_indexing::extractors::ExtractorFactory;
use mahalaxmi_indexing::languages::LanguageRegistry;
use mahalaxmi_indexing::types::SupportedLanguage;

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

/// All 8 supported language variants.
const ALL_LANGUAGES: [SupportedLanguage; 8] = [
    SupportedLanguage::Rust,
    SupportedLanguage::TypeScript,
    SupportedLanguage::JavaScript,
    SupportedLanguage::Python,
    SupportedLanguage::Go,
    SupportedLanguage::Java,
    SupportedLanguage::C,
    SupportedLanguage::Cpp,
];

#[test]
fn default_registry_has_all_languages() {
    let registry = LanguageRegistry::with_defaults();
    assert_eq!(
        registry.language_count(),
        8,
        "with_defaults() should register all 8 languages"
    );
    for lang in &ALL_LANGUAGES {
        assert!(
            registry.is_supported(lang),
            "{:?} should be supported in the default registry",
            lang
        );
    }
}

#[test]
fn registry_get_returns_config() {
    let registry = LanguageRegistry::with_defaults();
    for lang in &ALL_LANGUAGES {
        let config = registry.get(lang);
        assert!(
            config.is_some(),
            "get({:?}) should return Some on the default registry",
            lang
        );
    }
}

#[test]
fn registry_get_returns_none_for_unregistered() {
    let registry = LanguageRegistry::new();
    for lang in &ALL_LANGUAGES {
        assert!(
            registry.get(lang).is_none(),
            "get({:?}) should return None on an empty registry",
            lang
        );
    }
    assert_eq!(registry.language_count(), 0);
}

#[test]
fn registry_supported_languages_returns_all() {
    let registry = LanguageRegistry::with_defaults();
    let mut supported = registry.supported_languages();
    supported.sort_by_key(|l| l.as_str().to_owned());

    let mut expected: Vec<SupportedLanguage> = ALL_LANGUAGES.to_vec();
    expected.sort_by_key(|l| l.as_str().to_owned());

    assert_eq!(
        supported.len(),
        8,
        "supported_languages() should return 8 entries"
    );
    assert_eq!(supported, expected);
}

#[test]
fn extractor_factory_creates_all() {
    let i18n = test_i18n();
    let registry = LanguageRegistry::with_defaults();
    let extractors = ExtractorFactory::create_all(&registry, &i18n)
        .expect("create_all should succeed on the default registry");
    assert_eq!(
        extractors.len(),
        8,
        "create_all should produce 8 extractors"
    );
    for lang in &ALL_LANGUAGES {
        assert!(
            extractors.contains_key(lang),
            "create_all should produce an extractor for {:?}",
            lang
        );
    }
}

#[test]
fn extractor_factory_creates_single() {
    let i18n = test_i18n();
    let registry = LanguageRegistry::with_defaults();
    for lang in &ALL_LANGUAGES {
        let result = ExtractorFactory::create(*lang, &registry, &i18n);
        assert!(
            result.is_ok(),
            "create({:?}) should succeed; error: {:?}",
            lang,
            result.as_ref().err()
        );
        let extractor = result.unwrap();
        assert_eq!(
            extractor.language(),
            *lang,
            "extractor language should match the requested language"
        );
    }
}

#[test]
fn each_config_has_symbol_query() {
    let registry = LanguageRegistry::with_defaults();
    for lang in &ALL_LANGUAGES {
        let config = registry.get(lang).expect("config should exist");
        assert!(
            !config.symbol_query.trim().is_empty(),
            "{:?} should have a non-empty symbol_query",
            lang
        );
    }
}

#[test]
fn each_config_has_import_query() {
    let registry = LanguageRegistry::with_defaults();
    for lang in &ALL_LANGUAGES {
        let config = registry.get(lang).expect("config should exist");
        assert!(
            !config.import_query.trim().is_empty(),
            "{:?} should have a non-empty import_query",
            lang
        );
    }
}

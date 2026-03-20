//! Tests for MemoryInjector.

use mahalaxmi_memory::*;

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

fn make_populated_store(i18n: &I18nService) -> MemoryStore {
    let mut store = MemoryStore::new("test");

    store
        .insert(
            MemoryEntryBuilder::new(
                MemoryType::CodebaseFact,
                "Uses Rust",
                "The project uses Rust",
                MemorySource::System,
            )
            .confidence(0.8)
            .build(i18n)
            .unwrap(),
            i18n,
        )
        .unwrap();

    store
        .insert(
            MemoryEntryBuilder::new(
                MemoryType::Warning,
                "Avoid panics",
                "Do not use unwrap in production code",
                MemorySource::Worker {
                    worker_id: "w1".into(),
                },
            )
            .confidence(0.9)
            .build(i18n)
            .unwrap(),
            i18n,
        )
        .unwrap();

    store
        .insert(
            MemoryEntryBuilder::new(
                MemoryType::Convention,
                "Snake case",
                "All functions use snake_case naming",
                MemorySource::System,
            )
            .confidence(0.6)
            .build(i18n)
            .unwrap(),
            i18n,
        )
        .unwrap();

    store
        .insert(
            MemoryEntryBuilder::new(
                MemoryType::Decision,
                "Use tokio",
                "We chose tokio as the async runtime",
                MemorySource::Manager {
                    manager_id: "m1".into(),
                },
            )
            .confidence(0.85)
            .build(i18n)
            .unwrap(),
            i18n,
        )
        .unwrap();

    store
}

#[test]
fn inject_empty_store_returns_empty() {
    let store = MemoryStore::new("test");
    let injector = MemoryInjector::new(InjectorConfig::default());
    let result = injector.inject(&store, None);
    assert!(result.is_empty());
}

#[test]
fn inject_markdown_format() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        format: InjectionFormat::Markdown,
        max_tokens: 10000,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let result = injector.inject(&store, None);

    assert!(result.contains("###"));
    assert!(!result.is_empty());
}

#[test]
fn inject_plaintext_format() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        format: InjectionFormat::PlainText,
        max_tokens: 10000,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let result = injector.inject(&store, None);

    assert!(result.contains("---"));
    assert!(!result.is_empty());
}

#[test]
fn inject_xml_format() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        format: InjectionFormat::Xml,
        max_tokens: 10000,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let result = injector.inject(&store, None);

    assert!(result.contains("<memory"));
    assert!(result.contains("</memory>"));
}

#[test]
fn inject_respects_token_budget() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        max_tokens: 10,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let result = injector.inject(&store, None);

    let tokens = MemoryInjector::estimate_tokens(&result);
    assert!(tokens <= 10);
}

#[test]
fn inject_priority_by_confidence() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        max_tokens: 10000,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let result = injector.inject(&store, None);

    // Warning (0.9) should appear before Convention (0.6)
    let warn_pos = result.find("Avoid panics");
    let conv_pos = result.find("Snake case");
    assert!(warn_pos.is_some());
    assert!(conv_pos.is_some());
    assert!(warn_pos.unwrap() < conv_pos.unwrap());
}

#[test]
fn inject_min_confidence_filter() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        min_confidence: 0.85,
        max_tokens: 10000,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let result = injector.inject(&store, None);

    // Only Warning (0.9) and Decision (0.85) should pass
    assert!(result.contains("Avoid panics"));
    assert!(result.contains("Use tokio"));
    assert!(!result.contains("Snake case")); // Convention at 0.6
    assert!(!result.contains("Uses Rust")); // CodebaseFact at 0.8
}

#[test]
fn inject_type_filter() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        include_types: vec![MemoryType::Warning],
        min_confidence: 0.0,
        max_tokens: 10000,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let result = injector.inject(&store, None);

    assert!(result.contains("Avoid panics"));
    assert!(!result.contains("Uses Rust"));
    assert!(!result.contains("Snake case"));
}

#[test]
fn inject_with_query_filter() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        max_tokens: 10000,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let query = MemoryQuery::new().with_text_search("tokio");
    let result = injector.inject(&store, Some(&query));

    assert!(result.contains("Use tokio"));
    assert!(!result.contains("Avoid panics"));
}

#[test]
fn estimate_tokens_edge_cases() {
    assert_eq!(MemoryInjector::estimate_tokens(""), 1);
    assert_eq!(MemoryInjector::estimate_tokens("a"), 1);
    assert_eq!(MemoryInjector::estimate_tokens("ab"), 1);
    assert_eq!(MemoryInjector::estimate_tokens("abc"), 1);
    assert_eq!(MemoryInjector::estimate_tokens("abcd"), 1);
    assert_eq!(MemoryInjector::estimate_tokens("abcde"), 1);
    assert_eq!(MemoryInjector::estimate_tokens("abcdefgh"), 2);
    assert_eq!(MemoryInjector::estimate_tokens(&"x".repeat(100)), 25);
}

#[test]
fn inject_count() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        max_tokens: 10000,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let count = injector.inject_count(&store, None);
    assert_eq!(count, 4);
}

#[test]
fn inject_count_with_budget() {
    let i18n = test_i18n();
    let store = make_populated_store(&i18n);
    let config = InjectorConfig {
        max_tokens: 10,
        ..InjectorConfig::default()
    };
    let injector = MemoryInjector::new(config);
    let count = injector.inject_count(&store, None);
    assert!(count < 4);
}

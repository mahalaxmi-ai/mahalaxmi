//! Tests for MemoryQuery filtering.

use mahalaxmi_memory::*;

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

fn make_entry(
    i18n: &I18nService,
    title: &str,
    content: &str,
    mt: MemoryType,
    source: MemorySource,
) -> MemoryEntry {
    MemoryEntryBuilder::new(mt, title, content, source)
        .build(i18n)
        .unwrap()
}

#[test]
fn empty_query_matches_all() {
    let i18n = test_i18n();
    let entry = make_entry(
        &i18n,
        "test",
        "content",
        MemoryType::Warning,
        MemorySource::System,
    );
    let query = MemoryQuery::new();
    assert!(query.matches(&entry));
}

#[test]
fn type_filter_matches() {
    let i18n = test_i18n();
    let warning = make_entry(
        &i18n,
        "warn",
        "content",
        MemoryType::Warning,
        MemorySource::System,
    );
    let fact = make_entry(
        &i18n,
        "fact",
        "content",
        MemoryType::CodebaseFact,
        MemorySource::System,
    );

    let query = MemoryQuery::new().with_type(MemoryType::Warning);
    assert!(query.matches(&warning));
    assert!(!query.matches(&fact));
}

#[test]
fn multi_type_or_filter() {
    let i18n = test_i18n();
    let warning = make_entry(
        &i18n,
        "warn",
        "content",
        MemoryType::Warning,
        MemorySource::System,
    );
    let fact = make_entry(
        &i18n,
        "fact",
        "content",
        MemoryType::CodebaseFact,
        MemorySource::System,
    );
    let decision = make_entry(
        &i18n,
        "dec",
        "content",
        MemoryType::Decision,
        MemorySource::System,
    );

    let query = MemoryQuery::new().with_types(vec![MemoryType::Warning, MemoryType::CodebaseFact]);
    assert!(query.matches(&warning));
    assert!(query.matches(&fact));
    assert!(!query.matches(&decision));
}

#[test]
fn tag_filter_and_semantics() {
    let i18n = test_i18n();
    let mut entry = make_entry(
        &i18n,
        "test",
        "content",
        MemoryType::Warning,
        MemorySource::System,
    );
    entry.tags = vec!["rust".into(), "safety".into()];

    let query_single = MemoryQuery::new().with_tag("rust");
    assert!(query_single.matches(&entry));

    let query_both = MemoryQuery::new().with_tags(vec!["rust".into(), "safety".into()]);
    assert!(query_both.matches(&entry));

    let query_missing = MemoryQuery::new().with_tag("python");
    assert!(!query_missing.matches(&entry));
}

#[test]
fn min_confidence_filter() {
    let i18n = test_i18n();
    let mut entry = make_entry(
        &i18n,
        "test",
        "content",
        MemoryType::Convention,
        MemorySource::System,
    );
    entry.confidence = 0.4;

    let query_low = MemoryQuery::new().with_min_confidence(0.3);
    assert!(query_low.matches(&entry));

    let query_high = MemoryQuery::new().with_min_confidence(0.5);
    assert!(!query_high.matches(&entry));
}

#[test]
fn cycle_filter() {
    let i18n = test_i18n();
    let entry = MemoryEntryBuilder::new(
        MemoryType::Decision,
        "test",
        "content",
        MemorySource::System,
    )
    .cycle_id("cycle-1")
    .build(&i18n)
    .unwrap();

    let query_match = MemoryQuery::new().with_cycle("cycle-1");
    assert!(query_match.matches(&entry));

    let query_mismatch = MemoryQuery::new().with_cycle("cycle-2");
    assert!(!query_mismatch.matches(&entry));
}

#[test]
fn text_search_case_insensitive() {
    let i18n = test_i18n();
    let entry = make_entry(
        &i18n,
        "Important Warning",
        "Do not use UNSAFE code",
        MemoryType::Warning,
        MemorySource::System,
    );

    let query_title = MemoryQuery::new().with_text_search("important");
    assert!(query_title.matches(&entry));

    let query_content = MemoryQuery::new().with_text_search("unsafe");
    assert!(query_content.matches(&entry));

    let query_miss = MemoryQuery::new().with_text_search("python");
    assert!(!query_miss.matches(&entry));
}

#[test]
fn source_filter() {
    let i18n = test_i18n();
    let worker_entry = make_entry(
        &i18n,
        "test",
        "content",
        MemoryType::Warning,
        MemorySource::Worker {
            worker_id: "w1".into(),
        },
    );
    let system_entry = make_entry(
        &i18n,
        "test",
        "content",
        MemoryType::Warning,
        MemorySource::System,
    );

    let query = MemoryQuery::new().with_source("worker");
    assert!(query.matches(&worker_entry));
    assert!(!query.matches(&system_entry));
}

#[test]
fn file_path_filter() {
    let i18n = test_i18n();
    let mut entry = make_entry(
        &i18n,
        "test",
        "content",
        MemoryType::CodebaseFact,
        MemorySource::System,
    );
    entry.metadata.file_path = Some("src/main.rs".into());

    let query_match = MemoryQuery::new().with_file_path("src/main.rs");
    assert!(query_match.matches(&entry));

    let query_miss = MemoryQuery::new().with_file_path("src/lib.rs");
    assert!(!query_miss.matches(&entry));
}

#[test]
fn combined_filters() {
    let i18n = test_i18n();
    let mut entry = make_entry(
        &i18n,
        "Rust Convention",
        "Use snake_case",
        MemoryType::Convention,
        MemorySource::Worker {
            worker_id: "w1".into(),
        },
    );
    entry.tags = vec!["naming".into()];
    entry.confidence = 0.8;

    let query = MemoryQuery::new()
        .with_type(MemoryType::Convention)
        .with_tag("naming")
        .with_min_confidence(0.7)
        .with_source("worker")
        .with_text_search("snake");

    assert!(query.matches(&entry));

    let query_wrong_type = MemoryQuery::new()
        .with_type(MemoryType::Warning)
        .with_tag("naming");
    assert!(!query_wrong_type.matches(&entry));
}

#[test]
fn query_matches_source_helper() {
    let query_worker = MemoryQuery::new().with_source("worker");
    assert!(query_worker.matches_source(&MemorySource::Worker {
        worker_id: "w1".into()
    }));
    assert!(!query_worker.matches_source(&MemorySource::System));

    let query_any = MemoryQuery::new();
    assert!(query_any.matches_source(&MemorySource::System));
    assert!(query_any.matches_source(&MemorySource::User));
}

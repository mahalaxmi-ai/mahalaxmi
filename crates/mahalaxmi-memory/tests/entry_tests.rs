//! Tests for MemoryEntry and MemoryEntryBuilder.

use mahalaxmi_memory::*;
use std::collections::HashMap;

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

#[test]
fn builder_creates_valid_entry() {
    let i18n = test_i18n();
    let entry = MemoryEntryBuilder::new(
        MemoryType::CodebaseFact,
        "Project uses Rust",
        "The project is written in Rust with a workspace layout",
        MemorySource::Worker {
            worker_id: "w1".into(),
        },
    )
    .build(&i18n)
    .unwrap();

    assert_eq!(entry.title, "Project uses Rust");
    assert_eq!(entry.memory_type, MemoryType::CodebaseFact);
    assert!((entry.confidence - 0.7).abs() < f64::EPSILON);
    assert!(entry.tags.is_empty());
    assert!(entry.cycle_id.is_none());
}

#[test]
fn builder_uses_type_default_confidence() {
    let i18n = test_i18n();
    for mt in [
        MemoryType::CodebaseFact,
        MemoryType::Convention,
        MemoryType::Decision,
        MemoryType::Warning,
    ] {
        let entry = MemoryEntryBuilder::new(mt, "title", "content", MemorySource::System)
            .build(&i18n)
            .unwrap();
        assert!(
            (entry.confidence - mt.default_confidence()).abs() < f64::EPSILON,
            "{mt} default confidence mismatch"
        );
    }
}

#[test]
fn builder_overrides_confidence() {
    let i18n = test_i18n();
    let entry = MemoryEntryBuilder::new(
        MemoryType::Warning,
        "title",
        "content",
        MemorySource::System,
    )
    .confidence(0.42)
    .build(&i18n)
    .unwrap();

    assert!((entry.confidence - 0.42).abs() < f64::EPSILON);
}

#[test]
fn builder_rejects_empty_title() {
    let i18n = test_i18n();
    let result = MemoryEntryBuilder::new(MemoryType::Decision, "", "content", MemorySource::System)
        .build(&i18n);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_memory());
    assert!(err.to_string().contains("empty"));
}

#[test]
fn builder_rejects_title_over_200_chars() {
    let i18n = test_i18n();
    let long_title = "x".repeat(201);
    let result = MemoryEntryBuilder::new(
        MemoryType::Decision,
        long_title,
        "content",
        MemorySource::System,
    )
    .build(&i18n);
    assert!(result.is_err());
}

#[test]
fn builder_accepts_title_at_200_chars() {
    let i18n = test_i18n();
    let title = "x".repeat(200);
    let entry = MemoryEntryBuilder::new(
        MemoryType::Decision,
        title.clone(),
        "content",
        MemorySource::System,
    )
    .build(&i18n)
    .unwrap();
    assert_eq!(entry.title.len(), 200);
}

#[test]
fn builder_rejects_empty_content() {
    let i18n = test_i18n();
    let result = MemoryEntryBuilder::new(MemoryType::Warning, "title", "", MemorySource::System)
        .build(&i18n);
    assert!(result.is_err());
}

#[test]
fn builder_rejects_confidence_below_zero() {
    let i18n = test_i18n();
    let result = MemoryEntryBuilder::new(
        MemoryType::Warning,
        "title",
        "content",
        MemorySource::System,
    )
    .confidence(-0.1)
    .build(&i18n);
    assert!(result.is_err());
}

#[test]
fn builder_rejects_confidence_above_one() {
    let i18n = test_i18n();
    let result = MemoryEntryBuilder::new(
        MemoryType::Warning,
        "title",
        "content",
        MemorySource::System,
    )
    .confidence(1.01)
    .build(&i18n);
    assert!(result.is_err());
}

#[test]
fn builder_with_tags_and_cycle() {
    let i18n = test_i18n();
    let entry = MemoryEntryBuilder::new(
        MemoryType::Convention,
        "Use snake_case",
        "All functions use snake_case naming",
        MemorySource::System,
    )
    .tags(vec!["naming".into(), "rust".into()])
    .cycle_id("cycle-42")
    .build(&i18n)
    .unwrap();

    assert_eq!(entry.tags, vec!["naming", "rust"]);
    assert_eq!(entry.cycle_id.as_deref(), Some("cycle-42"));
}

#[test]
fn builder_with_metadata() {
    let i18n = test_i18n();
    let mut custom = HashMap::new();
    custom.insert("key1".into(), "val1".into());

    let metadata = MemoryMetadata {
        file_path: Some("src/main.rs".into()),
        line_range: Some((10, 20)),
        language: Some("rust".into()),
        related_entries: vec![],
        custom,
    };

    let entry = MemoryEntryBuilder::new(
        MemoryType::CodebaseFact,
        "main entry point",
        "The main function is in src/main.rs",
        MemorySource::System,
    )
    .metadata(metadata)
    .build(&i18n)
    .unwrap();

    assert_eq!(entry.metadata.file_path.as_deref(), Some("src/main.rs"));
    assert_eq!(entry.metadata.line_range, Some((10, 20)));
    assert_eq!(entry.metadata.language.as_deref(), Some("rust"));
    assert_eq!(entry.metadata.custom.get("key1").unwrap(), "val1");
}

#[test]
fn entry_serialization_roundtrip() {
    let i18n = test_i18n();
    let entry = MemoryEntryBuilder::new(
        MemoryType::Warning,
        "Watch out for panics",
        "Never use unwrap in production code",
        MemorySource::Worker {
            worker_id: "w1".into(),
        },
    )
    .tags(vec!["safety".into()])
    .build(&i18n)
    .unwrap();

    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: MemoryEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.title, entry.title);
    assert_eq!(deserialized.memory_type, entry.memory_type);
    assert_eq!(deserialized.id, entry.id);
}

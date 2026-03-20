//! Tests for MemoryStore.

use mahalaxmi_memory::*;

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

fn make_entry(i18n: &I18nService, title: &str, mt: MemoryType) -> MemoryEntry {
    MemoryEntryBuilder::new(mt, title, "test content", MemorySource::System)
        .build(i18n)
        .unwrap()
}

fn make_entry_with_confidence(
    i18n: &I18nService,
    title: &str,
    mt: MemoryType,
    confidence: f64,
) -> MemoryEntry {
    MemoryEntryBuilder::new(mt, title, "test content", MemorySource::System)
        .confidence(confidence)
        .build(i18n)
        .unwrap()
}

#[test]
fn empty_store_properties() {
    let store = MemoryStore::new("test-session");
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
    assert_eq!(store.session_id(), "test-session");
}

#[test]
fn insert_returns_id() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");
    let entry = make_entry(&i18n, "fact1", MemoryType::CodebaseFact);
    let id = store.insert(entry, &i18n).unwrap();
    assert!(store.get(&id).is_some());
}

#[test]
fn insert_and_get() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");
    let entry = make_entry(&i18n, "convention1", MemoryType::Convention);
    let id = store.insert(entry, &i18n).unwrap();

    let retrieved = store.get(&id).unwrap();
    assert_eq!(retrieved.title, "convention1");
    assert_eq!(retrieved.memory_type, MemoryType::Convention);
}

#[test]
fn full_store_rejects_insert() {
    let i18n = test_i18n();
    let mut store = MemoryStore::with_max_entries("test", 2);
    store
        .insert(make_entry(&i18n, "e1", MemoryType::Warning), &i18n)
        .unwrap();
    store
        .insert(make_entry(&i18n, "e2", MemoryType::Warning), &i18n)
        .unwrap();

    let result = store.insert(make_entry(&i18n, "e3", MemoryType::Warning), &i18n);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_memory());
}

#[test]
fn update_existing_entry() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");
    let entry = make_entry(&i18n, "original", MemoryType::Decision);
    let id = store.insert(entry, &i18n).unwrap();

    let mut updated = make_entry(&i18n, "updated", MemoryType::Decision);
    updated.id = id;
    store.update(&id, updated, &i18n).unwrap();

    assert_eq!(store.get(&id).unwrap().title, "updated");
}

#[test]
fn update_nonexistent_entry_fails() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");
    let id = MemoryId::new();
    let entry = make_entry(&i18n, "test", MemoryType::Warning);

    let result = store.update(&id, entry, &i18n);
    assert!(result.is_err());
}

#[test]
fn remove_entry() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");
    let entry = make_entry(&i18n, "to-remove", MemoryType::Warning);
    let id = store.insert(entry, &i18n).unwrap();

    let removed = store.remove(&id);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().title, "to-remove");
    assert!(store.is_empty());
}

#[test]
fn query_with_ordering_by_confidence() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");

    store
        .insert(
            make_entry_with_confidence(&i18n, "low", MemoryType::Warning, 0.3),
            &i18n,
        )
        .unwrap();
    store
        .insert(
            make_entry_with_confidence(&i18n, "high", MemoryType::Warning, 0.9),
            &i18n,
        )
        .unwrap();
    store
        .insert(
            make_entry_with_confidence(&i18n, "medium", MemoryType::Warning, 0.6),
            &i18n,
        )
        .unwrap();

    let query = MemoryQuery::new().order_by(QueryOrder::Confidence);
    let results = store.query(&query);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].title, "high");
    assert_eq!(results[1].title, "medium");
    assert_eq!(results[2].title, "low");
}

#[test]
fn query_with_limit() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");

    for i in 0..5 {
        store
            .insert(
                make_entry(&i18n, &format!("entry-{i}"), MemoryType::CodebaseFact),
                &i18n,
            )
            .unwrap();
    }

    let query = MemoryQuery::new().limit(2);
    let results = store.query(&query);
    assert_eq!(results.len(), 2);
}

#[test]
fn entries_by_type() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");

    store
        .insert(make_entry(&i18n, "w1", MemoryType::Warning), &i18n)
        .unwrap();
    store
        .insert(make_entry(&i18n, "w2", MemoryType::Warning), &i18n)
        .unwrap();
    store
        .insert(make_entry(&i18n, "f1", MemoryType::CodebaseFact), &i18n)
        .unwrap();

    let warnings = store.entries_by_type(MemoryType::Warning);
    assert_eq!(warnings.len(), 2);

    let facts = store.entries_by_type(MemoryType::CodebaseFact);
    assert_eq!(facts.len(), 1);

    let decisions = store.entries_by_type(MemoryType::Decision);
    assert_eq!(decisions.len(), 0);
}

#[test]
fn clear_removes_all() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");

    store
        .insert(make_entry(&i18n, "e1", MemoryType::Warning), &i18n)
        .unwrap();
    store
        .insert(make_entry(&i18n, "e2", MemoryType::Warning), &i18n)
        .unwrap();

    assert_eq!(store.len(), 2);
    store.clear();
    assert!(store.is_empty());
}

#[test]
fn stats_computation() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");

    store
        .insert(
            make_entry_with_confidence(&i18n, "f1", MemoryType::CodebaseFact, 0.8),
            &i18n,
        )
        .unwrap();
    store
        .insert(
            make_entry_with_confidence(&i18n, "w1", MemoryType::Warning, 0.6),
            &i18n,
        )
        .unwrap();

    let stats = store.stats();
    assert_eq!(stats.total_entries, 2);
    assert_eq!(*stats.by_type.get(&MemoryType::CodebaseFact).unwrap(), 1);
    assert_eq!(*stats.by_type.get(&MemoryType::Warning).unwrap(), 1);
    assert!((stats.avg_confidence - 0.7).abs() < f64::EPSILON);
    assert!(stats.oldest.is_some());
    assert!(stats.newest.is_some());
}

#[test]
fn stats_empty_store() {
    let store = MemoryStore::new("test");
    let stats = store.stats();
    assert_eq!(stats.total_entries, 0);
    assert!(stats.by_type.is_empty());
    assert!((stats.avg_confidence - 0.0).abs() < f64::EPSILON);
    assert!(stats.oldest.is_none());
    assert!(stats.newest.is_none());
}

#[test]
fn decay_confidence() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");

    let id = store
        .insert(
            make_entry_with_confidence(&i18n, "test", MemoryType::Warning, 1.0),
            &i18n,
        )
        .unwrap();

    store.decay_confidence(0.1);
    let entry = store.get(&id).unwrap();
    assert!((entry.confidence - 0.9).abs() < f64::EPSILON);
}

#[test]
fn boost_confidence_clamped() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");

    let id = store
        .insert(
            make_entry_with_confidence(&i18n, "test", MemoryType::Warning, 0.8),
            &i18n,
        )
        .unwrap();

    store.boost_confidence(&id, 0.5);
    let entry = store.get(&id).unwrap();
    assert!((entry.confidence - 1.0).abs() < f64::EPSILON);
}

#[test]
fn get_mut_allows_modification() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");
    let id = store
        .insert(make_entry(&i18n, "original", MemoryType::Warning), &i18n)
        .unwrap();

    if let Some(entry) = store.get_mut(&id) {
        entry.title = "modified".to_owned();
    }

    assert_eq!(store.get(&id).unwrap().title, "modified");
}

#[test]
fn all_entries_returns_all() {
    let i18n = test_i18n();
    let mut store = MemoryStore::new("test");

    store
        .insert(make_entry(&i18n, "e1", MemoryType::Warning), &i18n)
        .unwrap();
    store
        .insert(make_entry(&i18n, "e2", MemoryType::CodebaseFact), &i18n)
        .unwrap();
    store
        .insert(make_entry(&i18n, "e3", MemoryType::Decision), &i18n)
        .unwrap();

    assert_eq!(store.all_entries().len(), 3);
}

//! Tests for file-based persistence.

use mahalaxmi_memory::*;
use std::path::PathBuf;
use tempfile::TempDir;

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

fn make_store_with_entries(i18n: &I18nService) -> MemoryStore {
    let mut store = MemoryStore::new("test-session");
    store
        .insert(
            MemoryEntryBuilder::new(
                MemoryType::CodebaseFact,
                "Project uses Rust",
                "The project is a Rust workspace",
                MemorySource::System,
            )
            .tags(vec!["rust".into()])
            .build(i18n)
            .unwrap(),
            i18n,
        )
        .unwrap();
    store
        .insert(
            MemoryEntryBuilder::new(
                MemoryType::Warning,
                "No unwrap in prod",
                "Never use unwrap outside of tests",
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
}

fn store_path(dir: &TempDir) -> PathBuf {
    dir.path().join("test-session.json")
}

#[tokio::test]
async fn save_creates_file() {
    let i18n = test_i18n();
    let dir = TempDir::new().unwrap();
    let path = store_path(&dir);
    let store = make_store_with_entries(&i18n);
    let persistence = FileMemoryPersistence::new();

    persistence.save(&store, &path, &i18n).await.unwrap();
    assert!(path.exists());
}

#[tokio::test]
async fn save_creates_parent_directories() {
    let i18n = test_i18n();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nested").join("dir").join("store.json");
    let store = MemoryStore::new("test");
    let persistence = FileMemoryPersistence::new();

    persistence.save(&store, &path, &i18n).await.unwrap();
    assert!(path.exists());
}

#[tokio::test]
async fn save_and_load_roundtrip() {
    let i18n = test_i18n();
    let dir = TempDir::new().unwrap();
    let path = store_path(&dir);
    let store = make_store_with_entries(&i18n);
    let persistence = FileMemoryPersistence::new();

    persistence.save(&store, &path, &i18n).await.unwrap();
    let loaded = persistence.load(&path, &i18n).await.unwrap();

    assert_eq!(loaded.session_id(), store.session_id());
    assert_eq!(loaded.len(), store.len());
}

#[tokio::test]
async fn load_nonexistent_returns_error() {
    let i18n = test_i18n();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.json");
    let persistence = FileMemoryPersistence::new();

    let result = persistence.load(&path, &i18n).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn load_corrupt_file_returns_error() {
    let i18n = test_i18n();
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("corrupt.json");
    tokio::fs::write(&path, "{ invalid json").await.unwrap();
    let persistence = FileMemoryPersistence::new();

    let result = persistence.load(&path, &i18n).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn exists_true_when_file_present() {
    let i18n = test_i18n();
    let dir = TempDir::new().unwrap();
    let path = store_path(&dir);
    let store = MemoryStore::new("test");
    let persistence = FileMemoryPersistence::new();

    persistence.save(&store, &path, &i18n).await.unwrap();
    assert!(persistence.exists(&path).await);
}

#[tokio::test]
async fn exists_false_when_no_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("missing.json");
    let persistence = FileMemoryPersistence::new();

    assert!(!persistence.exists(&path).await);
}

#[tokio::test]
async fn delete_removes_file() {
    let i18n = test_i18n();
    let dir = TempDir::new().unwrap();
    let path = store_path(&dir);
    let store = MemoryStore::new("test");
    let persistence = FileMemoryPersistence::new();

    persistence.save(&store, &path, &i18n).await.unwrap();
    assert!(path.exists());

    persistence.delete(&path, &i18n).await.unwrap();
    assert!(!path.exists());
}

#[tokio::test]
async fn list_sessions() {
    let i18n = test_i18n();
    let dir = TempDir::new().unwrap();
    let persistence = FileMemoryPersistence::new();

    for name in ["alpha", "beta", "gamma"] {
        let store = MemoryStore::new(name);
        let path = dir.path().join(format!("{name}.json"));
        persistence.save(&store, &path, &i18n).await.unwrap();
    }

    // Add a non-JSON file to verify filtering
    tokio::fs::write(dir.path().join("ignore.txt"), "not json")
        .await
        .unwrap();

    let sessions = persistence.list_sessions(dir.path()).await;
    assert_eq!(sessions.len(), 3);
    assert!(sessions.contains(&"alpha".to_owned()));
    assert!(sessions.contains(&"beta".to_owned()));
    assert!(sessions.contains(&"gamma".to_owned()));
}

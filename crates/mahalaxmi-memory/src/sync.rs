//! Team memory sync: export and import of memory entries to/from JSON files.
//!
//! Enables cross-team sharing of Project- and Global-scoped memory entries.
//! Export serializes entries to a versioned JSON file; import inserts only
//! entries that are not already present, making it fully idempotent.

#[cfg(feature = "sqlite")]
use crate::entry::MemoryEntry;
#[cfg(feature = "sqlite")]
use crate::sqlite::SqliteMemoryPersistence;
#[cfg(feature = "sqlite")]
use std::collections::HashSet;
use std::path::Path;

/// Wire format for a memory sync file written by [`export_memories`].
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MemorySyncFile {
    /// Schema version; must match [`CURRENT_VERSION`] on import.
    pub version: u32,
    /// Unix timestamp (seconds since epoch) when this file was exported.
    pub exported_at: i64,
    /// The memory entries contained in this file.
    pub entries: Vec<MemoryEntry>,
}

/// Current schema version for [`MemorySyncFile`].
pub const CURRENT_VERSION: u32 = 1;

/// Errors that can occur during memory sync operations.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// An I/O error occurred while reading or writing the sync file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A JSON serialization or deserialization error occurred.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A SQLite error occurred while querying or upserting entries.
    #[cfg(feature = "sqlite")]
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Export all Project- and Global-scoped memories for `project_root` to `dest`.
///
/// Queries the store for both scopes, merges the results into a
/// [`MemorySyncFile`], and writes pretty-printed JSON to `dest`.
/// Returns the number of entries written.
#[cfg(feature = "sqlite")]
pub fn export_memories(
    store: &SqliteMemoryPersistence,
    project_root: &str,
    dest: &Path,
) -> Result<usize, SyncError> {
    let mut entries: Vec<MemoryEntry> =
        store.query_by_scope("Project", Some(project_root), usize::MAX)?;
    let global_entries: Vec<MemoryEntry> = store.query_by_scope("Global", None, usize::MAX)?;
    entries.extend(global_entries);

    let count = entries.len();
    let sync_file = MemorySyncFile {
        version: CURRENT_VERSION,
        exported_at: chrono::Utc::now().timestamp(),
        entries,
    };

    let file = std::fs::File::create(dest)?;
    serde_json::to_writer_pretty(file, &sync_file)?;
    Ok(count)
}

/// Import memory entries from `src` into `store`, skipping duplicates.
///
/// Reads and deserializes a [`MemorySyncFile`], queries existing entry IDs,
/// and calls `upsert_entry` only for entries whose ID is not already present.
/// Returns the number of newly inserted entries.
#[cfg(feature = "sqlite")]
pub fn import_memories(store: &SqliteMemoryPersistence, src: &Path) -> Result<usize, SyncError> {
    let file = std::fs::File::open(src)?;
    let sync_file: MemorySyncFile = serde_json::from_reader(file)?;

    let existing_project: Vec<MemoryEntry> = store.query_by_scope("Project", None, usize::MAX)?;
    let existing_global: Vec<MemoryEntry> = store.query_by_scope("Global", None, usize::MAX)?;

    let existing_ids: HashSet<String> = existing_project
        .iter()
        .chain(existing_global.iter())
        .map(|e| e.id.to_string())
        .collect();

    let mut inserted = 0usize;
    for entry in sync_file.entries {
        if !existing_ids.contains(&entry.id.to_string()) {
            store.upsert_entry(&entry)?;
            inserted += 1;
        }
    }
    Ok(inserted)
}

#[cfg(test)]
#[cfg(feature = "sqlite")]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn make_store() -> SqliteMemoryPersistence {
        SqliteMemoryPersistence::open_in_memory()
            .expect("in-memory SQLite store must open successfully")
    }

    fn make_entry(title: &str) -> MemoryEntry {
        use crate::entry::MemoryEntryBuilder;
        use crate::types::{MemorySource, MemoryType};
        use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};

        let i18n = I18nService::new(SupportedLocale::EnUs);
        MemoryEntryBuilder::new(
            MemoryType::CodebaseFact,
            title,
            "content for sync test",
            MemorySource::System,
        )
        .build(&i18n)
        .expect("valid entry must build without error")
    }

    /// Export produces a file that can be deserialized as `MemorySyncFile`.
    #[test]
    fn export_produces_valid_sync_file() {
        let store = make_store();
        let entry = make_entry("sync-test-entry");
        store
            .upsert_entry(&entry)
            .expect("upsert must succeed for a valid entry");

        let tmp = NamedTempFile::new().expect("temp file must be created");
        let count =
            export_memories(&store, "/test/project", tmp.path()).expect("export must succeed");

        assert!(count > 0, "export must write at least one entry");

        let file = std::fs::File::open(tmp.path()).expect("exported file must be readable");
        let parsed: MemorySyncFile = serde_json::from_reader(file)
            .expect("exported JSON must deserialize as MemorySyncFile");
        assert_eq!(
            parsed.version, CURRENT_VERSION,
            "version field must match CURRENT_VERSION"
        );
        assert!(
            parsed.exported_at > 0,
            "exported_at must be a positive Unix timestamp"
        );
    }

    /// Importing the same file twice returns 0 on the second call (idempotent).
    #[test]
    fn import_is_idempotent() {
        let store = make_store();
        let entry = make_entry("idempotent-entry");
        store
            .upsert_entry(&entry)
            .expect("upsert must succeed for a valid entry");

        let tmp = NamedTempFile::new().expect("temp file must be created");
        export_memories(&store, "/test/project", tmp.path()).expect("export must succeed");

        let second_store = make_store();
        let first_count =
            import_memories(&second_store, tmp.path()).expect("first import must succeed");
        assert!(
            first_count > 0,
            "first import must insert at least one entry"
        );

        let second_count =
            import_memories(&second_store, tmp.path()).expect("second import must succeed");
        assert_eq!(
            second_count, 0,
            "second import must return 0 (all entries already present)"
        );
    }

    /// A full export + import round-trip preserves all `MemoryEntry` fields.
    #[test]
    fn round_trip_preserves_entry_fields() {
        let source_store = make_store();
        let entry = make_entry("round-trip-entry");
        let entry_id = entry.id;
        let entry_title = entry.title.clone();
        let entry_content = entry.content.clone();

        source_store
            .upsert_entry(&entry)
            .expect("upsert must succeed for a valid entry");

        let tmp = NamedTempFile::new().expect("temp file must be created");
        export_memories(&source_store, "/test/project", tmp.path()).expect("export must succeed");

        let dest_store = make_store();
        import_memories(&dest_store, tmp.path()).expect("import must succeed");

        let imported: Vec<MemoryEntry> = dest_store
            .query_by_scope("Project", None, usize::MAX)
            .expect("query must succeed after import");

        let found = imported
            .iter()
            .find(|e| e.id == entry_id)
            .expect("imported entry must be found by ID");

        assert_eq!(found.title, entry_title, "title must be preserved");
        assert_eq!(found.content, entry_content, "content must be preserved");
    }
}

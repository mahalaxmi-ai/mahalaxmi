//! SQLite-backed memory persistence for cross-session organisational memory.
//!
//! All types in this module are gated behind the `sqlite` feature flag. Enable
//! it by adding `mahalaxmi-memory = { features = ["sqlite"] }` to your
//! `Cargo.toml`.

use crate::entry::{MemoryEntry, MemoryMetadata, MemoryScope};
use crate::types::{MemoryId, MemorySource, MemoryType};
use chrono::{TimeZone, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;

/// SQL DDL that creates the persistence schema.
///
/// All statements use `IF NOT EXISTS` so that applying the DDL twice on the
/// same connection is a no-op.
const SCHEMA_DDL: &str = "
CREATE TABLE IF NOT EXISTS memories (
    id               TEXT NOT NULL PRIMARY KEY,
    title            TEXT NOT NULL DEFAULT '',
    content          TEXT NOT NULL DEFAULT '',
    memory_type      TEXT NOT NULL DEFAULT '',
    source           TEXT NOT NULL DEFAULT '',
    scope            TEXT NOT NULL DEFAULT 'Session',
    project_root     TEXT,
    tags             TEXT NOT NULL DEFAULT '[]',
    confidence       REAL NOT NULL DEFAULT 0.0,
    cycle_id         TEXT,
    created_at       INTEGER NOT NULL DEFAULT 0,
    updated_at       INTEGER NOT NULL DEFAULT 0,
    last_accessed_at INTEGER NOT NULL DEFAULT 0,
    access_count     INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS memories_archive (
    id               TEXT NOT NULL PRIMARY KEY,
    title            TEXT NOT NULL DEFAULT '',
    content          TEXT NOT NULL DEFAULT '',
    memory_type      TEXT NOT NULL DEFAULT '',
    source           TEXT NOT NULL DEFAULT '',
    scope            TEXT NOT NULL DEFAULT 'Session',
    project_root     TEXT,
    tags             TEXT NOT NULL DEFAULT '[]',
    confidence       REAL NOT NULL DEFAULT 0.0,
    cycle_id         TEXT,
    created_at       INTEGER NOT NULL DEFAULT 0,
    updated_at       INTEGER NOT NULL DEFAULT 0,
    last_accessed_at INTEGER NOT NULL DEFAULT 0,
    access_count     INTEGER NOT NULL DEFAULT 0,
    archived_at      INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_memories_project    ON memories (project_root, scope);
CREATE INDEX IF NOT EXISTS idx_memories_confidence ON memories (confidence);
CREATE INDEX IF NOT EXISTS idx_memories_accessed   ON memories (last_accessed_at);
";

/// The column list used by every SELECT query that reconstructs a [`MemoryEntry`].
const SELECT_COLS: &str =
    "id, title, content, memory_type, source, tags, confidence, cycle_id, created_at, updated_at, scope";

/// Convert a [`MemoryScope`] to its database string representation.
///
/// - `Session`     → `"Session"`
/// - `Project`     → `"Project"`
/// - `Global`      → `"Global"`
/// - `Team(id)`    → `"team:{id}"`
fn scope_to_db_str(scope: &MemoryScope) -> String {
    match scope {
        MemoryScope::Session => "Session".to_string(),
        MemoryScope::Project => "Project".to_string(),
        MemoryScope::Global => "Global".to_string(),
        MemoryScope::Team(id) => format!("team:{id}"),
    }
}

/// Parse a database scope string back into a [`MemoryScope`].
///
/// Accepts both lower- and Pascal-case legacy values for backward compatibility
/// with entries written before this encoding was formalised.  Unknown strings
/// fall back to [`MemoryScope::Session`].
fn scope_from_db_str(s: &str) -> MemoryScope {
    match s {
        "Session" | "session" => MemoryScope::Session,
        "Project" | "project" => MemoryScope::Project,
        "Global" | "global" => MemoryScope::Global,
        other if other.starts_with("team:") => MemoryScope::Team(other[5..].to_string()),
        _ => MemoryScope::Session,
    }
}

/// SQLite-backed persistent store for cross-session memory entries.
///
/// Opens (or creates) a SQLite database file and applies the schema on first
/// use.  All read operations take `&self`; the underlying `rusqlite::Connection`
/// uses SQLite's default serialised threading mode which makes the handle safe
/// to move across threads.
pub struct SqliteMemoryPersistence {
    conn: Connection,
}

impl SqliteMemoryPersistence {
    /// Open or create a SQLite database file at `path`.
    ///
    /// The schema is applied immediately (idempotent due to `IF NOT EXISTS`).
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.conn.execute_batch(SCHEMA_DDL)?;
        Ok(store)
    }

    /// Open an in-memory SQLite database.
    ///
    /// Useful for unit tests — the database is discarded when the handle is
    /// dropped.
    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.conn.execute_batch(SCHEMA_DDL)?;
        Ok(store)
    }

    /// Re-apply the schema DDL to the connection.
    ///
    /// Because every statement uses `IF NOT EXISTS` this is safe to call any
    /// number of times on the same connection.
    #[cfg(test)]
    pub(crate) fn apply_schema(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(SCHEMA_DDL)
    }

    /// Insert or replace a memory entry.
    ///
    /// The scope string (`"Session"`, `"Project"`, `"Global"`, or `"team:{id}"`)
    /// and the optional `project_root` are derived from the entry's own `scope`
    /// field and `metadata.custom["project_root"]` value respectively.
    /// `tags` and `source` are serialised to JSON text; timestamps are stored
    /// as Unix seconds (i64).
    pub fn upsert_entry(&self, entry: &MemoryEntry) -> Result<(), rusqlite::Error> {
        let scope_str = scope_to_db_str(&entry.scope);
        let project_root = entry
            .metadata
            .custom
            .get("project_root")
            .map(String::as_str);

        let tags_json = serde_json::to_string(&entry.tags)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let source_json = serde_json::to_string(&entry.source)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let memory_type_json = serde_json::to_string(&entry.memory_type)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let created_ts = entry.created_at.timestamp();
        let updated_ts = entry.updated_at.timestamp();

        self.conn.execute(
            "INSERT OR REPLACE INTO memories \
             (id, title, content, memory_type, source, scope, project_root, tags, \
              confidence, cycle_id, created_at, updated_at, last_accessed_at, access_count) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?11, 0)",
            params![
                entry.id.to_string(),
                entry.title,
                entry.content,
                memory_type_json,
                source_json,
                scope_str,
                project_root,
                tags_json,
                entry.confidence,
                entry.cycle_id,
                created_ts,
                updated_ts,
            ],
        )?;
        Ok(())
    }

    /// Return up to `limit` entries for the given scope, ordered by confidence
    /// descending.
    ///
    /// When `project_root` is `None` the query matches rows where
    /// `project_root IS NULL`.
    pub fn query_by_scope(
        &self,
        scope: &str,
        project_root: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, rusqlite::Error> {
        let limit_i64: i64 = i64::try_from(limit).unwrap_or(i64::MAX);

        match project_root {
            Some(root) => {
                let mut stmt = self.conn.prepare(&format!(
                    "SELECT {SELECT_COLS} FROM memories \
                     WHERE scope=?1 AND project_root=?2 \
                     ORDER BY confidence DESC LIMIT ?3"
                ))?;
                let rows = stmt.query_map(params![scope, root, limit_i64], Self::row_to_entry)?;
                rows.collect()
            }
            None => {
                let mut stmt = self.conn.prepare(&format!(
                    "SELECT {SELECT_COLS} FROM memories \
                     WHERE scope=?1 AND project_root IS NULL \
                     ORDER BY confidence DESC LIMIT ?2"
                ))?;
                let rows = stmt.query_map(params![scope, limit_i64], Self::row_to_entry)?;
                rows.collect()
            }
        }
    }

    /// Increment the access count and refresh the last-accessed timestamp for
    /// the entry identified by `id`.
    pub fn record_access(&self, id: &str) -> Result<(), rusqlite::Error> {
        let now = Utc::now().timestamp();
        self.conn.execute(
            "UPDATE memories \
             SET access_count = access_count + 1, last_accessed_at = ?1 \
             WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    /// Move stale entries from `memories` to `memories_archive`.
    ///
    /// An entry is considered stale when its `last_accessed_at` timestamp is
    /// more than `threshold_secs` seconds in the past (i.e.
    /// `last_accessed_at < now - threshold_secs`).  The operation runs inside
    /// an explicit transaction for atomicity.  Returns the number of entries
    /// that were archived.
    pub fn archive_stale(
        &self,
        scope: &str,
        threshold_secs: i64,
    ) -> Result<usize, rusqlite::Error> {
        let cutoff = Utc::now().timestamp() - threshold_secs;
        let archived_at = Utc::now().timestamp();

        self.conn.execute_batch("BEGIN")?;

        let insert_result = self.conn.execute(
            "INSERT OR REPLACE INTO memories_archive \
             (id, title, content, memory_type, source, scope, project_root, tags, \
              confidence, cycle_id, created_at, updated_at, last_accessed_at, \
              access_count, archived_at) \
             SELECT id, title, content, memory_type, source, scope, project_root, tags, \
                    confidence, cycle_id, created_at, updated_at, last_accessed_at, \
                    access_count, ?1 \
             FROM memories \
             WHERE scope=?2 AND last_accessed_at < ?3",
            params![archived_at, scope, cutoff],
        );

        if let Err(e) = insert_result {
            let _ = self.conn.execute_batch("ROLLBACK");
            return Err(e);
        }

        let delete_result = self.conn.execute(
            "DELETE FROM memories WHERE scope=?1 AND last_accessed_at < ?2",
            params![scope, cutoff],
        );

        match delete_result {
            Ok(count) => {
                self.conn.execute_batch("COMMIT")?;
                Ok(count)
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    /// Deserialise a single database row into a [`MemoryEntry`].
    ///
    /// Column order must match [`SELECT_COLS`]:
    /// 0=id, 1=title, 2=content, 3=memory_type, 4=source, 5=tags,
    /// 6=confidence, 7=cycle_id, 8=created_at, 9=updated_at, 10=scope.
    fn row_to_entry(row: &rusqlite::Row<'_>) -> Result<MemoryEntry, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let uuid = Uuid::parse_str(&id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;
        let id = MemoryId::from_uuid(uuid);

        let title: String = row.get(1)?;
        let content: String = row.get(2)?;

        let memory_type_str: String = row.get(3)?;
        let memory_type: MemoryType = serde_json::from_str(&memory_type_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let source_str: String = row.get(4)?;
        let source: MemorySource = serde_json::from_str(&source_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let tags_str: String = row.get(5)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let confidence: f64 = row.get(6)?;
        let cycle_id: Option<String> = row.get(7)?;

        let created_ts: i64 = row.get(8)?;
        let created_at = Utc.timestamp_opt(created_ts, 0).single().ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                8,
                rusqlite::types::Type::Integer,
                format!("invalid timestamp: {created_ts}").into(),
            )
        })?;

        let updated_ts: i64 = row.get(9)?;
        let updated_at = Utc.timestamp_opt(updated_ts, 0).single().ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                9,
                rusqlite::types::Type::Integer,
                format!("invalid timestamp: {updated_ts}").into(),
            )
        })?;

        let scope_str: String = row.get(10).unwrap_or_else(|_| "Session".to_string());
        let scope = scope_from_db_str(&scope_str);

        Ok(MemoryEntry {
            id,
            title,
            content,
            memory_type,
            source,
            tags,
            confidence,
            cycle_id,
            created_at,
            updated_at,
            metadata: MemoryMetadata::default(),
            scope,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::MemoryEntryBuilder;
    use crate::types::{MemorySource, MemoryType};
    use chrono::Duration;

    fn make_entry() -> MemoryEntry {
        MemoryEntry {
            id: MemoryId::new(),
            memory_type: MemoryType::CodebaseFact,
            title: "Test Memory".to_owned(),
            content: "Some content".to_owned(),
            confidence: 0.85,
            source: MemorySource::System,
            tags: vec!["rust".to_owned(), "testing".to_owned()],
            cycle_id: Some("cycle-123".to_owned()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: MemoryMetadata::default(),
            scope: MemoryScope::Project,
        }
    }

    fn make_project_entry(project_root: &str) -> MemoryEntry {
        let mut entry = make_entry();
        entry
            .metadata
            .custom
            .insert("project_root".to_string(), project_root.to_string());
        entry
    }

    #[test]
    fn open_in_memory_creates_tables() {
        let store = SqliteMemoryPersistence::open_in_memory().unwrap();
        let r1: Result<i64, _> = store
            .conn
            .query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0));
        let r2: Result<i64, _> =
            store
                .conn
                .query_row("SELECT COUNT(*) FROM memories_archive", [], |row| {
                    row.get(0)
                });
        assert!(r1.is_ok(), "memories table should exist");
        assert!(r2.is_ok(), "memories_archive table should exist");
    }

    #[test]
    fn upsert_then_query_returns_entry() {
        let store = SqliteMemoryPersistence::open_in_memory().unwrap();
        let entry = make_project_entry("myproject");
        store.upsert_entry(&entry).unwrap();

        let results = store
            .query_by_scope("Project", Some("myproject"), 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Memory");
        assert!((results[0].confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn record_access_increments_count_to_one() {
        let store = SqliteMemoryPersistence::open_in_memory().unwrap();
        let entry = make_project_entry("myproject");
        let id_str = entry.id.to_string();
        store.upsert_entry(&entry).unwrap();

        store.record_access(&id_str).unwrap();

        let count: i64 = store
            .conn
            .query_row(
                "SELECT access_count FROM memories WHERE id=?1",
                params![id_str],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn archive_stale_moves_entry_and_leaves_none_with_large_threshold() {
        let store = SqliteMemoryPersistence::open_in_memory().unwrap();

        // Insert entry with timestamps 2 seconds in the past so that
        // archive_stale(0) (cutoff = now) will find it.
        let mut entry = make_project_entry("myproject");
        let past = Utc::now() - Duration::seconds(2);
        entry.created_at = past;
        entry.updated_at = past;
        let id_str = entry.id.to_string();
        store.upsert_entry(&entry).unwrap();

        // Large threshold (10 years) → nothing old enough to archive.
        let count_no_archive = store.archive_stale("Project", 315_360_000).unwrap();
        assert_eq!(
            count_no_archive, 0,
            "10-year threshold should archive 0 entries"
        );

        // threshold=0 → cutoff=now, entry last_accessed_at=now-2 < now → archived.
        let count = store.archive_stale("Project", 0).unwrap();
        assert_eq!(count, 1, "threshold=0 should archive the stale entry");

        let main_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE id=?1",
                params![id_str],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(main_count, 0, "entry must be absent from memories");

        let archive_count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM memories_archive WHERE id=?1",
                params![id_str],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            archive_count, 1,
            "entry must be present in memories_archive"
        );
    }

    #[test]
    fn schema_application_is_idempotent() {
        let store = SqliteMemoryPersistence::open_in_memory().unwrap();
        // Applying the schema a second time must not return an error.
        store.apply_schema().unwrap();
    }

    #[test]
    fn scope_to_db_str_roundtrip() {
        let cases = [
            (MemoryScope::Session, "Session"),
            (MemoryScope::Project, "Project"),
            (MemoryScope::Global, "Global"),
            (
                MemoryScope::Team("team-alpha".to_string()),
                "team:team-alpha",
            ),
        ];
        for (scope, expected) in &cases {
            assert_eq!(scope_to_db_str(scope), *expected);
            assert_eq!(scope_from_db_str(expected), *scope);
        }
    }

    #[test]
    fn scope_from_db_str_accepts_legacy_lowercase() {
        assert_eq!(scope_from_db_str("session"), MemoryScope::Session);
        assert_eq!(scope_from_db_str("project"), MemoryScope::Project);
        assert_eq!(scope_from_db_str("global"), MemoryScope::Global);
    }

    #[test]
    fn scope_from_db_str_unknown_falls_back_to_session() {
        assert_eq!(scope_from_db_str("unknown_value"), MemoryScope::Session);
    }

    /// Store a [`MemoryScope::Team`] entry and retrieve it; the deserialized
    /// scope must match the original team ID exactly.
    #[test]
    fn test_team_memory_scope_round_trip() {
        let store = SqliteMemoryPersistence::open_in_memory().unwrap();

        let i18n = {
            use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
            I18nService::new(SupportedLocale::EnUs)
        };

        let entry = MemoryEntryBuilder::new(
            MemoryType::CodebaseFact,
            "team scoped entry",
            "content for team scope test",
            MemorySource::System,
        )
        .scope(MemoryScope::Team("team-alpha".into()))
        .build(&i18n)
        .expect("valid entry must build");

        store.upsert_entry(&entry).expect("upsert must succeed");

        let results = store
            .query_by_scope("team:team-alpha", None, 10)
            .expect("query must succeed");

        assert_eq!(results.len(), 1, "exactly one team-alpha entry expected");
        assert_eq!(
            results[0].scope,
            MemoryScope::Team("team-alpha".into()),
            "retrieved scope must match the stored Team scope"
        );
    }

    /// A team-scoped entry must not appear when querying by Project scope.
    #[test]
    fn test_team_scope_does_not_match_project_scope() {
        let store = SqliteMemoryPersistence::open_in_memory().unwrap();

        let i18n = {
            use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
            I18nService::new(SupportedLocale::EnUs)
        };

        let team_entry = MemoryEntryBuilder::new(
            MemoryType::CodebaseFact,
            "team entry",
            "content",
            MemorySource::System,
        )
        .scope(MemoryScope::Team("team-alpha".into()))
        .build(&i18n)
        .expect("valid entry must build");

        store
            .upsert_entry(&team_entry)
            .expect("upsert must succeed");

        let project_results = store
            .query_by_scope("Project", None, 10)
            .expect("Project query must succeed");

        assert_eq!(
            project_results.len(),
            0,
            "team-scoped entry must not appear in Project scope query"
        );
    }
}

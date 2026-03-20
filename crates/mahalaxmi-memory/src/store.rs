//! In-memory store for shared memory entries.

use crate::entry::{MemoryEntry, MemoryScope};
use crate::query::{MemoryQuery, QueryOrder};
use crate::types::{MemoryId, MemoryType};
use chrono::{DateTime, Utc};
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics about the current state of the memory store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of entries.
    pub total_entries: usize,
    /// Count of entries by type.
    pub by_type: HashMap<MemoryType, usize>,
    /// Average confidence across all entries.
    pub avg_confidence: f64,
    /// Timestamp of the oldest entry.
    pub oldest: Option<DateTime<Utc>>,
    /// Timestamp of the newest entry.
    pub newest: Option<DateTime<Utc>>,
}

/// An in-memory store for cross-agent shared memory entries.
///
/// Each store belongs to a single session and enforces a maximum entry count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    entries: HashMap<MemoryId, MemoryEntry>,
    session_id: String,
    created_at: DateTime<Utc>,
    max_entries: usize,
}

impl MemoryStore {
    /// Create a new memory store for the given session with default max entries (1000).
    pub fn new(session_id: impl Into<String>) -> Self {
        Self::with_max_entries(session_id, 1000)
    }

    /// Create a new memory store with a custom max entries limit.
    pub fn with_max_entries(session_id: impl Into<String>, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            session_id: session_id.into(),
            created_at: Utc::now(),
            max_entries,
        }
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Insert a new entry into the store.
    ///
    /// Returns the assigned `MemoryId` on success.
    pub fn insert(&mut self, entry: MemoryEntry, i18n: &I18nService) -> MahalaxmiResult<MemoryId> {
        if self.entries.len() >= self.max_entries {
            let max_str = self.max_entries.to_string();
            return Err(MahalaxmiError::memory(
                i18n,
                keys::memory::STORE_FULL,
                &[("max", &max_str)],
            ));
        }

        let id = entry.id;
        self.entries.insert(id, entry);
        Ok(id)
    }

    /// Get an entry by ID.
    pub fn get(&self, id: &MemoryId) -> Option<&MemoryEntry> {
        self.entries.get(id)
    }

    /// Get a mutable reference to an entry by ID.
    pub fn get_mut(&mut self, id: &MemoryId) -> Option<&mut MemoryEntry> {
        self.entries.get_mut(id)
    }

    /// Update an existing entry.
    pub fn update(
        &mut self,
        id: &MemoryId,
        entry: MemoryEntry,
        i18n: &I18nService,
    ) -> MahalaxmiResult<()> {
        if !self.entries.contains_key(id) {
            let id_str = id.to_string();
            return Err(MahalaxmiError::memory(
                i18n,
                keys::memory::NOT_FOUND,
                &[("id", &id_str)],
            ));
        }
        self.entries.insert(*id, entry);
        Ok(())
    }

    /// Remove an entry by ID.
    pub fn remove(&mut self, id: &MemoryId) -> Option<MemoryEntry> {
        self.entries.remove(id)
    }

    /// Query entries matching the given filters, with ordering and limit applied.
    pub fn query(&self, query: &MemoryQuery) -> Vec<&MemoryEntry> {
        let mut results: Vec<&MemoryEntry> =
            self.entries.values().filter(|e| query.matches(e)).collect();

        match query.order {
            Some(QueryOrder::Confidence) => {
                results.sort_by(|a, b| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            Some(QueryOrder::CreatedAt) => {
                results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            Some(QueryOrder::UpdatedAt) => {
                results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
            Some(QueryOrder::Relevance) => {
                results.sort_by(|a, b| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.created_at.cmp(&a.created_at))
                });
            }
            None => {}
        }

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }

    /// Get all entries of a specific type.
    pub fn entries_by_type(&self, memory_type: MemoryType) -> Vec<&MemoryEntry> {
        self.entries
            .values()
            .filter(|e| e.memory_type == memory_type)
            .collect()
    }

    /// Get all entries.
    pub fn all_entries(&self) -> Vec<&MemoryEntry> {
        self.entries.values().collect()
    }

    /// Remove all entries from the store.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Compute aggregate statistics about the store.
    pub fn stats(&self) -> MemoryStats {
        let total_entries = self.entries.len();

        let mut by_type: HashMap<MemoryType, usize> = HashMap::new();
        let mut sum_confidence = 0.0;
        let mut oldest: Option<DateTime<Utc>> = None;
        let mut newest: Option<DateTime<Utc>> = None;

        for entry in self.entries.values() {
            *by_type.entry(entry.memory_type).or_insert(0) += 1;
            sum_confidence += entry.confidence;

            match oldest {
                Some(ref o) if entry.created_at < *o => oldest = Some(entry.created_at),
                None => oldest = Some(entry.created_at),
                _ => {}
            }

            match newest {
                Some(ref n) if entry.created_at > *n => newest = Some(entry.created_at),
                None => newest = Some(entry.created_at),
                _ => {}
            }
        }

        let avg_confidence = if total_entries > 0 {
            sum_confidence / total_entries as f64
        } else {
            0.0
        };

        MemoryStats {
            total_entries,
            by_type,
            avg_confidence,
            oldest,
            newest,
        }
    }

    /// Decay confidence for all entries by the given rate.
    ///
    /// Each entry's confidence is multiplied by `(1.0 - rate)`, clamped to 0.0.
    pub fn decay_confidence(&mut self, rate: f64) {
        let factor = (1.0 - rate).max(0.0);
        for entry in self.entries.values_mut() {
            entry.confidence = (entry.confidence * factor).max(0.0);
        }
    }

    /// Boost confidence for a specific entry.
    ///
    /// The entry's confidence is increased by `boost`, clamped to 1.0.
    pub fn boost_confidence(&mut self, id: &MemoryId, boost: f64) {
        if let Some(entry) = self.entries.get_mut(id) {
            entry.confidence = (entry.confidence + boost).min(1.0);
        }
    }

    /// Attempt to promote a Session-scoped entry to Project scope.
    ///
    /// Returns `true` if promotion occurred. Does NOT persist to SQLite; caller must
    /// call `SqliteMemoryPersistence::upsert_entry` after a successful promotion.
    pub fn maybe_promote(
        &mut self,
        id: &MemoryId,
        project_root: &std::path::Path,
        min_confidence: f64,
        _min_cycles: usize,
    ) -> bool {
        let entry = match self.entries.get_mut(id) {
            Some(e) => e,
            None => return false,
        };
        if entry.scope != MemoryScope::Session {
            return false;
        }
        if entry.confidence < min_confidence {
            return false;
        }
        entry.scope = MemoryScope::Project;
        entry.metadata.custom.insert(
            "project_root".to_string(),
            project_root.to_string_lossy().into_owned(),
        );
        true
    }

    /// Return memories visible in current context.
    ///
    /// Delegates to `query()` in v1; scope-aware multi-backend filtering will be added
    /// in a subsequent iteration.
    pub fn query_contextual(&self, query: &MemoryQuery) -> Vec<&MemoryEntry> {
        self.query(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{MemoryEntryBuilder, MemoryScope};
    use crate::types::MemorySource;
    use mahalaxmi_core::i18n::locale::SupportedLocale;

    fn test_i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    fn make_entry(i18n: &I18nService, title: &str, mt: MemoryType) -> MemoryEntry {
        MemoryEntryBuilder::new(mt, title, "test content", MemorySource::System)
            .build(i18n)
            .unwrap()
    }

    #[test]
    fn empty_store() {
        let store = MemoryStore::new("test-session");
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert_eq!(store.session_id(), "test-session");
    }

    #[test]
    fn insert_and_get() {
        let i18n = test_i18n();
        let mut store = MemoryStore::new("test");
        let entry = make_entry(&i18n, "fact1", MemoryType::CodebaseFact);
        let id = store.insert(entry, &i18n).unwrap();

        assert_eq!(store.len(), 1);
        assert!(store.get(&id).is_some());
        assert_eq!(store.get(&id).unwrap().title, "fact1");
    }

    #[test]
    fn full_store_rejects_insert() {
        let i18n = test_i18n();
        let mut store = MemoryStore::with_max_entries("test", 1);
        let entry1 = make_entry(&i18n, "e1", MemoryType::Warning);
        store.insert(entry1, &i18n).unwrap();

        let entry2 = make_entry(&i18n, "e2", MemoryType::Warning);
        let result = store.insert(entry2, &i18n);
        assert!(result.is_err());
    }

    #[test]
    fn stats_computation() {
        let i18n = test_i18n();
        let mut store = MemoryStore::new("test");
        store
            .insert(make_entry(&i18n, "f1", MemoryType::CodebaseFact), &i18n)
            .unwrap();
        store
            .insert(make_entry(&i18n, "w1", MemoryType::Warning), &i18n)
            .unwrap();

        let stats = store.stats();
        assert_eq!(stats.total_entries, 2);
        assert!(stats.avg_confidence > 0.0);
        assert!(stats.oldest.is_some());
        assert!(stats.newest.is_some());
    }

    #[test]
    fn maybe_promote_returns_false_when_not_found() {
        let mut store = MemoryStore::new("test");
        let missing_id = MemoryId::new();
        let promoted = store.maybe_promote(&missing_id, std::path::Path::new("/project"), 0.5, 1);
        assert!(!promoted);
    }

    #[test]
    fn maybe_promote_returns_false_when_already_project() {
        let i18n = test_i18n();
        let mut store = MemoryStore::new("test");
        let entry = MemoryEntryBuilder::new(
            MemoryType::CodebaseFact,
            "title",
            "content",
            MemorySource::System,
        )
        .confidence(0.9)
        .scope(MemoryScope::Project)
        .build(&i18n)
        .unwrap();
        let id = store.insert(entry, &i18n).unwrap();
        let promoted = store.maybe_promote(&id, std::path::Path::new("/project"), 0.5, 1);
        assert!(!promoted);
    }

    #[test]
    fn maybe_promote_returns_false_when_confidence_too_low() {
        let i18n = test_i18n();
        let mut store = MemoryStore::new("test");
        let entry = MemoryEntryBuilder::new(
            MemoryType::CodebaseFact,
            "title",
            "content",
            MemorySource::System,
        )
        .confidence(0.5)
        .build(&i18n)
        .unwrap();
        let id = store.insert(entry, &i18n).unwrap();
        let promoted = store.maybe_promote(&id, std::path::Path::new("/project"), 0.8, 1);
        assert!(!promoted);
        assert_eq!(store.get(&id).unwrap().scope, MemoryScope::Session);
    }

    #[test]
    fn maybe_promote_returns_true_and_mutates_scope() {
        let i18n = test_i18n();
        let mut store = MemoryStore::new("test");
        let entry = MemoryEntryBuilder::new(
            MemoryType::Decision,
            "title",
            "content",
            MemorySource::System,
        )
        .confidence(0.9)
        .build(&i18n)
        .unwrap();
        let id = store.insert(entry, &i18n).unwrap();
        let promoted = store.maybe_promote(&id, std::path::Path::new("/my/project"), 0.8, 1);
        assert!(promoted);
        let updated = store.get(&id).unwrap();
        assert_eq!(updated.scope, MemoryScope::Project);
        assert_eq!(
            updated
                .metadata
                .custom
                .get("project_root")
                .map(String::as_str),
            Some("/my/project")
        );
    }

    #[test]
    fn query_contextual_matches_query_for_session_entries() {
        let i18n = test_i18n();
        let mut store = MemoryStore::new("test");
        store
            .insert(
                make_entry(&i18n, "session-entry", MemoryType::Convention),
                &i18n,
            )
            .unwrap();

        let query = MemoryQuery::new().with_type(MemoryType::Convention);
        let direct = store.query(&query);
        let contextual = store.query_contextual(&query);

        assert_eq!(direct.len(), contextual.len());
        assert_eq!(direct.len(), 1);
        assert_eq!(contextual[0].title, "session-entry");
    }

    /// Tests for `MemoryScope::Team` — grouped under `team_scope_tests` so that
    /// `cargo test -- team_scope` filters to both tests.
    mod team_scope_tests {
        use super::*;

        /// A [`MemoryScope::Team`] entry stored in the in-memory store must be
        /// retrievable by ID with its scope intact.
        #[test]
        fn test_team_memory_scope_round_trip() {
            let i18n = test_i18n();
            let mut store = MemoryStore::new("test");

            let entry = MemoryEntryBuilder::new(
                MemoryType::CodebaseFact,
                "team scoped entry",
                "content for team scope test",
                MemorySource::System,
            )
            .scope(MemoryScope::Team("team-alpha".into()))
            .build(&i18n)
            .unwrap();

            let id = store.insert(entry, &i18n).unwrap();
            let retrieved = store.get(&id).unwrap();

            assert_eq!(
                retrieved.scope,
                MemoryScope::Team("team-alpha".into()),
                "retrieved scope must match the stored Team scope"
            );
        }

        /// A team-scoped entry must not be treated as a project-scoped entry.
        /// Different team IDs must also be unequal.
        #[test]
        fn test_team_scope_does_not_match_project_scope() {
            let i18n = test_i18n();
            let mut store = MemoryStore::new("test");

            let team_entry = MemoryEntryBuilder::new(
                MemoryType::CodebaseFact,
                "team entry",
                "team content",
                MemorySource::System,
            )
            .scope(MemoryScope::Team("team-alpha".into()))
            .build(&i18n)
            .unwrap();

            let project_entry = MemoryEntryBuilder::new(
                MemoryType::CodebaseFact,
                "project entry",
                "project content",
                MemorySource::System,
            )
            .scope(MemoryScope::Project)
            .build(&i18n)
            .unwrap();

            store.insert(team_entry, &i18n).unwrap();
            store.insert(project_entry, &i18n).unwrap();

            // Only project-scoped entries should appear when filtering by Project scope.
            let project_only: Vec<&MemoryEntry> = store
                .all_entries()
                .into_iter()
                .filter(|e| e.scope == MemoryScope::Project)
                .collect();

            assert_eq!(
                project_only.len(),
                1,
                "exactly one Project-scoped entry expected"
            );
            assert_eq!(project_only[0].title, "project entry");
            assert_ne!(
                project_only[0].scope,
                MemoryScope::Team("team-alpha".into()),
                "project entry scope must not equal Team scope"
            );

            // Distinct team IDs are not equal.
            assert_ne!(
                MemoryScope::Team("team-alpha".into()),
                MemoryScope::Team("team-beta".into()),
                "different team IDs must not be equal"
            );
        }
    }
}

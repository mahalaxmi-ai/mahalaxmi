//! Memory entry types and builder.

use crate::types::{MemoryId, MemorySource, MemoryType};
use chrono::{DateTime, Utc};
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Persistence scope controlling lifetime and storage backend of a memory entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub enum MemoryScope {
    /// Stored in HashMap only. Cleared on application restart.
    #[default]
    Session,
    /// Persisted to SQLite with project_root filter. Archived after 90 days of no access.
    Project,
    /// Persisted to SQLite with NULL project_root. Cross-project. Archived after 365 days.
    Global,
    /// Team-scoped memory shared across all developers on a specific team.
    ///
    /// The `String` payload is the team identifier. Stored in SQLite with scope column
    /// value `team:{team_id}`. Not promoted to Global during `maybe_promote()`. Decays
    /// at the same rate as Project-scoped memory.
    #[serde(rename = "team")]
    Team(String),
}

impl MemoryScope {
    /// Returns the string key used when storing this scope in a SQLite `scope` column.
    ///
    /// Unit scopes map to their PascalCase names; `Team(id)` maps to `"team:{id}"`.
    /// An empty `team_id` string stores as `"team:"` and retrieves the same way.
    pub fn db_key(&self) -> String {
        match self {
            MemoryScope::Session => "Session".to_owned(),
            MemoryScope::Project => "Project".to_owned(),
            MemoryScope::Global => "Global".to_owned(),
            MemoryScope::Team(id) => format!("team:{id}"),
        }
    }
}

/// A single shared memory entry.
///
/// Represents a piece of knowledge discovered or decided during an orchestration cycle
/// that can be shared across agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier.
    pub id: MemoryId,
    /// Classification of this memory.
    pub memory_type: MemoryType,
    /// Short title describing the memory.
    pub title: String,
    /// Full content of the memory.
    pub content: String,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
    /// Who created this memory.
    pub source: MemorySource,
    /// Searchable tags.
    pub tags: Vec<String>,
    /// Associated orchestration cycle, if any.
    pub cycle_id: Option<String>,
    /// When this entry was created.
    pub created_at: DateTime<Utc>,
    /// When this entry was last updated.
    pub updated_at: DateTime<Utc>,
    /// Additional metadata.
    pub metadata: MemoryMetadata,
    /// Persistence scope; defaults to Session for backward-compatibility with existing saved entries.
    #[serde(default)]
    pub scope: MemoryScope,
}

/// Additional metadata for a memory entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// Associated file path.
    pub file_path: Option<String>,
    /// Associated line range (start, end).
    pub line_range: Option<(usize, usize)>,
    /// Programming language.
    pub language: Option<String>,
    /// Related memory entry IDs.
    pub related_entries: Vec<MemoryId>,
    /// Arbitrary key-value metadata.
    pub custom: HashMap<String, String>,
}

/// Builder for constructing `MemoryEntry` instances with validation.
pub struct MemoryEntryBuilder {
    memory_type: MemoryType,
    title: String,
    content: String,
    source: MemorySource,
    confidence: Option<f64>,
    tags: Vec<String>,
    cycle_id: Option<String>,
    metadata: MemoryMetadata,
    scope: MemoryScope,
}

impl MemoryEntryBuilder {
    /// Create a new builder with required fields.
    pub fn new(
        memory_type: MemoryType,
        title: impl Into<String>,
        content: impl Into<String>,
        source: MemorySource,
    ) -> Self {
        Self {
            memory_type,
            title: title.into(),
            content: content.into(),
            source,
            confidence: None,
            tags: Vec::new(),
            cycle_id: None,
            metadata: MemoryMetadata::default(),
            scope: MemoryScope::default(),
        }
    }

    /// Set the confidence score (defaults to the type's default).
    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Add tags.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set the cycle ID.
    pub fn cycle_id(mut self, cycle_id: impl Into<String>) -> Self {
        self.cycle_id = Some(cycle_id.into());
        self
    }

    /// Set metadata.
    pub fn metadata(mut self, metadata: MemoryMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the persistence scope.
    pub fn scope(mut self, scope: MemoryScope) -> Self {
        self.scope = scope;
        self
    }

    /// Build and validate the memory entry.
    pub fn build(self, i18n: &I18nService) -> MahalaxmiResult<MemoryEntry> {
        if self.title.is_empty() {
            return Err(MahalaxmiError::memory(
                i18n,
                keys::memory::INVALID_ENTRY,
                &[("reason", "title must not be empty")],
            ));
        }

        if self.title.len() > 200 {
            return Err(MahalaxmiError::memory(
                i18n,
                keys::memory::INVALID_ENTRY,
                &[("reason", "title must be 200 characters or fewer")],
            ));
        }

        if self.content.is_empty() {
            return Err(MahalaxmiError::memory(
                i18n,
                keys::memory::INVALID_ENTRY,
                &[("reason", "content must not be empty")],
            ));
        }

        let confidence = self
            .confidence
            .unwrap_or_else(|| self.memory_type.default_confidence());

        if !(0.0..=1.0).contains(&confidence) {
            let value_str = confidence.to_string();
            return Err(MahalaxmiError::memory(
                i18n,
                keys::memory::INVALID_CONFIDENCE,
                &[("value", &value_str)],
            ));
        }

        let now = Utc::now();
        Ok(MemoryEntry {
            id: MemoryId::new(),
            memory_type: self.memory_type,
            title: self.title,
            content: self.content,
            confidence,
            source: self.source,
            tags: self.tags,
            cycle_id: self.cycle_id,
            created_at: now,
            updated_at: now,
            metadata: self.metadata,
            scope: self.scope,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahalaxmi_core::i18n::locale::SupportedLocale;

    fn test_i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    #[test]
    fn builder_valid_entry() {
        let i18n = test_i18n();
        let entry = MemoryEntryBuilder::new(
            MemoryType::CodebaseFact,
            "Test Title",
            "Test content",
            MemorySource::System,
        )
        .build(&i18n)
        .unwrap();

        assert_eq!(entry.title, "Test Title");
        assert_eq!(entry.content, "Test content");
        assert_eq!(entry.memory_type, MemoryType::CodebaseFact);
        assert!((entry.confidence - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_rejects_empty_title() {
        let i18n = test_i18n();
        let result =
            MemoryEntryBuilder::new(MemoryType::Warning, "", "content", MemorySource::System)
                .build(&i18n);

        assert!(result.is_err());
        assert!(result.unwrap_err().is_memory());
    }

    #[test]
    fn builder_rejects_long_title() {
        let i18n = test_i18n();
        let long_title = "a".repeat(201);
        let result = MemoryEntryBuilder::new(
            MemoryType::Warning,
            long_title,
            "content",
            MemorySource::System,
        )
        .build(&i18n);

        assert!(result.is_err());
    }

    #[test]
    fn builder_rejects_invalid_confidence() {
        let i18n = test_i18n();
        let result = MemoryEntryBuilder::new(
            MemoryType::Warning,
            "title",
            "content",
            MemorySource::System,
        )
        .confidence(1.5)
        .build(&i18n);

        assert!(result.is_err());
    }

    #[test]
    fn memory_scope_default_is_session() {
        assert_eq!(MemoryScope::default(), MemoryScope::Session);
    }

    #[test]
    fn memory_entry_without_scope_deserializes_as_session() {
        let json = serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "memory_type": "codebase_fact",
            "title": "Test",
            "content": "Test content",
            "confidence": 0.7,
            "source": {"kind": "System"},
            "tags": [],
            "cycle_id": null,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "metadata": {
                "file_path": null,
                "line_range": null,
                "language": null,
                "related_entries": [],
                "custom": {}
            }
        });
        let entry: MemoryEntry = serde_json::from_value(json).unwrap();
        assert_eq!(entry.scope, MemoryScope::Session);
    }

    #[test]
    fn memory_scope_serializes_as_pascal_case() {
        assert_eq!(
            serde_json::to_string(&MemoryScope::Session).unwrap(),
            "\"Session\""
        );
        assert_eq!(
            serde_json::to_string(&MemoryScope::Project).unwrap(),
            "\"Project\""
        );
        assert_eq!(
            serde_json::to_string(&MemoryScope::Global).unwrap(),
            "\"Global\""
        );
    }

    #[test]
    fn builder_scope_method_sets_scope() {
        let i18n = test_i18n();
        let entry = MemoryEntryBuilder::new(
            MemoryType::Decision,
            "title",
            "content",
            MemorySource::System,
        )
        .scope(MemoryScope::Project)
        .build(&i18n)
        .unwrap();

        assert_eq!(entry.scope, MemoryScope::Project);
    }

    #[test]
    fn builder_default_scope_is_session() {
        let i18n = test_i18n();
        let entry = MemoryEntryBuilder::new(
            MemoryType::CodebaseFact,
            "title",
            "content",
            MemorySource::System,
        )
        .build(&i18n)
        .unwrap();

        assert_eq!(entry.scope, MemoryScope::Session);
    }

    #[test]
    fn test_memory_scope_team_serde() {
        let scope = MemoryScope::Team("eng-team".into());
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(json, r#"{"team":"eng-team"}"#);
        let deserialized: MemoryScope = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MemoryScope::Team("eng-team".into()));
    }

    #[test]
    fn test_team_scope_key_prefix() {
        let scope = MemoryScope::Team("my-team".into());
        assert_eq!(scope.db_key(), "team:my-team");

        let empty = MemoryScope::Team(String::new());
        assert_eq!(empty.db_key(), "team:");

        assert_eq!(MemoryScope::Session.db_key(), "Session");
        assert_eq!(MemoryScope::Project.db_key(), "Project");
        assert_eq!(MemoryScope::Global.db_key(), "Global");
    }
}

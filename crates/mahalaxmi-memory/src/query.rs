//! Query builder for filtering and ordering memory entries.

use crate::entry::MemoryEntry;
use crate::types::{MemorySource, MemoryType};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Ordering criteria for query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryOrder {
    /// Sort by confidence score (highest first).
    Confidence,
    /// Sort by creation time (newest first).
    CreatedAt,
    /// Sort by last update time (newest first).
    UpdatedAt,
    /// Composite relevance: confidence descending, then recency.
    Relevance,
}

impl QueryOrder {
    /// Returns the order as a static string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Confidence => "confidence",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
            Self::Relevance => "relevance",
        }
    }
}

impl fmt::Display for QueryOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Builder-style query for filtering memory entries.
///
/// Filters are combined with AND semantics (all must match).
/// Multiple types are OR'd together; multiple tags are AND'd.
#[derive(Debug, Clone, Default)]
pub struct MemoryQuery {
    /// Filter by memory types (OR'd).
    pub(crate) types: Vec<MemoryType>,
    /// Filter by tags (AND'd — entry must have all specified tags).
    pub(crate) tags: Vec<String>,
    /// Minimum confidence threshold.
    pub(crate) min_confidence: Option<f64>,
    /// Filter by cycle ID.
    pub(crate) cycle_id: Option<String>,
    /// Case-insensitive text search in title and content.
    pub(crate) text_search: Option<String>,
    /// Filter by source kind.
    pub(crate) source_kind: Option<String>,
    /// Filter by file path (exact match).
    pub(crate) file_path: Option<String>,
    /// Ordering of results.
    pub(crate) order: Option<QueryOrder>,
    /// Maximum number of results.
    pub(crate) limit: Option<usize>,
}

impl MemoryQuery {
    /// Create a new empty query (matches all entries).
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by a single memory type.
    pub fn with_type(mut self, memory_type: MemoryType) -> Self {
        self.types.push(memory_type);
        self
    }

    /// Filter by multiple memory types (OR'd).
    pub fn with_types(mut self, types: Vec<MemoryType>) -> Self {
        self.types.extend(types);
        self
    }

    /// Require a single tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Require multiple tags (AND'd).
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Set minimum confidence threshold.
    pub fn with_min_confidence(mut self, min: f64) -> Self {
        self.min_confidence = Some(min);
        self
    }

    /// Filter by cycle ID.
    pub fn with_cycle(mut self, cycle_id: impl Into<String>) -> Self {
        self.cycle_id = Some(cycle_id.into());
        self
    }

    /// Case-insensitive text search in title and content.
    pub fn with_text_search(mut self, text: impl Into<String>) -> Self {
        self.text_search = Some(text.into());
        self
    }

    /// Filter by source kind (e.g. "worker", "manager", "system", "user").
    pub fn with_source(mut self, source_kind: impl Into<String>) -> Self {
        self.source_kind = Some(source_kind.into());
        self
    }

    /// Filter by file path (exact match on metadata.file_path).
    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Set the ordering of results.
    pub fn order_by(mut self, order: QueryOrder) -> Self {
        self.order = Some(order);
        self
    }

    /// Limit the number of results.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if an entry matches all filters in this query.
    pub fn matches(&self, entry: &MemoryEntry) -> bool {
        if !self.types.is_empty() && !self.types.contains(&entry.memory_type) {
            return false;
        }

        if !self.tags.is_empty() {
            for tag in &self.tags {
                if !entry.tags.contains(tag) {
                    return false;
                }
            }
        }

        if let Some(min) = self.min_confidence {
            if entry.confidence < min {
                return false;
            }
        }

        if let Some(ref cycle) = self.cycle_id {
            match &entry.cycle_id {
                Some(entry_cycle) if entry_cycle == cycle => {}
                _ => return false,
            }
        }

        if let Some(ref text) = self.text_search {
            let lower = text.to_lowercase();
            let title_match = entry.title.to_lowercase().contains(&lower);
            let content_match = entry.content.to_lowercase().contains(&lower);
            if !title_match && !content_match {
                return false;
            }
        }

        if let Some(ref kind) = self.source_kind {
            if entry.source.as_str() != kind {
                return false;
            }
        }

        if let Some(ref path) = self.file_path {
            match &entry.metadata.file_path {
                Some(entry_path) if entry_path == path => {}
                _ => return false,
            }
        }

        true
    }

    /// Check if a source matches the query's source filter.
    pub fn matches_source(&self, source: &MemorySource) -> bool {
        match &self.source_kind {
            Some(kind) => source.as_str() == kind,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_order_display() {
        assert_eq!(format!("{}", QueryOrder::Confidence), "confidence");
        assert_eq!(format!("{}", QueryOrder::Relevance), "relevance");
    }

    #[test]
    fn empty_query_matches_all() {
        let query = MemoryQuery::new();
        assert!(query.types.is_empty());
        assert!(query.tags.is_empty());
        assert!(query.min_confidence.is_none());
    }

    #[test]
    fn query_builder_chaining() {
        let query = MemoryQuery::new()
            .with_type(MemoryType::Warning)
            .with_tag("rust")
            .with_min_confidence(0.5)
            .order_by(QueryOrder::Confidence)
            .limit(10);

        assert_eq!(query.types.len(), 1);
        assert_eq!(query.tags.len(), 1);
        assert_eq!(query.min_confidence, Some(0.5));
        assert_eq!(query.order, Some(QueryOrder::Confidence));
        assert_eq!(query.limit, Some(10));
    }
}

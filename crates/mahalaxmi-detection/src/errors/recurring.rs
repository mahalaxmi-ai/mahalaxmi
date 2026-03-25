use chrono::{DateTime, Utc};
use mahalaxmi_core::types::RootCauseCategory;
use serde::{Deserialize, Serialize};

/// A recurring error pattern that has been observed multiple times.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringError {
    /// A normalized version of the error message (for grouping).
    pub normalized_message: String,
    /// Number of times this error has occurred.
    pub occurrence_count: u32,
    /// Timestamps of each occurrence.
    pub occurrences: Vec<DateTime<Utc>>,
    /// The categorized root cause (if determined).
    pub category: RootCauseCategory,
    /// Additional context about the error.
    pub context: Option<String>,
    /// First time this error was seen.
    pub first_seen: DateTime<Utc>,
    /// Most recent occurrence.
    pub last_seen: DateTime<Utc>,
}

impl RecurringError {
    /// Create a new recurring error from an initial occurrence.
    pub fn new(normalized_message: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            normalized_message: normalized_message.into(),
            occurrence_count: 1,
            occurrences: vec![now],
            category: RootCauseCategory::Unknown,
            context: None,
            first_seen: now,
            last_seen: now,
        }
    }

    /// Record another occurrence of this error.
    pub fn record_occurrence(&mut self) {
        let now = Utc::now();
        self.occurrence_count += 1;
        self.occurrences.push(now);
        self.last_seen = now;
    }

    /// Set the root cause category.
    pub fn with_category(mut self, category: RootCauseCategory) -> Self {
        self.category = category;
        self
    }

    /// Set additional context.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Average time between occurrences in seconds.
    pub fn average_interval_seconds(&self) -> Option<f64> {
        if self.occurrences.len() < 2 {
            return None;
        }
        let total = self
            .last_seen
            .signed_duration_since(self.first_seen)
            .num_seconds() as f64;
        Some(total / (self.occurrences.len() - 1) as f64)
    }
}

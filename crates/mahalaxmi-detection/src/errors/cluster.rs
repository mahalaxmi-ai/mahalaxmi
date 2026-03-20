use serde::{Deserialize, Serialize};

use crate::errors::recurring::RecurringError;

/// A cluster of related errors that may share a common root cause.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCluster {
    /// Human-readable label for this cluster.
    pub label: String,
    /// The recurring errors in this cluster.
    pub errors: Vec<RecurringError>,
    /// Suspected root cause description.
    pub suspected_cause: Option<String>,
    /// Confidence level (0.0 to 1.0) that errors in this cluster are related.
    pub confidence: f64,
}

impl ErrorCluster {
    /// Create a new error cluster with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            errors: Vec::new(),
            suspected_cause: None,
            confidence: 0.0,
        }
    }

    /// Add a recurring error to this cluster.
    pub fn with_error(mut self, error: RecurringError) -> Self {
        self.errors.push(error);
        self
    }

    /// Set the suspected root cause.
    pub fn with_suspected_cause(mut self, cause: impl Into<String>) -> Self {
        self.suspected_cause = Some(cause.into());
        self
    }

    /// Set the confidence level.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Total occurrences across all errors in this cluster.
    pub fn total_occurrences(&self) -> u32 {
        self.errors.iter().map(|e| e.occurrence_count).sum()
    }

    /// Number of distinct error patterns in this cluster.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

use mahalaxmi_core::types::{ActionType, RootCauseCategory};
use serde::{Deserialize, Serialize};

/// A hypothesis about the root cause of a set of errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseHypothesis {
    /// The hypothesized root cause category.
    pub category: RootCauseCategory,
    /// Human-readable description of the hypothesis.
    pub description: String,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f64,
    /// Evidence supporting this hypothesis.
    pub evidence: Vec<String>,
    /// Suggested action to resolve the root cause.
    pub suggested_action: ActionType,
}

impl RootCauseHypothesis {
    /// Create a new root cause hypothesis.
    pub fn new(category: RootCauseCategory, description: impl Into<String>) -> Self {
        Self {
            category,
            description: description.into(),
            confidence: 0.0,
            evidence: Vec::new(),
            suggested_action: ActionType::ContinueProcessing,
        }
    }

    /// Set the confidence level.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add evidence supporting this hypothesis.
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    /// Set the suggested action.
    pub fn with_action(mut self, action: ActionType) -> Self {
        self.suggested_action = action;
        self
    }
}

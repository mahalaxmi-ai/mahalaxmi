//! Worker result models for the orchestration engine.
//!
//! Tracks the outcome of a worker execution including token usage
//! for cost accounting and performance monitoring.

use mahalaxmi_providers::cost::TokenUsage;
use serde::{Deserialize, Serialize};

/// The result of a completed worker execution.
///
/// Captures token usage for cost tracking and performance analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkerResult {
    /// Token usage recorded during this worker's execution.
    #[serde(default)]
    pub token_usage: TokenUsage,
}

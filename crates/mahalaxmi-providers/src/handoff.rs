//! Cross-provider handoff — context sharing when tasks move between providers.
//!
//! When a task is routed from one provider to another (due to failover, cost
//! optimization, or provider specialization), the `HandoffContext` captures the
//! relevant state from the source provider so the target can resume work.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mahalaxmi_core::types::{ProviderId, WorkerId};
use serde::{Deserialize, Serialize};

/// Context transferred between providers during a handoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffContext {
    /// Worker ID being handed off.
    pub worker_id: WorkerId,
    /// Source provider that was working on the task.
    pub source_provider: ProviderId,
    /// Target provider receiving the task.
    pub target_provider: ProviderId,
    /// Summary of work completed so far.
    pub progress_summary: String,
    /// Files that were modified by the source provider.
    pub modified_files: Vec<String>,
    /// Key discoveries or conventions found during source work.
    pub discoveries: Vec<String>,
    /// Errors encountered by the source provider (if any).
    pub errors: Vec<String>,
    /// Arbitrary metadata for provider-specific context.
    pub metadata: HashMap<String, String>,
    /// When the handoff was initiated.
    pub timestamp: DateTime<Utc>,
}

impl HandoffContext {
    /// Create a new handoff context.
    pub fn new(
        worker_id: WorkerId,
        source_provider: ProviderId,
        target_provider: ProviderId,
    ) -> Self {
        Self {
            worker_id,
            source_provider,
            target_provider,
            progress_summary: String::new(),
            modified_files: Vec::new(),
            discoveries: Vec::new(),
            errors: Vec::new(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    /// Set the progress summary.
    pub fn with_progress(mut self, summary: impl Into<String>) -> Self {
        self.progress_summary = summary.into();
        self
    }

    /// Add a modified file.
    pub fn with_modified_file(mut self, path: impl Into<String>) -> Self {
        self.modified_files.push(path.into());
        self
    }

    /// Add a discovery.
    pub fn with_discovery(mut self, discovery: impl Into<String>) -> Self {
        self.discoveries.push(discovery.into());
        self
    }

    /// Add an error from the source provider.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if the source provider encountered errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Build a handoff prompt for the target provider.
    ///
    /// Generates a structured context string that can be prepended to the
    /// target provider's prompt so it knows what happened before.
    pub fn build_handoff_prompt(&self) -> String {
        let mut prompt = String::new();
        prompt.push_str("## Handoff Context\n\n");
        prompt.push_str(&format!(
            "This task was previously worked on by {} and is being handed off to you.\n\n",
            self.source_provider.as_str()
        ));

        if !self.progress_summary.is_empty() {
            prompt.push_str("### Progress So Far\n");
            prompt.push_str(&self.progress_summary);
            prompt.push_str("\n\n");
        }

        if !self.modified_files.is_empty() {
            prompt.push_str("### Files Modified\n");
            for file in &self.modified_files {
                prompt.push_str(&format!("- {}\n", file));
            }
            prompt.push('\n');
        }

        if !self.discoveries.is_empty() {
            prompt.push_str("### Discoveries\n");
            for discovery in &self.discoveries {
                prompt.push_str(&format!("- {}\n", discovery));
            }
            prompt.push('\n');
        }

        if !self.errors.is_empty() {
            prompt.push_str("### Previous Errors\n");
            for error in &self.errors {
                prompt.push_str(&format!("- {}\n", error));
            }
            prompt.push('\n');
        }

        prompt
    }
}

/// Outcome of a handoff attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffResult {
    /// Handoff succeeded — target provider accepted the task.
    Success,
    /// Handoff failed — target provider is unavailable.
    TargetUnavailable,
    /// Handoff failed — no suitable target found.
    NoTarget,
}

/// Tracks active handoffs for the orchestration cycle.
#[derive(Debug, Default)]
pub struct HandoffTracker {
    /// History of completed handoffs.
    history: Vec<HandoffContext>,
    /// Count of handoffs per worker (to prevent infinite loops).
    handoff_counts: HashMap<WorkerId, u32>,
    /// Maximum allowed handoffs per worker.
    max_handoffs_per_worker: u32,
}

impl HandoffTracker {
    /// Create a new handoff tracker.
    pub fn new(max_handoffs: u32) -> Self {
        Self {
            history: Vec::new(),
            handoff_counts: HashMap::new(),
            max_handoffs_per_worker: max_handoffs,
        }
    }

    /// Record a completed handoff.
    pub fn record(&mut self, context: HandoffContext) {
        let count = self.handoff_counts.entry(context.worker_id).or_insert(0);
        *count += 1;
        self.history.push(context);
    }

    /// Check if a worker can be handed off again.
    pub fn can_handoff(&self, worker_id: &WorkerId) -> bool {
        self.handoff_counts.get(worker_id).copied().unwrap_or(0) < self.max_handoffs_per_worker
    }

    /// Get the number of handoffs for a worker.
    pub fn handoff_count(&self, worker_id: &WorkerId) -> u32 {
        self.handoff_counts.get(worker_id).copied().unwrap_or(0)
    }

    /// Get the full handoff history.
    pub fn history(&self) -> &[HandoffContext] {
        &self.history
    }

    /// Get the most recent handoff for a worker.
    pub fn last_handoff(&self, worker_id: &WorkerId) -> Option<&HandoffContext> {
        self.history
            .iter()
            .rev()
            .find(|h| &h.worker_id == worker_id)
    }

    /// Total number of handoffs across all workers.
    pub fn total_handoffs(&self) -> usize {
        self.history.len()
    }
}

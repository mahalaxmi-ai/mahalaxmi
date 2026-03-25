use chrono::{DateTime, Utc};
use mahalaxmi_core::types::{ProviderId, TaskId, WorkerId, WorkerStatus};
use mahalaxmi_providers::cost::TokenUsage;
use serde::{Deserialize, Serialize};

use crate::agent::AgentSpec;
use crate::verification::VerificationResult;

/// A worker entry in the execution queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedWorker {
    /// Worker ID.
    pub worker_id: WorkerId,
    /// Task assigned to this worker.
    pub task_id: TaskId,
    /// Current worker status.
    pub status: WorkerStatus,
    /// Task IDs this worker depends on.
    pub dependencies: Vec<TaskId>,
    /// Number of times this worker has failed and been retried.
    pub failure_count: u32,
    /// Maximum retry attempts before permanent failure.
    pub max_retries: u32,
    /// Timestamp of the last failure (for retry backoff).
    pub last_failure_time: Option<DateTime<Utc>>,
    /// Timestamp when this worker started execution.
    pub started_at: Option<DateTime<Utc>>,
    /// Timestamp when this worker completed or failed.
    pub finished_at: Option<DateTime<Utc>>,
    /// Number of verification-triggered retries performed.
    pub verification_retries: usize,
    /// Error context from last failed verification (for re-injection).
    pub retry_context: Option<String>,
    /// Last verification result.
    pub last_verification: Option<VerificationResult>,
    /// Context preamble for this worker (populated before activation).
    pub context_preamble: Option<String>,
    /// Full assembled prompt from WorkerPromptBuilder (populated before activation).
    pub assembled_prompt: Option<String>,
    /// Provider assigned by the multi-provider router.
    pub provider_id: Option<ProviderId>,
    /// Ordered fallback providers if the primary fails.
    pub fallback_provider_ids: Vec<ProviderId>,
    /// Agent spec assigned to this worker (specialist persona).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_spec: Option<AgentSpec>,
    /// Files successfully modified before failure (from worktree diff).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub partial_files_modified: Vec<String>,
    /// Partial output captured before failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_output: Option<String>,
    /// Token usage recorded at session completion. None until the AI session ends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
}

impl QueuedWorker {
    /// Create a new queued worker.
    pub fn new(
        worker_id: WorkerId,
        task_id: TaskId,
        dependencies: Vec<TaskId>,
        max_retries: u32,
    ) -> Self {
        let status = if dependencies.is_empty() {
            WorkerStatus::Pending
        } else {
            WorkerStatus::Blocked
        };
        Self {
            worker_id,
            task_id,
            status,
            dependencies,
            failure_count: 0,
            max_retries,
            last_failure_time: None,
            started_at: None,
            finished_at: None,
            verification_retries: 0,
            retry_context: None,
            last_verification: None,
            context_preamble: None,
            assembled_prompt: None,
            provider_id: None,
            fallback_provider_ids: Vec::new(),
            agent_spec: None,
            partial_files_modified: Vec::new(),
            partial_output: None,
            token_usage: None,
        }
    }

    /// Check if all dependencies are in the completed set.
    pub fn are_dependencies_satisfied(
        &self,
        completed: &std::collections::HashSet<TaskId>,
    ) -> bool {
        self.dependencies.iter().all(|dep| completed.contains(dep))
    }

    /// Check if the worker is eligible for retry.
    ///
    /// Returns true if the worker has failed, has retries remaining,
    /// and enough time has elapsed since the last failure (exponential backoff).
    pub fn is_ready_for_retry(&self, now: DateTime<Utc>) -> bool {
        if self.status != WorkerStatus::Failed {
            return false;
        }
        if self.failure_count >= self.max_retries {
            return false;
        }
        match self.last_failure_time {
            Some(last) => {
                let backoff_seconds = 2i64.saturating_pow(self.failure_count);
                let elapsed = now.signed_duration_since(last).num_seconds();
                elapsed >= backoff_seconds
            }
            None => true,
        }
    }
}

/// Statistics about the worker queue.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueStatistics {
    /// Total number of workers in the queue.
    pub total: u32,
    /// Workers that have completed successfully.
    pub completed: u32,
    /// Workers currently executing.
    pub active: u32,
    /// Workers waiting to be scheduled.
    pub pending: u32,
    /// Workers blocked on dependencies.
    pub blocked: u32,
    /// Workers currently running verification checks.
    pub verifying: u32,
    /// Workers that have failed.
    pub failed: u32,
    /// Workers skipped because a dependency permanently failed.
    pub skipped: u32,
}

impl QueueStatistics {
    /// Returns true if all workers have completed (successfully).
    pub fn all_completed(&self) -> bool {
        self.completed == self.total && self.total > 0
    }

    /// Returns the completion percentage (0.0 to 100.0).
    pub fn completion_percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.completed as f64 / self.total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_worker_new_token_usage_is_none() {
        let worker_id = WorkerId::new(0);
        let task_id = TaskId::new("task-0");
        let worker = QueuedWorker::new(worker_id, task_id, Vec::new(), 3);
        assert!(worker.token_usage.is_none());
    }
}

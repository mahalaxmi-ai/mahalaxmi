use chrono::{DateTime, Utc};
use mahalaxmi_core::types::{CycleId, ManagerId, OrchestrationCycleState, WorkerId};
use serde::{Deserialize, Serialize};

/// A preview of a single task in an execution plan, sent to the frontend for
/// plan review before worker dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTaskPreview {
    /// Unique task identifier (matches `WorkerTask.task_id`).
    pub id: String,
    /// Human-readable task title.
    pub title: String,
    /// Detailed task description.
    pub description: String,
    /// Files expected to be modified by this task.
    pub affected_files: Vec<String>,
    /// Execution wave (phase number) this task belongs to.
    pub wave: u32,
    /// Estimated complexity on a 1-10 scale.
    pub complexity: u32,
}

/// Build a [`PlanTaskPreview`] from a plan phase number and worker task.
///
/// Sets `wave = phase_number`, `id = task.task_id.to_string()`,
/// and `complexity = task.complexity`.
pub fn task_to_preview(
    phase_number: u32,
    task: &crate::models::plan::WorkerTask,
) -> PlanTaskPreview {
    PlanTaskPreview {
        id: task.task_id.to_string(),
        title: task.title.clone(),
        description: task.description.clone(),
        affected_files: task.affected_files.clone(),
        wave: phase_number,
        complexity: task.complexity,
    }
}

/// Events emitted during orchestration cycle execution.
///
/// These events drive UI updates, logging, and audit trails.
///
/// Marked `#[non_exhaustive]` so that downstream crates that match on this
/// enum are not broken when new variants are added (they must include a
/// wildcard arm).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationEvent {
    /// A new orchestration cycle has started.
    CycleStarted {
        cycle_id: CycleId,
        timestamp: DateTime<Utc>,
    },
    /// The cycle state machine has transitioned.
    StateChanged {
        cycle_id: CycleId,
        old_state: OrchestrationCycleState,
        new_state: OrchestrationCycleState,
        timestamp: DateTime<Utc>,
    },
    /// A manager has completed its analysis.
    ManagerCompleted {
        cycle_id: CycleId,
        manager_id: ManagerId,
        task_count: u32,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },
    /// Consensus has been reached on the execution plan.
    ConsensusReached {
        cycle_id: CycleId,
        agreed_count: u32,
        dissenting_count: u32,
        timestamp: DateTime<Utc>,
    },
    /// An execution plan has been created.
    PlanCreated {
        cycle_id: CycleId,
        phase_count: u32,
        worker_count: u32,
        timestamp: DateTime<Utc>,
    },
    /// A worker has started executing its task.
    WorkerStarted {
        cycle_id: CycleId,
        worker_id: WorkerId,
        task_summary: String,
        timestamp: DateTime<Utc>,
    },
    /// A worker has completed its task.
    WorkerCompleted {
        cycle_id: CycleId,
        worker_id: WorkerId,
        success: bool,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },
    /// A worker has failed.
    WorkerFailed {
        cycle_id: CycleId,
        worker_id: WorkerId,
        error_summary: String,
        retry_count: u32,
        timestamp: DateTime<Utc>,
    },
    /// A worker's provider was switched to a fallback after failure.
    ProviderSwitched {
        cycle_id: CycleId,
        worker_id: WorkerId,
        from_provider: String,
        to_provider: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    /// A worker's worktree merge produced conflicts.
    WorkerConflict {
        cycle_id: CycleId,
        worker_id: WorkerId,
        conflicting_files: Vec<String>,
        diff_summary: String,
        timestamp: DateTime<Utc>,
    },
    /// The orchestration cycle has completed.
    CycleCompleted {
        cycle_id: CycleId,
        total_duration_ms: u64,
        success_rate: f64,
        /// Number of workers that completed successfully.
        completed_workers: u32,
        /// Number of workers that failed permanently.
        failed_workers: u32,
        /// Total workers dispatched this cycle.
        total_workers: u32,
        timestamp: DateTime<Utc>,
    },
    /// Worker count was auto-scaled to match the task count from consensus.
    ///
    /// Emitted after `begin_worker_execution()` when `worker_count == 0` (auto mode).
    AutoScaled {
        /// The worker count requested by the user (0 = auto).
        requested: u32,
        /// The effective count after auto-scaling and tier clamping.
        effective: u32,
        /// The maximum allowed by the current license tier.
        tier_limit: u32,
    },
    /// Cumulative cycle cost has been updated after a worker completed.
    ///
    /// Emitted by the driver once per worker completion. The frontend uses this
    /// event to drive the live cost ticker displayed inside `CycleStatus`.
    CycleCostUpdated {
        /// Cycle this cost belongs to.
        cycle_id: CycleId,
        /// Worker that just completed (most recent contributor to the cost).
        worker_id: WorkerId,
        /// Cumulative input tokens consumed across all completed workers so far.
        total_input_tokens: u64,
        /// Cumulative output tokens produced across all completed workers so far.
        total_output_tokens: u64,
        /// Cumulative estimated cost in USD across all completed workers so far.
        estimated_cost_usd: f64,
        /// Per-provider breakdown of cumulative cost: `(provider_id, cost_usd)`.
        cost_by_provider: Vec<(String, f64)>,
        /// Wall-clock timestamp of this event.
        timestamp: DateTime<Utc>,
    },
    /// The execution plan is ready for developer review.
    ///
    /// Emitted when `enable_plan_review = true` in config, immediately after
    /// `DiscoveringWorkerRequirements → AwaitingPlanApproval` transition.
    /// The frontend renders the plan review UI and calls
    /// `approve_execution_plan` to unblock the cycle.
    PlanReady {
        /// Cycle whose plan is awaiting review.
        cycle_id: CycleId,
        /// Task previews for display in the plan review UI.
        tasks: Vec<PlanTaskPreview>,
    },
    /// Cumulative cycle cost has reached or exceeded the configured budget limit.
    ///
    /// Emitted by the driver when `cumulative_cost_usd >= max_cycle_cost_usd`.
    /// The driver then awaits a response via the `continue_over_budget` Tauri
    /// command before dispatching further workers.
    BudgetExceeded {
        /// Cycle whose cost exceeded the budget.
        cycle_id: CycleId,
        /// Total estimated cost at the time the limit was breached.
        estimated_cost_usd: f64,
        /// The configured budget that was reached.
        budget_usd: f64,
        /// Wall-clock timestamp of this event.
        timestamp: DateTime<Utc>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::plan::WorkerTask;
    use mahalaxmi_core::types::{CycleId, TaskId, WorkerId};

    fn make_task(id: &str, title: &str, complexity: u32) -> WorkerTask {
        let mut t = WorkerTask::new(TaskId::new(id), WorkerId::new(0), title, "desc");
        t.complexity = complexity;
        t.affected_files = vec!["src/lib.rs".to_owned()];
        t
    }

    #[test]
    fn task_to_preview_sets_wave_and_id() {
        let task = make_task("task-7", "Refactor auth", 8);
        let preview = task_to_preview(2, &task);
        assert_eq!(preview.wave, 2);
        assert_eq!(preview.id, "task-7");
        assert_eq!(preview.complexity, 8);
        assert_eq!(preview.title, "Refactor auth");
        assert_eq!(preview.affected_files, vec!["src/lib.rs"]);
    }

    #[test]
    fn plan_ready_event_serializes_and_deserializes() {
        let cycle_id = CycleId::new();
        let task = make_task("task-1", "Add feature", 5);
        let preview = task_to_preview(1, &task);
        let event = OrchestrationEvent::PlanReady {
            cycle_id,
            tasks: vec![preview],
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let recovered: OrchestrationEvent = serde_json::from_str(&json).expect("deserialize");
        if let OrchestrationEvent::PlanReady {
            cycle_id: recovered_id,
            tasks,
        } = recovered
        {
            assert_eq!(recovered_id, cycle_id);
            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].id, "task-1");
            assert_eq!(tasks[0].wave, 1);
        } else {
            panic!("Expected PlanReady variant");
        }
    }
}

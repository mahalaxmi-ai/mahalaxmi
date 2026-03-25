//! Data models for the orchestration engine.
//!
//! Includes manager proposals, consensus results, execution plans,
//! worker queue state, and orchestration events.

pub mod consensus;
pub mod events;
pub mod plan;
pub mod proposal;
pub mod queue;
pub mod report;
pub mod review;
pub mod validation;
pub mod worker;

pub use consensus::{
    BatchAnalysisOutput, CompletionVerdict, ConsensusConfiguration, ConsensusMetrics,
    ConsensusResult, ConsensusTask, DissentingTask,
};
pub use events::OrchestrationEvent;
pub use plan::{ExecutionPhase, ExecutionPlan, WorkerTask};
pub use proposal::{ManagerProposal, ProposedAgentSpec, ProposedTask};
pub use queue::{QueueStatistics, QueuedWorker};
pub use report::{CycleCostSummary, CycleOutcome, CycleReport, ProviderCostBreakdown};
pub use review::{ReviewIssue, ReviewResult, ReviewSeverity};
pub use validation::{
    AcceptanceCommandResult, AcceptanceCriterionResult, FulfillmentStatus, GapSeverity,
    RequirementAssessment, RequirementGap, ValidationVerdict,
};
pub use worker::WorkerResult;

/// Simplified task entry used inside a [`ManagerProposalSnapshot`].
///
/// Carries only the fields relevant for the manager deliberation panel —
/// a stable `id`, human-readable `title`, `description`, and a rough
/// complexity estimate. Full proposal data lives in [`ProposedTask`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ManagerProposedTask {
    /// Stable identifier derived from the normalized task title.
    pub id: String,
    /// Human-readable title of the task.
    pub title: String,
    /// Brief description of the work to be done.
    pub description: String,
    /// Rough complexity estimate on a 0-10 scale.
    #[serde(default)]
    pub estimated_complexity: u8,
}

/// Snapshot of a single manager's proposal, captured immediately after
/// consensus completes. Displayed in the Manager Deliberation Panel.
///
/// Populated by the orchestration driver via
/// [`crate::state::AppState::last_manager_proposals`] and broadcast as a
/// `manager_proposals_ready` Tauri event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ManagerProposalSnapshot {
    /// Identifier of the manager agent that produced this proposal.
    pub manager_id: String,
    /// Provider used by this manager (e.g. `"claude-code"`).
    pub provider: String,
    /// Simplified task list for display in the deliberation panel.
    pub tasks: Vec<ManagerProposedTask>,
    /// Wall-clock time at which this snapshot was captured.
    pub snapshot_at: chrono::DateTime<chrono::Utc>,
    /// Relative weight of this manager's proposal in the consensus.
    /// Defaults to `1.0` (equal weighting).
    #[serde(default = "default_snapshot_weight")]
    pub weight: f32,
}

fn default_snapshot_weight() -> f32 {
    1.0
}

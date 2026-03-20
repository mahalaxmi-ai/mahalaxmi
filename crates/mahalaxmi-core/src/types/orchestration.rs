//! Orchestration domain types shared across crates.
//!
//! These types are defined in `mahalaxmi-core` so that both `mahalaxmi-orchestration`
//! and `mahalaxmi-detection` can use them without cross-dependency.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for an orchestration cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CycleId(Uuid);

impl CycleId {
    /// Create a new random cycle ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID value.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CycleId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CycleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a worker within an execution plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerId(u32);

impl WorkerId {
    /// Create a new worker ID from a numeric value.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the inner numeric value.
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for WorkerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "worker-{}", self.0)
    }
}

/// Unique identifier for a manager in the orchestration system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ManagerId(String);

impl ManagerId {
    /// Create a new manager ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the manager ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ManagerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a task within an execution plan.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(String);

impl TaskId {
    /// Create a new task ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the task ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// State of the orchestration cycle state machine.
///
/// Tracks the lifecycle of a single orchestration cycle from idle through
/// manager processing, consensus, worker execution, and completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrchestrationCycleState {
    /// No cycle is active.
    Idle,
    /// Manager terminals are being set up.
    InitializingManager,
    /// Managers are processing (analyzing codebase, generating proposals).
    ManagerProcessing,
    /// Manager results are being analyzed for consensus.
    AnalyzingManagerResults,
    /// Worker requirements are being discovered from consensus results.
    DiscoveringWorkerRequirements,
    /// Cycle is paused for developer plan review before worker dispatch.
    ///
    /// Only reached when `enable_plan_review = true` in config. The cycle
    /// remains in this state until the developer approves (or rejects) the
    /// execution plan via `approve_execution_plan`.
    AwaitingPlanApproval,
    /// Worker terminals are being initialized.
    InitializingWorkers,
    /// Workers are executing their assigned tasks.
    WorkersProcessing,
    /// Cycle is in the process of completing.
    CompletingCycle,
    /// Cycle is restarting (after error recovery or explicit restart).
    RestartingCycle,
    /// An error has occurred during the cycle.
    Error,
    /// The cycle has been stopped.
    Stopped,
}

impl OrchestrationCycleState {
    /// Returns true if this state represents an active (in-progress) cycle.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::InitializingManager
                | Self::ManagerProcessing
                | Self::AnalyzingManagerResults
                | Self::DiscoveringWorkerRequirements
                | Self::AwaitingPlanApproval
                | Self::InitializingWorkers
                | Self::WorkersProcessing
                | Self::CompletingCycle
                | Self::RestartingCycle
        )
    }

    /// Returns true if this state represents a terminal (final) state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped)
    }

    /// Returns the set of valid next states from the current state.
    pub fn valid_transitions(&self) -> &'static [OrchestrationCycleState] {
        match self {
            Self::Idle => &[Self::InitializingManager, Self::Stopped],
            Self::InitializingManager => &[Self::ManagerProcessing, Self::Error, Self::Stopped],
            Self::ManagerProcessing => &[Self::AnalyzingManagerResults, Self::Error, Self::Stopped],
            Self::AnalyzingManagerResults => &[
                Self::DiscoveringWorkerRequirements,
                Self::Error,
                Self::Stopped,
            ],
            Self::DiscoveringWorkerRequirements => &[
                Self::AwaitingPlanApproval,
                Self::InitializingWorkers,
                Self::Error,
                Self::Stopped,
            ],
            Self::AwaitingPlanApproval => &[Self::InitializingWorkers, Self::Error, Self::Stopped],
            Self::InitializingWorkers => &[Self::WorkersProcessing, Self::Error, Self::Stopped],
            Self::WorkersProcessing => &[Self::CompletingCycle, Self::Error, Self::Stopped],
            Self::CompletingCycle => &[
                Self::Idle,
                Self::RestartingCycle,
                Self::Error,
                Self::Stopped,
            ],
            Self::RestartingCycle => &[Self::Idle, Self::Error, Self::Stopped],
            Self::Error => &[Self::RestartingCycle, Self::Stopped],
            Self::Stopped => &[],
        }
    }

    /// Returns true if transitioning to the given state is valid.
    pub fn can_transition_to(&self, next: OrchestrationCycleState) -> bool {
        self.valid_transitions().contains(&next)
    }
}

impl fmt::Display for OrchestrationCycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Idle => "Idle",
            Self::InitializingManager => "InitializingManager",
            Self::ManagerProcessing => "ManagerProcessing",
            Self::AnalyzingManagerResults => "AnalyzingManagerResults",
            Self::DiscoveringWorkerRequirements => "DiscoveringWorkerRequirements",
            Self::AwaitingPlanApproval => "AwaitingPlanApproval",
            Self::InitializingWorkers => "InitializingWorkers",
            Self::WorkersProcessing => "WorkersProcessing",
            Self::CompletingCycle => "CompletingCycle",
            Self::RestartingCycle => "RestartingCycle",
            Self::Error => "Error",
            Self::Stopped => "Stopped",
        };
        write!(f, "{}", label)
    }
}

/// Status of a worker in the execution queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerStatus {
    /// Worker is waiting to be scheduled.
    Pending,
    /// Worker is actively executing its task.
    Active,
    /// Worker is waiting for dependencies to complete.
    Blocked,
    /// Worker process is alive but has produced no PTY output for longer than
    /// the idle warning threshold. May recover if output resumes.
    Stalled,
    /// Worker is running self-verification checks (tests, lint, self-review).
    Verifying,
    /// Worker has completed its task successfully.
    Completed,
    /// Worker has failed (may be retried).
    Failed,
    /// Worker was skipped because a dependency permanently failed.
    Skipped,
}

impl WorkerStatus {
    /// Returns true if the worker is in a terminal (finished) state.
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Skipped)
    }

    /// Returns true if the worker can accept new work.
    pub fn is_schedulable(&self) -> bool {
        matches!(self, Self::Pending | Self::Blocked)
    }

    /// Returns true if the worker is actively consuming a terminal slot.
    pub fn is_active_slot(&self) -> bool {
        matches!(self, Self::Active | Self::Stalled | Self::Verifying)
    }
}

impl fmt::Display for WorkerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Pending => "Pending",
            Self::Active => "Active",
            Self::Blocked => "Blocked",
            Self::Stalled => "Stalled",
            Self::Verifying => "Verifying",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
        };
        write!(f, "{}", label)
    }
}

/// Type of verification check to run after worker task completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum VerificationCheck {
    /// Run the project's test suite.
    Tests,
    /// Run lint/static analysis tools.
    Lint,
    /// Ask the AI provider to self-review its changes.
    SelfReview,
}

impl VerificationCheck {
    /// Returns the check name as a static string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tests => "tests",
            Self::Lint => "lint",
            Self::SelfReview => "self-review",
        }
    }

    /// Returns the execution priority (lower = runs first).
    pub fn priority(&self) -> u8 {
        match self {
            Self::Tests => 0,
            Self::Lint => 1,
            Self::SelfReview => 2,
        }
    }
}

impl fmt::Display for VerificationCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Git merge strategy for handling worker worktree branches after completion.
///
/// Controls how completed worker branches are integrated back into the project.
/// Configured per-project in the setup wizard, with global defaults in
/// `OrchestrationConfig`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitMergeStrategy {
    /// Merge worker branch directly into the target branch (current behavior).
    #[default]
    DirectMerge,
    /// Push worker branch to remote and create a pull/merge request.
    BranchAndPr,
    /// Leave worker branches for manual handling (no merge, no PR).
    BranchOnly,
    /// No worktree isolation — all workers share the project root.
    Disabled,
}

impl GitMergeStrategy {
    /// Parse from the config string format.
    pub fn from_config_str(s: &str) -> Option<Self> {
        match s {
            "direct_merge" => Some(Self::DirectMerge),
            "branch_and_pr" => Some(Self::BranchAndPr),
            "branch_only" => Some(Self::BranchOnly),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }

    /// Convert to the config string format.
    pub fn as_config_str(&self) -> &'static str {
        match self {
            Self::DirectMerge => "direct_merge",
            Self::BranchAndPr => "branch_and_pr",
            Self::BranchOnly => "branch_only",
            Self::Disabled => "disabled",
        }
    }
}

impl fmt::Display for GitMergeStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_config_str())
    }
}

/// PR platform for `BranchAndPr` strategy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitPrPlatform {
    /// GitHub (`gh` CLI).
    #[default]
    GitHub,
    /// GitLab (`glab` CLI).
    GitLab,
}

impl GitPrPlatform {
    /// Parse from the config string format.
    pub fn from_config_str(s: &str) -> Option<Self> {
        match s {
            "github" => Some(Self::GitHub),
            "gitlab" => Some(Self::GitLab),
            _ => None,
        }
    }

    /// Convert to the config string format.
    pub fn as_config_str(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::GitLab => "gitlab",
        }
    }
}

impl fmt::Display for GitPrPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_config_str())
    }
}

/// Consensus strategy for merging multiple manager proposals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsensusStrategy {
    /// Include all unique tasks from all managers.
    Union,
    /// Include only tasks proposed by every manager.
    Intersection,
    /// Include tasks meeting a minimum agreement threshold.
    WeightedVoting,
    /// Factor in both vote frequency and complexity scores.
    ComplexityWeighted,
}

impl ConsensusStrategy {
    /// Parse from the config string format used in OrchestrationConfig.
    ///
    /// `"unanimous"` maps to `Union` — at the proposal-merge level, unanimous
    /// agreement across managers is achieved by taking the union of all
    /// proposals and requiring each to appear in every manager's output.
    /// This bridges the coding domain's `ConsensusAlgorithm::Unanimous` to
    /// the strategy that enforces full-agreement at the merge step.
    pub fn from_config_str(s: &str) -> Option<Self> {
        match s {
            "union" | "unanimous" => Some(Self::Union),
            "intersection" => Some(Self::Intersection),
            "weighted_voting" => Some(Self::WeightedVoting),
            "complexity_weighted" => Some(Self::ComplexityWeighted),
            _ => None,
        }
    }

    /// Convert to the config string format used in OrchestrationConfig.
    pub fn as_config_str(&self) -> &'static str {
        match self {
            Self::Union => "union",
            Self::Intersection => "intersection",
            Self::WeightedVoting => "weighted_voting",
            Self::ComplexityWeighted => "complexity_weighted",
        }
    }
}

impl fmt::Display for ConsensusStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_config_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_merge_strategy_from_config_str() {
        assert_eq!(
            GitMergeStrategy::from_config_str("direct_merge"),
            Some(GitMergeStrategy::DirectMerge)
        );
        assert_eq!(
            GitMergeStrategy::from_config_str("branch_and_pr"),
            Some(GitMergeStrategy::BranchAndPr)
        );
        assert_eq!(
            GitMergeStrategy::from_config_str("branch_only"),
            Some(GitMergeStrategy::BranchOnly)
        );
        assert_eq!(
            GitMergeStrategy::from_config_str("disabled"),
            Some(GitMergeStrategy::Disabled)
        );
        assert_eq!(GitMergeStrategy::from_config_str("invalid"), None);
    }

    #[test]
    fn git_merge_strategy_roundtrip() {
        for strategy in [
            GitMergeStrategy::DirectMerge,
            GitMergeStrategy::BranchAndPr,
            GitMergeStrategy::BranchOnly,
            GitMergeStrategy::Disabled,
        ] {
            let s = strategy.as_config_str();
            assert_eq!(GitMergeStrategy::from_config_str(s), Some(strategy));
        }
    }

    #[test]
    fn git_merge_strategy_default() {
        assert_eq!(GitMergeStrategy::default(), GitMergeStrategy::DirectMerge);
    }

    #[test]
    fn git_merge_strategy_display() {
        assert_eq!(GitMergeStrategy::DirectMerge.to_string(), "direct_merge");
        assert_eq!(GitMergeStrategy::BranchAndPr.to_string(), "branch_and_pr");
    }

    #[test]
    fn git_pr_platform_from_config_str() {
        assert_eq!(
            GitPrPlatform::from_config_str("github"),
            Some(GitPrPlatform::GitHub)
        );
        assert_eq!(
            GitPrPlatform::from_config_str("gitlab"),
            Some(GitPrPlatform::GitLab)
        );
        assert_eq!(GitPrPlatform::from_config_str("invalid"), None);
    }

    #[test]
    fn git_pr_platform_roundtrip() {
        for platform in [GitPrPlatform::GitHub, GitPrPlatform::GitLab] {
            let s = platform.as_config_str();
            assert_eq!(GitPrPlatform::from_config_str(s), Some(platform));
        }
    }

    #[test]
    fn git_pr_platform_default() {
        assert_eq!(GitPrPlatform::default(), GitPrPlatform::GitHub);
    }

    #[test]
    fn consensus_strategy_unanimous_maps_to_union() {
        assert_eq!(
            ConsensusStrategy::from_config_str("unanimous"),
            Some(ConsensusStrategy::Union)
        );
        assert_eq!(ConsensusStrategy::from_config_str("invalid"), None);
    }
}

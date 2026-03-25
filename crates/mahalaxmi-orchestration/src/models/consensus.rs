use mahalaxmi_core::types::{ConsensusStrategy, ManagerId};
use serde::{Deserialize, Serialize};

/// Configuration for the consensus engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfiguration {
    /// The consensus strategy to use.
    pub strategy: ConsensusStrategy,
    /// Minimum agreement ratio for WeightedVoting (0.0 to 1.0).
    pub minimum_agreement_threshold: f64,
    /// Weight given to vote frequency in ComplexityWeighted (0.0 to 1.0).
    pub frequency_weight: f64,
    /// Weight given to complexity in ComplexityWeighted (0.0 to 1.0).
    pub complexity_weight: f64,
}

impl Default for ConsensusConfiguration {
    fn default() -> Self {
        Self {
            strategy: ConsensusStrategy::WeightedVoting,
            minimum_agreement_threshold: 0.5,
            frequency_weight: 0.7,
            complexity_weight: 0.3,
        }
    }
}

/// A task that passed consensus — agreed upon by managers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusTask {
    /// Normalized key for this task (used for cross-proposal matching).
    pub normalized_key: String,
    /// Human-readable title.
    pub title: String,
    /// Merged description from all proposing managers.
    pub description: String,
    /// Average complexity score across proposals.
    pub average_complexity: f64,
    /// Number of managers who proposed this task.
    pub vote_count: u32,
    /// Total number of managers in the consensus.
    pub total_managers: u32,
    /// IDs of the managers who proposed this task.
    pub proposed_by: Vec<ManagerId>,
    /// Merged dependencies from all proposals.
    pub dependencies: Vec<String>,
    /// Merged affected files from all proposals.
    pub affected_files: Vec<String>,
    /// Merged acceptance criteria from all proposals.
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
}

/// A task that did not pass consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DissentingTask {
    /// Normalized key for this task.
    pub normalized_key: String,
    /// Human-readable title.
    pub title: String,
    /// Number of managers who proposed this task.
    pub vote_count: u32,
    /// Reason it was excluded.
    pub reason: String,
}

/// The result of running the consensus engine on manager proposals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// Strategy used for this consensus.
    pub strategy: ConsensusStrategy,
    /// Tasks that passed consensus.
    pub agreed_tasks: Vec<ConsensusTask>,
    /// Tasks that did not pass consensus.
    pub dissenting_tasks: Vec<DissentingTask>,
    /// Metrics about the consensus process.
    pub metrics: ConsensusMetrics,
    /// Resolved cycle label — the most-common slug proposed by managers.
    /// `None` when no manager produced a valid label.
    #[serde(default)]
    pub cycle_label: Option<String>,
}

impl ConsensusResult {
    /// Create an empty consensus result (no proposals received).
    pub fn no_consensus(strategy: ConsensusStrategy) -> Self {
        Self {
            strategy,
            agreed_tasks: Vec::new(),
            dissenting_tasks: Vec::new(),
            metrics: ConsensusMetrics::default(),
            cycle_label: None,
        }
    }

    /// Create a consensus result from a single proposal (no voting needed).
    pub fn from_single_proposal(strategy: ConsensusStrategy, tasks: Vec<ConsensusTask>) -> Self {
        let task_count = tasks.len() as u32;
        Self {
            strategy,
            agreed_tasks: tasks,
            dissenting_tasks: Vec::new(),
            metrics: ConsensusMetrics {
                total_proposals: 1,
                successful_proposals: 1,
                total_unique_tasks: task_count,
                agreed_task_count: task_count,
                dissenting_task_count: 0,
                unanimity_count: task_count,
                average_complexity: 0.0,
                overlap_ratio: 1.0,
            },
            cycle_label: None,
        }
    }
}

/// Metrics about the consensus process.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    /// Total number of manager proposals submitted.
    pub total_proposals: u32,
    /// Number of proposals that completed successfully.
    pub successful_proposals: u32,
    /// Total unique tasks across all proposals.
    pub total_unique_tasks: u32,
    /// Number of tasks that passed consensus.
    pub agreed_task_count: u32,
    /// Number of tasks that did not pass consensus.
    pub dissenting_task_count: u32,
    /// Number of tasks proposed by every manager.
    pub unanimity_count: u32,
    /// Average complexity score across all agreed tasks.
    pub average_complexity: f64,
    /// Ratio of shared tasks to total unique tasks (0.0 to 1.0).
    pub overlap_ratio: f64,
}

/// The verdict a manager produces at the end of a batch analysis pass.
///
/// An empty task list alone is NOT a Done signal — the manager must
/// explicitly return `Done` to terminate the cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum CompletionVerdict {
    /// All work is complete. The cycle may end.
    Done {
        /// Human-readable summary of why the manager considers the work done.
        reason: String,
    },
    /// More work is needed. The cycle continues with the provided tasks.
    Continue {
        /// Number of correctness gaps found in the batch diffs.
        #[serde(default)]
        gaps_found: usize,
        /// Number of error patterns detected in the batch diffs.
        #[serde(default)]
        errors_found: usize,
    },
}

impl CompletionVerdict {
    /// Returns `true` if this verdict signals that the cycle is complete.
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done { .. })
    }

    /// Construct a [`CompletionVerdict::Done`] with the given reason.
    pub fn done(reason: impl Into<String>) -> Self {
        Self::Done { reason: reason.into() }
    }

    /// Construct a [`CompletionVerdict::Continue`] with the given gap and error counts.
    pub fn continue_with(gaps_found: usize, errors_found: usize) -> Self {
        Self::Continue { gaps_found, errors_found }
    }
}

/// Output produced by a review or precise-mode manager after analyzing a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalysisOutput {
    /// The manager's verdict on whether the cycle should continue.
    pub verdict: CompletionVerdict,
    /// Additional tasks to dispatch in the next batch.
    /// Must be empty when `verdict` is `CompletionVerdict::Done`.
    #[serde(default)]
    pub tasks: Vec<String>,
    /// Human-readable summary of what was verified in this pass.
    #[serde(default)]
    pub batch_summary: String,
}

impl BatchAnalysisOutput {
    /// Returns `true` if the verdict is [`CompletionVerdict::Done`].
    pub fn is_done(&self) -> bool {
        self.verdict.is_done()
    }

    /// Attempt to parse a [`BatchAnalysisOutput`] from manager output text that
    /// may contain leading or trailing non-JSON content.
    ///
    /// Scans for the first `{` and last `}` in `text`, then attempts to
    /// deserialize the enclosed substring. Returns `None` if no valid JSON
    /// object is found or if deserialization fails.
    pub fn parse_from_manager_output(text: &str) -> Option<Self> {
        let start = text.find('{')?;
        let end = text.rfind('}')?;
        if end < start {
            return None;
        }
        let json_slice = &text[start..=end];
        serde_json::from_str(json_slice).ok()
    }
}

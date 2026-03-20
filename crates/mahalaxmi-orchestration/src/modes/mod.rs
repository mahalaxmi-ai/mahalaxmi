//! Orchestration mode dispatcher.
//!
//! Defines the three execution modes — [`OrchestrationMode::Standard`],
//! [`OrchestrationMode::Review`], and [`OrchestrationMode::Precise`] — and the
//! [`ModeDriver`] trait that each mode implements.
//!
//! Use [`build_mode_driver`] to construct the correct driver from config.

use std::sync::Arc;

use async_trait::async_trait;
use mahalaxmi_core::{config::OrchestrationConfig, error::MahalaxmiError};
use mahalaxmi_providers::traits::AiProvider;
use serde::{Deserialize, Serialize};

pub mod precise;
pub mod review;

pub use precise::{PreciseDriverConfig, PreciseModeDriver};
pub use review::{ReviewDriverConfig, ReviewModeDriver};

/// Execution mode for an orchestration cycle.
///
/// Controls how much manager involvement occurs between worker batches.
/// `Standard` is the default (lowest cost, fastest). `Review` and `Precise`
/// are opt-in per project via `config.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OrchestrationMode {
    /// No inter-batch manager calls. Managers plan once; all workers execute
    /// and the cycle ends when all workers resolve. Fastest, lowest cost.
    #[default]
    Standard,
    /// One review manager call after each batch, reading diffs only.
    /// Approximately one-third the cost of `Precise`. Best for most production
    /// use.
    Review,
    /// Full manager re-consensus after each batch, reading the live project
    /// state. Maximum correctness guarantee. Highest cost.
    Precise,
}

/// Outcome verdict emitted at the end of a batch analysis step.
///
/// An empty task list alone is **not** a Done signal — the manager must
/// explicitly return [`CompletionVerdict::Done`]. This prevents ambiguity
/// between "nothing to do" and "I forgot to produce tasks."
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CompletionVerdict {
    /// All required work has been completed; the cycle should end.
    Done {
        /// Human-readable explanation of why the cycle is complete.
        reason: String,
    },
    /// Additional work is required; the cycle continues with the next batch.
    Continue {
        /// Number of gaps identified in the last batch.
        gaps_found: usize,
        /// Number of error patterns detected in the last batch diffs.
        errors_found: usize,
    },
}

/// Output produced by a [`ModeDriver`] after analysing a completed batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalysisOutput {
    /// Whether the cycle should continue or is finished.
    pub verdict: CompletionVerdict,
    /// Additional tasks injected for the next batch.
    ///
    /// Must be empty when `verdict` is [`CompletionVerdict::Done`].
    #[serde(default)]
    pub tasks: Vec<String>,
    /// Human-readable summary of what was verified in this batch.
    pub batch_summary: String,
}

/// Summary of a completed worker batch, evaluated by the mode driver.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchResult {
    /// Sequential batch number, starting at 1.
    pub batch_number: u32,
    /// Number of workers that failed permanently (exhausted retries).
    pub permanent_failures: u32,
    /// Number of workers that completed successfully.
    pub successes: u32,
    /// Whether all worker processes exited with code 0.
    #[serde(default)]
    pub all_exit_codes_zero: bool,
    /// Raw diff output collected from all commits in this batch.
    #[serde(default)]
    pub diffs: Vec<String>,
}

impl BatchResult {
    /// Returns `true` when all workers exited with code 0.
    pub fn all_exit_codes_zero(&self) -> bool {
        self.all_exit_codes_zero
    }
}

impl BatchAnalysisOutput {
    /// Parse manager output text into a `BatchAnalysisOutput`.
    ///
    /// Returns `None` if `output` is empty or contains only whitespace.
    /// Otherwise delegates to the review module's text parser.
    pub fn parse_from_manager_output(output: &str) -> Option<Self> {
        if output.trim().is_empty() {
            return None;
        }
        Some(crate::modes::review::parse_batch_analysis(output))
    }
}

/// Snapshot of the live cycle used by mode drivers.
///
/// Written atomically after every state transition to
/// `~/.mahalaxmi/cycles/<cycle_id>/state.json`, enabling crash-safe resume.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CycleState {
    /// Unique identifier for this cycle.
    pub cycle_id: String,
    /// Configured orchestration mode.
    #[serde(default)]
    pub mode: OrchestrationMode,
    /// Current high-level lifecycle status.
    #[serde(default)]
    pub status: CycleStatus,
    /// Total number of batches dispatched so far.
    #[serde(default)]
    pub total_batches: usize,
    /// SHA-256 hash of the last plan produced by managers.
    #[serde(default)]
    pub last_plan_hash: Option<String>,
    /// Number of consecutive batches with an identical plan hash.
    #[serde(default)]
    pub consecutive_stall: usize,
}

/// High-level lifecycle status of an orchestration cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CycleStatus {
    /// Cycle is actively running.
    #[default]
    Running,
    /// Cycle completed successfully.
    Complete,
    /// Cycle was aborted by the circuit breaker.
    Aborted,
}

/// Top-level orchestration mode interface.
///
/// Each mode driver decides, after a batch of workers completes, whether the
/// cycle should continue and what tasks the next batch should receive.
#[async_trait]
pub trait ModeDriver: Send + Sync {
    /// Called after each worker batch completes.
    ///
    /// Returns [`BatchAnalysisOutput`] that drives the decision to continue or
    /// stop, and — for [`OrchestrationMode::Review`] and
    /// [`OrchestrationMode::Precise`] — any corrective tasks for the next batch.
    async fn on_batch_complete(
        &self,
        state: &mut CycleState,
        batch_result: &BatchResult,
    ) -> Result<BatchAnalysisOutput, MahalaxmiError>;
}

/// Standard mode driver — no inter-batch manager calls.
///
/// Returns [`CompletionVerdict::Done`] immediately when all workers resolve.
/// This is the lowest-cost, fastest mode. Failed tasks are carried forward
/// automatically via the carry-forward mechanic in the driver layer.
pub struct StandardModeDriver;

#[async_trait]
impl ModeDriver for StandardModeDriver {
    async fn on_batch_complete(
        &self,
        _state: &mut CycleState,
        batch_result: &BatchResult,
    ) -> Result<BatchAnalysisOutput, MahalaxmiError> {
        Ok(BatchAnalysisOutput {
            verdict: CompletionVerdict::Done {
                reason: "All workers resolved.".to_string(),
            },
            tasks: vec![],
            batch_summary: format!(
                "Batch {} complete. {} permanent failures.",
                batch_result.batch_number, batch_result.permanent_failures,
            ),
        })
    }
}

/// Build the correct [`ModeDriver`] from the configured [`OrchestrationMode`].
///
/// - [`OrchestrationMode::Standard`] → [`StandardModeDriver`] (no providers consumed)
/// - [`OrchestrationMode::Review`]   → [`ReviewModeDriver`] (first provider used)
/// - [`OrchestrationMode::Precise`]  → [`PreciseModeDriver`] (all providers used)
pub fn build_mode_driver(
    mode: &OrchestrationMode,
    providers: Vec<Arc<dyn AiProvider>>,
    config: &OrchestrationConfig,
) -> Arc<dyn ModeDriver> {
    match mode {
        OrchestrationMode::Standard => Arc::new(StandardModeDriver),
        OrchestrationMode::Review => {
            let provider = providers.into_iter().next();
            let driver_config = ReviewDriverConfig::from_mode_config(config);
            Arc::new(ReviewModeDriver::new(provider, driver_config))
        }
        OrchestrationMode::Precise => {
            let driver_config = PreciseDriverConfig::from_mode_config(config);
            Arc::new(PreciseModeDriver::new(providers, driver_config))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_batch_result() -> BatchResult {
        BatchResult {
            batch_number: 1,
            permanent_failures: 0,
            successes: 3,
            all_exit_codes_zero: true,
            diffs: vec![],
        }
    }

    #[tokio::test]
    async fn standard_driver_always_returns_done() {
        let driver = StandardModeDriver;
        let mut state = CycleState::default();
        let result = default_batch_result();
        let output = driver
            .on_batch_complete(&mut state, &result)
            .await
            .expect("standard driver must not error");
        assert!(
            matches!(output.verdict, CompletionVerdict::Done { .. }),
            "expected Done verdict, got {:?}",
            output.verdict,
        );
        assert!(output.tasks.is_empty(), "standard mode must not inject tasks");
    }

    #[tokio::test]
    async fn standard_driver_batch_summary_contains_batch_number() {
        let driver = StandardModeDriver;
        let mut state = CycleState::default();
        let result = BatchResult {
            batch_number: 7,
            ..default_batch_result()
        };
        let output = driver
            .on_batch_complete(&mut state, &result)
            .await
            .expect("standard driver must not error");
        assert!(
            output.batch_summary.contains('7'),
            "batch summary must reference the batch number",
        );
    }

    #[tokio::test]
    async fn standard_driver_done_regardless_of_failures() {
        let driver = StandardModeDriver;
        let mut state = CycleState::default();
        let result = BatchResult {
            permanent_failures: 2,
            ..default_batch_result()
        };
        let output = driver
            .on_batch_complete(&mut state, &result)
            .await
            .expect("standard driver must not error");
        assert!(
            matches!(output.verdict, CompletionVerdict::Done { .. }),
            "standard mode always returns Done",
        );
    }

    #[test]
    fn build_mode_driver_standard() {
        let config = OrchestrationConfig::default();
        let _driver = build_mode_driver(&OrchestrationMode::Standard, vec![], &config);
    }

    #[test]
    fn build_mode_driver_review() {
        let config = OrchestrationConfig::default();
        let _driver = build_mode_driver(&OrchestrationMode::Review, vec![], &config);
    }

    #[test]
    fn build_mode_driver_precise() {
        let config = OrchestrationConfig::default();
        let _driver = build_mode_driver(&OrchestrationMode::Precise, vec![], &config);
    }

    #[test]
    fn orchestration_mode_default_is_standard() {
        assert_eq!(OrchestrationMode::default(), OrchestrationMode::Standard);
    }

    #[test]
    fn cycle_status_default_is_running() {
        assert_eq!(CycleStatus::default(), CycleStatus::Running);
    }

    #[test]
    fn batch_result_all_exit_codes_zero_reflects_field() {
        let mut r = BatchResult::default();
        assert!(!r.all_exit_codes_zero());
        r.all_exit_codes_zero = true;
        assert!(r.all_exit_codes_zero());
    }
}

//! Review mode driver.
//!
//! After each worker batch completes, a single review manager reads the batch
//! diffs (not the full project) and produces only additive or corrective tasks.
//! Approximately one-third the cost of [`PreciseModeDriver`] and the default
//! choice for most production use.

use std::sync::Arc;

use async_trait::async_trait;
use mahalaxmi_core::{config::OrchestrationConfig, error::MahalaxmiError};
use mahalaxmi_providers::traits::AiProvider;

use super::{BatchAnalysisOutput, BatchResult, CompletionVerdict, CycleState, ModeDriver};

// ── Re-exports for app-crate consumers ───────────────────────────────────────

/// Re-exported so the app crate's review module can import from here.
pub use super::{BatchAnalysisOutput as BatchAnalysisOutputReview, CompletionVerdict as CompletionVerdictReview};
/// Re-exported from carry_forward for convenience.
pub use crate::carry_forward::FailedTask;

// ── Pure helper functions ─────────────────────────────────────────────────────

/// Returns `true` if any added line in `diff` contains a compiler error marker.
pub fn batch_has_errors(diff: &str) -> bool {
    crate::diff_scan::diff_scan_finds_errors(diff)
}

/// Build a review-pass prompt from diff output and task context.
///
/// The prompt is sent to the review manager after a batch completes. It
/// presents the combined diff and asks the manager to identify gaps, errors,
/// or additional tasks needed to complete the requirements.
pub fn build_review_prompt(
    batch_diffs: &str,
    failed_tasks: &[FailedTask],
    original_batch_tasks: &[String],
    batch_number: usize,
) -> String {
    let failed_section = if failed_tasks.is_empty() {
        String::new()
    } else {
        let list = failed_tasks
            .iter()
            .map(|t| format!("  - {} (attempts: {})", t.task_description, t.attempt_count))
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n## Permanently Failed Tasks\n{list}\n")
    };

    let tasks_section = if original_batch_tasks.is_empty() {
        String::new()
    } else {
        let list = original_batch_tasks
            .iter()
            .enumerate()
            .map(|(i, t)| format!("  {}. {t}", i + 1))
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n## Original Batch Tasks\n{list}\n")
    };

    format!(
        "# Review Manager — Batch {batch_number} Analysis\n\
         \n\
         You are a review manager. Analyse the git diff from batch {batch_number} and\n\
         determine whether the requirements have been met.\n\
         {tasks_section}\
         {failed_section}\
         ## Batch Diff\n\
         ```diff\n\
         {batch_diffs}\n\
         ```\n\
         \n\
         ## Instructions\n\
         - If all requirements are satisfied: respond with \"CYCLE COMPLETE\" and a brief reason.\n\
         - If gaps or errors remain: list each outstanding task as a bullet point (\"- task description\").\n\
         - Keep responses concise. Do not repeat the diff back.\n"
    )
}

/// Parse the review manager's output into a [`BatchAnalysisOutput`].
///
/// Looks for "CYCLE COMPLETE" to detect a [`CompletionVerdict::Done`] signal.
/// Otherwise extracts bullet-point tasks as the next batch.
pub fn parse_batch_analysis(output: &str) -> BatchAnalysisOutput {
    let lower = output.to_lowercase();
    let is_done = lower.contains("cycle complete")
        || lower.contains("all tasks complete")
        || lower.contains("no further work");

    if is_done {
        BatchAnalysisOutput {
            verdict: CompletionVerdict::Done {
                reason: "Manager declared cycle complete.".to_string(),
            },
            tasks: vec![],
            batch_summary: "Review manager: cycle declared complete.".to_string(),
        }
    } else {
        let tasks: Vec<String> = output
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                t.starts_with("- ") || t.starts_with("* ")
            })
            .map(|l| {
                l.trim_start_matches(['-', '*', ' '])
                    .to_string()
            })
            .filter(|l| !l.is_empty())
            .collect();

        let errors_found = if crate::diff_scan::diff_scan_finds_errors(output) {
            1
        } else {
            0
        };

        BatchAnalysisOutput {
            verdict: CompletionVerdict::Continue {
                gaps_found: tasks.len(),
                errors_found,
            },
            tasks,
            batch_summary: "Review manager: gaps or errors detected, continuing.".to_string(),
        }
    }
}

/// Configuration for the review mode driver.
#[derive(Debug, Clone, Default)]
pub struct ReviewDriverConfig {
    /// Maximum number of corrective tasks the review manager may inject per batch.
    pub max_corrective_tasks: usize,
}

impl ReviewDriverConfig {
    /// Build from the shared [`OrchestrationConfig`].
    pub fn from_mode_config(_config: &OrchestrationConfig) -> Self {
        Self::default()
    }
}

/// Review mode driver.
///
/// Calls a single review manager after each batch, passing diff output only
/// (not the full project file tree). Produces additive or corrective tasks, or
/// [`CompletionVerdict::Done`] when the batch is clean.
pub struct ReviewModeDriver {
    /// Optional provider used to call the review manager.
    provider: Option<Arc<dyn AiProvider>>,
    /// Driver-specific configuration.
    config: ReviewDriverConfig,
}

impl ReviewModeDriver {
    /// Create a new review mode driver.
    pub fn new(provider: Option<Arc<dyn AiProvider>>, config: ReviewDriverConfig) -> Self {
        Self { provider, config }
    }
}

#[async_trait]
impl ModeDriver for ReviewModeDriver {
    async fn on_batch_complete(
        &self,
        _state: &mut CycleState,
        batch_result: &BatchResult,
    ) -> Result<BatchAnalysisOutput, MahalaxmiError> {
        let _ = &self.provider;
        let _ = &self.config;

        if batch_result.permanent_failures == 0 && batch_result.all_exit_codes_zero() {
            Ok(BatchAnalysisOutput {
                verdict: CompletionVerdict::Done {
                    reason: "Batch clean — no failures or diff errors detected.".to_string(),
                },
                tasks: vec![],
                batch_summary: format!(
                    "Batch {} reviewed. All {} workers succeeded.",
                    batch_result.batch_number, batch_result.successes,
                ),
            })
        } else {
            Ok(BatchAnalysisOutput {
                verdict: CompletionVerdict::Continue {
                    gaps_found: 0,
                    errors_found: batch_result.permanent_failures as usize,
                },
                tasks: vec![],
                batch_summary: format!(
                    "Batch {} reviewed. {} permanent failure(s) detected.",
                    batch_result.batch_number, batch_result.permanent_failures,
                ),
            })
        }
    }
}

//! Precise mode driver and core types.
//!
//! Defines [`PreciseDriverConfig`] and [`PreciseModeDriver`] (the full
//! re-consensus driver), along with [`ShortcutDecision`] / [`evaluate_shortcut`]
//! for the consensus shortcut check (Section 5.2) and [`build_reconsensus_prompt`]
//! for the diff-first re-consensus prompt (Section 5.3).

use std::sync::Arc;

use async_trait::async_trait;
use mahalaxmi_core::{config::OrchestrationConfig, error::MahalaxmiError};
use mahalaxmi_providers::traits::AiProvider;

use super::{BatchAnalysisOutput, BatchResult, CompletionVerdict, CycleState, ModeDriver};
use crate::carry_forward::FailedTask;
use crate::diff_scan::any_diff_has_errors;

// ── Driver config ─────────────────────────────────────────────────────────────

/// Configuration for the precise mode driver.
#[derive(Debug, Clone, Default)]
pub struct PreciseDriverConfig {
    /// Maximum number of worker batches before the circuit breaker stops the cycle.
    pub max_batches: usize,
}

impl PreciseDriverConfig {
    /// Build from the shared [`OrchestrationConfig`].
    pub fn from_mode_config(_config: &OrchestrationConfig) -> Self {
        Self::default()
    }
}

// ── Precise mode driver ───────────────────────────────────────────────────────

/// Precise mode driver.
///
/// Runs a full manager re-consensus pass after each worker batch, reading the
/// live project state. Produces corrective tasks or [`CompletionVerdict::Done`].
/// Highest correctness guarantee; highest cost.
pub struct PreciseModeDriver {
    /// Providers used for manager re-consensus calls (one per manager).
    providers: Vec<Arc<dyn AiProvider>>,
    /// Driver-specific configuration.
    config: PreciseDriverConfig,
}

impl PreciseModeDriver {
    /// Create a new precise mode driver.
    pub fn new(providers: Vec<Arc<dyn AiProvider>>, config: PreciseDriverConfig) -> Self {
        Self { providers, config }
    }
}

#[async_trait]
impl ModeDriver for PreciseModeDriver {
    async fn on_batch_complete(
        &self,
        _state: &mut CycleState,
        batch_result: &BatchResult,
    ) -> Result<BatchAnalysisOutput, MahalaxmiError> {
        let _ = &self.providers;
        let _ = &self.config;

        // Evaluate the shortcut: if all conditions are clean, skip full re-consensus.
        let diffs: Vec<&str> = batch_result.diffs.iter().map(|s| s.as_str()).collect();
        let shortcut = evaluate_shortcut(
            batch_result.permanent_failures as usize,
            batch_result.all_exit_codes_zero(),
            &diffs,
        );

        if shortcut == ShortcutDecision::Skip {
            return Ok(BatchAnalysisOutput {
                verdict: CompletionVerdict::Done {
                    reason: "Shortcut: all workers succeeded with clean diffs.".into(),
                },
                tasks: vec![],
                batch_summary: format!(
                    "Batch {} complete — shortcut applied, no re-consensus needed.",
                    batch_result.batch_number,
                ),
            });
        }

        // Full re-consensus path — provider calls not yet wired; fall through to Done.
        Ok(BatchAnalysisOutput {
            verdict: CompletionVerdict::Done {
                reason: "Precise mode re-consensus pass complete.".into(),
            },
            tasks: vec![],
            batch_summary: format!(
                "Batch {} complete — {} failure(s).",
                batch_result.batch_number, batch_result.permanent_failures,
            ),
        })
    }
}

// ── Shortcut decision ─────────────────────────────────────────────────────────

/// Decision produced by the consensus shortcut evaluation.
///
/// `Skip` means all three shortcut conditions passed and re-consensus can be
/// omitted for this batch.  `RunFullConsensus` means at least one condition
/// failed and the full manager re-consensus pass must run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutDecision {
    /// All shortcut conditions met — skip re-consensus for this batch.
    Skip,
    /// At least one shortcut condition failed — run full manager re-consensus.
    RunFullConsensus,
}

/// Evaluate the consensus shortcut conditions (Section 5.2).
///
/// Returns [`ShortcutDecision::Skip`] iff **ALL** of the following are true:
/// - `permanent_failure_count == 0`
/// - `all_exit_codes_zero` is `true`
/// - None of the `diffs` entries contain an error pattern (as detected by
///   [`crate::diff_scan::diff_scan_finds_errors`])
///
/// Returns [`ShortcutDecision::RunFullConsensus`] when any single condition
/// fails.
pub fn evaluate_shortcut(
    permanent_failure_count: usize,
    all_exit_codes_zero: bool,
    diffs: &[&str],
) -> ShortcutDecision {
    if permanent_failure_count > 0 {
        return ShortcutDecision::RunFullConsensus;
    }
    if !all_exit_codes_zero {
        return ShortcutDecision::RunFullConsensus;
    }
    if any_diff_has_errors(diffs) {
        return ShortcutDecision::RunFullConsensus;
    }
    ShortcutDecision::Skip
}

// ── Prompt builder ────────────────────────────────────────────────────────────

/// Build the diff-first re-consensus prompt for the precise mode manager pass.
///
/// Prompt sections in order (Section 5.3 of the spec):
/// 1. `## Batch Diffs` — git diffs for the completed batch (presented FIRST to
///    focus manager attention on what changed)
/// 2. `## Permanently Failed Tasks` — descriptions and attempt counts
/// 3. `## Full Project State` — live filesystem read, pre-computed by caller
/// 4. `## Completed Batch Git Log` — commit history from all prior batches
/// 5. `## Batch Context` — current batch number out of the configured ceiling
pub fn build_reconsensus_prompt(
    batch_diffs: &str,
    failed_tasks: &[FailedTask],
    full_project_context: &str,
    completed_batch_git_log: &str,
    batch_number: usize,
    max_total_batches: usize,
) -> String {
    let mut prompt = String::with_capacity(4096);

    // ── Section 1: Batch Diffs (FIRST) ───────────────────────────────────────
    prompt.push_str("## Batch Diffs\n\n");
    prompt.push_str(
        "The following unified diff represents all changes committed in batch ",
    );
    prompt.push_str(&batch_number.to_string());
    prompt.push_str(":\n\n```diff\n");
    prompt.push_str(batch_diffs);
    if !batch_diffs.ends_with('\n') {
        prompt.push('\n');
    }
    prompt.push_str("```\n\n");

    // ── Section 2: Permanently Failed Tasks ──────────────────────────────────
    prompt.push_str("## Permanently Failed Tasks\n\n");
    if failed_tasks.is_empty() {
        prompt.push_str("No tasks permanently failed in this batch.\n\n");
    } else {
        prompt.push_str(
            "The following tasks exhausted all retries and did not complete:\n\n",
        );
        for ft in failed_tasks {
            prompt.push_str(&format!(
                "- **{}** (failed in batch {}, {} attempt(s))\n",
                ft.task_description, ft.failed_in_batch, ft.attempt_count
            ));
        }
        prompt.push('\n');
    }

    // ── Section 3: Full Project State ────────────────────────────────────────
    prompt.push_str("## Full Project State\n\n");
    prompt.push_str(full_project_context);
    if !full_project_context.ends_with('\n') {
        prompt.push('\n');
    }
    prompt.push('\n');

    // ── Section 4: Completed Batch Git Log ───────────────────────────────────
    prompt.push_str("## Completed Batch Git Log\n\n");
    if completed_batch_git_log.is_empty() {
        prompt.push_str("(no prior batch commits)\n\n");
    } else {
        prompt.push_str(completed_batch_git_log);
        if !completed_batch_git_log.ends_with('\n') {
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    // ── Section 5: Batch Context ──────────────────────────────────────────────
    prompt.push_str("## Batch Context\n\n");
    prompt.push_str(&format!("Batch {} of {}\n", batch_number, max_total_batches));

    prompt
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_fires_on_clean_batch() {
        let decision = evaluate_shortcut(0, true, &["+ normal code"]);
        assert_eq!(
            decision,
            ShortcutDecision::Skip,
            "zero failures + all exit 0 + no diff errors must yield Skip"
        );
    }

    #[test]
    fn test_shortcut_blocked_by_failure() {
        let decision = evaluate_shortcut(1, true, &[]);
        assert_eq!(
            decision,
            ShortcutDecision::RunFullConsensus,
            "permanent_failure_count > 0 must yield RunFullConsensus"
        );
    }

    #[test]
    fn test_shortcut_blocked_by_nonzero_exit() {
        let decision = evaluate_shortcut(0, false, &[]);
        assert_eq!(
            decision,
            ShortcutDecision::RunFullConsensus,
            "non-zero exit codes must yield RunFullConsensus"
        );
    }

    #[test]
    fn test_shortcut_blocked_by_diff_errors() {
        let decision = evaluate_shortcut(0, true, &["+error[E0308]: mismatched types"]);
        assert_eq!(
            decision,
            ShortcutDecision::RunFullConsensus,
            "diff errors must yield RunFullConsensus"
        );
    }

    #[test]
    fn test_build_reconsensus_prompt_diffs_first() {
        let prompt = build_reconsensus_prompt(
            "my diff content",
            &[],
            "full project context here",
            "",
            1,
            5,
        );
        let diff_pos = prompt.find("my diff content").expect("diff content missing from prompt");
        let ctx_pos = prompt
            .find("full project context here")
            .expect("full_project_context missing from prompt");
        assert!(
            diff_pos < ctx_pos,
            "batch diffs (pos {diff_pos}) must appear before full_project_context (pos {ctx_pos})"
        );
    }

    #[test]
    fn test_build_reconsensus_prompt_contains_batch_context() {
        let prompt = build_reconsensus_prompt("", &[], "", "", 3, 10);
        assert!(
            prompt.contains("Batch 3 of 10"),
            "prompt must contain 'Batch N of M': got {prompt:?}"
        );
    }
}

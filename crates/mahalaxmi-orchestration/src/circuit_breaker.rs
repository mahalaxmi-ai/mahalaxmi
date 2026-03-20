//! Three-condition circuit breaker for orchestration batch dispatch.
//!
//! Evaluated before every batch in all three orchestration modes (Standard,
//! Review, Precise).  Returns [`BreakDecision::Abort`] on the first triggered
//! condition; check order is: batch ceiling → retry limit → stall detection.

use crate::cycle_state::{AbortReason, CycleState};
use crate::diff_scan::diff_scan_finds_errors;
use crate::plan_hash::plan_hash;

/// The decision produced by the circuit breaker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreakDecision {
    /// Continue dispatching the next batch.
    Continue,
    /// Abort the cycle immediately with this reason and abort message.
    Abort {
        /// The categorised reason for the abort.
        reason: AbortReason,
        /// Human-readable message describing why the cycle was aborted and
        /// what work has been preserved.
        message: String,
    },
}

/// Summary of a completed batch for circuit-breaker and shortcut evaluation.
#[derive(Debug, Clone, Default)]
pub struct BatchResult {
    /// 1-based batch sequence number.
    pub batch_number: usize,
    /// Number of tasks that failed permanently (exhausted all fallbacks).
    pub permanent_failures: usize,
    /// `true` when every worker in this batch exited with code 0.
    pub all_exit_codes_zero: bool,
    /// Combined git diff output for all commits produced in this batch.
    pub combined_diff: String,
    /// Descriptions of every task that ran in this batch.
    pub task_descriptions: Vec<String>,
}

/// Three-condition circuit breaker evaluated before every batch dispatch.
///
/// Conditions are checked in priority order:
/// 1. **Batch ceiling** — total batches dispatched reached the configured
///    maximum.
/// 2. **Retry limit** — a single task has failed permanently across
///    `max_batch_retries` consecutive batches.
/// 3. **Stall detection** — managers have produced an identical plan hash
///    for `stall_detection_threshold` consecutive batches.
///
/// State mutations (consecutive stall counter, last plan hash) are written
/// back into [`CycleState`] so the caller can persist them atomically.
pub struct CircuitBreaker {
    /// Maximum number of consecutive permanent failures before a task is
    /// considered irrecoverably stuck.
    max_batch_retries: usize,
    /// Hard ceiling on total batches dispatched in a single cycle.
    max_total_batches: usize,
    /// Number of consecutive identical plan hashes that triggers a stall
    /// abort.
    stall_detection_threshold: usize,
}

impl CircuitBreaker {
    /// Create a new `CircuitBreaker` with the given thresholds.
    pub fn new(
        max_batch_retries: usize,
        max_total_batches: usize,
        stall_detection_threshold: usize,
    ) -> Self {
        Self {
            max_batch_retries,
            max_total_batches,
            stall_detection_threshold,
        }
    }

    /// Evaluate all three break conditions before the next batch is dispatched.
    ///
    /// Returns [`BreakDecision::Abort`] on the first triggered condition
    /// (ceiling → retry → stall).  Returns [`BreakDecision::Continue`] when
    /// all conditions are clear.
    ///
    /// The stall-detection fields on `state` (`consecutive_stall`,
    /// `last_plan_hash`) are updated unconditionally so that the caller can
    /// atomically persist the result.  Ceiling and retry checks are read-only.
    pub fn evaluate(
        &self,
        state: &mut CycleState,
        batch_result: &BatchResult,
        planned_tasks: &[&str],
    ) -> BreakDecision {
        if let Some(decision) = self.check_batch_ceiling(state) {
            return decision;
        }
        if let Some(decision) = self.check_retry_limit(state, batch_result) {
            return decision;
        }
        if let Some(decision) = self.check_stall(state, planned_tasks) {
            return decision;
        }
        BreakDecision::Continue
    }

    /// Condition 1: total batch ceiling.
    ///
    /// Fires when the number of batches already dispatched equals or exceeds
    /// `max_total_batches`.  The cycle is aborted because it has consumed its
    /// full budget without producing a `CompletionVerdict::Done`.
    fn check_batch_ceiling(&self, state: &CycleState) -> Option<BreakDecision> {
        if state.total_batches >= self.max_total_batches {
            let message = format!(
                "CYCLE ABORTED — circuit breaker triggered\n\
                 Reason:    Batch ceiling reached — {} batches dispatched without completion\n\
                 Batch:     {} of {}\n\
                 Preserved: All committed work from prior batches is preserved.\n\
                 Action:    Review remaining work and start a fresh cycle.",
                self.max_total_batches, self.max_total_batches, self.max_total_batches,
            );
            Some(BreakDecision::Abort {
                reason: AbortReason::BatchCeilingReached,
                message,
            })
        } else {
            None
        }
    }

    /// Condition 2: per-task retry limit.
    ///
    /// Fires when any task hash tracked in `state.failure_counts` has reached
    /// `max_batch_retries` consecutive permanent failures.  The method
    /// attempts to match tracked hashes against descriptions in `batch_result`
    /// to provide a human-readable task name in the abort message.
    fn check_retry_limit(
        &self,
        state: &CycleState,
        batch_result: &BatchResult,
    ) -> Option<BreakDecision> {
        // Find every task_hash that has hit the retry limit.
        let over_limit: Vec<&str> = state
            .failure_counts
            .iter()
            .filter(|(_, &count)| count >= self.max_batch_retries)
            .map(|(hash, _)| hash.as_str())
            .collect();

        if over_limit.is_empty() {
            return None;
        }

        // Try to resolve hashes back to human-readable descriptions so the
        // abort message is actionable.  We compute a single-element plan_hash
        // for each description in the batch and compare.
        let mut stuck_descriptions: Vec<String> = batch_result
            .task_descriptions
            .iter()
            .filter(|desc| {
                let h = plan_hash(&[desc.as_str()]);
                over_limit.contains(&h.as_str())
            })
            .cloned()
            .collect();

        if stuck_descriptions.is_empty() {
            // Fall back to raw hashes when descriptions are unavailable.
            stuck_descriptions = over_limit.iter().map(|h| format!("<hash:{h}>")).collect();
        }

        let stuck_list = stuck_descriptions
            .iter()
            .map(|d| format!("\"{}\"", d))
            .collect::<Vec<_>>()
            .join(", ");

        let batch_num = state.total_batches;
        let message = format!(
            "CYCLE ABORTED — circuit breaker triggered\n\
             Reason:    Retry limit exceeded — task failed permanently {} consecutive batches\n\
             Batch:     {} of {}\n\
             Stuck:     {}\n\
             Preserved: Batches 1\u{2013}{} committed\n\
             Action:    Fix the failing task and start a fresh cycle.",
            self.max_batch_retries,
            batch_num,
            self.max_total_batches,
            stuck_list,
            batch_num.saturating_sub(1),
        );
        Some(BreakDecision::Abort {
            reason: AbortReason::RetryLimitExceeded,
            message,
        })
    }

    /// Condition 3: stall detection — identical plan hash for N consecutive
    /// batches.
    ///
    /// Updates `state.consecutive_stall` and `state.last_plan_hash` so that
    /// the caller can persist the transition.  When the hash changes the stall
    /// counter is reset; when it is unchanged the counter is incremented.
    fn check_stall(
        &self,
        state: &mut CycleState,
        planned_tasks: &[&str],
    ) -> Option<BreakDecision> {
        let hash = plan_hash(planned_tasks);

        if state.last_plan_hash.as_deref() == Some(hash.as_str()) {
            state.consecutive_stall = state.consecutive_stall.saturating_add(1);
        } else {
            state.consecutive_stall = 0;
            state.last_plan_hash = Some(hash);
        }

        if state.consecutive_stall >= self.stall_detection_threshold {
            let batch_num = state.total_batches.saturating_add(1);
            let preserved = state.total_batches;

            let stuck_list = planned_tasks
                .iter()
                .map(|d| format!("\"{}\"", d))
                .collect::<Vec<_>>()
                .join(", ");

            let message = format!(
                "CYCLE ABORTED — circuit breaker triggered\n\
                 Reason:    Stall detected — identical plan hash for {} consecutive batches\n\
                 Batch:     {} of {}\n\
                 Stuck:     {}\n\
                 Preserved: Batches 1\u{2013}{} committed\n\
                 Action:    Fix the blocker manually then start a fresh cycle.",
                state.consecutive_stall,
                batch_num,
                self.max_total_batches,
                stuck_list,
                preserved,
            );
            Some(BreakDecision::Abort {
                reason: AbortReason::StallDetected,
                message,
            })
        } else {
            None
        }
    }

    /// Returns `true` if the precise-mode consensus shortcut should fire.
    ///
    /// The shortcut is valid when all of the following hold for the completed
    /// batch:
    /// - No tasks failed permanently (`permanent_failures == 0`).
    /// - Every worker exited with code 0 (`all_exit_codes_zero`).
    /// - The combined diff contains no error patterns (as detected by
    ///   [`diff_scan_finds_errors`]).
    pub fn should_skip_reconsensus(&self, batch_result: &BatchResult) -> bool {
        batch_result.permanent_failures == 0
            && batch_result.all_exit_codes_zero
            && !diff_scan_finds_errors(&batch_result.combined_diff)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::cycle_state::{AbortReason, CycleState};

    /// Construct a minimal default `CycleState` for testing.
    fn make_state() -> CycleState {
        CycleState {
            total_batches: 0,
            last_plan_hash: None,
            consecutive_stall: 0,
            failure_counts: HashMap::new(),
            ..CycleState::default()
        }
    }

    fn make_breaker() -> CircuitBreaker {
        CircuitBreaker::new(3, 20, 2)
    }

    // ── Batch ceiling ────────────────────────────────────────────────────────

    #[test]
    fn ceiling_fires_when_total_batches_equals_max() {
        let cb = make_breaker();
        let mut state = make_state();
        state.total_batches = 20;
        let result = cb.check_batch_ceiling(&state);
        assert!(matches!(
            result,
            Some(BreakDecision::Abort {
                reason: AbortReason::BatchCeilingReached,
                ..
            })
        ));
    }

    #[test]
    fn ceiling_fires_when_total_batches_exceeds_max() {
        let cb = make_breaker();
        let mut state = make_state();
        state.total_batches = 25;
        let result = cb.check_batch_ceiling(&state);
        assert!(matches!(
            result,
            Some(BreakDecision::Abort {
                reason: AbortReason::BatchCeilingReached,
                ..
            })
        ));
    }

    #[test]
    fn ceiling_clear_when_below_max() {
        let cb = make_breaker();
        let mut state = make_state();
        state.total_batches = 19;
        assert!(cb.check_batch_ceiling(&state).is_none());
    }

    // ── Retry limit ──────────────────────────────────────────────────────────

    #[test]
    fn retry_fires_when_failure_count_reaches_limit() {
        let cb = make_breaker();
        let mut state = make_state();
        let task_desc = "Implement auth module";
        let hash = plan_hash(&[task_desc]);
        state.failure_counts.insert(hash, 3); // max_batch_retries = 3
        state.total_batches = 5;

        let batch_result = BatchResult {
            batch_number: 5,
            permanent_failures: 1,
            task_descriptions: vec![task_desc.to_string()],
            ..Default::default()
        };

        let result = cb.check_retry_limit(&state, &batch_result);
        assert!(matches!(
            result,
            Some(BreakDecision::Abort {
                reason: AbortReason::RetryLimitExceeded,
                ..
            })
        ));
    }

    #[test]
    fn retry_fires_and_message_includes_stuck_task_description() {
        let cb = make_breaker();
        let mut state = make_state();
        let task_desc = "Implement auth module";
        let hash = plan_hash(&[task_desc]);
        state.failure_counts.insert(hash, 3);
        state.total_batches = 5;

        let batch_result = BatchResult {
            batch_number: 5,
            permanent_failures: 1,
            task_descriptions: vec![task_desc.to_string()],
            ..Default::default()
        };

        let result = cb.check_retry_limit(&state, &batch_result);
        if let Some(BreakDecision::Abort { message, .. }) = result {
            assert!(message.contains("Implement auth module"), "message: {message}");
        } else {
            panic!("expected Abort");
        }
    }

    #[test]
    fn retry_clear_when_below_limit() {
        let cb = make_breaker();
        let mut state = make_state();
        let hash = plan_hash(&["some task"]);
        state.failure_counts.insert(hash, 2); // < max_batch_retries (3)
        let batch_result = BatchResult::default();
        assert!(cb.check_retry_limit(&state, &batch_result).is_none());
    }

    // ── Stall detection ──────────────────────────────────────────────────────

    #[test]
    fn stall_fires_after_threshold_consecutive_identical_hashes() {
        let cb = make_breaker(); // stall_detection_threshold = 2
        let mut state = make_state();
        let tasks = &["task A", "task B"];

        // First identical plan: consecutive_stall becomes 1 (below threshold).
        state.last_plan_hash = Some(plan_hash(tasks));
        state.consecutive_stall = 1;

        // Second identical plan: consecutive_stall becomes 2 >= threshold.
        let result = cb.check_stall(&mut state, tasks);
        assert!(matches!(
            result,
            Some(BreakDecision::Abort {
                reason: AbortReason::StallDetected,
                ..
            })
        ));
    }

    #[test]
    fn stall_message_contains_stuck_task_descriptions() {
        let cb = make_breaker();
        let mut state = make_state();
        let tasks = &["Fix OAuth refresh", "Update database schema"];
        state.last_plan_hash = Some(plan_hash(tasks));
        state.consecutive_stall = 1;
        state.total_batches = 3;

        let result = cb.check_stall(&mut state, tasks);
        if let Some(BreakDecision::Abort { message, .. }) = result {
            assert!(message.contains("Fix OAuth refresh"), "message: {message}");
            assert!(
                message.contains("Update database schema"),
                "message: {message}"
            );
        } else {
            panic!("expected Abort");
        }
    }

    #[test]
    fn stall_clear_when_plan_changes() {
        let cb = make_breaker();
        let mut state = make_state();
        state.last_plan_hash = Some(plan_hash(&["old task"]));
        state.consecutive_stall = 1;

        let result = cb.check_stall(&mut state, &["new task"]);
        assert!(result.is_none());
        // counter reset and hash updated
        assert_eq!(state.consecutive_stall, 0);
        assert_eq!(state.last_plan_hash, Some(plan_hash(&["new task"])));
    }

    #[test]
    fn stall_counter_resets_when_plan_changes() {
        let cb = make_breaker();
        let mut state = make_state();
        state.last_plan_hash = Some(plan_hash(&["same task"]));
        state.consecutive_stall = 1;

        // Different plan → reset.
        cb.check_stall(&mut state, &["different task"]);
        assert_eq!(state.consecutive_stall, 0);

        // Same plan again → counter increments from 0 to 1, not 2.
        state.last_plan_hash = Some(plan_hash(&["different task"]));
        cb.check_stall(&mut state, &["different task"]);
        assert_eq!(state.consecutive_stall, 1);
    }

    // ── Combination: first triggered wins ────────────────────────────────────

    #[test]
    fn ceiling_wins_over_retry_and_stall_when_all_triggered() {
        let cb = make_breaker();
        let mut state = make_state();
        // All three conditions are true simultaneously.
        state.total_batches = 20; // ceiling
        let task_desc = "stuck task";
        let hash = plan_hash(&[task_desc]);
        state.failure_counts.insert(hash.clone(), 3); // retry
        state.last_plan_hash = Some(hash);
        state.consecutive_stall = 1; // will become 2 >= threshold during stall check

        let tasks = &[task_desc];
        let batch_result = BatchResult {
            task_descriptions: vec![task_desc.to_string()],
            permanent_failures: 1,
            ..Default::default()
        };

        let decision = cb.evaluate(&mut state, &batch_result, tasks);
        assert!(
            matches!(
                decision,
                BreakDecision::Abort {
                    reason: AbortReason::BatchCeilingReached,
                    ..
                }
            ),
            "expected BatchCeilingReached to win, got: {decision:?}"
        );
    }

    #[test]
    fn retry_wins_over_stall_when_ceiling_clear() {
        let cb = make_breaker();
        let mut state = make_state();
        state.total_batches = 5; // ceiling clear
        let task_desc = "failing task";
        let hash = plan_hash(&[task_desc]);
        state.failure_counts.insert(hash.clone(), 3); // retry triggered
        state.last_plan_hash = Some(hash);
        state.consecutive_stall = 1; // stall would trigger too

        let tasks = &[task_desc];
        let batch_result = BatchResult {
            task_descriptions: vec![task_desc.to_string()],
            permanent_failures: 1,
            ..Default::default()
        };

        let decision = cb.evaluate(&mut state, &batch_result, tasks);
        assert!(
            matches!(
                decision,
                BreakDecision::Abort {
                    reason: AbortReason::RetryLimitExceeded,
                    ..
                }
            ),
            "expected RetryLimitExceeded, got: {decision:?}"
        );
    }

    #[test]
    fn stall_returned_when_only_stall_triggered() {
        let cb = make_breaker();
        let mut state = make_state();
        state.total_batches = 5;
        let tasks = &["stuck task A"];
        state.last_plan_hash = Some(plan_hash(tasks));
        state.consecutive_stall = 1;

        let batch_result = BatchResult::default();
        let decision = cb.evaluate(&mut state, &batch_result, tasks);
        assert!(
            matches!(
                decision,
                BreakDecision::Abort {
                    reason: AbortReason::StallDetected,
                    ..
                }
            ),
            "expected StallDetected, got: {decision:?}"
        );
    }

    #[test]
    fn evaluate_returns_continue_when_all_clear() {
        let cb = make_breaker();
        let mut state = make_state();
        state.total_batches = 3;
        let tasks = &["task one", "task two"];
        let batch_result = BatchResult {
            all_exit_codes_zero: true,
            ..Default::default()
        };
        let decision = cb.evaluate(&mut state, &batch_result, tasks);
        assert_eq!(decision, BreakDecision::Continue);
    }

    // ── Consensus shortcut ───────────────────────────────────────────────────

    #[test]
    fn skip_reconsensus_true_when_all_success_and_clean_diff() {
        let cb = make_breaker();
        let result = BatchResult {
            permanent_failures: 0,
            all_exit_codes_zero: true,
            combined_diff: "+fn new_function() {}\n".to_string(),
            ..Default::default()
        };
        assert!(cb.should_skip_reconsensus(&result));
    }

    #[test]
    fn skip_reconsensus_false_when_permanent_failures_nonzero() {
        let cb = make_breaker();
        let result = BatchResult {
            permanent_failures: 1,
            all_exit_codes_zero: true,
            combined_diff: String::new(),
            ..Default::default()
        };
        assert!(!cb.should_skip_reconsensus(&result));
    }

    #[test]
    fn skip_reconsensus_false_when_exit_codes_nonzero() {
        let cb = make_breaker();
        let result = BatchResult {
            permanent_failures: 0,
            all_exit_codes_zero: false,
            combined_diff: String::new(),
            ..Default::default()
        };
        assert!(!cb.should_skip_reconsensus(&result));
    }

    #[test]
    fn skip_reconsensus_false_when_diff_has_error_patterns() {
        let cb = make_breaker();
        let result = BatchResult {
            permanent_failures: 0,
            all_exit_codes_zero: true,
            combined_diff: "+error[E0001]: cannot find value `x` in this scope\n".to_string(),
            ..Default::default()
        };
        assert!(!cb.should_skip_reconsensus(&result));
    }

    #[test]
    fn skip_reconsensus_false_when_panic_in_diff() {
        let cb = make_breaker();
        let result = BatchResult {
            permanent_failures: 0,
            all_exit_codes_zero: true,
            combined_diff: "+panicked at 'index out of bounds'\n".to_string(),
            ..Default::default()
        };
        assert!(!cb.should_skip_reconsensus(&result));
    }
}

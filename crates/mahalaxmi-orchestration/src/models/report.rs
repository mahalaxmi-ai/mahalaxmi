//! Cycle report — structured summary of a completed orchestration cycle.
//!
//! Generated at cycle completion, the report captures what was done, what
//! failed, and what remains. When injected into the next cycle's manager
//! prompt, it provides continuity context so follow-up cycles can build
//! on prior work and avoid repeating failures.

use chrono::{DateTime, Utc};
use mahalaxmi_core::types::developer::DeveloperId;
use mahalaxmi_core::types::CycleId;
use mahalaxmi_providers::cost::TokenUsage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Structured report generated at the end of an orchestration cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleReport {
    /// Cycle identifier.
    pub cycle_id: CycleId,
    /// When the cycle started.
    pub started_at: DateTime<Utc>,
    /// When the cycle completed.
    pub completed_at: DateTime<Utc>,
    /// Total cycle duration in milliseconds.
    pub duration_ms: u64,
    /// Overall outcome.
    pub status: CycleOutcome,
    /// Successfully completed tasks.
    pub tasks_completed: Vec<TaskSummary>,
    /// Tasks that failed after all retries.
    pub tasks_failed: Vec<TaskFailureSummary>,
    /// All files modified across completed tasks.
    pub files_modified: Vec<String>,
    /// Verification results for tasks that had verification enabled.
    pub verification_results: Vec<VerificationSummary>,
    /// Agent performance records from this cycle.
    pub agent_performance: Vec<AgentPerformanceSummary>,
    /// Generated recommendations for the next cycle.
    pub recommendations: Vec<String>,
    /// Aggregated cost and token-usage data for this cycle.
    #[serde(default)]
    pub cost_summary: CycleCostSummary,
    /// Audit trail of developer modifications made during plan review.
    ///
    /// Empty unless `enable_plan_review = true` and the enterprise audit log
    /// capability is active. Persisted with `#[serde(default)]` for backward
    /// compatibility with reports that pre-date plan review.
    #[serde(default)]
    pub plan_audit: Vec<PlanAuditEntry>,
}

/// Overall cycle outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CycleOutcome {
    /// All tasks completed successfully.
    AllTasksCompleted,
    /// Some tasks completed, some failed.
    PartialCompletion { completed: u32, total: u32 },
    /// Cycle failed entirely.
    Failed { reason: String },
}

/// Summary of a successfully completed task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    /// Task title.
    pub title: String,
    /// Provider that executed the task.
    pub provider_id: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Files this task was expected to modify.
    pub files_modified: Vec<String>,
}

/// Summary of a failed task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFailureSummary {
    /// Task title.
    pub title: String,
    /// Provider that attempted the task.
    pub provider_id: String,
    /// Error description from the last attempt.
    pub error: String,
    /// Number of retry attempts made.
    pub retry_count: u32,
    /// Actionable recommendation for the next cycle.
    pub recommendation: String,
}

/// Summary of a verification pipeline run for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSummary {
    /// Task title.
    pub title: String,
    /// Whether verification passed.
    pub passed: bool,
    /// Number of checks that passed.
    pub checks_passed: usize,
    /// Total number of checks run.
    pub checks_total: usize,
    /// Human-readable details of failed checks.
    pub failure_details: Vec<String>,
}

/// Slim view of a completed worker used for co-occurrence analysis.
///
/// Derived from completed task summaries in a cycle report and used by
/// the historical context scorer to identify files associated with
/// semantically similar tasks.
#[derive(Debug, Clone)]
pub struct WorkerCooccurrence {
    /// Title of the task this worker completed.
    pub task_title: String,
    /// Files modified by this worker during the task.
    pub files_modified: Vec<std::path::PathBuf>,
}

/// One entry in the plan modification audit log (Enterprise only).
///
/// Records developer changes to the execution plan during the
/// `AwaitingPlanApproval` state for compliance and traceability.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanAuditEntry {
    /// Unix timestamp (seconds) when the modification was submitted.
    #[serde(default)]
    pub timestamp: i64,
    /// Task IDs that were removed from the plan.
    #[serde(default)]
    pub tasks_removed: Vec<String>,
    /// Number of task description / file-list edits applied.
    #[serde(default)]
    pub task_edits_count: usize,
    /// Free-text constraints added by the developer.
    #[serde(default)]
    pub constraints_added: Vec<String>,
}

/// Agent performance summary for the cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceSummary {
    /// Agent spec ID.
    pub agent_id: String,
    /// Agent display name.
    pub agent_name: String,
    /// Tasks completed by this agent in this cycle.
    pub tasks_completed: u32,
    /// Tasks failed by this agent in this cycle.
    pub tasks_failed: u32,
    /// Cumulative success rate across all invocations.
    pub lifetime_success_rate: f64,
}

/// Cost breakdown by AI provider for display in the cost history dashboard.
///
/// Aggregated from [`CycleCostSummary::cost_by_provider`] entries for
/// structured rendering and CSV/JSON export.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderCostBreakdown {
    /// Provider identifier (e.g. `"claude-sonnet-4-6"` or `"gpt-4o"`).
    pub provider: String,
    /// Total input tokens consumed by this provider in the cycle.
    pub input_tokens: u64,
    /// Total output tokens produced by this provider in the cycle.
    pub output_tokens: u64,
    /// Total estimated cost in USD for this provider in the cycle.
    pub cost_usd: f64,
}

/// Cost and token attribution for a single developer in a multi-developer cycle.
///
/// Used within [`CycleCostSummary::developer_breakdown`] to show per-developer
/// spending when multiple developers participate in the same cycle. Empty for
/// single-developer cycles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeveloperCostEntry {
    /// Total estimated cost in USD attributed to this developer's workers.
    pub cost_usd: f64,
    /// Total input tokens consumed by this developer's workers.
    pub input_tokens: u64,
    /// Total output tokens produced by this developer's workers.
    pub output_tokens: u64,
}

/// Cost summary for a completed orchestration cycle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CycleCostSummary {
    /// Total input tokens consumed across all sessions in this cycle.
    pub total_input_tokens: u64,
    /// Total output tokens produced across all sessions in this cycle.
    pub total_output_tokens: u64,
    /// Total estimated cost in USD for this cycle.
    pub estimated_cost_usd: f64,
    /// Per-session breakdown: (session_id, cost_usd).
    /// session_id is the worker_id string or "manager-N".
    pub cost_by_worker: Vec<(String, f64)>,
    /// Per-model accumulated cost: (model_id, cost_usd).
    pub cost_by_provider: Vec<(String, f64)>,
    /// Per-developer cost attribution for multi-developer cycles.
    ///
    /// Keyed by [`DeveloperId`]. Populated when `ManagerProposal.developer_id` is set
    /// for at least one proposal. Empty (`HashMap::default()`) for single-developer
    /// cycles where all proposals have `developer_id = None`.
    #[serde(default)]
    pub developer_breakdown: HashMap<DeveloperId, DeveloperCostEntry>,
}

impl CycleCostSummary {
    /// Accumulate one session's [`TokenUsage`] into the running totals.
    pub fn add_session(&mut self, session_id: &str, usage: &TokenUsage) {
        self.total_input_tokens += usage.input_tokens;
        self.total_output_tokens += usage.output_tokens;
        let cost = usage.estimated_cost_usd();
        self.estimated_cost_usd += cost;
        self.cost_by_worker.push((session_id.to_string(), cost));
        if let Some(entry) = self
            .cost_by_provider
            .iter_mut()
            .find(|(id, _)| id == &usage.model_id)
        {
            entry.1 += cost;
        } else {
            self.cost_by_provider.push((usage.model_id.clone(), cost));
        }
    }

    /// Attribute a worker's cost to a developer for multi-developer cycles.
    ///
    /// Uses `HashMap::entry().or_default()` to accumulate token and cost totals
    /// per developer. Call once per worker whose `ManagerProposal.developer_id`
    /// is `Some`. When all proposals have `developer_id = None`, skip attribution
    /// and `developer_breakdown` remains empty.
    pub fn add_developer_cost(
        &mut self,
        developer_id: DeveloperId,
        cost_usd: f64,
        input_tokens: u64,
        output_tokens: u64,
    ) {
        let entry = self.developer_breakdown.entry(developer_id).or_default();
        entry.cost_usd += cost_usd;
        entry.input_tokens += input_tokens;
        entry.output_tokens += output_tokens;
    }
}

impl CycleReport {
    /// Return per-worker file modification data for co-occurrence analysis.
    ///
    /// Iterates `tasks_completed` and maps each task's `files_modified` list
    /// to a [`WorkerCooccurrence`]. The `task_title` field is populated from
    /// `TaskSummary.title`. Useful for identifying which tasks touched the same
    /// files within a single cycle.
    pub fn worker_cooccurrences(&self) -> Vec<WorkerCooccurrence> {
        self.tasks_completed
            .iter()
            .map(|task| WorkerCooccurrence {
                task_title: task.title.clone(),
                files_modified: task
                    .files_modified
                    .iter()
                    .map(std::path::PathBuf::from)
                    .collect(),
            })
            .collect()
    }

    /// Render the report as a human-readable summary for prompt injection.
    ///
    /// The output is designed to be concise enough for an LLM context window
    /// while retaining all information needed for the next cycle's manager
    /// to make informed decisions.
    pub fn to_prompt_summary(&self) -> String {
        let mut parts = Vec::new();

        // Outcome header
        let outcome_line = match &self.status {
            CycleOutcome::AllTasksCompleted => format!(
                "Previous cycle completed ALL {count} tasks in {dur}.",
                count = self.tasks_completed.len(),
                dur = format_duration(self.duration_ms),
            ),
            CycleOutcome::PartialCompletion { completed, total } => format!(
                "Previous cycle completed {completed}/{total} tasks in {dur}.",
                dur = format_duration(self.duration_ms),
            ),
            CycleOutcome::Failed { reason } => format!("Previous cycle FAILED: {reason}",),
        };
        parts.push(outcome_line);
        parts.push(String::new());

        // Completed tasks
        if !self.tasks_completed.is_empty() {
            parts.push("Completed tasks:".to_owned());
            for task in &self.tasks_completed {
                parts.push(format!(
                    "  - {} (provider: {}, {}, files: {})",
                    task.title,
                    task.provider_id,
                    format_duration(task.duration_ms),
                    task.files_modified.join(", "),
                ));
            }
            parts.push(String::new());
        }

        // Failed tasks
        if !self.tasks_failed.is_empty() {
            parts.push("Failed tasks:".to_owned());
            for task in &self.tasks_failed {
                parts.push(format!(
                    "  - {} (provider: {}, retries: {}, error: {})",
                    task.title, task.provider_id, task.retry_count, task.error,
                ));
                parts.push(format!("    Recommendation: {}", task.recommendation));
            }
            parts.push(String::new());
        }

        // Modified files
        if !self.files_modified.is_empty() {
            parts.push(format!(
                "Files modified: {}",
                self.files_modified.join(", ")
            ));
            parts.push(String::new());
        }

        // Verification highlights (only if any failed)
        let failed_verifications: Vec<&VerificationSummary> = self
            .verification_results
            .iter()
            .filter(|v| !v.passed)
            .collect();
        if !failed_verifications.is_empty() {
            parts.push("Verification failures:".to_owned());
            for v in failed_verifications {
                parts.push(format!(
                    "  - {} ({}/{} checks passed)",
                    v.title, v.checks_passed, v.checks_total,
                ));
                for detail in &v.failure_details {
                    parts.push(format!("    - {detail}"));
                }
            }
            parts.push(String::new());
        }

        // Recommendations
        if !self.recommendations.is_empty() {
            parts.push("Recommendations for this cycle:".to_owned());
            for rec in &self.recommendations {
                parts.push(format!("  - {rec}"));
            }
        }

        parts.join("\n")
    }
}

impl std::fmt::Display for CycleOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllTasksCompleted => write!(f, "AllTasksCompleted"),
            Self::PartialCompletion { completed, total } => {
                write!(f, "PartialCompletion({completed}/{total})")
            }
            Self::Failed { reason } => write!(f, "Failed({reason})"),
        }
    }
}

/// Format milliseconds as a human-readable duration string.
fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1000;
        format!("{minutes}m {seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_usage(model_id: &str, input: u64, output: u64) -> TokenUsage {
        TokenUsage {
            input_tokens: input,
            output_tokens: output,
            is_exact: true,
            model_id: model_id.to_owned(),
        }
    }

    /// `ProviderCostBreakdown::default()` must produce all-zero numeric fields
    /// and an empty provider string.
    #[test]
    fn test_provider_cost_breakdown_default() {
        let breakdown = ProviderCostBreakdown::default();
        assert_eq!(breakdown.provider, "");
        assert_eq!(breakdown.input_tokens, 0);
        assert_eq!(breakdown.output_tokens, 0);
        assert_eq!(breakdown.cost_usd, 0.0);
    }

    /// `CycleCostSummary::developer_breakdown` must default to an empty map
    /// and the field must be absent-tolerant when deserializing legacy JSON.
    #[test]
    fn cycle_cost_summary_developer_breakdown_defaults_to_empty() {
        let summary = CycleCostSummary::default();
        assert!(
            summary.developer_breakdown.is_empty(),
            "developer_breakdown must default to an empty HashMap"
        );

        let json = r#"{
            "total_input_tokens": 0,
            "total_output_tokens": 0,
            "estimated_cost_usd": 0.0,
            "cost_by_worker": [],
            "cost_by_provider": []
        }"#;
        let deserialized: CycleCostSummary = serde_json::from_str(json).expect("deserialize");
        assert!(
            deserialized.developer_breakdown.is_empty(),
            "developer_breakdown must deserialize as empty when absent from JSON"
        );
    }

    #[test]
    fn cycle_cost_summary_default_is_zero() {
        let summary = CycleCostSummary::default();
        assert_eq!(summary.total_input_tokens, 0);
        assert_eq!(summary.total_output_tokens, 0);
        assert_eq!(summary.estimated_cost_usd, 0.0);
        assert!(summary.cost_by_worker.is_empty());
        assert!(summary.cost_by_provider.is_empty());
    }

    #[test]
    fn add_session_same_provider_merges_cost() {
        let mut summary = CycleCostSummary::default();
        let usage = make_usage("claude-sonnet-4-6", 1_000_000, 0);
        summary.add_session("worker-0", &usage);
        summary.add_session("worker-1", &usage);
        assert_eq!(summary.cost_by_provider.len(), 1);
        let (_, cost) = &summary.cost_by_provider[0];
        assert!((cost - 6.0).abs() < 1e-9);
    }

    #[test]
    fn add_session_different_providers_produces_two_entries() {
        let mut summary = CycleCostSummary::default();
        let usage_a = make_usage("claude-sonnet-4-6", 1_000_000, 0);
        let usage_b = make_usage("gpt-4o", 1_000_000, 0);
        summary.add_session("worker-0", &usage_a);
        summary.add_session("worker-1", &usage_b);
        assert_eq!(summary.cost_by_provider.len(), 2);
    }

    #[test]
    fn add_session_accumulates_token_counts() {
        let mut summary = CycleCostSummary::default();
        let usage_a = make_usage("claude-sonnet-4-6", 100, 50);
        let usage_b = make_usage("gpt-4o", 200, 75);
        summary.add_session("worker-0", &usage_a);
        summary.add_session("worker-1", &usage_b);
        assert_eq!(summary.total_input_tokens, 300);
        assert_eq!(summary.total_output_tokens, 125);
    }

    #[test]
    fn plan_audit_entry_clone_debug_serde() {
        let entry = PlanAuditEntry {
            timestamp: 1_700_000_000,
            tasks_removed: vec!["task-1".to_string()],
            task_edits_count: 2,
            constraints_added: vec!["must use async".to_string()],
        };
        let cloned = entry.clone();
        let _ = format!("{cloned:?}");
        let json = serde_json::to_string(&entry).expect("serialize");
        let deserialized: PlanAuditEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.timestamp, 1_700_000_000);
        assert_eq!(deserialized.tasks_removed, vec!["task-1"]);
        assert_eq!(deserialized.task_edits_count, 2);
    }

    #[test]
    fn cycle_report_deserializes_without_plan_audit_key() {
        let json = r#"{
            "cycle_id": "00000000-0000-0000-0000-000000000002",
            "started_at": "2024-01-01T00:00:00Z",
            "completed_at": "2024-01-01T00:01:00Z",
            "duration_ms": 60000,
            "status": "AllTasksCompleted",
            "tasks_completed": [],
            "tasks_failed": [],
            "files_modified": [],
            "verification_results": [],
            "agent_performance": [],
            "recommendations": []
        }"#;
        let report: CycleReport = serde_json::from_str(json).expect("deserialize");
        assert!(report.plan_audit.is_empty());
    }

    #[test]
    fn cycle_report_deserializes_without_cost_summary_key() {
        let json = r#"{
            "cycle_id": "00000000-0000-0000-0000-000000000001",
            "started_at": "2024-01-01T00:00:00Z",
            "completed_at": "2024-01-01T00:01:00Z",
            "duration_ms": 60000,
            "status": "AllTasksCompleted",
            "tasks_completed": [],
            "tasks_failed": [],
            "files_modified": [],
            "verification_results": [],
            "agent_performance": [],
            "recommendations": []
        }"#;
        let report: CycleReport = serde_json::from_str(json).expect("deserialize");
        assert_eq!(report.cost_summary.total_input_tokens, 0);
        assert_eq!(report.cost_summary.total_output_tokens, 0);
        assert_eq!(report.cost_summary.estimated_cost_usd, 0.0);
        assert!(report.cost_summary.cost_by_worker.is_empty());
        assert!(report.cost_summary.cost_by_provider.is_empty());
    }

    #[test]
    fn cycle_report_missing_plan_audit_deserializes_to_empty_vec() {
        let json = r#"{
            "cycle_id": "00000000-0000-0000-0000-000000000001",
            "started_at": "2024-01-01T00:00:00Z",
            "completed_at": "2024-01-01T00:01:00Z",
            "duration_ms": 60000,
            "status": "AllTasksCompleted",
            "tasks_completed": [],
            "tasks_failed": [],
            "files_modified": [],
            "verification_results": [],
            "agent_performance": [],
            "recommendations": []
        }"#;
        let report: CycleReport = serde_json::from_str(json).expect("deserialize");
        assert!(
            report.plan_audit.is_empty(),
            "plan_audit must default to empty vec when absent from JSON"
        );
    }

    #[test]
    fn test_team_cost_attribution_sums_to_total() {
        use mahalaxmi_core::types::developer::DeveloperId;

        let mut summary = CycleCostSummary::default();

        let dev_a = DeveloperId::from("alice");
        let dev_b = DeveloperId::from("bob");

        summary.add_developer_cost(dev_a, 1.5, 100_000, 50_000);
        summary.add_developer_cost(dev_b, 2.5, 200_000, 80_000);
        summary.estimated_cost_usd = 4.0;

        let dev_total: f64 = summary
            .developer_breakdown
            .values()
            .map(|b| b.cost_usd)
            .sum();

        assert!(
            (dev_total - summary.estimated_cost_usd).abs() < 0.001,
            "developer costs {dev_total} should sum to total {}",
            summary.estimated_cost_usd
        );
        assert_eq!(summary.developer_breakdown.len(), 2);
    }

    #[test]
    fn plan_audit_entry_is_clone_debug_serde() {
        let entry = PlanAuditEntry {
            timestamp: 1_700_000_000,
            tasks_removed: vec!["task-1".to_owned()],
            task_edits_count: 2,
            constraints_added: vec!["No network calls".to_owned()],
        };
        let cloned = entry.clone();
        assert_eq!(cloned.timestamp, entry.timestamp);
        assert_eq!(format!("{:?}", entry).contains("PlanAuditEntry"), true);
        let json = serde_json::to_string(&entry).expect("serialize");
        let recovered: PlanAuditEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(recovered.tasks_removed, entry.tasks_removed);
        assert_eq!(recovered.task_edits_count, entry.task_edits_count);
        assert_eq!(recovered.constraints_added, entry.constraints_added);
    }

    #[test]
    fn worker_cooccurrences_maps_completed_tasks() {
        use chrono::Utc;
        use mahalaxmi_core::types::CycleId;

        let report = CycleReport {
            cycle_id: CycleId::new(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            duration_ms: 1000,
            status: CycleOutcome::AllTasksCompleted,
            tasks_completed: vec![
                TaskSummary {
                    title: "Task A".to_owned(),
                    provider_id: "claude".to_owned(),
                    duration_ms: 100,
                    files_modified: vec!["src/a.rs".to_owned(), "src/b.rs".to_owned()],
                },
                TaskSummary {
                    title: "Task B".to_owned(),
                    provider_id: "claude".to_owned(),
                    duration_ms: 200,
                    files_modified: vec!["src/b.rs".to_owned()],
                },
            ],
            tasks_failed: vec![],
            files_modified: vec![],
            verification_results: vec![],
            agent_performance: vec![],
            recommendations: vec![],
            cost_summary: CycleCostSummary::default(),
            plan_audit: vec![],
        };

        let cooccurrences = report.worker_cooccurrences();
        assert_eq!(cooccurrences.len(), 2);
        assert_eq!(cooccurrences[0].task_title, "Task A");
        assert_eq!(cooccurrences[0].files_modified.len(), 2);
        assert_eq!(cooccurrences[1].task_title, "Task B");
        assert_eq!(cooccurrences[1].files_modified.len(), 1);
    }
}

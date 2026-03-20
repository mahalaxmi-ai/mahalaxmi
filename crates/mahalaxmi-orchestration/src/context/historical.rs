//! Historical co-occurrence context scoring for context routing.
//!
//! Scores files by their co-occurrence with semantically similar tasks in the
//! previous orchestration cycle. A file modified in all tasks similar to the
//! current one scores 1.0; files from weakly similar tasks score proportionally.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::consensus::normalizer::normalize_task_key;
use crate::consensus::similarity::token_jaccard as jaccard_token_similarity;
use crate::models::plan::WorkerTask;
use crate::models::report::CycleReport;

/// Minimum Jaccard similarity required before a prior task's files contribute
/// to the co-occurrence score.
const MIN_SIMILARITY: f64 = 0.1;

/// Score files by co-occurrence with similar tasks from the last cycle.
///
/// Algorithm:
/// 1. For each completed worker in the previous cycle, compute the Jaccard
///    similarity between the worker's task title and the current task title.
/// 2. Workers with similarity below [`MIN_SIMILARITY`] are skipped.
/// 3. Accumulate a weighted score for each file: `weight += similarity`.
/// 4. Normalise by the total weight so scores fall in `[0, 1]`.
///    A file modified by every similar worker scores 1.0.
///
/// Returns an empty map when `report` is `None` or when no prior task is
/// similar enough.
pub fn score_all(task: &WorkerTask, report: Option<&CycleReport>) -> HashMap<PathBuf, f64> {
    let Some(report) = report else {
        return HashMap::new();
    };

    let task_key = normalize_task_key(&task.title);
    let mut file_weights: HashMap<PathBuf, f64> = HashMap::new();
    let mut total_weight = 0.0_f64;

    for task_summary in &report.tasks_completed {
        let sim = jaccard_token_similarity(&normalize_task_key(&task_summary.title), &task_key);
        if sim < MIN_SIMILARITY {
            continue;
        }
        total_weight += sim;
        for file_str in &task_summary.files_modified {
            *file_weights.entry(PathBuf::from(file_str)).or_insert(0.0) += sim;
        }
    }

    if total_weight == 0.0 {
        return HashMap::new();
    }

    file_weights
        .into_iter()
        .map(|(p, w)| (p, (w / total_weight).min(1.0)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use mahalaxmi_core::types::{CycleId, TaskId, WorkerId};

    use crate::models::report::{CycleCostSummary, CycleOutcome, TaskSummary};

    fn make_empty_report() -> CycleReport {
        CycleReport {
            cycle_id: CycleId::new(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            duration_ms: 0,
            status: CycleOutcome::AllTasksCompleted,
            tasks_completed: vec![],
            tasks_failed: vec![],
            files_modified: vec![],
            verification_results: vec![],
            agent_performance: vec![],
            recommendations: vec![],
            cost_summary: CycleCostSummary::default(),
            plan_audit: vec![],
        }
    }

    fn make_task(title: &str) -> WorkerTask {
        WorkerTask::new(
            TaskId::new("task-0"),
            WorkerId::new(0),
            title,
            "Test description",
        )
    }

    #[test]
    fn returns_empty_when_report_is_none() {
        let task = make_task("OAuth login");
        let scores = score_all(&task, None);
        assert!(scores.is_empty());
    }

    #[test]
    fn returns_empty_when_no_completed_tasks() {
        let task = make_task("OAuth login");
        let report = make_empty_report();
        let scores = score_all(&task, Some(&report));
        assert!(scores.is_empty());
    }

    #[test]
    fn file_modified_by_all_similar_workers_scores_one() {
        let mut report = make_empty_report();
        let shared_file = "/src/auth.rs".to_string();
        report.tasks_completed = vec![
            TaskSummary {
                title: "OAuth login".to_string(),
                provider_id: "claude".to_string(),
                duration_ms: 1000,
                files_modified: vec![shared_file.clone()],
            },
            TaskSummary {
                title: "OAuth login".to_string(),
                provider_id: "claude".to_string(),
                duration_ms: 1000,
                files_modified: vec![shared_file.clone()],
            },
        ];

        let task = make_task("OAuth login");
        let scores = score_all(&task, Some(&report));

        let score = scores
            .get(&PathBuf::from(&shared_file))
            .copied()
            .expect("file should be scored");
        assert!((score - 1.0).abs() < 1e-9, "expected 1.0, got {score}");
    }

    #[test]
    fn dissimilar_task_does_not_contribute() {
        let mut report = make_empty_report();
        report.tasks_completed = vec![TaskSummary {
            title: "database migration".to_string(),
            provider_id: "claude".to_string(),
            duration_ms: 1000,
            files_modified: vec!["/src/db.rs".to_string()],
        }];

        let task = make_task("OAuth login");
        let scores = score_all(&task, Some(&report));
        // "database migration" shares no tokens with "oauth login" → empty.
        assert!(scores.is_empty());
    }

    #[test]
    fn below_min_similarity_threshold_excluded() {
        let mut report = make_empty_report();
        // Pick a title that shares exactly one token (similarity just below 0.1
        // when there are many unique tokens on each side).
        report.tasks_completed = vec![TaskSummary {
            title: "login widget xyz abc def ghi jkl mno".to_string(),
            provider_id: "claude".to_string(),
            duration_ms: 500,
            files_modified: vec!["/src/widget.rs".to_string()],
        }];

        // Current task: completely different tokens.
        let task = make_task("payment processing module pqr stu vwx yzz qqq rrr sss");
        let scores = score_all(&task, Some(&report));
        assert!(
            scores.is_empty(),
            "dissimilar task should produce empty scores"
        );
    }
}

//! Graph-proximity context scoring for context routing.
//!
//! Scores indexed files by their import-graph distance to the task's
//! affected files. A file directly imported by an affected file scores 0.5
//! (distance 1), one two hops away scores ~0.333, and so on. Files beyond
//! `MAX_DEPTH` hops are omitted.

use mahalaxmi_indexing::CodebaseIndex;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::models::plan::WorkerTask;

/// Maximum BFS depth when traversing the import graph.
const MAX_DEPTH: usize = 4;

/// Score all indexed files by import-graph proximity to the task's affected files.
///
/// Score formula: `1.0 / (1.0 + distance)`, giving:
/// - distance 0 → 1.0 (the affected file itself)
/// - distance 1 → 0.5
/// - distance 2 → ~0.333
/// - distance 3 → 0.25
/// - distance 4 → 0.2
///
/// Returns an empty map when `task.affected_files` is empty.
/// Files not reachable within [`MAX_DEPTH`] hops are omitted.
pub fn score_all(task: &WorkerTask, index: &CodebaseIndex) -> HashMap<PathBuf, f64> {
    let sources: Vec<PathBuf> = task.affected_files.iter().map(PathBuf::from).collect();
    if sources.is_empty() {
        return HashMap::new();
    }

    let graph = index.graph();
    let mut scores = HashMap::new();

    for path in graph.files() {
        if let Some(dist) = graph.bfs_distance(path, &sources, MAX_DEPTH) {
            let score = 1.0 / (1.0 + dist as f64);
            if score > 0.0 {
                scores.insert(path.clone(), score);
            }
        }
    }

    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahalaxmi_core::config::IndexingConfig;
    use mahalaxmi_core::i18n::locale::SupportedLocale;
    use mahalaxmi_core::i18n::I18nService;
    use mahalaxmi_core::types::{TaskId, WorkerId};
    use mahalaxmi_indexing::{CodebaseIndex, FileDependency, FileDependencyGraph};

    fn test_i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    fn make_task_with_files(affected: Vec<String>) -> WorkerTask {
        let mut task = WorkerTask::new(
            TaskId::new("task-0"),
            WorkerId::new(0),
            "Test task",
            "Test description",
        );
        task.affected_files = affected;
        task
    }

    #[test]
    fn empty_affected_files_returns_empty_map() {
        let i18n = test_i18n();
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        std::fs::write(root.join("a.rs"), "fn a() {}").expect("write");

        let config = IndexingConfig::default();
        let index = CodebaseIndex::build(&root, &config, &i18n).expect("build");

        let task = make_task_with_files(vec![]);
        let scores = score_all(&task, &index);
        assert!(scores.is_empty());
        std::mem::forget(dir);
    }

    #[test]
    fn distance_zero_scores_one() {
        let i18n = test_i18n();
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        let a_path = root.join("a.rs");
        std::fs::write(&a_path, "fn a() {}").expect("write");

        let config = IndexingConfig::default();
        let index = CodebaseIndex::build(&root, &config, &i18n).expect("build");

        let task = make_task_with_files(vec![a_path.to_string_lossy().to_string()]);
        let scores = score_all(&task, &index);

        let score = scores.get(&a_path).copied().unwrap_or(0.0);
        assert!(
            (score - 1.0).abs() < 1e-9,
            "source file should score 1.0, got {score}"
        );
        std::mem::forget(dir);
    }

    #[test]
    fn bfs_distance_formula_distance_one_is_half() {
        let mut graph = FileDependencyGraph::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        graph.add_dependency(FileDependency::new(a.clone(), b.clone(), "b"));

        let dist = graph.bfs_distance(&b, &[a.clone()], MAX_DEPTH);
        assert_eq!(dist, Some(1));
        let score = 1.0 / (1.0 + 1_f64);
        assert!((score - 0.5).abs() < 1e-3);
    }

    #[test]
    fn bfs_distance_formula_distance_two_is_one_third() {
        let mut graph = FileDependencyGraph::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        let c = PathBuf::from("/c.rs");
        graph.add_dependency(FileDependency::new(a.clone(), b.clone(), "b"));
        graph.add_dependency(FileDependency::new(b.clone(), c.clone(), "c"));

        let dist = graph.bfs_distance(&c, &[a.clone()], MAX_DEPTH);
        assert_eq!(dist, Some(2));
        let score = 1.0 / (1.0 + 2_f64);
        assert!((score - 0.333_333_333).abs() < 1e-6);
    }

    #[test]
    fn unreachable_files_absent_from_result() {
        let mut graph = FileDependencyGraph::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        let c = PathBuf::from("/c.rs"); // no edge to c
        graph.add_dependency(FileDependency::new(a.clone(), b.clone(), "b"));
        graph.add_file(c.clone());

        let dist = graph.bfs_distance(&c, &[a.clone()], MAX_DEPTH);
        assert_eq!(dist, None, "unreachable file should return None");
    }
}

//! Lexical context scoring for context routing.
//!
//! Scores indexed files by the Jaccard token similarity between the task's
//! title + description and the symbol names and file stems in each file.
//! Higher similarity means the file is more likely to be relevant to the task.

use mahalaxmi_indexing::CodebaseIndex;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::consensus::normalizer::normalize_task_key;
use crate::consensus::similarity::token_jaccard as jaccard_token_similarity;
use crate::models::plan::WorkerTask;

/// Score all indexed files by lexical overlap with the task title and description.
///
/// For each file in the codebase index the score is the maximum Jaccard token
/// similarity across:
/// - All symbol names defined in that file.
/// - The file's own stem (e.g., `auth_handler` for `src/auth_handler.rs`).
///
/// Returns a map from file path to score in `[0, 1]`.
/// Files with no overlap are omitted from the result.
pub fn score_all(task: &WorkerTask, index: &CodebaseIndex) -> HashMap<PathBuf, f64> {
    let task_key = normalize_task_key(&format!("{} {}", task.title, task.description));
    let mut scores: HashMap<PathBuf, f64> = HashMap::new();

    for file_path in index.graph().files() {
        // Score every symbol defined in this file.
        for symbol in index.symbols_in_file(file_path) {
            let sim = jaccard_token_similarity(&normalize_task_key(&symbol.name), &task_key);
            let entry = scores.entry(file_path.clone()).or_insert(0.0);
            if sim > *entry {
                *entry = sim;
            }
        }

        // Also score the file stem itself.
        let stem = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let sim = jaccard_token_similarity(&normalize_task_key(stem), &task_key);
        let entry = scores.entry(file_path.clone()).or_insert(0.0);
        if sim > *entry {
            *entry = sim;
        }
    }

    // Remove zero-score entries to keep the map compact.
    scores.retain(|_, v| *v > 0.0);
    scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahalaxmi_core::config::IndexingConfig;
    use mahalaxmi_core::i18n::locale::SupportedLocale;
    use mahalaxmi_core::i18n::I18nService;
    use mahalaxmi_core::types::{TaskId, WorkerId};
    use mahalaxmi_indexing::CodebaseIndex;

    fn test_i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    fn make_task(title: &str, description: &str) -> WorkerTask {
        WorkerTask::new(TaskId::new("task-0"), WorkerId::new(0), title, description)
    }

    fn build_two_file_index(
        file_a_name: &str,
        file_a_content: &str,
        file_b_name: &str,
        file_b_content: &str,
    ) -> (CodebaseIndex, PathBuf, PathBuf) {
        let i18n = test_i18n();
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();

        let path_a = root.join(file_a_name);
        let path_b = root.join(file_b_name);
        std::fs::write(&path_a, file_a_content).expect("write a");
        std::fs::write(&path_b, file_b_content).expect("write b");

        let config = IndexingConfig::default();
        let index = CodebaseIndex::build(&root, &config, &i18n).expect("build index");

        // Keep dir alive via leak; tests are short-lived.
        std::mem::forget(dir);
        (index, path_a, path_b)
    }

    #[test]
    fn empty_task_returns_near_zero_scores() {
        let i18n = test_i18n();
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        std::fs::write(root.join("any_file.rs"), "pub fn any_file() {}").expect("write");

        let config = IndexingConfig::default();
        let index = CodebaseIndex::build(&root, &config, &i18n).expect("build index");

        let task = make_task("", "");
        let scores = score_all(&task, &index);
        // All Jaccard similarities against an empty token set are 0 → map should be empty.
        assert!(
            scores.is_empty(),
            "expected empty scores for empty task, got {scores:?}"
        );
        std::mem::forget(dir);
    }

    #[test]
    fn matching_file_scores_higher_than_unrelated() {
        let (index, auth_path, db_path) = build_two_file_index(
            "oauth_handler.rs",
            "pub fn oauth_handler() {}",
            "database_pool.rs",
            "pub fn database_pool() {}",
        );

        let task = make_task("oauth login", "Implement OAuth 2.0 login flow");
        let scores = score_all(&task, &index);

        let oauth_score = scores.get(&auth_path).copied().unwrap_or(0.0);
        let db_score = scores.get(&db_path).copied().unwrap_or(0.0);

        assert!(
            oauth_score > db_score,
            "oauth_handler ({oauth_score}) should score higher than database_pool ({db_score})"
        );
    }

    #[test]
    fn file_stem_contributes_to_score() {
        let i18n = test_i18n();
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();

        // File whose stem exactly matches the task keyword.
        let path = root.join("billing_processor.rs");
        // Content intentionally does NOT mention "billing" to isolate stem scoring.
        std::fs::write(&path, "pub fn run() {}").expect("write");

        let config = IndexingConfig::default();
        let index = CodebaseIndex::build(&root, &config, &i18n).expect("build index");

        let task = make_task("billing processor", "Handle billing logic");
        let scores = score_all(&task, &index);

        let score = scores.get(&path).copied().unwrap_or(0.0);
        assert!(
            score > 0.0,
            "billing_processor.rs stem should score > 0 for 'billing processor' task"
        );
        std::mem::forget(dir);
    }
}

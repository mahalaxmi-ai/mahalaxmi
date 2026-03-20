use std::path::PathBuf;

use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_orchestration::context::{
    extract_file_paths, score_files, FileRelevance, RelevanceReason,
};
use mahalaxmi_orchestration::models::plan::WorkerTask;

fn make_task(description: &str, affected: &[&str]) -> WorkerTask {
    let mut task = WorkerTask::new(
        TaskId::new("task-1"),
        WorkerId::new(1),
        "Test Task",
        description,
    );
    for file in affected {
        task = task.with_affected_file(*file);
    }
    task
}

#[test]
fn extract_file_paths_unix() {
    let text = "Modify src/lib.rs and update src/main.rs";
    let paths = extract_file_paths(text);
    assert_eq!(paths.len(), 2);
    assert!(paths.contains(&PathBuf::from("src/lib.rs")));
    assert!(paths.contains(&PathBuf::from("src/main.rs")));
}

#[test]
fn extract_file_paths_windows() {
    let text = r"Check src\config\mod.rs for issues";
    let paths = extract_file_paths(text);
    assert_eq!(paths.len(), 1);
    assert!(paths.contains(&PathBuf::from(r"src\config\mod.rs")));
}

#[test]
fn extract_file_paths_deduplicates() {
    let text = "Fix src/lib.rs and test src/lib.rs";
    let paths = extract_file_paths(text);
    assert_eq!(paths.len(), 1);
}

#[test]
fn extract_file_paths_no_bare_extensions() {
    let text = "version 1.0 and hello world";
    let paths = extract_file_paths(text);
    // "1.0" has no separator so should not match
    assert!(paths.is_empty());
}

#[test]
fn score_directly_mentioned_score_1_0() {
    let task = make_task("Fix src/lib.rs", &[]);
    let scored = score_files(&task, None);
    assert!(!scored.is_empty());
    let first = &scored[0];
    assert_eq!(first.file_path, PathBuf::from("src/lib.rs"));
    assert!((first.score - 1.0).abs() < f64::EPSILON);
    assert!(first.is_directly_mentioned());
}

#[test]
fn score_affected_file_score_0_8() {
    let task = make_task("Do something", &["src/models/user.rs"]);
    let scored = score_files(&task, None);
    let affected = scored
        .iter()
        .find(|f| f.file_path == PathBuf::from("src/models/user.rs"));
    assert!(affected.is_some());
    let affected = affected.unwrap();
    assert!((affected.score - 0.8).abs() < f64::EPSILON);
}

#[test]
fn score_both_mentioned_and_affected_picks_max() {
    let task = make_task("Fix src/lib.rs thoroughly", &["src/lib.rs"]);
    let scored = score_files(&task, None);
    let entry = scored
        .iter()
        .find(|f| f.file_path == PathBuf::from("src/lib.rs"));
    assert!(entry.is_some());
    let entry = entry.unwrap();
    // Should have both reasons, score should be max (1.0)
    assert_eq!(entry.reasons.len(), 2);
    assert!((entry.score - 1.0).abs() < f64::EPSILON);
}

#[test]
fn score_files_sorted_by_score_descending() {
    let task = make_task("Fix src/lib.rs", &["src/models/user.rs"]);
    let scored = score_files(&task, None);
    for i in 1..scored.len() {
        assert!(scored[i - 1].score >= scored[i].score);
    }
}

#[test]
fn score_empty_task_returns_empty() {
    let task = make_task("No files mentioned", &[]);
    let scored = score_files(&task, None);
    assert!(scored.is_empty());
}

#[test]
fn score_files_infers_test_path() {
    let task = make_task("", &["src/queue.rs"]);
    let scored = score_files(&task, None);
    // Should have the affected file plus an inferred test file
    assert!(scored.len() >= 1);
    let has_test = scored.iter().any(|f| {
        f.reasons
            .iter()
            .any(|r| matches!(r, RelevanceReason::TestFile { .. }))
    });
    assert!(has_test);
}

#[test]
fn relevance_reason_display() {
    let reason = RelevanceReason::DirectlyMentioned {
        in_field: "description".to_owned(),
    };
    let display = format!("{}", reason);
    assert!(display.contains("directly mentioned"));
    assert!(display.contains("description"));
}

#[test]
fn file_relevance_primary_reason() {
    let mut fr = FileRelevance::new("src/lib.rs");
    fr.add_reason(RelevanceReason::TestFile {
        tests_for: PathBuf::from("src/main.rs"),
    });
    fr.add_reason(RelevanceReason::DirectlyMentioned {
        in_field: "description".to_owned(),
    });
    let primary = fr.primary_reason().unwrap();
    assert!((primary.score() - 1.0).abs() < f64::EPSILON);
}

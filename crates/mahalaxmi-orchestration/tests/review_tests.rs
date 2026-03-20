//! Tests for the cross-provider review chain (P6).

use mahalaxmi_core::config::ContextFormat;
use mahalaxmi_orchestration::models::review::{ReviewIssue, ReviewResult, ReviewSeverity};
use mahalaxmi_orchestration::prompt::review_builder::{ReviewPromptBuilder, ReviewPromptConfig};

// =========================================================================
// ReviewResult parsing
// =========================================================================

#[test]
fn parse_review_result_from_clean_json() {
    let output = r#"{"approved": true, "issues": [], "suggestions": ["Add more tests"]}"#;
    let result =
        ReviewResult::parse_from_output(output, "Task Alpha", "reviewer-provider", 5000).unwrap();

    assert!(result.approved);
    assert!(result.issues.is_empty());
    assert_eq!(result.suggestions.len(), 1);
    assert_eq!(result.suggestions[0], "Add more tests");
    assert_eq!(result.task_title, "Task Alpha");
    assert_eq!(result.reviewer_provider_id, "reviewer-provider");
    assert_eq!(result.duration_ms, 5000);
}

#[test]
fn parse_review_result_from_markdown_fenced_json() {
    let output = r#"Here is my review:

```json
{
  "approved": false,
  "issues": [
    {
      "severity": "critical",
      "file_path": "src/auth.rs",
      "description": "SQL injection vulnerability in login query"
    }
  ],
  "suggestions": []
}
```

I hope this helps."#;

    let result =
        ReviewResult::parse_from_output(output, "Auth Task", "openai-foundry", 8000).unwrap();

    assert!(!result.approved);
    assert_eq!(result.issues.len(), 1);
    assert_eq!(result.issues[0].severity, ReviewSeverity::Critical);
    assert_eq!(result.issues[0].file_path, "src/auth.rs");
    assert!(result.issues[0].description.contains("SQL injection"));
}

#[test]
fn parse_review_result_with_multiple_issues() {
    let output = r#"{
  "approved": false,
  "issues": [
    {"severity": "critical", "file_path": "src/db.rs", "description": "Missing connection pool cleanup"},
    {"severity": "warning", "file_path": "src/api.rs", "description": "No timeout on HTTP requests"},
    {"severity": "suggestion", "file_path": "src/lib.rs", "description": "Consider using structured logging"}
  ],
  "suggestions": ["Add benchmarks"]
}"#;

    let result = ReviewResult::parse_from_output(output, "DB Task", "claude-code", 12000).unwrap();

    assert!(!result.approved);
    assert_eq!(result.issues.len(), 3);
    assert_eq!(result.critical_count(), 1);
    assert_eq!(result.warning_count(), 1);
    assert_eq!(result.suggestions.len(), 1);
}

#[test]
fn parse_review_result_returns_none_for_garbage() {
    let output = "This is not JSON at all, just random text without any structure.";
    let result = ReviewResult::parse_from_output(output, "Task", "provider", 1000);
    assert!(result.is_none());
}

#[test]
fn parse_review_result_unknown_severity_defaults_to_suggestion() {
    let output = r#"{
  "approved": true,
  "issues": [
    {"severity": "info", "file_path": "src/main.rs", "description": "Minor note"}
  ],
  "suggestions": []
}"#;

    let result = ReviewResult::parse_from_output(output, "Task", "provider", 1000).unwrap();
    assert_eq!(result.issues[0].severity, ReviewSeverity::Suggestion);
}

// =========================================================================
// ReviewResult methods
// =========================================================================

#[test]
fn critical_count_and_warning_count() {
    let result = ReviewResult {
        task_title: "Test".into(),
        reviewer_provider_id: "p".into(),
        approved: false,
        issues: vec![
            ReviewIssue {
                severity: ReviewSeverity::Critical,
                file_path: "a.rs".into(),
                description: "bug".into(),
            },
            ReviewIssue {
                severity: ReviewSeverity::Critical,
                file_path: "b.rs".into(),
                description: "bug2".into(),
            },
            ReviewIssue {
                severity: ReviewSeverity::Warning,
                file_path: "c.rs".into(),
                description: "warn".into(),
            },
            ReviewIssue {
                severity: ReviewSeverity::Suggestion,
                file_path: "d.rs".into(),
                description: "sug".into(),
            },
        ],
        suggestions: vec![],
        duration_ms: 1000,
    };

    assert_eq!(result.critical_count(), 2);
    assert_eq!(result.warning_count(), 1);
}

#[test]
fn to_retry_context_contains_issues() {
    let result = ReviewResult {
        task_title: "Auth".into(),
        reviewer_provider_id: "openai".into(),
        approved: false,
        issues: vec![
            ReviewIssue {
                severity: ReviewSeverity::Critical,
                file_path: "src/auth.rs".into(),
                description: "Missing input validation".into(),
            },
            ReviewIssue {
                severity: ReviewSeverity::Warning,
                file_path: "".into(),
                description: "Consider adding rate limiting".into(),
            },
        ],
        suggestions: vec!["Add integration tests".into()],
        duration_ms: 5000,
    };

    let context = result.to_retry_context();
    assert!(context.contains("Cross-Provider Review Feedback"));
    assert!(context.contains("REJECTED"));
    assert!(context.contains("[CRITICAL]"));
    assert!(context.contains("(src/auth.rs)"));
    assert!(context.contains("Missing input validation"));
    assert!(context.contains("[WARNING]"));
    assert!(context.contains("rate limiting"));
    assert!(context.contains("Add integration tests"));
}

#[test]
fn to_retry_context_approved_shows_suggestions() {
    let result = ReviewResult {
        task_title: "Task".into(),
        reviewer_provider_id: "p".into(),
        approved: true,
        issues: vec![],
        suggestions: vec!["Consider caching".into()],
        duration_ms: 1000,
    };

    let context = result.to_retry_context();
    assert!(context.contains("APPROVED"));
    assert!(context.contains("Consider caching"));
}

// =========================================================================
// ReviewResult serialization
// =========================================================================

#[test]
fn review_result_serialization_roundtrip() {
    let result = ReviewResult {
        task_title: "Build endpoint".into(),
        reviewer_provider_id: "claude-code".into(),
        approved: false,
        issues: vec![ReviewIssue {
            severity: ReviewSeverity::Critical,
            file_path: "src/api.rs".into(),
            description: "Unhandled error".into(),
        }],
        suggestions: vec!["Add logging".into()],
        duration_ms: 7000,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: ReviewResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.approved, false);
    assert_eq!(deserialized.issues.len(), 1);
    assert_eq!(deserialized.issues[0].severity, ReviewSeverity::Critical);
    assert_eq!(deserialized.suggestions.len(), 1);
}

// =========================================================================
// ReviewPromptBuilder
// =========================================================================

#[test]
fn review_prompt_contains_constraint_format() {
    let config = ReviewPromptConfig {
        task_title: "Build auth module".into(),
        task_description: "Implement JWT authentication".into(),
        worker_output: "Here is the code I wrote...".into(),
        affected_files: vec!["src/auth.rs".into(), "src/middleware.rs".into()],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
    };

    let prompt = ReviewPromptBuilder::build(&config);
    assert!(prompt.contains("HARD CONSTRAINTS"));
    assert!(prompt.contains("VIOLATION"));
    assert!(prompt.contains("approved=false ONLY for Critical issues"));
}

#[test]
fn review_prompt_includes_task_context() {
    let config = ReviewPromptConfig {
        task_title: "Fix database queries".into(),
        task_description: "Optimize slow queries in the user table".into(),
        worker_output: "Updated queries...".into(),
        affected_files: vec!["src/db.rs".into()],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
    };

    let prompt = ReviewPromptBuilder::build(&config);
    assert!(prompt.contains("Fix database queries"));
    assert!(prompt.contains("Optimize slow queries"));
    assert!(prompt.contains("src/db.rs"));
}

#[test]
fn review_prompt_includes_worker_output() {
    let config = ReviewPromptConfig {
        task_title: "Task".into(),
        task_description: "Do something".into(),
        worker_output: "UNIQUE_WORKER_OUTPUT_MARKER_12345".into(),
        affected_files: vec![],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
    };

    let prompt = ReviewPromptBuilder::build(&config);
    assert!(prompt.contains("UNIQUE_WORKER_OUTPUT_MARKER_12345"));
}

#[test]
fn review_prompt_truncates_long_output() {
    let config = ReviewPromptConfig {
        task_title: "Task".into(),
        task_description: "Do something".into(),
        worker_output: "x".repeat(20_000),
        affected_files: vec![],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
    };

    let prompt = ReviewPromptBuilder::build(&config);
    assert!(prompt.contains("Output truncated"));
    // Should not contain the full 20k output
    assert!(prompt.len() < 20_000);
}

#[test]
fn review_prompt_uses_xml_for_claude() {
    let config = ReviewPromptConfig {
        task_title: "Task".into(),
        task_description: "Desc".into(),
        worker_output: "Output".into(),
        affected_files: vec![],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
    };

    let prompt = ReviewPromptBuilder::build(&config);
    assert!(prompt.contains("<review_constraints>"));
    assert!(prompt.contains("<task_under_review>"));
    assert!(prompt.contains("<worker_output>"));
    assert!(prompt.contains("<output_format>"));
}

#[test]
fn review_prompt_uses_markdown_for_non_claude() {
    let config = ReviewPromptConfig {
        task_title: "Task".into(),
        task_description: "Desc".into(),
        worker_output: "Output".into(),
        affected_files: vec![],
        provider_id: "openai-foundry".into(),
        format: ContextFormat::Auto,
    };

    let prompt = ReviewPromptBuilder::build(&config);
    assert!(prompt.contains("## Review Constraints"));
    assert!(prompt.contains("## Task Under Review"));
    assert!(prompt.contains("## Worker Output"));
    assert!(prompt.contains("## Output Format"));
}

#[test]
fn review_prompt_includes_output_json_schema() {
    let config = ReviewPromptConfig {
        task_title: "Task".into(),
        task_description: "Desc".into(),
        worker_output: "Output".into(),
        affected_files: vec![],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
    };

    let prompt = ReviewPromptBuilder::build(&config);
    assert!(prompt.contains("\"approved\""));
    assert!(prompt.contains("\"issues\""));
    assert!(prompt.contains("\"severity\""));
    assert!(prompt.contains("\"suggestions\""));
}

// =========================================================================
// ReviewSeverity display
// =========================================================================

#[test]
fn review_severity_display() {
    assert_eq!(ReviewSeverity::Critical.to_string(), "Critical");
    assert_eq!(ReviewSeverity::Warning.to_string(), "Warning");
    assert_eq!(ReviewSeverity::Suggestion.to_string(), "Suggestion");
}

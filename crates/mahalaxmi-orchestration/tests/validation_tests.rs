//! Tests for post-cycle requirement validation (Phase 4 — P2–P5).

use mahalaxmi_core::config::ContextFormat;
use mahalaxmi_orchestration::models::validation::{
    AcceptanceCriterionResult, FulfillmentStatus, GapSeverity, RequirementGap, ValidationVerdict,
};
use mahalaxmi_orchestration::prompt::builder::{ManagerPromptBuilder, ManagerPromptConfig};
use mahalaxmi_orchestration::prompt::validator_builder::{
    CommandResult, TaskValidationSummary, ValidatorPromptBuilder, ValidatorPromptConfig,
};

// =========================================================================
// ValidationVerdict parsing
// =========================================================================

#[test]
fn parse_verdict_from_clean_json() {
    let output =
        r#"{"status": "fulfilled", "summary": "All requirements met", "confidence": 0.95}"#;
    let result =
        ValidationVerdict::parse_from_output(output, "cycle-1", "claude-code", 5000).unwrap();

    assert_eq!(result.status, FulfillmentStatus::Fulfilled);
    assert_eq!(result.summary, "All requirements met");
    assert!((result.confidence - 0.95).abs() < f64::EPSILON);
    assert!(result.gaps.is_empty());
    assert!(result.criteria_results.is_empty());
    assert_eq!(result.cycle_id, "cycle-1");
    assert_eq!(result.validator_provider_id, "claude-code");
    assert_eq!(result.duration_ms, 5000);
}

#[test]
fn parse_verdict_from_markdown_fenced_json() {
    let output = r#"Here is my validation:

```json
{
  "status": "partially_fulfilled",
  "summary": "Most requirements met but OAuth setup is missing",
  "confidence": 0.7,
  "gaps": [
    {
      "requirement": "OAuth provider setup",
      "severity": "critical",
      "expected": "OAuth provider configured with Google/GitHub",
      "actual": "No OAuth provider found",
      "suggested_action": "Add a task for OAuth provider setup with Google/GitHub",
      "affected_files": ["src/auth/oauth.rs"]
    }
  ],
  "criteria_results": [
    {"criterion": "Login endpoint works", "task_title": "Auth task", "passed": true, "evidence": "Tests pass"},
    {"criterion": "OAuth flow complete", "task_title": "OAuth task", "passed": false, "evidence": "No OAuth provider configured"}
  ]
}
```

I hope this helps."#;

    let result =
        ValidationVerdict::parse_from_output(output, "cycle-1", "openai-foundry", 8000).unwrap();

    assert_eq!(result.status, FulfillmentStatus::PartiallyFulfilled);
    assert_eq!(result.gaps.len(), 1);
    assert_eq!(result.gaps[0].severity, GapSeverity::Critical);
    assert_eq!(result.gaps[0].requirement, "OAuth provider setup");
    assert_eq!(result.gaps[0].affected_files, vec!["src/auth/oauth.rs"]);
    assert_eq!(result.criteria_results.len(), 2);
    assert!(result.criteria_results[0].passed);
    assert!(!result.criteria_results[1].passed);
}

#[test]
fn parse_verdict_with_gaps_and_criteria() {
    let output = r#"{
  "status": "not_fulfilled",
  "summary": "Major gaps found",
  "confidence": 0.85,
  "gaps": [
    {"requirement": "Database migration", "severity": "critical", "expected": "Migration files created", "actual": "No migration files", "suggested_action": "Add migration task", "affected_files": ["migrations/"]},
    {"requirement": "Error handling", "severity": "major", "expected": "Standardized API errors", "actual": "Inconsistent error responses", "suggested_action": "Implement error middleware", "affected_files": ["src/api.rs", "src/errors.rs"]},
    {"requirement": "Logging", "severity": "minor", "expected": "Structured logging", "actual": "No logging", "suggested_action": "Add tracing", "affected_files": []}
  ],
  "criteria_results": [
    {"criterion": "cargo build succeeds", "task_title": "", "passed": true, "evidence": "Build passes"},
    {"criterion": "all tests pass", "task_title": "", "passed": false, "evidence": "3 tests failing"},
    {"criterion": "API returns valid JSON", "task_title": "", "passed": true, "evidence": "Manual test confirms"}
  ]
}"#;

    let result =
        ValidationVerdict::parse_from_output(output, "cycle-1", "claude-code", 12000).unwrap();

    assert_eq!(result.status, FulfillmentStatus::NotFulfilled);
    assert_eq!(result.gaps.len(), 3);
    assert_eq!(result.criteria_results.len(), 3);
    assert_eq!(result.criteria_passed_count(), 2);
    assert_eq!(result.criteria_total(), 3);

    let (critical, major, minor) = result.gap_count_by_severity();
    assert_eq!(critical, 1);
    assert_eq!(major, 1);
    assert_eq!(minor, 1);
}

#[test]
fn parse_verdict_returns_none_for_garbage() {
    let output = "This is not JSON at all, just random text without any structure.";
    let result = ValidationVerdict::parse_from_output(output, "cycle-1", "provider", 1000);
    assert!(result.is_none());
}

#[test]
fn parse_verdict_returns_none_for_unknown_status() {
    let output = r#"{"status": "banana", "summary": "Invalid status"}"#;
    let result = ValidationVerdict::parse_from_output(output, "cycle-1", "provider", 1000);
    assert!(result.is_none());
}

#[test]
fn parse_verdict_unknown_severity_defaults_to_minor() {
    let output = r#"{
  "status": "partially_fulfilled",
  "gaps": [
    {"requirement": "Test", "severity": "info", "expected": "", "actual": "", "suggested_action": "", "affected_files": []}
  ]
}"#;

    let result = ValidationVerdict::parse_from_output(output, "cycle-1", "provider", 1000).unwrap();
    assert_eq!(result.gaps[0].severity, GapSeverity::Minor);
}

#[test]
fn parse_verdict_alternative_status_values() {
    // "partial" variant
    let output = r#"{"status": "partial"}"#;
    let result = ValidationVerdict::parse_from_output(output, "c", "p", 100).unwrap();
    assert_eq!(result.status, FulfillmentStatus::PartiallyFulfilled);

    // "failed" variant
    let output = r#"{"status": "failed"}"#;
    let result = ValidationVerdict::parse_from_output(output, "c", "p", 100).unwrap();
    assert_eq!(result.status, FulfillmentStatus::NotFulfilled);

    // case insensitive
    let output = r#"{"status": "FULFILLED"}"#;
    let result = ValidationVerdict::parse_from_output(output, "c", "p", 100).unwrap();
    assert_eq!(result.status, FulfillmentStatus::Fulfilled);
}

// =========================================================================
// ValidationVerdict methods
// =========================================================================

/// Helper to build a minimal verdict for method tests.
fn make_verdict(
    status: FulfillmentStatus,
    gaps: Vec<RequirementGap>,
    criteria: Vec<AcceptanceCriterionResult>,
) -> ValidationVerdict {
    ValidationVerdict {
        cycle_id: "test-cycle".into(),
        validator_provider_id: "test-provider".into(),
        status,
        requirement_assessments: vec![],
        gaps,
        criteria_results: criteria,
        command_results: vec![],
        summary: String::new(),
        confidence: 0.5,
        duration_ms: 1000,
        security_scan: None,
    }
}

/// Helper to build a RequirementGap.
fn make_gap(
    requirement: &str,
    severity: GapSeverity,
    expected: &str,
    actual: &str,
    action: &str,
    files: Vec<&str>,
) -> RequirementGap {
    RequirementGap {
        requirement: requirement.into(),
        severity,
        expected: expected.into(),
        actual: actual.into(),
        suggested_action: action.into(),
        affected_files: files.into_iter().map(|s| s.into()).collect(),
    }
}

/// Helper to build an AcceptanceCriterionResult.
fn make_criterion(
    criterion: &str,
    task_title: &str,
    passed: bool,
    evidence: &str,
) -> AcceptanceCriterionResult {
    AcceptanceCriterionResult {
        criterion: criterion.into(),
        task_title: task_title.into(),
        passed,
        evidence: evidence.into(),
    }
}

#[test]
fn gap_count_by_severity() {
    let verdict = make_verdict(
        FulfillmentStatus::NotFulfilled,
        vec![
            make_gap("A", GapSeverity::Critical, "", "", "", vec![]),
            make_gap("B", GapSeverity::Critical, "", "", "", vec![]),
            make_gap("C", GapSeverity::Major, "", "", "", vec![]),
            make_gap("D", GapSeverity::Minor, "", "", "", vec![]),
        ],
        vec![],
    );

    let (critical, major, minor) = verdict.gap_count_by_severity();
    assert_eq!(critical, 2);
    assert_eq!(major, 1);
    assert_eq!(minor, 1);
}

#[test]
fn criteria_passed_count() {
    let verdict = make_verdict(
        FulfillmentStatus::PartiallyFulfilled,
        vec![],
        vec![
            make_criterion("Build succeeds", "", true, "OK"),
            make_criterion("Tests pass", "", false, "2 failing"),
            make_criterion("Lint clean", "", true, "No warnings"),
        ],
    );

    assert_eq!(verdict.criteria_passed_count(), 2);
    assert_eq!(verdict.criteria_total(), 3);
}

// =========================================================================
// to_prompt_summary
// =========================================================================

#[test]
fn to_prompt_summary_fulfilled() {
    let mut verdict = make_verdict(
        FulfillmentStatus::Fulfilled,
        vec![],
        vec![make_criterion("Build succeeds", "", true, "cargo build OK")],
    );
    verdict.summary = "All requirements satisfied.".into();
    verdict.confidence = 0.95;

    let summary = verdict.to_prompt_summary();
    assert!(summary.contains("FULFILLED all requirements"));
    assert!(summary.contains("All requirements satisfied."));
    // No gaps section
    assert!(!summary.contains("Requirement gaps"));
}

#[test]
fn to_prompt_summary_with_gaps() {
    let mut verdict = make_verdict(
        FulfillmentStatus::PartiallyFulfilled,
        vec![make_gap(
            "OAuth setup",
            GapSeverity::Critical,
            "OAuth provider configured",
            "No OAuth provider found",
            "Add OAuth configuration task",
            vec!["src/auth.rs"],
        )],
        vec![
            make_criterion("Login works", "", true, "Tests pass"),
            make_criterion("OAuth flow works", "", false, "Not implemented"),
        ],
    );
    verdict.summary = "Partial completion.".into();
    verdict.confidence = 0.8;

    let summary = verdict.to_prompt_summary();
    assert!(summary.contains("PARTIALLY FULFILLED"));
    assert!(summary.contains("Requirement gaps to address:"));
    assert!(summary.contains("[CRITICAL] OAuth setup"));
    assert!(summary.contains("Expected: OAuth provider configured"));
    assert!(summary.contains("Actual: No OAuth provider found"));
    assert!(summary.contains("Action: Add OAuth configuration task"));
    assert!(summary.contains("Files: src/auth.rs"));
    assert!(summary.contains("Failed acceptance criteria:"));
    assert!(summary.contains("OAuth flow works"));
}

// =========================================================================
// Serialization
// =========================================================================

#[test]
fn verdict_serialization_roundtrip() {
    let mut verdict = make_verdict(
        FulfillmentStatus::PartiallyFulfilled,
        vec![make_gap(
            "Error handling",
            GapSeverity::Major,
            "Complete",
            "Incomplete",
            "Add middleware",
            vec!["src/api.rs"],
        )],
        vec![make_criterion("Build passes", "", true, "OK")],
    );
    verdict.summary = "Partial".into();
    verdict.confidence = 0.75;
    verdict.duration_ms = 7000;

    let json = serde_json::to_string(&verdict).unwrap();
    let deserialized: ValidationVerdict = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.status, FulfillmentStatus::PartiallyFulfilled);
    assert_eq!(deserialized.gaps.len(), 1);
    assert_eq!(deserialized.gaps[0].severity, GapSeverity::Major);
    assert_eq!(deserialized.criteria_results.len(), 1);
    assert!(deserialized.criteria_results[0].passed);
    assert!((deserialized.confidence - 0.75).abs() < f64::EPSILON);
}

// =========================================================================
// Display impls
// =========================================================================

#[test]
fn fulfillment_status_display() {
    assert_eq!(FulfillmentStatus::Fulfilled.to_string(), "Fulfilled");
    assert_eq!(
        FulfillmentStatus::PartiallyFulfilled.to_string(),
        "Partially Fulfilled"
    );
    assert_eq!(FulfillmentStatus::NotFulfilled.to_string(), "Not Fulfilled");
}

#[test]
fn gap_severity_display() {
    assert_eq!(GapSeverity::Critical.to_string(), "Critical");
    assert_eq!(GapSeverity::Major.to_string(), "Major");
    assert_eq!(GapSeverity::Minor.to_string(), "Minor");
}

// =========================================================================
// ValidatorPromptBuilder
// =========================================================================

#[test]
fn validator_prompt_contains_constraint_format() {
    let config = ValidatorPromptConfig {
        requirements: "Add OAuth authentication".into(),
        all_acceptance_criteria: vec!["cargo build succeeds".into()],
        tasks_summary: vec![TaskValidationSummary {
            title: "Implement login".into(),
            completed: true,
            output_excerpt: "Done".into(),
            acceptance_criteria: vec!["Login works".into()],
        }],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };

    let prompt = ValidatorPromptBuilder::build(&config);
    assert!(prompt.contains("HARD CONSTRAINTS"));
    assert!(prompt.contains("VIOLATION"));
    assert!(prompt.contains("status=\"fulfilled\" ONLY when ALL requirements"));
}

#[test]
fn validator_prompt_includes_requirements_and_criteria() {
    let config = ValidatorPromptConfig {
        requirements: "Build REST API with pagination".into(),
        all_acceptance_criteria: vec![
            "GET /items returns paginated list".into(),
            "cargo test passes".into(),
        ],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };

    let prompt = ValidatorPromptBuilder::build(&config);
    assert!(prompt.contains("Build REST API with pagination"));
    assert!(prompt.contains("GET /items returns paginated list"));
    assert!(prompt.contains("cargo test passes"));
}

#[test]
fn validator_prompt_includes_task_summaries() {
    let config = ValidatorPromptConfig {
        requirements: "Test".into(),
        tasks_summary: vec![
            TaskValidationSummary {
                title: "Setup DB".into(),
                completed: true,
                output_excerpt: "Migration applied".into(),
                acceptance_criteria: vec!["Schema created".into()],
            },
            TaskValidationSummary {
                title: "Add endpoints".into(),
                completed: false,
                output_excerpt: "Timeout".into(),
                acceptance_criteria: vec![],
            },
        ],
        failed_tasks: vec!["Add endpoints".into()],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };

    let prompt = ValidatorPromptBuilder::build(&config);
    assert!(prompt.contains("[COMPLETED] Setup DB"));
    assert!(prompt.contains("[FAILED] Add endpoints"));
    assert!(prompt.contains("Migration applied"));
    assert!(prompt.contains("Schema created"));
}

#[test]
fn validator_prompt_includes_command_results() {
    let config = ValidatorPromptConfig {
        requirements: "Test".into(),
        command_results: vec![
            CommandResult {
                command: "cargo build".into(),
                success: true,
                output: "Finished dev profile".into(),
            },
            CommandResult {
                command: "cargo test".into(),
                success: false,
                output: "3 tests failed".into(),
            },
        ],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };

    let prompt = ValidatorPromptBuilder::build(&config);
    assert!(prompt.contains("[PASS] $ cargo build"));
    assert!(prompt.contains("[FAIL] $ cargo test"));
    assert!(prompt.contains("3 tests failed"));
}

#[test]
fn validator_prompt_uses_xml_for_claude() {
    let config = ValidatorPromptConfig {
        requirements: "Test".into(),
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };

    let prompt = ValidatorPromptBuilder::build(&config);
    assert!(prompt.contains("<validation_constraints>"));
    assert!(prompt.contains("<original_requirements>"));
    assert!(prompt.contains("<output_format>"));
}

#[test]
fn validator_prompt_uses_markdown_for_non_claude() {
    let config = ValidatorPromptConfig {
        requirements: "Test".into(),
        provider_id: "openai-foundry".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };

    let prompt = ValidatorPromptBuilder::build(&config);
    assert!(prompt.contains("## Validation Constraints"));
    assert!(prompt.contains("## Original Requirements"));
    assert!(prompt.contains("## Output Format"));
}

#[test]
fn validator_prompt_includes_output_json_schema() {
    let config = ValidatorPromptConfig {
        requirements: "Test".into(),
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };

    let prompt = ValidatorPromptBuilder::build(&config);
    assert!(prompt.contains("\"status\""));
    assert!(prompt.contains("\"gaps\""));
    assert!(prompt.contains("\"criteria_results\""));
    assert!(prompt.contains("\"confidence\""));
    assert!(prompt.contains("\"suggested_action\""));
}

#[test]
fn validator_prompt_truncates_long_output() {
    let config = ValidatorPromptConfig {
        requirements: "Test".into(),
        combined_output: "x".repeat(20_000),
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };

    let prompt = ValidatorPromptBuilder::build(&config);
    assert!(prompt.contains("Output truncated"));
    assert!(prompt.len() < 20_000);
}

#[test]
fn validator_prompt_includes_files_modified() {
    let config = ValidatorPromptConfig {
        requirements: "Test".into(),
        files_modified: vec!["src/auth.rs".into(), "src/api.rs".into()],
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };

    let prompt = ValidatorPromptBuilder::build(&config);
    assert!(prompt.contains("src/auth.rs"));
    assert!(prompt.contains("src/api.rs"));
}

// =========================================================================
// P5: gaps_as_proposed_tasks
// =========================================================================

#[test]
fn gaps_as_proposed_tasks_critical_gap() {
    let verdict = make_verdict(
        FulfillmentStatus::PartiallyFulfilled,
        vec![make_gap(
            "OAuth setup",
            GapSeverity::Critical,
            "OAuth provider configured",
            "No OAuth provider",
            "Add OAuth configuration task",
            vec!["src/auth.rs"],
        )],
        vec![],
    );

    let tasks = verdict.gaps_as_proposed_tasks();
    assert_eq!(tasks.len(), 1);

    let task = &tasks[0];
    assert!(task.title.contains("OAuth setup"));
    assert!(task
        .description
        .contains("Expected: OAuth provider configured"));
    assert!(task
        .description
        .contains("Action: Add OAuth configuration task"));
    assert_eq!(task.complexity, 8);
    assert_eq!(task.priority, 1);
    assert!(task
        .acceptance_criteria
        .iter()
        .any(|c| c.contains("OAuth setup")));
    assert_eq!(task.affected_files, vec!["src/auth.rs"]);
}

#[test]
fn gaps_as_proposed_tasks_empty_gaps() {
    let verdict = make_verdict(FulfillmentStatus::Fulfilled, vec![], vec![]);

    let tasks = verdict.gaps_as_proposed_tasks();
    assert!(tasks.is_empty());
}

#[test]
fn gaps_as_proposed_tasks_filters_minor_gaps() {
    let verdict = make_verdict(
        FulfillmentStatus::NotFulfilled,
        vec![
            make_gap(
                "Critical issue",
                GapSeverity::Critical,
                "Expected",
                "Actual",
                "Fix critical",
                vec!["a.rs"],
            ),
            make_gap(
                "Major issue",
                GapSeverity::Major,
                "Expected",
                "Actual",
                "Fix major",
                vec![],
            ),
            make_gap(
                "Minor issue",
                GapSeverity::Minor,
                "Expected",
                "Actual",
                "Fix minor",
                vec![],
            ),
        ],
        vec![],
    );

    let tasks = verdict.gaps_as_proposed_tasks();
    // Minor gaps are filtered out — only Critical + Major generate tasks
    assert_eq!(tasks.len(), 2);

    // Critical
    assert_eq!(tasks[0].complexity, 8);
    assert_eq!(tasks[0].priority, 1);
    assert_eq!(tasks[0].affected_files, vec!["a.rs"]);

    // Major
    assert_eq!(tasks[1].complexity, 5);
    assert_eq!(tasks[1].priority, 50);
    assert!(tasks[1].affected_files.is_empty());
}

// =========================================================================
// P5: previous_validation_verdict in manager prompt
// =========================================================================

#[test]
fn manager_prompt_includes_previous_validation_verdict() {
    let config = ManagerPromptConfig {
        requirements: "Build a web app".into(),
        repo_map: "src/\n  main.rs".into(),
        provider_id: "claude-code".into(),
        worker_count: 3,
        previous_validation_verdict: Some("Validation verdict: Partially Fulfilled (confidence: 70%)\n\nRequirement gaps:\n  - [Critical] OAuth setup".into()),
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);
    assert!(prompt.contains("<previous_validation>"));
    assert!(prompt.contains("Partially Fulfilled"));
    assert!(prompt.contains("[Critical] OAuth setup"));
}

#[test]
fn manager_prompt_skips_validation_verdict_when_none() {
    let config = ManagerPromptConfig {
        requirements: "Build a web app".into(),
        repo_map: "src/\n  main.rs".into(),
        provider_id: "claude-code".into(),
        worker_count: 3,
        previous_validation_verdict: None,
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);
    assert!(!prompt.contains("previous_validation"));
    assert!(!prompt.contains("Previous Validation Verdict"));
}

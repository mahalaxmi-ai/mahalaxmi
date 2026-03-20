use std::sync::Arc;

use mahalaxmi_core::config::VerificationConfig;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::VerificationCheck;
use mahalaxmi_detection::verification::{
    LintIssue, LintResult, LintSeverity, LintTool, TestFailure, TestFramework, TestResult,
};
use mahalaxmi_orchestration::{
    format_retry_context, CheckResult, VerificationPipeline, VerificationResult, VerificationStatus,
};

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

fn test_pipeline(config: VerificationConfig) -> VerificationPipeline {
    VerificationPipeline::new(config, Arc::new(i18n()))
}

// ---------------------------------------------------------------------------
// VerificationStatus tests
// ---------------------------------------------------------------------------

#[test]
fn verification_status_is_success_for_passed() {
    assert!(VerificationStatus::Passed.is_success());
}

#[test]
fn verification_status_is_not_success_for_failed() {
    assert!(!VerificationStatus::Failed.is_success());
    assert!(!VerificationStatus::Skipped.is_success());
    assert!(!VerificationStatus::TimedOut.is_success());
}

#[test]
fn verification_status_display() {
    assert_eq!(VerificationStatus::Passed.to_string(), "Passed");
    assert_eq!(VerificationStatus::Failed.to_string(), "Failed");
    assert_eq!(VerificationStatus::Skipped.to_string(), "Skipped");
    assert_eq!(VerificationStatus::TimedOut.to_string(), "TimedOut");
}

// ---------------------------------------------------------------------------
// VerificationResult factory methods
// ---------------------------------------------------------------------------

#[test]
fn verification_result_passed_factory() {
    let checks = vec![CheckResult {
        check: VerificationCheck::Tests,
        passed: true,
        details: "3 passed, 0 failed, 0 skipped".to_owned(),
        test_result: None,
        lint_result: None,
    }];
    let result = VerificationResult::passed(checks, 150);

    assert_eq!(result.status, VerificationStatus::Passed);
    assert!(result.retry_context.is_none());
    assert_eq!(result.duration_ms, 150);
    assert_eq!(result.checks_run.len(), 1);
}

#[test]
fn verification_result_failed_factory() {
    let checks = vec![CheckResult {
        check: VerificationCheck::Tests,
        passed: false,
        details: "1 passed, 1 failed, 0 skipped".to_owned(),
        test_result: None,
        lint_result: None,
    }];
    let result = VerificationResult::failed(checks, "Fix the failing test.".to_owned(), 200);

    assert_eq!(result.status, VerificationStatus::Failed);
    assert!(result.retry_context.is_some());
    assert_eq!(result.retry_context.unwrap(), "Fix the failing test.");
    assert_eq!(result.duration_ms, 200);
}

#[test]
fn verification_result_skipped_factory() {
    let result = VerificationResult::skipped();

    assert_eq!(result.status, VerificationStatus::Skipped);
    assert!(result.checks_run.is_empty());
    assert!(result.retry_context.is_none());
    assert_eq!(result.duration_ms, 0);
}

#[test]
fn verification_result_passed_and_failed_counts() {
    let checks = vec![
        CheckResult {
            check: VerificationCheck::Tests,
            passed: true,
            details: "ok".to_owned(),
            test_result: None,
            lint_result: None,
        },
        CheckResult {
            check: VerificationCheck::Lint,
            passed: false,
            details: "1 error".to_owned(),
            test_result: None,
            lint_result: None,
        },
        CheckResult {
            check: VerificationCheck::SelfReview,
            passed: true,
            details: "ok".to_owned(),
            test_result: None,
            lint_result: None,
        },
    ];
    let result = VerificationResult::failed(checks, "ctx".to_owned(), 50);

    assert_eq!(result.passed_count(), 2);
    assert_eq!(result.failed_count(), 1);
}

// ---------------------------------------------------------------------------
// VerificationPipeline tests
// ---------------------------------------------------------------------------

#[test]
fn pipeline_not_enabled_when_disabled() {
    let config = VerificationConfig {
        enabled: false,
        checks: vec![VerificationCheck::Tests],
        max_retries: 2,
        fail_fast: true,
        auto_fix: true,
        timeout_seconds: 300,
        parse_verification_failures: true,
        require_verification_output: false,
        run_build_check_per_worker: false,
    };
    let pipeline = test_pipeline(config);
    assert!(!pipeline.is_enabled());
}

#[test]
fn pipeline_not_enabled_when_no_checks() {
    let config = VerificationConfig {
        enabled: true,
        checks: vec![],
        max_retries: 2,
        fail_fast: true,
        auto_fix: true,
        timeout_seconds: 300,
        parse_verification_failures: true,
        require_verification_output: false,
        run_build_check_per_worker: false,
    };
    let pipeline = test_pipeline(config);
    assert!(!pipeline.is_enabled());
}

#[test]
fn pipeline_enabled_with_checks() {
    let config = VerificationConfig {
        enabled: true,
        checks: vec![VerificationCheck::Tests],
        max_retries: 2,
        fail_fast: true,
        auto_fix: true,
        timeout_seconds: 300,
        parse_verification_failures: true,
        require_verification_output: false,
        run_build_check_per_worker: false,
    };
    let pipeline = test_pipeline(config);
    assert!(pipeline.is_enabled());
}

#[test]
fn pipeline_run_skipped_when_disabled() {
    let config = VerificationConfig {
        enabled: false,
        checks: vec![VerificationCheck::Tests],
        max_retries: 2,
        fail_fast: true,
        auto_fix: true,
        timeout_seconds: 300,
        parse_verification_failures: true,
        require_verification_output: false,
        run_build_check_per_worker: false,
    };
    let pipeline = test_pipeline(config);
    let result = pipeline.run("some output");

    assert_eq!(result.status, VerificationStatus::Skipped);
    assert!(result.checks_run.is_empty());
}

#[test]
fn pipeline_run_with_no_test_output() {
    let config = VerificationConfig {
        enabled: true,
        checks: vec![VerificationCheck::Tests],
        max_retries: 2,
        fail_fast: true,
        auto_fix: true,
        timeout_seconds: 300,
        parse_verification_failures: true,
        require_verification_output: false,
        run_build_check_per_worker: false,
    };
    let pipeline = test_pipeline(config);
    let result = pipeline.run("hello world, this has no test output");

    assert_eq!(result.status, VerificationStatus::Passed);
    assert_eq!(result.checks_run.len(), 1);
    assert!(result.checks_run[0].passed);
    assert_eq!(result.checks_run[0].details, "No test output detected");
}

#[test]
fn pipeline_run_with_passing_cargo_test() {
    let config = VerificationConfig {
        enabled: true,
        checks: vec![VerificationCheck::Tests],
        max_retries: 2,
        fail_fast: true,
        auto_fix: true,
        timeout_seconds: 300,
        parse_verification_failures: true,
        require_verification_output: false,
        run_build_check_per_worker: false,
    };
    let pipeline = test_pipeline(config);

    let output = "\
running 3 tests
test tests::test_one ... ok
test tests::test_two ... ok
test tests::test_three ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";

    let result = pipeline.run(output);

    assert_eq!(result.status, VerificationStatus::Passed);
    assert!(result.retry_context.is_none());
    assert_eq!(result.checks_run.len(), 1);
    assert!(result.checks_run[0].passed);
    assert!(result.checks_run[0].test_result.is_some());

    let tr = result.checks_run[0].test_result.as_ref().unwrap();
    assert_eq!(tr.passed, 3);
    assert_eq!(tr.failed, 0);
    assert_eq!(tr.framework, TestFramework::CargoTest);
}

#[test]
fn pipeline_run_with_failing_cargo_test() {
    let config = VerificationConfig {
        enabled: true,
        checks: vec![VerificationCheck::Tests],
        max_retries: 2,
        fail_fast: true,
        auto_fix: true,
        timeout_seconds: 300,
        parse_verification_failures: true,
        require_verification_output: false,
        run_build_check_per_worker: false,
    };
    let pipeline = test_pipeline(config);

    let output = "\
running 2 tests
test tests::test_one ... ok
test tests::test_two ... FAILED

failures:

---- tests::test_two stdout ----
thread 'tests::test_two' panicked at 'assertion failed: (left == right)
  left: 1,
 right: 2', src/lib.rs:42:5

failures:
    tests::test_two

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out";

    let result = pipeline.run(output);

    assert_eq!(result.status, VerificationStatus::Failed);
    assert!(result.retry_context.is_some());
    assert_eq!(result.checks_run.len(), 1);
    assert!(!result.checks_run[0].passed);

    let tr = result.checks_run[0].test_result.as_ref().unwrap();
    assert_eq!(tr.passed, 1);
    assert_eq!(tr.failed, 1);
    assert_eq!(tr.framework, TestFramework::CargoTest);

    let ctx = result.retry_context.unwrap();
    assert!(ctx.contains("Verification Failed"));
    assert!(ctx.contains("Test Failures"));
}

// ---------------------------------------------------------------------------
// format_retry_context tests
// ---------------------------------------------------------------------------

#[test]
fn format_retry_context_empty_when_all_pass() {
    let result = VerificationResult {
        status: VerificationStatus::Failed,
        checks_run: vec![
            CheckResult {
                check: VerificationCheck::Tests,
                passed: true,
                details: "3 passed, 0 failed, 0 skipped".to_owned(),
                test_result: None,
                lint_result: None,
            },
            CheckResult {
                check: VerificationCheck::Lint,
                passed: true,
                details: "0 errors, 0 warnings".to_owned(),
                test_result: None,
                lint_result: None,
            },
        ],
        retry_context: None,
        duration_ms: 100,
    };
    let ctx = format_retry_context(&result);

    // Should contain the header and footer but no failure sections
    assert!(ctx.contains("## Verification Failed"));
    assert!(ctx.contains("Please fix the above issues"));
    assert!(!ctx.contains("### Test Failures"));
    assert!(!ctx.contains("### Lint Issues"));
}

#[test]
fn format_retry_context_includes_test_failures() {
    let test_result = TestResult {
        framework: TestFramework::CargoTest,
        total: 3,
        passed: 2,
        failed: 1,
        skipped: 0,
        duration_ms: None,
        failures: vec![TestFailure {
            test_name: "tests::test_two".to_owned(),
            message: "assertion failed: left == right".to_owned(),
            file_path: Some("src/lib.rs".to_owned()),
            line: Some(42),
            stdout: None,
        }],
    };

    let result = VerificationResult {
        status: VerificationStatus::Failed,
        checks_run: vec![CheckResult {
            check: VerificationCheck::Tests,
            passed: false,
            details: "2 passed, 1 failed, 0 skipped".to_owned(),
            test_result: Some(test_result),
            lint_result: None,
        }],
        retry_context: None,
        duration_ms: 50,
    };
    let ctx = format_retry_context(&result);

    assert!(ctx.contains("### Test Failures (1 of 3 tests failed)"));
    assert!(ctx.contains("`tests::test_two`"));
    assert!(ctx.contains("src/lib.rs:42"));
    assert!(ctx.contains("assertion failed: left == right"));
}

#[test]
fn format_retry_context_includes_lint_issues() {
    let lint_result = LintResult {
        tool: LintTool::Clippy,
        total_issues: 2,
        errors: 1,
        warnings: 1,
        issues: vec![
            LintIssue {
                severity: LintSeverity::Error,
                message: "unused variable `x`".to_owned(),
                file_path: "src/main.rs".to_owned(),
                line: 10,
                column: Some(5),
                rule: Some("clippy::unused_variables".to_owned()),
            },
            LintIssue {
                severity: LintSeverity::Warning,
                message: "needless return".to_owned(),
                file_path: "src/lib.rs".to_owned(),
                line: 25,
                column: None,
                rule: Some("clippy::needless_return".to_owned()),
            },
        ],
    };

    let result = VerificationResult {
        status: VerificationStatus::Failed,
        checks_run: vec![CheckResult {
            check: VerificationCheck::Lint,
            passed: false,
            details: "1 errors, 1 warnings".to_owned(),
            test_result: None,
            lint_result: Some(lint_result),
        }],
        retry_context: None,
        duration_ms: 30,
    };
    let ctx = format_retry_context(&result);

    assert!(ctx.contains("### Lint Issues"));
    assert!(ctx.contains("ERROR src/main.rs:10:5"));
    assert!(ctx.contains("unused variable `x`"));
    assert!(ctx.contains("[clippy::unused_variables]"));
    assert!(ctx.contains("WARNING src/lib.rs:25"));
    assert!(ctx.contains("needless return"));
    assert!(ctx.contains("[clippy::needless_return]"));
}

#[test]
fn format_retry_context_includes_self_review() {
    let result = VerificationResult {
        status: VerificationStatus::Failed,
        checks_run: vec![CheckResult {
            check: VerificationCheck::SelfReview,
            passed: false,
            details: "Self-review found issues in the implementation".to_owned(),
            test_result: None,
            lint_result: None,
        }],
        retry_context: None,
        duration_ms: 20,
    };
    let ctx = format_retry_context(&result);

    assert!(ctx.contains("### Self-Review:"));
    assert!(ctx.contains("Self-review found issues in the implementation"));
}

// ---------------------------------------------------------------------------
// CheckResult / VerificationResult serde roundtrip tests
// ---------------------------------------------------------------------------

#[test]
fn check_result_serde_roundtrip() {
    let original = CheckResult {
        check: VerificationCheck::Lint,
        passed: false,
        details: "1 errors, 0 warnings".to_owned(),
        test_result: None,
        lint_result: Some(LintResult {
            tool: LintTool::Clippy,
            total_issues: 1,
            errors: 1,
            warnings: 0,
            issues: vec![LintIssue {
                severity: LintSeverity::Error,
                message: "unused import".to_owned(),
                file_path: "src/main.rs".to_owned(),
                line: 3,
                column: Some(5),
                rule: Some("unused_imports".to_owned()),
            }],
        }),
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: CheckResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.check, original.check);
    assert_eq!(deserialized.passed, original.passed);
    assert_eq!(deserialized.details, original.details);
    assert!(deserialized.test_result.is_none());
    assert!(deserialized.lint_result.is_some());

    let lr = deserialized.lint_result.unwrap();
    assert_eq!(lr.tool, LintTool::Clippy);
    assert_eq!(lr.errors, 1);
    assert_eq!(lr.issues.len(), 1);
    assert_eq!(lr.issues[0].message, "unused import");
}

#[test]
fn verification_result_serde_roundtrip() {
    let original = VerificationResult::failed(
        vec![
            CheckResult {
                check: VerificationCheck::Tests,
                passed: true,
                details: "5 passed, 0 failed, 0 skipped".to_owned(),
                test_result: Some(TestResult {
                    framework: TestFramework::CargoTest,
                    total: 5,
                    passed: 5,
                    failed: 0,
                    skipped: 0,
                    duration_ms: Some(120),
                    failures: vec![],
                }),
                lint_result: None,
            },
            CheckResult {
                check: VerificationCheck::Lint,
                passed: false,
                details: "2 errors, 1 warnings".to_owned(),
                test_result: None,
                lint_result: Some(LintResult {
                    tool: LintTool::Eslint,
                    total_issues: 3,
                    errors: 2,
                    warnings: 1,
                    issues: vec![],
                }),
            },
        ],
        "Retry: fix lint errors".to_owned(),
        250,
    );

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: VerificationResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.status, VerificationStatus::Failed);
    assert_eq!(deserialized.duration_ms, 250);
    assert_eq!(deserialized.checks_run.len(), 2);
    assert_eq!(
        deserialized.retry_context.as_deref(),
        Some("Retry: fix lint errors")
    );
    assert_eq!(deserialized.passed_count(), 1);
    assert_eq!(deserialized.failed_count(), 1);

    // Verify the embedded test result survived the roundtrip
    let tr = deserialized.checks_run[0].test_result.as_ref().unwrap();
    assert_eq!(tr.framework, TestFramework::CargoTest);
    assert_eq!(tr.total, 5);
    assert_eq!(tr.duration_ms, Some(120));

    // Verify the embedded lint result survived the roundtrip
    let lr = deserialized.checks_run[1].lint_result.as_ref().unwrap();
    assert_eq!(lr.tool, LintTool::Eslint);
    assert_eq!(lr.total_issues, 3);
}

//! Verification pipeline for worker self-verification.
//!
//! Sequences verification checks (tests, lint, self-review) and produces
//! structured results with retry context for re-injection on failure.

use std::sync::Arc;
use std::time::Instant;

use mahalaxmi_core::config::VerificationConfig;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::VerificationCheck;
use mahalaxmi_detection::verification::{LintResult, TestResult};
use serde::{Deserialize, Serialize};

/// Overall status of the verification pipeline run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// All configured checks passed.
    Passed,
    /// One or more checks failed.
    Failed,
    /// Verification was skipped (not enabled or no checks configured).
    Skipped,
    /// Verification timed out.
    TimedOut,
}

impl VerificationStatus {
    /// Returns true only if the status is `Passed`.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Passed)
    }
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Passed => "Passed",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
            Self::TimedOut => "TimedOut",
        };
        write!(f, "{}", label)
    }
}

/// Result of a single verification check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Which check was run.
    pub check: VerificationCheck,
    /// Whether the check passed.
    pub passed: bool,
    /// Human-readable details (summary or error message).
    pub details: String,
    /// Parsed test results, if this was a Tests check.
    pub test_result: Option<TestResult>,
    /// Parsed lint results, if this was a Lint check.
    pub lint_result: Option<LintResult>,
}

/// Aggregate result of the full verification pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Overall verification status.
    pub status: VerificationStatus,
    /// Results from each check that was run.
    pub checks_run: Vec<CheckResult>,
    /// Formatted retry context string for re-injection (None if passed).
    pub retry_context: Option<String>,
    /// Total duration of all checks in milliseconds.
    pub duration_ms: u64,
}

impl VerificationResult {
    /// Create a result indicating all checks passed.
    pub fn passed(checks: Vec<CheckResult>, duration_ms: u64) -> Self {
        Self {
            status: VerificationStatus::Passed,
            checks_run: checks,
            retry_context: None,
            duration_ms,
        }
    }

    /// Create a result indicating one or more checks failed.
    pub fn failed(checks: Vec<CheckResult>, retry_context: String, duration_ms: u64) -> Self {
        Self {
            status: VerificationStatus::Failed,
            checks_run: checks,
            retry_context: Some(retry_context),
            duration_ms,
        }
    }

    /// Create a result indicating verification was skipped.
    pub fn skipped() -> Self {
        Self {
            status: VerificationStatus::Skipped,
            checks_run: Vec::new(),
            retry_context: None,
            duration_ms: 0,
        }
    }

    /// Number of checks that passed.
    pub fn passed_count(&self) -> usize {
        self.checks_run.iter().filter(|c| c.passed).count()
    }

    /// Number of checks that failed.
    pub fn failed_count(&self) -> usize {
        self.checks_run.iter().filter(|c| !c.passed).count()
    }
}

/// Pipeline that runs configured verification checks against worker output.
pub struct VerificationPipeline {
    config: VerificationConfig,
    #[allow(dead_code)]
    i18n: Arc<I18nService>,
}

impl VerificationPipeline {
    /// Create a new pipeline with the given configuration.
    pub fn new(config: VerificationConfig, i18n: Arc<I18nService>) -> Self {
        Self { config, i18n }
    }

    /// Whether verification is enabled and has checks configured.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.checks.is_empty()
    }

    /// Run all configured verification checks against the given output.
    ///
    /// Checks run in priority order (Tests -> Lint -> SelfReview).
    /// If `fail_fast` is enabled, stops on first failure.
    /// Returns structured [`VerificationResult`].
    pub fn run(&self, output: &str) -> VerificationResult {
        if !self.is_enabled() {
            return VerificationResult::skipped();
        }

        let start = Instant::now();
        let mut checks_run = Vec::new();
        let mut all_passed = true;

        // Sort checks by priority to ensure deterministic execution order
        let mut ordered_checks = self.config.checks.clone();
        ordered_checks.sort_by_key(|c| c.priority());

        for check in &ordered_checks {
            let result = self.run_check(*check, output);
            if !result.passed {
                all_passed = false;
            }
            checks_run.push(result);

            if !all_passed && self.config.fail_fast {
                break;
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        if all_passed {
            VerificationResult::passed(checks_run, duration_ms)
        } else {
            let retry_context = format_retry_context(&VerificationResult {
                status: VerificationStatus::Failed,
                checks_run: checks_run.clone(),
                retry_context: None,
                duration_ms,
            });
            VerificationResult::failed(checks_run, retry_context, duration_ms)
        }
    }

    /// Run a single check against the given output.
    fn run_check(&self, check: VerificationCheck, output: &str) -> CheckResult {
        match check {
            VerificationCheck::Tests => self.run_tests_check(output),
            VerificationCheck::Lint => self.run_lint_check(output),
            VerificationCheck::SelfReview => self.run_self_review_check(output),
        }
    }

    /// Run the Tests check — delegates to test_parser::parse_test_output().
    fn run_tests_check(&self, output: &str) -> CheckResult {
        use mahalaxmi_detection::verification::test_parser::parse_test_output;

        match parse_test_output(output) {
            Some(test_result) => {
                let passed = test_result.all_passed();
                let details = test_result.summary();
                CheckResult {
                    check: VerificationCheck::Tests,
                    passed,
                    details,
                    test_result: Some(test_result),
                    lint_result: None,
                }
            }
            None => CheckResult {
                check: VerificationCheck::Tests,
                passed: true,
                details: "No test output detected".to_owned(),
                test_result: None,
                lint_result: None,
            },
        }
    }

    /// Run the Lint check — delegates to lint_parser::parse_lint_output().
    fn run_lint_check(&self, output: &str) -> CheckResult {
        use mahalaxmi_detection::verification::lint_parser::parse_lint_output;

        match parse_lint_output(output) {
            Some(lint_result) => {
                let passed = !lint_result.has_errors();
                let details = lint_result.summary();
                CheckResult {
                    check: VerificationCheck::Lint,
                    passed,
                    details,
                    test_result: None,
                    lint_result: Some(lint_result),
                }
            }
            None => CheckResult {
                check: VerificationCheck::Lint,
                passed: true,
                details: "No lint output detected".to_owned(),
                test_result: None,
                lint_result: None,
            },
        }
    }

    /// Run the SelfReview check.
    ///
    /// Generates a structured review request. Actual AI invocation
    /// happens at the orchestration layer; this check prepares the request.
    fn run_self_review_check(&self, output: &str) -> CheckResult {
        // Self-review always "passes" at the pipeline level — the actual
        // AI review is handled by the orchestration layer. We prepare the
        // output summary for injection into the review prompt.
        let details = if output.len() > 500 {
            format!("Output available for self-review ({} bytes)", output.len())
        } else {
            "Output available for self-review".to_owned()
        };

        CheckResult {
            check: VerificationCheck::SelfReview,
            passed: true,
            details,
            test_result: None,
            lint_result: None,
        }
    }
}

/// Format verification failures into structured text for re-injection
/// into the worker's next prompt.
///
/// Groups by check type, includes specific failure details
/// (test names, lint issues with file/line).
pub fn format_retry_context(result: &VerificationResult) -> String {
    let mut parts = Vec::new();
    parts.push("## Verification Failed — Retry Context".to_owned());
    parts.push(String::new());

    for check in &result.checks_run {
        if check.passed {
            continue;
        }

        match check.check {
            VerificationCheck::Tests => {
                if let Some(ref tr) = check.test_result {
                    parts.push(format!(
                        "### Test Failures ({} of {} tests failed)",
                        tr.failed, tr.total
                    ));
                    for failure in &tr.failures {
                        let location = match (&failure.file_path, failure.line) {
                            (Some(path), Some(line)) => format!(" at {}:{}", path, line),
                            (Some(path), None) => format!(" at {}", path),
                            _ => String::new(),
                        };
                        parts.push(format!(
                            "- `{}`{} — {}",
                            failure.test_name, location, failure.message
                        ));
                    }
                } else {
                    parts.push(format!("### Tests: {}", check.details));
                }
            }
            VerificationCheck::Lint => {
                if let Some(ref lr) = check.lint_result {
                    parts.push(format!("### Lint Issues ({})", lr.summary()));
                    for issue in &lr.issues {
                        let col_str = issue.column.map(|c| format!(":{}", c)).unwrap_or_default();
                        let rule_str = issue
                            .rule
                            .as_ref()
                            .map(|r| format!(" [{}]", r))
                            .unwrap_or_default();
                        parts.push(format!(
                            "- {} {}:{}{} — {}{}",
                            issue.severity.as_str().to_uppercase(),
                            issue.file_path,
                            issue.line,
                            col_str,
                            issue.message,
                            rule_str,
                        ));
                    }
                } else {
                    parts.push(format!("### Lint: {}", check.details));
                }
            }
            VerificationCheck::SelfReview => {
                parts.push(format!("### Self-Review: {}", check.details));
            }
        }

        parts.push(String::new());
    }

    parts.push("Please fix the above issues and try again.".to_owned());
    parts.join("\n")
}

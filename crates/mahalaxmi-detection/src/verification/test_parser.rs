//! Test runner output parsers.
//!
//! Extracts structured [`TestResult`] data from the terminal output of
//! common test frameworks: Cargo test, pytest, Jest, and Go test.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifies the test framework that produced the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestFramework {
    /// Rust `cargo test` runner.
    CargoTest,
    /// Python `pytest` runner.
    Pytest,
    /// JavaScript/TypeScript `jest` runner.
    Jest,
    /// Go `go test` runner.
    GoTest,
}

impl TestFramework {
    /// Returns the canonical string identifier for this framework.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CargoTest => "cargo-test",
            Self::Pytest => "pytest",
            Self::Jest => "jest",
            Self::GoTest => "go-test",
        }
    }
}

impl fmt::Display for TestFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single test failure with optional location and captured output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestFailure {
    /// Fully-qualified test name (e.g. `module::test_name`).
    pub test_name: String,
    /// Failure message or assertion description.
    pub message: String,
    /// Source file where the test is defined, if available.
    pub file_path: Option<String>,
    /// Line number of the failure, if available.
    pub line: Option<usize>,
    /// Captured stdout/stderr from the failing test.
    pub stdout: Option<String>,
}

/// Aggregated result from a single test-runner invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestResult {
    /// The framework that produced this result.
    pub framework: TestFramework,
    /// Total number of tests executed.
    pub total: usize,
    /// Number of tests that passed.
    pub passed: usize,
    /// Number of tests that failed.
    pub failed: usize,
    /// Number of tests that were skipped or ignored.
    pub skipped: usize,
    /// Wall-clock duration in milliseconds, if reported.
    pub duration_ms: Option<u64>,
    /// Details of each failing test.
    pub failures: Vec<TestFailure>,
}

impl TestResult {
    /// Returns `true` when no tests failed.
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Human-readable one-line summary (e.g. "42 passed, 2 failed, 1 skipped").
    pub fn summary(&self) -> String {
        format!(
            "{} passed, {} failed, {} skipped",
            self.passed, self.failed, self.skipped
        )
    }
}

/// Auto-detect the test framework from raw output and parse results.
///
/// Detection order:
/// 1. `test result:` marker indicates `cargo test`.
/// 2. `pytest` or `test session starts` with `passed` indicates pytest.
/// 3. `Test Suites:` or `Tests:` with `passed` and `total` indicates Jest.
/// 4. `--- PASS:` / `--- FAIL:` / leading `ok ` / `FAIL\t` indicates Go test.
///
/// Returns `None` if the framework cannot be identified or the output
/// cannot be parsed.
pub fn parse_test_output(output: &str) -> Option<TestResult> {
    if output.contains("test result:") {
        if let Some(r) = parse_cargo_test(output) {
            return Some(r);
        }
    }

    if output.contains("passed")
        && (output.contains("pytest") || output.contains("test session starts"))
    {
        if let Some(r) = parse_pytest(output) {
            return Some(r);
        }
    }

    if (output.contains("Test Suites:") || output.contains("Tests:"))
        && output.contains("passed")
        && output.contains("total")
    {
        if let Some(r) = parse_jest(output) {
            return Some(r);
        }
    }

    if output.contains("--- PASS:")
        || output.contains("--- FAIL:")
        || output.starts_with("ok ")
        || output.contains("\nok ")
        || output.starts_with("FAIL\t")
        || output.contains("\nFAIL\t")
    {
        if let Some(r) = parse_go_test(output) {
            return Some(r);
        }
    }

    None
}

/// Parse `cargo test` output into a [`TestResult`].
///
/// Looks for the standard summary line
/// `test result: ok|FAILED. N passed; N failed; N ignored` and
/// extracts individual failure blocks delimited by
/// `---- <name> stdout ----`.
pub fn parse_cargo_test(output: &str) -> Option<TestResult> {
    let summary_re =
        Regex::new(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored")
            .ok()?;

    let caps = summary_re.captures(output)?;
    let passed: usize = caps.get(1)?.as_str().parse().ok()?;
    let failed: usize = caps.get(2)?.as_str().parse().ok()?;
    let skipped: usize = caps.get(3)?.as_str().parse().ok()?;
    let total = passed + failed + skipped;

    // Extract duration
    let duration_ms = Regex::new(r"finished in ([\d.]+)s")
        .ok()
        .and_then(|re| re.captures(output))
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok())
        .map(|secs| (secs * 1000.0) as u64);

    // Extract failure blocks
    let failure_block_re = Regex::new(r"---- (.+?) stdout ----\n([\s\S]*?)(?:\n\n|\z)").ok()?;
    let location_re = Regex::new(r"(?:thread '.+' panicked at )?'?([^']+)',\s*(.+?):(\d+)").ok();
    let mut failures = Vec::new();

    for cap in failure_block_re.captures_iter(output) {
        let test_name = cap
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let block = cap
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        // Try to extract file path and line from the block
        let (message, file_path, line) = if let Some(loc_re) = &location_re {
            if let Some(loc_caps) = loc_re.captures(&block) {
                let msg = loc_caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let fp = loc_caps.get(2).map(|m| m.as_str().to_string());
                let ln = loc_caps
                    .get(3)
                    .and_then(|m| m.as_str().parse::<usize>().ok());
                (msg, fp, ln)
            } else {
                (first_nonempty_line(&block), None, None)
            }
        } else {
            (first_nonempty_line(&block), None, None)
        };

        failures.push(TestFailure {
            test_name,
            message,
            file_path,
            line,
            stdout: if block.is_empty() { None } else { Some(block) },
        });
    }

    Some(TestResult {
        framework: TestFramework::CargoTest,
        total,
        passed,
        failed,
        skipped,
        duration_ms,
        failures,
    })
}

/// Parse `pytest` output into a [`TestResult`].
///
/// Recognises the short summary line containing counts like
/// `3 passed, 1 failed, 2 skipped` and `FAILED` markers.
pub fn parse_pytest(output: &str) -> Option<TestResult> {
    let passed = extract_count(output, r"(\d+) passed");
    let failed = extract_count(output, r"(\d+) failed");
    let skipped = extract_count(output, r"(\d+) skipped");
    let total = passed + failed + skipped;

    // Need at least one meaningful count
    if total == 0 {
        return None;
    }

    // Duration
    let duration_ms = Regex::new(r"in ([\d.]+)s")
        .ok()
        .and_then(|re| re.captures(output))
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok())
        .map(|secs| (secs * 1000.0) as u64);

    // Failure details: FAILED path/file.py::test_name - message
    let failure_re = Regex::new(r"(?m)^FAILED (.+?)::(.+?) - (.+)$").ok()?;
    let mut failures = Vec::new();
    for cap in failure_re.captures_iter(output) {
        let file_path = cap.get(1).map(|m| m.as_str().to_string());
        let test_name = cap
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let message = cap
            .get(3)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        failures.push(TestFailure {
            test_name,
            message,
            file_path,
            line: None,
            stdout: None,
        });
    }

    Some(TestResult {
        framework: TestFramework::Pytest,
        total,
        passed,
        failed,
        skipped,
        duration_ms,
        failures,
    })
}

/// Parse `jest` output into a [`TestResult`].
///
/// Looks for the `Tests:` summary line
/// `Tests:  N failed, N passed, N total` and `FAIL` file markers.
pub fn parse_jest(output: &str) -> Option<TestResult> {
    let tests_re = Regex::new(r"Tests:\s+(?:(\d+) failed,\s+)?(\d+) passed,\s+(\d+) total").ok()?;
    let caps = tests_re.captures(output)?;

    let failed: usize = caps
        .get(1)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);
    let passed: usize = caps.get(2)?.as_str().parse().ok()?;
    let total: usize = caps.get(3)?.as_str().parse().ok()?;
    let skipped = total.saturating_sub(passed + failed);

    // Duration: Time:  1.234 s
    let duration_ms = Regex::new(r"Time:\s+([\d.]+)\s*s")
        .ok()
        .and_then(|re| re.captures(output))
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok())
        .map(|secs| (secs * 1000.0) as u64);

    // Failing test files: FAIL path/to/file.test.js
    let fail_file_re = Regex::new(r"(?m)^\s*FAIL (.+)$").ok()?;
    let mut failures = Vec::new();
    for cap in fail_file_re.captures_iter(output) {
        let file_path = cap.get(1).map(|m| m.as_str().trim().to_string());
        failures.push(TestFailure {
            test_name: file_path.clone().unwrap_or_default(),
            message: "test suite failed".to_string(),
            file_path,
            line: None,
            stdout: None,
        });
    }

    Some(TestResult {
        framework: TestFramework::Jest,
        total,
        passed,
        failed,
        skipped,
        duration_ms,
        failures,
    })
}

/// Parse `go test` output into a [`TestResult`].
///
/// Processes `--- PASS:` / `--- FAIL:` lines for individual test
/// outcomes and `ok` / `FAIL` package summary lines for timing.
pub fn parse_go_test(output: &str) -> Option<TestResult> {
    let result_re = Regex::new(r"(?m)^--- (PASS|FAIL|SKIP): (\S+)").ok()?;

    let mut passed: usize = 0;
    let mut failed: usize = 0;
    let mut skipped: usize = 0;
    let mut failures = Vec::new();

    for cap in result_re.captures_iter(output) {
        let status = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let test_name = cap
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        match status {
            "PASS" => passed += 1,
            "FAIL" => {
                failed += 1;
                failures.push(TestFailure {
                    test_name,
                    message: "test failed".to_string(),
                    file_path: None,
                    line: None,
                    stdout: None,
                });
            }
            "SKIP" => skipped += 1,
            _ => {}
        }
    }

    // Also count from package summary lines if no individual results found
    if passed == 0 && failed == 0 && skipped == 0 {
        let ok_re = Regex::new(r"(?m)^ok\s+\S+").ok()?;
        let fail_pkg_re = Regex::new(r"(?m)^FAIL\t\S+").ok()?;
        let ok_count = ok_re.find_iter(output).count();
        let fail_count = fail_pkg_re.find_iter(output).count();
        if ok_count == 0 && fail_count == 0 {
            return None;
        }
        passed = ok_count;
        failed = fail_count;

        for cap in fail_pkg_re.captures_iter(output) {
            let pkg = cap
                .get(0)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            failures.push(TestFailure {
                test_name: pkg,
                message: "package tests failed".to_string(),
                file_path: None,
                line: None,
                stdout: None,
            });
        }
    }

    let total = passed + failed + skipped;

    // Duration from package summary: ok  pkg  1.234s
    let duration_ms = Regex::new(r"(?m)(?:ok|FAIL)\s+\S+\s+([\d.]+)s")
        .ok()
        .and_then(|re| {
            let mut max_ms: Option<u64> = None;
            for cap in re.captures_iter(output) {
                if let Some(m) = cap.get(1) {
                    if let Ok(secs) = m.as_str().parse::<f64>() {
                        let ms = (secs * 1000.0) as u64;
                        max_ms = Some(max_ms.map_or(ms, |prev: u64| prev.max(ms)));
                    }
                }
            }
            max_ms
        });

    Some(TestResult {
        framework: TestFramework::GoTest,
        total,
        passed,
        failed,
        skipped,
        duration_ms,
        failures,
    })
}

/// Helper: extract a count from the first regex match of `(\d+) label`.
fn extract_count(output: &str, pattern: &str) -> usize {
    Regex::new(pattern)
        .ok()
        .and_then(|re| re.captures(output))
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<usize>().ok())
        .unwrap_or(0)
}

/// Helper: return the first non-empty line of a string, or the whole
/// string trimmed if it is a single line.
fn first_nonempty_line(s: &str) -> String {
    s.lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(s)
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Cargo Test -------------------------------------------------------

    #[test]
    fn cargo_test_all_pass() {
        let output = "\
running 42 tests
test foo::bar ... ok
test baz::qux ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s
";
        let result = parse_cargo_test(output).expect("should parse");
        assert_eq!(result.framework, TestFramework::CargoTest);
        assert_eq!(result.passed, 42);
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.total, 42);
        assert!(result.all_passed());
        assert_eq!(result.duration_ms, Some(1230));
        assert!(result.failures.is_empty());
    }

    #[test]
    fn cargo_test_with_failures() {
        let output = "\
running 10 tests
test works ... ok
test broken ... FAILED

failures:

---- broken stdout ----
thread 'broken' panicked at 'assertion failed', src/lib.rs:42

failures:
    broken

test result: FAILED. 8 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.5s
";
        let result = parse_cargo_test(output).expect("should parse");
        assert_eq!(result.passed, 8);
        assert_eq!(result.failed, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.total, 10);
        assert!(!result.all_passed());
        assert_eq!(result.duration_ms, Some(500));
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].test_name, "broken");
    }

    #[test]
    fn cargo_test_summary_format() {
        let result = TestResult {
            framework: TestFramework::CargoTest,
            total: 45,
            passed: 42,
            failed: 2,
            skipped: 1,
            duration_ms: None,
            failures: vec![],
        };
        assert_eq!(result.summary(), "42 passed, 2 failed, 1 skipped");
    }

    // -- Pytest -----------------------------------------------------------

    #[test]
    fn pytest_all_pass() {
        let output = "\
============================= test session starts ==============================
collected 15 items

tests/test_math.py ............... [100%]

============================== 15 passed in 0.42s ==============================
";
        let result = parse_pytest(output).expect("should parse");
        assert_eq!(result.framework, TestFramework::Pytest);
        assert_eq!(result.passed, 15);
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 0);
        assert!(result.all_passed());
        assert_eq!(result.duration_ms, Some(420));
    }

    #[test]
    fn pytest_with_failures() {
        let output = "\
============================= test session starts ==============================
collected 10 items

tests/test_foo.py .....F..s. [100%]

=================================== FAILURES ===================================
FAILED tests/test_foo.py::test_divide - ZeroDivisionError: division by zero

========================= 7 passed, 1 failed, 1 skipped in 1.1s ========================
";
        let result = parse_pytest(output).expect("should parse");
        assert_eq!(result.passed, 7);
        assert_eq!(result.failed, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.total, 9);
        assert!(!result.all_passed());
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].test_name, "test_divide");
        assert_eq!(
            result.failures[0].file_path.as_deref(),
            Some("tests/test_foo.py")
        );
        assert_eq!(
            result.failures[0].message,
            "ZeroDivisionError: division by zero"
        );
    }

    // -- Jest -------------------------------------------------------------

    #[test]
    fn jest_all_pass() {
        let output = "\
 PASS src/utils.test.ts
 PASS src/app.test.ts

Test Suites:  2 passed, 2 total
Tests:        12 passed, 12 total
Snapshots:    0 total
Time:         3.456 s
";
        let result = parse_jest(output).expect("should parse");
        assert_eq!(result.framework, TestFramework::Jest);
        assert_eq!(result.passed, 12);
        assert_eq!(result.failed, 0);
        assert_eq!(result.total, 12);
        assert!(result.all_passed());
        assert_eq!(result.duration_ms, Some(3456));
    }

    #[test]
    fn jest_with_failures() {
        let output = "\
 FAIL src/broken.test.ts
  \u{25cf} should work

    expect(received).toBe(expected)

 PASS src/utils.test.ts

Test Suites:  1 failed, 1 passed, 2 total
Tests:        2 failed, 5 passed, 7 total
Snapshots:    0 total
Time:         2.1 s
";
        let result = parse_jest(output).expect("should parse");
        assert_eq!(result.passed, 5);
        assert_eq!(result.failed, 2);
        assert_eq!(result.total, 7);
        assert!(!result.all_passed());
        assert_eq!(result.duration_ms, Some(2100));
        assert_eq!(result.failures.len(), 1);
        assert_eq!(
            result.failures[0].file_path.as_deref(),
            Some("src/broken.test.ts")
        );
    }

    // -- Go Test ----------------------------------------------------------

    #[test]
    fn go_test_all_pass() {
        let output = "\
=== RUN   TestAdd
--- PASS: TestAdd (0.00s)
=== RUN   TestSub
--- PASS: TestSub (0.00s)
PASS
ok  \texample.com/math\t0.003s
";
        let result = parse_go_test(output).expect("should parse");
        assert_eq!(result.framework, TestFramework::GoTest);
        assert_eq!(result.passed, 2);
        assert_eq!(result.failed, 0);
        assert!(result.all_passed());
        assert_eq!(result.duration_ms, Some(3));
    }

    #[test]
    fn go_test_with_failures() {
        let output = "\
=== RUN   TestGood
--- PASS: TestGood (0.00s)
=== RUN   TestBad
--- FAIL: TestBad (0.01s)
    main_test.go:15: expected 4 got 5
FAIL
FAIL\texample.com/math\t0.015s
";
        let result = parse_go_test(output).expect("should parse");
        assert_eq!(result.passed, 1);
        assert_eq!(result.failed, 1);
        assert!(!result.all_passed());
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].test_name, "TestBad");
    }

    #[test]
    fn go_test_skip() {
        let output = "\
=== RUN   TestSkipped
--- SKIP: TestSkipped (0.00s)
    main_test.go:10: skipping in short mode
=== RUN   TestOk
--- PASS: TestOk (0.00s)
ok  \texample.com/math\t0.002s
";
        let result = parse_go_test(output).expect("should parse");
        assert_eq!(result.passed, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.failed, 0);
        assert!(result.all_passed());
    }

    // -- Auto-detection ---------------------------------------------------

    #[test]
    fn auto_detect_cargo() {
        let output = "test result: ok. 5 passed; 0 failed; 0 ignored; finished in 0.1s\n";
        let result = parse_test_output(output).expect("should detect cargo");
        assert_eq!(result.framework, TestFramework::CargoTest);
    }

    #[test]
    fn auto_detect_pytest() {
        let output = "===== test session starts =====\n10 passed in 0.5s\n";
        let result = parse_test_output(output).expect("should detect pytest");
        assert_eq!(result.framework, TestFramework::Pytest);
    }

    #[test]
    fn auto_detect_jest() {
        let output = "Tests:  3 passed, 3 total\nTime:  1.2 s\n";
        let result = parse_test_output(output).expect("should detect jest");
        assert_eq!(result.framework, TestFramework::Jest);
    }

    #[test]
    fn auto_detect_go() {
        let output = "--- PASS: TestHello (0.00s)\nok  \texample.com/hello\t0.001s\n";
        let result = parse_test_output(output).expect("should detect go");
        assert_eq!(result.framework, TestFramework::GoTest);
    }

    #[test]
    fn auto_detect_unknown() {
        let result = parse_test_output("some random output");
        assert!(result.is_none());
    }

    // -- Framework Display ------------------------------------------------

    #[test]
    fn framework_display() {
        assert_eq!(TestFramework::CargoTest.to_string(), "cargo-test");
        assert_eq!(TestFramework::Pytest.to_string(), "pytest");
        assert_eq!(TestFramework::Jest.to_string(), "jest");
        assert_eq!(TestFramework::GoTest.to_string(), "go-test");
    }

    #[test]
    fn framework_as_str() {
        assert_eq!(TestFramework::CargoTest.as_str(), "cargo-test");
        assert_eq!(TestFramework::Pytest.as_str(), "pytest");
        assert_eq!(TestFramework::Jest.as_str(), "jest");
        assert_eq!(TestFramework::GoTest.as_str(), "go-test");
    }
}

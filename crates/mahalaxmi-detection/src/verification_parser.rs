//! Parser for `VERIFICATION:` lines in worker terminal output (REQ-002).
//!
//! Workers are instructed to emit structured self-verification lines before
//! signalling `TASK COMPLETE`.  The orchestration engine reads these lines and
//! blocks completion when any criterion is marked FAIL.
//!
//! Accepted format (either form):
//! ```text
//! VERIFICATION: [0] criterion text → PASS: supporting evidence
//! VERIFICATION: criterion text → FAIL: reason why it failed
//! ```

use regex::Regex;

/// A single parsed `VERIFICATION:` line from worker terminal output.
#[derive(Debug, Clone)]
pub struct VerificationLineResult {
    /// Optional index from `[N]` prefix.
    pub index: u32,
    /// The criterion being verified.
    pub criterion: String,
    /// `true` if the line recorded PASS, `false` if FAIL.
    pub passed: bool,
    /// Supporting evidence or failure reason.
    pub evidence: String,
}

/// Parse all `VERIFICATION:` lines from worker terminal output.
///
/// Lines that do not match the expected format are silently skipped so that
/// noisy terminal output does not cause false positives.
pub fn parse_verification_lines(output: &str) -> Vec<VerificationLineResult> {
    // Compile once. The pattern matches both `→` (U+2192) and `->` separators.
    let re = Regex::new(
        r"VERIFICATION:\s*(?:\[(\d+)\])?\s*([^→>-]+?)(?:→|->)\s*(PASS|FAIL):\s*(.*)",
    )
    .expect("verification_parser regex is always valid");

    output
        .lines()
        .filter_map(|line| {
            re.captures(line).map(|caps| VerificationLineResult {
                index: caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0),
                criterion: caps[2].trim().to_string(),
                passed: &caps[3] == "PASS",
                evidence: caps[4].trim().to_string(),
            })
        })
        .collect()
}

/// Return the subset of `VERIFICATION:` lines that recorded `FAIL`, or `None`
/// if either there are no such lines or no verification lines at all.
///
/// Returning `None` instead of an empty `Vec` lets callers distinguish
/// "no VERIFICATION output" from "all criteria passed".
pub fn has_verification_failures(output: &str) -> Option<Vec<VerificationLineResult>> {
    let results = parse_verification_lines(output);
    if results.is_empty() {
        return None;
    }
    let failures: Vec<_> = results.into_iter().filter(|r| !r.passed).collect();
    if failures.is_empty() {
        None
    } else {
        Some(failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pass_line() {
        let output = "VERIFICATION: [0] function exists → PASS: found in AuthService.ts";
        let results = parse_verification_lines(output);
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].criterion, "function exists");
        assert_eq!(results[0].evidence, "found in AuthService.ts");
    }

    #[test]
    fn parses_fail_line() {
        let output = "VERIFICATION: [1] tests pass → FAIL: 3 test failures";
        let results = parse_verification_lines(output);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert_eq!(results[0].evidence, "3 test failures");
    }

    #[test]
    fn parses_without_index() {
        let output = "VERIFICATION: endpoint returns 401 → PASS: verified with curl";
        let results = parse_verification_lines(output);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].index, 0);
        assert!(results[0].passed);
    }

    #[test]
    fn mixed_pass_fail_returns_only_failures() {
        let output = "VERIFICATION: [0] crit A → PASS: ok\nVERIFICATION: [1] crit B → FAIL: broken";
        let failures = has_verification_failures(output);
        assert!(failures.is_some());
        let failures = failures.unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].criterion, "crit B");
    }

    #[test]
    fn all_pass_returns_none() {
        let output = "VERIFICATION: [0] crit A → PASS: ok\nVERIFICATION: [1] crit B → PASS: ok";
        assert!(has_verification_failures(output).is_none());
    }

    #[test]
    fn no_verification_lines_returns_none() {
        let output = "Task complete\nNo verification output here";
        assert!(has_verification_failures(output).is_none());
        assert!(parse_verification_lines(output).is_empty());
    }

    #[test]
    fn arrow_ascii_alternative() {
        let output = "VERIFICATION: [0] thing done -> PASS: evidence here";
        let results = parse_verification_lines(output);
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }
}

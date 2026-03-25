/// Scan a batch's combined diff output for error patterns.
///
/// Returns `true` if any error patterns are detected in added lines,
/// `false` if the diff appears clean.
///
/// Only examines lines beginning with `+` (added lines). Lines beginning
/// with `+++` (file headers) are excluded to avoid false positives.
pub fn diff_scan_finds_errors(diff: &str) -> bool {
    for line in diff.lines() {
        if line.starts_with("+++") {
            continue;
        }
        if let Some(content) = line.strip_prefix('+') {
            for pattern in ERROR_PATTERNS {
                if content.contains(pattern) {
                    return true;
                }
            }
        }
    }
    false
}

/// Scan multiple diffs and return `true` if any one of them contains error patterns.
///
/// Useful for batch analysis where a single failing diff should escalate consensus.
pub fn any_diff_has_errors(diffs: &[&str]) -> bool {
    diffs.iter().any(|d| diff_scan_finds_errors(d))
}

/// The specific patterns that trigger an error detection.
/// Exported for testing.
pub const ERROR_PATTERNS: &[&str] = &[
    "error[E",
    "error: aborting",
    "error: could not compile",
    "Build FAILED",
    "test result: FAILED",
    "failures:",
    "panicked at",
    "stack backtrace:",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_diff_returns_false() {
        let diff = "+fn main() {}\n+    let x = 42;\n+}\n";
        assert!(!diff_scan_finds_errors(diff));
    }

    #[test]
    fn test_compile_error_in_added_line() {
        let diff = "+error[E0001]: cannot find value `x` in this scope\n";
        assert!(diff_scan_finds_errors(diff));
    }

    #[test]
    fn test_removed_error_line_ignored() {
        let diff = "-error[E0001]: cannot find value `x` in this scope\n";
        assert!(!diff_scan_finds_errors(diff));
    }

    #[test]
    fn test_file_header_ignored() {
        let diff = "+++ b/src/file.rs\n+fn main() {}\n";
        assert!(!diff_scan_finds_errors(diff));
    }

    #[test]
    fn test_panic_trace_detected() {
        let diff = "+panicked at 'index out of bounds: the len is 2 but the index is 5'\n";
        assert!(diff_scan_finds_errors(diff));
    }

    #[test]
    fn test_test_failure_detected() {
        let diff = "+test result: FAILED. 1 passed; 2 failed\n";
        assert!(diff_scan_finds_errors(diff));
    }

    #[test]
    fn test_empty_diff() {
        assert!(!diff_scan_finds_errors(""));
    }

    #[test]
    fn test_any_diff_has_errors_returns_true_when_one_fails() {
        let result = any_diff_has_errors(&["clean diff", "+error[E0308]: mismatched types"]);
        assert!(result);
    }

    #[test]
    fn test_any_diff_has_errors_returns_false_when_all_clean() {
        let result = any_diff_has_errors(&["+fn foo() {}", "+let x = 1;"]);
        assert!(!result);
    }

    #[test]
    fn test_error_e0308_detected() {
        let diff = "+error[E0308]: mismatched types\n";
        assert!(diff_scan_finds_errors(diff));
    }
}

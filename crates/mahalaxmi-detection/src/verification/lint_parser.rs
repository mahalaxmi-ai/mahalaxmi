//! Lint output parser for multiple lint tools.
//!
//! Parses structured and semi-structured output from popular lint tools
//! (Clippy, ESLint, Pylint, golint/staticcheck) into a normalized
//! [`LintResult`] representation. Supports auto-detection of the lint tool
//! from raw terminal output.

use std::fmt;

use regex::Regex;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Supported lint tools whose output can be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LintTool {
    /// Rust's Clippy linter.
    Clippy,
    /// JavaScript/TypeScript ESLint.
    Eslint,
    /// Python Pylint.
    Pylint,
    /// Go's golint / staticcheck.
    Golint,
}

impl LintTool {
    /// Return a lowercase string identifier for this tool.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clippy => "clippy",
            Self::Eslint => "eslint",
            Self::Pylint => "pylint",
            Self::Golint => "golint",
        }
    }
}

impl fmt::Display for LintTool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Severity level for a lint diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LintSeverity {
    /// Hint-level diagnostic (lowest severity).
    Hint,
    /// Informational diagnostic.
    Info,
    /// Warning-level diagnostic.
    Warning,
    /// Error-level diagnostic (highest severity).
    Error,
}

impl LintSeverity {
    /// Return a lowercase string identifier for this severity.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hint => "hint",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for LintSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single lint issue (diagnostic) reported by a lint tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintIssue {
    /// Severity of this diagnostic.
    pub severity: LintSeverity,
    /// Human-readable diagnostic message.
    pub message: String,
    /// Path to the source file that triggered the diagnostic.
    pub file_path: String,
    /// 1-based line number in the source file.
    pub line: usize,
    /// Optional 1-based column number.
    pub column: Option<usize>,
    /// Optional lint rule identifier (e.g. `clippy::needless_return`).
    pub rule: Option<String>,
}

/// Aggregated result of parsing lint tool output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintResult {
    /// Which lint tool produced this result.
    pub tool: LintTool,
    /// Total number of issues found.
    pub total_issues: usize,
    /// Number of error-level issues.
    pub errors: usize,
    /// Number of warning-level issues.
    pub warnings: usize,
    /// Individual issues parsed from the output.
    pub issues: Vec<LintIssue>,
}

impl LintResult {
    /// Returns `true` when the lint run found no errors and no warnings.
    pub fn is_clean(&self) -> bool {
        self.errors == 0 && self.warnings == 0
    }

    /// Returns `true` when the lint run found at least one error.
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    /// Produce a short human-readable summary such as `"3 errors, 7 warnings"`.
    pub fn summary(&self) -> String {
        format!("{} errors, {} warnings", self.errors, self.warnings)
    }
}

// ---------------------------------------------------------------------------
// Auto-detection entry point
// ---------------------------------------------------------------------------

/// Auto-detect the lint tool from raw terminal output and parse results.
///
/// Detection heuristics (tried in order):
/// 1. Clippy -- presence of `warning[clippy::` or `error[E` with `-->` arrows
/// 2. ESLint -- `problems` summary or eslint-style indented severity lines
/// 3. Pylint -- `[CRWEF]\d{4}` diagnostic codes
/// 4. golint -- `.go:\d+:\d+:` location patterns without other tool markers
///
/// Returns `None` when the output does not match any known tool.
pub fn parse_lint_output(output: &str) -> Option<LintResult> {
    // 1. Clippy detection
    if let (Some(clippy_marker), Some(arrow_marker)) = (
        Regex::new(r"warning\[clippy::|error\[E\d|= note: `#\[warn\(").ok(),
        Regex::new(r"-->").ok(),
    ) {
        if clippy_marker.is_match(output) && arrow_marker.is_match(output) {
            return parse_clippy(output);
        }
    }

    // 2. ESLint detection
    if let (Some(eslint_summary), Some(eslint_lines)) = (
        Regex::new(r"\d+ problems? \(\d+ errors?, \d+ warnings?\)").ok(),
        Regex::new(r"(?m)^\s+\d+:\d+\s+(error|warning)\s+").ok(),
    ) {
        if eslint_summary.is_match(output) || eslint_lines.is_match(output) {
            return parse_eslint(output);
        }
    }

    // 3. Pylint detection
    if let Ok(pylint_code) = Regex::new(r"[CRWEF]\d{4}") {
        if pylint_code.is_match(output) {
            return parse_pylint(output);
        }
    }

    // 4. golint detection
    if let Ok(go_loc) = Regex::new(r"\.go:\d+:\d+:") {
        if go_loc.is_match(output) {
            return parse_golint(output);
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Clippy parser
// ---------------------------------------------------------------------------

/// Parse Rust Clippy output into a [`LintResult`].
///
/// Recognises diagnostic blocks of the form:
/// ```text
/// warning[clippy::needless_return]: ...message...
///   --> src/main.rs:42:5
/// ```
///
/// Summary lines (e.g. `generated N warning(s)`) and build errors
/// (`could not compile`) are skipped.
pub fn parse_clippy(output: &str) -> Option<LintResult> {
    // Match a diagnostic header: `warning: msg` or `warning[rule]: msg` or `error[E0123]: msg`
    let diag_re = Regex::new(r"(?m)^(warning|error)(?:\[(.+?)\])?: (.+)$").ok()?;
    // Match a location line: `  --> path:line:col`
    let loc_re = Regex::new(r"(?m)^\s+-->\s+(.+?):(\d+):(\d+)").ok()?;
    // Lines to skip
    let skip_re =
        Regex::new(r"(?i)generated \d+ warning|could not compile|aborting due to").ok()?;

    let mut issues: Vec<LintIssue> = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    let mut idx = 0;
    while idx < lines.len() {
        let line = lines[idx];

        // Try to match a diagnostic header
        if let Some(caps) = diag_re.captures(line) {
            let severity_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let rule = caps.get(2).map(|m| m.as_str().to_string());
            let message = caps
                .get(3)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            // Skip summary / build error lines
            if skip_re.is_match(line) {
                idx += 1;
                continue;
            }

            let severity = match severity_str {
                "error" => LintSeverity::Error,
                _ => LintSeverity::Warning,
            };

            // Look ahead for the location line (within the next few lines)
            let mut found_loc = false;
            let lookahead_limit = (idx + 6).min(lines.len());
            for (j, item) in lines.iter().enumerate().take(lookahead_limit).skip(idx + 1) {
                if let Some(loc_caps) = loc_re.captures(item) {
                    let file_path = loc_caps
                        .get(1)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    let line_num: usize = loc_caps
                        .get(2)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(0);
                    let col: usize = loc_caps
                        .get(3)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(0);

                    issues.push(LintIssue {
                        severity,
                        message: message.clone(),
                        file_path,
                        line: line_num,
                        column: Some(col),
                        rule: rule.clone(),
                    });
                    found_loc = true;
                    idx = j + 1;
                    break;
                }
            }

            if !found_loc {
                idx += 1;
            }
        } else {
            idx += 1;
        }
    }

    let errors = issues
        .iter()
        .filter(|i| i.severity == LintSeverity::Error)
        .count();
    let warnings = issues
        .iter()
        .filter(|i| i.severity == LintSeverity::Warning)
        .count();

    Some(LintResult {
        tool: LintTool::Clippy,
        total_issues: issues.len(),
        errors,
        warnings,
        issues,
    })
}

// ---------------------------------------------------------------------------
// ESLint parser
// ---------------------------------------------------------------------------

/// Parse ESLint output into a [`LintResult`].
///
/// ESLint outputs per-file sections:
/// ```text
/// /path/to/file.js
///   12:5  error  Unexpected var  no-var
///   20:1  warning  Missing semicolon  semi
///
/// 2 problems (1 error, 1 warning)
/// ```
pub fn parse_eslint(output: &str) -> Option<LintResult> {
    // Issue line:  line:col  severity  message  rule
    let issue_re =
        Regex::new(r"(?m)^\s+(\d+):(\d+)\s+(error|warning)\s+(.+?)\s{2,}(\S+)\s*$").ok()?;
    // Summary line
    let summary_re = Regex::new(r"(?m)(\d+) problems? \((\d+) errors?, (\d+) warnings?\)").ok()?;
    // File path line: non-indented, non-empty
    let file_re = Regex::new(r"(?m)^(\S.+)$").ok()?;

    let mut issues: Vec<LintIssue> = Vec::new();
    let mut current_file = String::new();

    for line in output.lines() {
        // Check if this is an issue line first (indented)
        if let Some(caps) = issue_re.captures(line) {
            let line_num: usize = caps
                .get(1)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            let col: usize = caps
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            let severity_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let message = caps
                .get(4)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            let rule = caps.get(5).map(|m| m.as_str().to_string());

            let severity = match severity_str {
                "error" => LintSeverity::Error,
                _ => LintSeverity::Warning,
            };

            issues.push(LintIssue {
                severity,
                message,
                file_path: current_file.clone(),
                line: line_num,
                column: Some(col),
                rule,
            });
        } else if !line.trim().is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
            // Might be a file path -- but skip the summary line
            if summary_re.is_match(line) {
                continue;
            }
            // Also skip lines that are clearly not file paths
            if let Some(m) = file_re.captures(line) {
                let candidate = m.get(1).map(|c| c.as_str().trim()).unwrap_or("");
                if !candidate.is_empty() {
                    current_file = candidate.to_string();
                }
            }
        }
    }

    let errors = issues
        .iter()
        .filter(|i| i.severity == LintSeverity::Error)
        .count();
    let warnings = issues
        .iter()
        .filter(|i| i.severity == LintSeverity::Warning)
        .count();

    Some(LintResult {
        tool: LintTool::Eslint,
        total_issues: issues.len(),
        errors,
        warnings,
        issues,
    })
}

// ---------------------------------------------------------------------------
// Pylint parser
// ---------------------------------------------------------------------------

/// Parse Pylint output into a [`LintResult`].
///
/// Pylint outputs lines like:
/// ```text
/// mymodule/foo.py:10:0: C0114: Missing module docstring (missing-module-docstring)
/// mymodule/bar.py:25:4: E0602: Undefined variable 'x' (undefined-variable)
/// ```
///
/// Code-letter mapping:
/// - **C** (convention) -> [`LintSeverity::Info`]
/// - **R** (refactor)   -> [`LintSeverity::Info`]
/// - **W** (warning)    -> [`LintSeverity::Warning`]
/// - **E** (error)      -> [`LintSeverity::Error`]
/// - **F** (fatal)      -> [`LintSeverity::Error`]
pub fn parse_pylint(output: &str) -> Option<LintResult> {
    let line_re =
        Regex::new(r"(?m)^(.+?):(\d+):(\d+): ([CRWEF]\d{4}): (.+?) \((.+?)\)\s*$").ok()?;

    let mut issues: Vec<LintIssue> = Vec::new();

    for caps in line_re.captures_iter(output) {
        let file_path = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let line_num: usize = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let col: usize = caps
            .get(3)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let code = caps
            .get(4)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let message = caps
            .get(5)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let rule_name = caps.get(6).map(|m| m.as_str().to_string());

        let severity = match code.chars().next() {
            Some('C') | Some('R') => LintSeverity::Info,
            Some('W') => LintSeverity::Warning,
            Some('E') | Some('F') => LintSeverity::Error,
            _ => LintSeverity::Warning,
        };

        issues.push(LintIssue {
            severity,
            message,
            file_path,
            line: line_num,
            column: Some(col),
            rule: rule_name,
        });
    }

    let errors = issues
        .iter()
        .filter(|i| i.severity == LintSeverity::Error)
        .count();
    let warnings = issues
        .iter()
        .filter(|i| i.severity == LintSeverity::Warning)
        .count();

    Some(LintResult {
        tool: LintTool::Pylint,
        total_issues: issues.len(),
        errors,
        warnings,
        issues,
    })
}

// ---------------------------------------------------------------------------
// golint / staticcheck parser
// ---------------------------------------------------------------------------

/// Parse golint or staticcheck output into a [`LintResult`].
///
/// Lines are expected in the form:
/// ```text
/// main.go:15:2: exported function Foo should have comment or be unexported
/// main.go:20:5: some issue (SA1000)
/// ```
///
/// All issues default to [`LintSeverity::Warning`].
pub fn parse_golint(output: &str) -> Option<LintResult> {
    // With optional trailing check-id in parens
    let line_re = Regex::new(r"(?m)^(.+?\.go):(\d+):(\d+):\s+(.+?)(?:\s+\((\S+)\))?\s*$").ok()?;

    let mut issues: Vec<LintIssue> = Vec::new();

    for caps in line_re.captures_iter(output) {
        let file_path = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let line_num: usize = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let col: usize = caps
            .get(3)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let message = caps
            .get(4)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let rule = caps.get(5).map(|m| m.as_str().to_string());

        issues.push(LintIssue {
            severity: LintSeverity::Warning,
            message,
            file_path,
            line: line_num,
            column: Some(col),
            rule,
        });
    }

    let warnings = issues.len();

    Some(LintResult {
        tool: LintTool::Golint,
        total_issues: issues.len(),
        errors: 0,
        warnings,
        issues,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- LintTool --

    #[test]
    fn lint_tool_as_str() {
        assert_eq!(LintTool::Clippy.as_str(), "clippy");
        assert_eq!(LintTool::Eslint.as_str(), "eslint");
        assert_eq!(LintTool::Pylint.as_str(), "pylint");
        assert_eq!(LintTool::Golint.as_str(), "golint");
    }

    #[test]
    fn lint_tool_display() {
        assert_eq!(format!("{}", LintTool::Clippy), "clippy");
        assert_eq!(format!("{}", LintTool::Eslint), "eslint");
    }

    // -- LintSeverity --

    #[test]
    fn lint_severity_as_str() {
        assert_eq!(LintSeverity::Hint.as_str(), "hint");
        assert_eq!(LintSeverity::Info.as_str(), "info");
        assert_eq!(LintSeverity::Warning.as_str(), "warning");
        assert_eq!(LintSeverity::Error.as_str(), "error");
    }

    #[test]
    fn lint_severity_display() {
        assert_eq!(format!("{}", LintSeverity::Warning), "warning");
        assert_eq!(format!("{}", LintSeverity::Error), "error");
    }

    #[test]
    fn lint_severity_ordering() {
        assert!(LintSeverity::Hint < LintSeverity::Info);
        assert!(LintSeverity::Info < LintSeverity::Warning);
        assert!(LintSeverity::Warning < LintSeverity::Error);
    }

    // -- LintResult --

    #[test]
    fn lint_result_is_clean() {
        let result = LintResult {
            tool: LintTool::Clippy,
            total_issues: 0,
            errors: 0,
            warnings: 0,
            issues: vec![],
        };
        assert!(result.is_clean());
    }

    #[test]
    fn lint_result_not_clean_with_warnings() {
        let result = LintResult {
            tool: LintTool::Clippy,
            total_issues: 1,
            errors: 0,
            warnings: 1,
            issues: vec![],
        };
        assert!(!result.is_clean());
    }

    #[test]
    fn lint_result_has_errors() {
        let result = LintResult {
            tool: LintTool::Clippy,
            total_issues: 2,
            errors: 1,
            warnings: 1,
            issues: vec![],
        };
        assert!(result.has_errors());
    }

    #[test]
    fn lint_result_summary() {
        let result = LintResult {
            tool: LintTool::Eslint,
            total_issues: 10,
            errors: 3,
            warnings: 7,
            issues: vec![],
        };
        assert_eq!(result.summary(), "3 errors, 7 warnings");
    }

    // -- Clippy parser --

    #[test]
    fn parse_clippy_basic() {
        let output = "\
warning[clippy::needless_return]: unneeded `return` statement
  --> src/lib.rs:42:5
   |
42 |     return value;
   |     ^^^^^^^^^^^^^ help: remove `return`
   |
   = note: `#[warn(clippy::needless_return)]` on by default

warning: `my_crate` (lib) generated 1 warning
";
        let result = parse_clippy(output).expect("should parse");
        assert_eq!(result.tool, LintTool::Clippy);
        assert_eq!(result.total_issues, 1);
        assert_eq!(result.warnings, 1);
        assert_eq!(result.errors, 0);
        assert_eq!(result.issues[0].file_path, "src/lib.rs");
        assert_eq!(result.issues[0].line, 42);
        assert_eq!(result.issues[0].column, Some(5));
        assert_eq!(
            result.issues[0].rule.as_deref(),
            Some("clippy::needless_return")
        );
    }

    #[test]
    fn parse_clippy_error_and_warning() {
        let output = "\
error[E0382]: use of moved value: `x`
  --> src/main.rs:10:15
   |
10 |     let y = x;
   |               ^ value used here after move
   |

warning[clippy::unused_variable]: unused variable: `z`
  --> src/main.rs:15:9
   |
15 |     let z = 5;
   |         ^ help: if this is intentional, prefix it with an underscore: `_z`

error: aborting due to 1 previous error; 1 warning emitted
";
        let result = parse_clippy(output).expect("should parse");
        assert_eq!(result.errors, 1);
        assert_eq!(result.warnings, 1);
        assert_eq!(result.total_issues, 2);
    }

    #[test]
    fn parse_clippy_skips_could_not_compile() {
        let output = "\
warning[clippy::dead_code]: function is never used
  --> src/lib.rs:5:4
   |
5  | fn unused() {}
   |    ^^^^^^

error: could not compile `my_crate` (lib) due to previous error
";
        let result = parse_clippy(output).expect("should parse");
        // The "could not compile" line should not generate an issue
        assert_eq!(result.total_issues, 1);
        assert_eq!(result.warnings, 1);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn parse_clippy_empty_output() {
        let result = parse_clippy("").expect("should return empty result");
        assert!(result.is_clean());
        assert!(result.issues.is_empty());
    }

    // -- ESLint parser --

    #[test]
    fn parse_eslint_basic() {
        let output = "\
/home/user/project/src/index.js
   3:5   error    Unexpected var, use let or const instead  no-var
  10:1   warning  Missing semicolon                         semi
  15:10  error    'foo' is not defined                      no-undef

3 problems (2 errors, 1 warning)
";
        let result = parse_eslint(output).expect("should parse");
        assert_eq!(result.tool, LintTool::Eslint);
        assert_eq!(result.total_issues, 3);
        assert_eq!(result.errors, 2);
        assert_eq!(result.warnings, 1);
        assert_eq!(
            result.issues[0].file_path,
            "/home/user/project/src/index.js"
        );
        assert_eq!(result.issues[0].line, 3);
        assert_eq!(result.issues[0].column, Some(5));
        assert_eq!(result.issues[0].rule.as_deref(), Some("no-var"));
    }

    #[test]
    fn parse_eslint_multiple_files() {
        let output = "\
/home/user/src/a.js
  1:1  error  Unexpected var  no-var

/home/user/src/b.js
  5:3  warning  Missing return  consistent-return

2 problems (1 error, 1 warning)
";
        let result = parse_eslint(output).expect("should parse");
        assert_eq!(result.total_issues, 2);
        assert_eq!(result.issues[0].file_path, "/home/user/src/a.js");
        assert_eq!(result.issues[1].file_path, "/home/user/src/b.js");
    }

    #[test]
    fn parse_eslint_empty_output() {
        let result = parse_eslint("").expect("should return empty result");
        assert!(result.is_clean());
    }

    // -- Pylint parser --

    #[test]
    fn parse_pylint_basic() {
        let output = "\
mymodule/foo.py:10:0: C0114: Missing module docstring (missing-module-docstring)
mymodule/bar.py:25:4: E0602: Undefined variable 'x' (undefined-variable)
mymodule/baz.py:30:0: W0611: Unused import os (unused-import)
mymodule/qux.py:1:0: R0903: Too few public methods (0/2) (too-few-public-methods)
mymodule/fail.py:1:0: F0001: No module named nonexistent (fatal)
";
        let result = parse_pylint(output).expect("should parse");
        assert_eq!(result.tool, LintTool::Pylint);
        assert_eq!(result.total_issues, 5);
        // C0114 -> Info, E0602 -> Error, W0611 -> Warning, R0903 -> Info, F0001 -> Error
        assert_eq!(result.errors, 2); // E + F
        assert_eq!(result.warnings, 1); // W

        assert_eq!(result.issues[0].severity, LintSeverity::Info);
        assert_eq!(
            result.issues[0].rule.as_deref(),
            Some("missing-module-docstring")
        );
        assert_eq!(result.issues[1].severity, LintSeverity::Error);
        assert_eq!(result.issues[1].file_path, "mymodule/bar.py");
        assert_eq!(result.issues[2].severity, LintSeverity::Warning);
        assert_eq!(result.issues[3].severity, LintSeverity::Info);
        assert_eq!(result.issues[4].severity, LintSeverity::Error);
    }

    #[test]
    fn parse_pylint_empty_output() {
        let result = parse_pylint("").expect("should return empty result");
        assert!(result.is_clean());
    }

    // -- golint parser --

    #[test]
    fn parse_golint_basic() {
        let output = "\
main.go:15:2: exported function Foo should have comment or be unexported
cmd/server.go:20:5: error strings should not be capitalized (SA1000)
";
        let result = parse_golint(output).expect("should parse");
        assert_eq!(result.tool, LintTool::Golint);
        assert_eq!(result.total_issues, 2);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 2);
        assert_eq!(result.issues[0].file_path, "main.go");
        assert_eq!(result.issues[0].line, 15);
        assert_eq!(result.issues[0].column, Some(2));
        assert!(result.issues[0].rule.is_none());
        assert_eq!(result.issues[1].rule.as_deref(), Some("SA1000"));
    }

    #[test]
    fn parse_golint_empty_output() {
        let result = parse_golint("").expect("should return empty result");
        assert!(result.is_clean());
    }

    // -- Auto-detection --

    #[test]
    fn auto_detect_clippy() {
        let output = "\
warning[clippy::needless_return]: unneeded `return` statement
  --> src/lib.rs:10:5
   |
   = note: `#[warn(clippy::needless_return)]` on by default
";
        let result = parse_lint_output(output).expect("should auto-detect clippy");
        assert_eq!(result.tool, LintTool::Clippy);
    }

    #[test]
    fn auto_detect_eslint() {
        let output = "\
/home/user/src/index.js
  3:5  error  Unexpected var  no-var

1 problem (1 error, 0 warnings)
";
        let result = parse_lint_output(output).expect("should auto-detect eslint");
        assert_eq!(result.tool, LintTool::Eslint);
    }

    #[test]
    fn auto_detect_pylint() {
        let output =
            "mymodule/foo.py:10:0: C0114: Missing module docstring (missing-module-docstring)\n";
        let result = parse_lint_output(output).expect("should auto-detect pylint");
        assert_eq!(result.tool, LintTool::Pylint);
    }

    #[test]
    fn auto_detect_golint() {
        let output = "main.go:15:2: exported function Foo should have comment\n";
        let result = parse_lint_output(output).expect("should auto-detect golint");
        assert_eq!(result.tool, LintTool::Golint);
    }

    #[test]
    fn auto_detect_unknown() {
        let result = parse_lint_output("everything is fine, no lint issues here");
        assert!(result.is_none());
    }

    // -- Serialization roundtrip --

    #[test]
    fn lint_issue_serde_roundtrip() {
        let issue = LintIssue {
            severity: LintSeverity::Error,
            message: "test error".to_string(),
            file_path: "src/main.rs".to_string(),
            line: 42,
            column: Some(5),
            rule: Some("E0382".to_string()),
        };
        let json = serde_json::to_string(&issue).expect("serialize");
        let deserialized: LintIssue = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(issue, deserialized);
    }

    #[test]
    fn lint_result_serde_roundtrip() {
        let result = LintResult {
            tool: LintTool::Clippy,
            total_issues: 1,
            errors: 1,
            warnings: 0,
            issues: vec![LintIssue {
                severity: LintSeverity::Error,
                message: "moved value".to_string(),
                file_path: "src/lib.rs".to_string(),
                line: 10,
                column: None,
                rule: None,
            }],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: LintResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }
}

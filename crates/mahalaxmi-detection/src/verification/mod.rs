//! Worker self-verification output parsers.
//!
//! Parsers for extracting structured results from test runner and
//! lint tool output across multiple frameworks and languages.

pub mod lint_parser;
pub mod test_parser;

pub use lint_parser::{LintIssue, LintResult, LintSeverity, LintTool};
pub use test_parser::{TestFailure, TestFramework, TestResult};

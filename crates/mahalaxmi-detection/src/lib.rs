//! State detection and auto-response for Mahalaxmi.
//!
//! Pattern-matching rules for detecting terminal states, completion markers,
//! error patterns, and provider-specific prompts from PTY output streams.

pub mod errors;
pub mod matcher;
pub mod pattern;
pub mod result;
pub mod rule;
pub mod rule_set;
pub mod verification;
pub mod verification_parser;

pub use errors::analysis::ErrorPatternAnalysis;
pub use errors::cluster::ErrorCluster;
pub use errors::hypothesis::RootCauseHypothesis;
pub use errors::recurring::RecurringError;
pub use matcher::RuleMatcher;
pub use pattern::{CompiledPattern, DetectionPattern};
pub use result::DetectionResult;
pub use rule::DetectionRule;
pub use rule_set::BuiltinRuleSets;
pub use verification::{
    LintIssue, LintResult, LintSeverity, LintTool, TestFailure, TestFramework, TestResult,
};
pub use verification_parser::{
    has_verification_failures, parse_verification_lines, VerificationLineResult,
};

pub use mahalaxmi_core::config::MahalaxmiConfig;
pub use mahalaxmi_core::error::MahalaxmiError;
pub use mahalaxmi_core::i18n::locale::SupportedLocale;
pub use mahalaxmi_core::i18n::I18nService;
pub use mahalaxmi_core::MahalaxmiResult;

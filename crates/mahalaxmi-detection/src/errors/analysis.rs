use mahalaxmi_core::types::RootCauseCategory;
use serde::{Deserialize, Serialize};

use crate::errors::cluster::ErrorCluster;
use crate::errors::hypothesis::RootCauseHypothesis;
use crate::errors::recurring::RecurringError;

/// The result of analyzing error patterns from worker execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPatternAnalysis {
    /// Individual recurring error patterns.
    pub recurring_errors: Vec<RecurringError>,
    /// Clusters of related errors.
    pub clusters: Vec<ErrorCluster>,
    /// Root cause hypotheses generated from the analysis.
    pub hypotheses: Vec<RootCauseHypothesis>,
}

impl ErrorPatternAnalysis {
    /// Create an empty analysis.
    pub fn empty() -> Self {
        Self {
            recurring_errors: Vec::new(),
            clusters: Vec::new(),
            hypotheses: Vec::new(),
        }
    }

    /// Add a recurring error to the analysis.
    pub fn with_recurring_error(mut self, error: RecurringError) -> Self {
        self.recurring_errors.push(error);
        self
    }

    /// Add an error cluster to the analysis.
    pub fn with_cluster(mut self, cluster: ErrorCluster) -> Self {
        self.clusters.push(cluster);
        self
    }

    /// Add a root cause hypothesis.
    pub fn with_hypothesis(mut self, hypothesis: RootCauseHypothesis) -> Self {
        self.hypotheses.push(hypothesis);
        self
    }

    /// Returns true if any recurring errors have been recorded.
    pub fn has_recurring_errors(&self) -> bool {
        !self.recurring_errors.is_empty()
    }

    /// Total count of all recurring error occurrences.
    pub fn total_error_occurrences(&self) -> u32 {
        self.recurring_errors
            .iter()
            .map(|e| e.occurrence_count)
            .sum()
    }
}

/// Normalize an error message for grouping.
///
/// Strips variable parts (file paths, line numbers, timestamps) to group
/// similar errors together.
pub fn normalize_error_message(message: &str) -> String {
    let mut normalized = message.to_lowercase();

    // Replace file paths (absolute and relative)
    let path_re = regex::Regex::new(r"(/[\w./-]+|[A-Z]:\\[\w.\\-]+)")
        .unwrap_or_else(|_| regex::Regex::new(r"NEVER_MATCH").expect("fallback regex"));
    normalized = path_re.replace_all(&normalized, "<path>").to_string();

    // Replace line numbers
    let line_re = regex::Regex::new(r"line\s+\d+")
        .unwrap_or_else(|_| regex::Regex::new(r"NEVER_MATCH").expect("fallback regex"));
    normalized = line_re.replace_all(&normalized, "line <N>").to_string();

    // Replace numeric values
    let num_re = regex::Regex::new(r"\b\d{4,}\b")
        .unwrap_or_else(|_| regex::Regex::new(r"NEVER_MATCH").expect("fallback regex"));
    normalized = num_re.replace_all(&normalized, "<NUM>").to_string();

    // Trim and collapse whitespace
    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Categorize an error message into a root cause category.
///
/// Uses keyword matching to assign an initial category. The hypothesis
/// engine may later refine this based on additional evidence.
pub fn categorize_error(message: &str) -> RootCauseCategory {
    let lower = message.to_lowercase();

    if lower.contains("authentication")
        || lower.contains("unauthorized")
        || lower.contains("api key")
        || lower.contains("credentials")
    {
        return RootCauseCategory::Authentication;
    }

    if lower.contains("network")
        || lower.contains("connection refused")
        || lower.contains("timeout")
        || lower.contains("dns")
    {
        if lower.contains("timeout") {
            return RootCauseCategory::Timeout;
        }
        return RootCauseCategory::Network;
    }

    if lower.contains("permission denied")
        || lower.contains("access denied")
        || lower.contains("forbidden")
    {
        return RootCauseCategory::Permission;
    }

    if lower.contains("file not found")
        || lower.contains("no such file")
        || lower.contains("enoent")
    {
        return RootCauseCategory::FileSystem;
    }

    if lower.contains("dependency")
        || lower.contains("module not found")
        || lower.contains("package")
        || lower.contains("version")
    {
        return RootCauseCategory::Dependency;
    }

    if lower.contains("syntax error")
        || lower.contains("parse error")
        || lower.contains("unexpected token")
    {
        return RootCauseCategory::Syntax;
    }

    if lower.contains("out of memory")
        || lower.contains("oom")
        || lower.contains("disk full")
        || lower.contains("no space")
    {
        return RootCauseCategory::Resource;
    }

    if lower.contains("rate limit") || lower.contains("too many requests") || lower.contains("429")
    {
        return RootCauseCategory::RateLimit;
    }

    if lower.contains("build failed")
        || lower.contains("compilation error")
        || lower.contains("cargo build")
    {
        return RootCauseCategory::Build;
    }

    if lower.contains("test failed") || lower.contains("assertion") || lower.contains("test error")
    {
        return RootCauseCategory::Test;
    }

    if lower.contains("merge conflict") || lower.contains("git") || lower.contains("rebase") {
        return RootCauseCategory::VersionControl;
    }

    if lower.contains("service unavailable") || lower.contains("503") || lower.contains("502") {
        return RootCauseCategory::ExternalService;
    }

    if lower.contains("config")
        || lower.contains("configuration")
        || lower.contains("invalid setting")
    {
        return RootCauseCategory::Configuration;
    }

    if lower.contains("runtime error") || lower.contains("panic") || lower.contains("segfault") {
        return RootCauseCategory::Runtime;
    }

    RootCauseCategory::Unknown
}

/// Analyze a collection of error messages to find patterns.
///
/// Groups errors by their normalized message, identifies recurring patterns,
/// and generates basic root cause hypotheses.
pub fn analyze_errors(messages: &[&str]) -> ErrorPatternAnalysis {
    let mut recurring_map: std::collections::HashMap<String, RecurringError> =
        std::collections::HashMap::new();

    for message in messages {
        let normalized = normalize_error_message(message);
        let category = categorize_error(message);

        recurring_map
            .entry(normalized.clone())
            .and_modify(|e| e.record_occurrence())
            .or_insert_with(|| RecurringError::new(&normalized).with_category(category));
    }

    let recurring_errors: Vec<RecurringError> = recurring_map.into_values().collect();

    let hypotheses: Vec<RootCauseHypothesis> = recurring_errors
        .iter()
        .filter(|e| e.occurrence_count >= 2)
        .map(|e| {
            RootCauseHypothesis::new(
                e.category,
                format!("Recurring {} error: {}", e.category, e.normalized_message),
            )
            .with_confidence(0.5 + (e.occurrence_count as f64 * 0.1).min(0.4))
            .with_evidence(format!("Occurred {} times", e.occurrence_count))
        })
        .collect();

    ErrorPatternAnalysis {
        recurring_errors,
        clusters: Vec::new(),
        hypotheses,
    }
}

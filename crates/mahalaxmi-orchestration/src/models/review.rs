//! Cross-provider review chain — types for post-completion code review.
//!
//! After a worker completes its task, the review chain optionally routes
//! the output to a *different* AI provider for review. This provides a
//! cross-validation layer: the reviewing provider can catch issues that
//! the generating provider missed.

use serde::{Deserialize, Serialize};

/// Result of a cross-provider review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// Task title that was reviewed.
    pub task_title: String,
    /// Provider that performed the review.
    pub reviewer_provider_id: String,
    /// Whether the review approved the output.
    pub approved: bool,
    /// Specific issues found during review.
    pub issues: Vec<ReviewIssue>,
    /// Suggestions for improvement (non-blocking).
    pub suggestions: Vec<String>,
    /// Duration of the review in milliseconds.
    pub duration_ms: u64,
}

impl ReviewResult {
    /// Count of critical issues (those that block approval).
    pub fn critical_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == ReviewSeverity::Critical)
            .count()
    }

    /// Count of warning issues (noted but don't block).
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == ReviewSeverity::Warning)
            .count()
    }

    /// Parse a ReviewResult from the review provider's JSON output.
    ///
    /// Attempts to extract a JSON object from the output text, handling
    /// cases where the provider wraps the JSON in markdown code fences
    /// or adds explanatory text around it.
    pub fn parse_from_output(
        output: &str,
        task_title: &str,
        reviewer_provider_id: &str,
        duration_ms: u64,
    ) -> Option<Self> {
        // Try to find JSON in the output
        let json_str = extract_json_block(output)?;

        #[derive(Deserialize)]
        struct RawReview {
            #[serde(default)]
            approved: bool,
            #[serde(default)]
            issues: Vec<RawIssue>,
            #[serde(default)]
            suggestions: Vec<String>,
        }

        #[derive(Deserialize)]
        struct RawIssue {
            #[serde(default)]
            severity: String,
            #[serde(default)]
            file_path: String,
            #[serde(default)]
            description: String,
        }

        let raw: RawReview = serde_json::from_str(json_str).ok()?;

        let issues = raw
            .issues
            .into_iter()
            .map(|i| ReviewIssue {
                severity: match i.severity.to_lowercase().as_str() {
                    "critical" => ReviewSeverity::Critical,
                    "warning" => ReviewSeverity::Warning,
                    _ => ReviewSeverity::Suggestion,
                },
                file_path: i.file_path,
                description: i.description,
            })
            .collect();

        Some(Self {
            task_title: task_title.to_owned(),
            reviewer_provider_id: reviewer_provider_id.to_owned(),
            approved: raw.approved,
            issues,
            suggestions: raw.suggestions,
            duration_ms,
        })
    }

    /// Format the review result as context for a worker retry prompt.
    pub fn to_retry_context(&self) -> String {
        let mut parts = Vec::new();
        parts.push("## Cross-Provider Review Feedback".to_owned());
        parts.push(String::new());

        if self.approved {
            parts.push("Review APPROVED with suggestions:".to_owned());
        } else {
            parts.push("Review REJECTED — fix the following issues:".to_owned());
        }
        parts.push(String::new());

        for issue in &self.issues {
            let severity = match issue.severity {
                ReviewSeverity::Critical => "CRITICAL",
                ReviewSeverity::Warning => "WARNING",
                ReviewSeverity::Suggestion => "SUGGESTION",
            };
            let file_note = if issue.file_path.is_empty() {
                String::new()
            } else {
                format!(" ({})", issue.file_path)
            };
            parts.push(format!("- [{severity}]{file_note}: {}", issue.description));
        }

        if !self.suggestions.is_empty() {
            parts.push(String::new());
            parts.push("Suggestions:".to_owned());
            for sug in &self.suggestions {
                parts.push(format!("- {sug}"));
            }
        }

        parts.join("\n")
    }
}

/// A specific issue found during review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// Severity level.
    pub severity: ReviewSeverity,
    /// File path related to the issue (empty if general).
    pub file_path: String,
    /// Description of the issue.
    pub description: String,
}

/// Severity level for a review issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSeverity {
    /// Blocks acceptance — must be fixed.
    Critical,
    /// Noted but does not block acceptance.
    Warning,
    /// Optional improvement suggestion.
    Suggestion,
}

impl std::fmt::Display for ReviewSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical"),
            Self::Warning => write!(f, "Warning"),
            Self::Suggestion => write!(f, "Suggestion"),
        }
    }
}

/// Extract a JSON block from text that may contain markdown fences or prose.
fn extract_json_block(text: &str) -> Option<&str> {
    // Try markdown code fence first: ```json ... ```
    if let Some(start) = text.find("```json") {
        let json_start = start + 7; // skip "```json"
        if let Some(end) = text[json_start..].find("```") {
            let block = text[json_start..json_start + end].trim();
            if !block.is_empty() {
                return Some(block);
            }
        }
    }

    // Try plain code fence: ``` ... ```
    if let Some(start) = text.find("```") {
        let fence_start = start + 3;
        if let Some(end) = text[fence_start..].find("```") {
            let block = text[fence_start..fence_start + end].trim();
            if block.starts_with('{') {
                return Some(block);
            }
        }
    }

    // Try bare JSON object
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                let block = &text[start..=end];
                return Some(block);
            }
        }
    }

    None
}

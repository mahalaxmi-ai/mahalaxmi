//! Review prompt builder — assembles the prompt for cross-provider code review.
//!
//! After a worker completes its task, the review chain sends the worker's output
//! to a different AI provider for review. This builder creates the constraint-style
//! prompt that instructs the reviewer to evaluate the work and produce a structured
//! JSON response.

use mahalaxmi_core::config::ContextFormat;

use super::builder::{format_section, resolve_format};

/// Configuration for building a review prompt.
pub struct ReviewPromptConfig {
    /// Task title being reviewed.
    pub task_title: String,
    /// Task description / acceptance criteria.
    pub task_description: String,
    /// Worker's terminal output (the work to review).
    pub worker_output: String,
    /// Files the task was expected to modify.
    pub affected_files: Vec<String>,
    /// Provider performing the review (for format selection).
    pub provider_id: String,
    /// Context format override.
    pub format: ContextFormat,
}

impl Default for ReviewPromptConfig {
    fn default() -> Self {
        Self {
            task_title: String::new(),
            task_description: String::new(),
            worker_output: String::new(),
            affected_files: Vec::new(),
            provider_id: String::new(),
            format: ContextFormat::Auto,
        }
    }
}

/// Builds the prompt sent to the reviewing AI provider.
pub struct ReviewPromptBuilder;

impl ReviewPromptBuilder {
    /// Build the full review prompt.
    pub fn build(config: &ReviewPromptConfig) -> String {
        let effective_format = resolve_format(config.format, &config.provider_id);
        let mut prompt = String::with_capacity(4096);

        // System role
        prompt.push_str(
            "You are a senior code reviewer. Your task is to review another AI agent's \
             work and evaluate whether it meets the acceptance criteria. \
             Be precise, be fair, and focus on correctness and security.",
        );
        prompt.push_str("\n\n");

        // Constraints
        prompt.push_str(&format_section(
            "Review Constraints",
            "review_constraints",
            &Self::review_constraints(),
            effective_format,
        ));
        prompt.push_str("\n\n");

        // Task context
        prompt.push_str(&format_section(
            "Task Under Review",
            "task_under_review",
            &format!(
                "Title: {title}\n\nDescription:\n{description}\n\nExpected files: {files}",
                title = config.task_title,
                description = config.task_description,
                files = if config.affected_files.is_empty() {
                    "(none specified)".to_owned()
                } else {
                    config.affected_files.join(", ")
                },
            ),
            effective_format,
        ));
        prompt.push_str("\n\n");

        // Worker output
        let output_preview = if config.worker_output.len() > 8000 {
            format!(
                "{}...\n\n[Output truncated — {} total bytes]",
                &config.worker_output[..8000],
                config.worker_output.len()
            )
        } else {
            config.worker_output.clone()
        };

        prompt.push_str(&format_section(
            "Worker Output",
            "worker_output",
            &output_preview,
            effective_format,
        ));
        prompt.push_str("\n\n");

        // Output format
        prompt.push_str(&Self::output_format(effective_format));

        prompt
    }

    /// Constraint-style review instructions.
    fn review_constraints() -> String {
        "\
HARD CONSTRAINTS — Follow these exactly:

C1: Set approved=false ONLY for Critical issues (security vulnerabilities,
    correctness bugs, data loss risks, missing error handling on failure paths).
    VIOLATION: Rejecting code for style preferences or minor improvements.

C2: Set approved=true if the code meets the acceptance criteria, even if
    you have Warning or Suggestion-level feedback.
    VIOLATION: Blocking acceptance for non-critical issues.

C3: Every issue MUST include a specific file_path and description.
    VIOLATION: Vague feedback like \"code needs improvement\" without specifics.

C4: Do NOT re-implement the task. Only review what was produced.
    VIOLATION: Writing new code instead of reviewing existing output.

C5: Respond with ONLY the JSON structure specified below. No prose.
    VIOLATION: Any text outside the JSON block."
            .to_owned()
    }

    /// Output format specification for the reviewer.
    fn output_format(format: ContextFormat) -> String {
        let json_example = r#"{
  "approved": true,
  "issues": [
    {
      "severity": "warning",
      "file_path": "src/auth.rs",
      "description": "Missing rate limiting on login endpoint"
    }
  ],
  "suggestions": [
    "Consider adding integration tests for the error paths"
  ]
}"#;

        let instructions = "\
OUTPUT CONSTRAINT: Respond with EXACTLY this JSON structure.\n\
\n\
Field constraints:\n\
- \"approved\": boolean, REQUIRED. true = accept, false = reject (critical issues only).\n\
- \"issues\": array, REQUIRED (empty [] if no issues).\n\
  - \"severity\": \"critical\" | \"warning\" | \"suggestion\"\n\
  - \"file_path\": string, file path related to the issue\n\
  - \"description\": string, specific description of the issue\n\
- \"suggestions\": array of strings, REQUIRED (empty [] if none).";

        match format {
            ContextFormat::Xml | ContextFormat::Auto => {
                format!(
                    "<output_format>\n{instructions}\n\nExample:\n{json_example}\n</output_format>"
                )
            }
            ContextFormat::Markdown => {
                format!(
                    "## Output Format\n\n{instructions}\n\nExample:\n```json\n{json_example}\n```"
                )
            }
            ContextFormat::PlainText => {
                format!("=== Output Format ===\n{instructions}\n\nExample:\n{json_example}\n---")
            }
        }
    }
}

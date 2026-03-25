//! Validator prompt builder — assembles the prompt for post-cycle requirement validation.
//!
//! After all workers complete, the validator agent holistically assesses whether
//! the combination of completed tasks fulfills the user's original requirements.
//! This builder creates the constraint-style prompt that instructs the validator
//! to evaluate cycle output and produce a structured JSON verdict.

use mahalaxmi_core::config::ContextFormat;
use serde::{Deserialize, Serialize};

use super::builder::{format_section, resolve_format};

/// Full content of a file modified during the cycle, injected for deep structural validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    /// Path of the file relative to the project root.
    pub path: String,
    /// Full (or truncated) content of the file.
    pub content: String,
    /// Original size in bytes before any truncation.
    pub original_size: usize,
    /// Whether `content` was truncated from the original file.
    pub truncated: bool,
}

/// Configuration for building a validator prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPromptConfig {
    /// The user's original requirements text.
    #[serde(default)]
    pub requirements: String,
    /// All acceptance criteria collected across tasks.
    #[serde(default)]
    pub all_acceptance_criteria: Vec<String>,
    /// Summary of each task's title, status, and output.
    #[serde(default)]
    pub tasks_summary: Vec<TaskValidationSummary>,
    /// Tasks that failed during the cycle.
    #[serde(default)]
    pub failed_tasks: Vec<String>,
    /// Combined terminal output from all workers (truncated if needed).
    #[serde(default)]
    pub combined_output: String,
    /// All files modified across the cycle.
    #[serde(default)]
    pub files_modified: Vec<String>,
    /// Results of acceptance commands (e.g., `cargo build`, `cargo test`).
    #[serde(default)]
    pub command_results: Vec<CommandResult>,
    /// Provider performing the validation (for format selection).
    #[serde(default)]
    pub provider_id: String,
    /// Context format override.
    #[serde(default = "default_context_format")]
    pub format: ContextFormat,
    /// The git diff of changes introduced in this cycle. When Some, used as primary evidence for validation.
    #[serde(default)]
    pub git_diff: Option<String>,
    /// When true, instruct the validator to return raw JSON without markdown fences (P6).
    #[serde(default)]
    pub supports_structured_output: bool,
    /// Full contents of key files modified during this cycle.
    ///
    /// When non-empty, a `## Modified File Contents` section is appended after
    /// the git diff section so the validator can verify integration, imports,
    /// and overall file structure directly from source.
    #[serde(default)]
    pub modified_file_contents: Vec<FileContent>,
}

fn default_context_format() -> ContextFormat {
    ContextFormat::Auto
}

/// Summary of one task for validation context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskValidationSummary {
    /// Task title.
    #[serde(default)]
    pub title: String,
    /// Whether the task completed successfully.
    #[serde(default)]
    pub completed: bool,
    /// Brief output excerpt or status.
    #[serde(default)]
    pub output_excerpt: String,
    /// Acceptance criteria specific to this task.
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
}

/// Result of running an acceptance command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// The command that was run (e.g., "cargo test").
    #[serde(default)]
    pub command: String,
    /// Whether the command succeeded (exit code 0).
    #[serde(default)]
    pub success: bool,
    /// Command output (truncated).
    #[serde(default)]
    pub output: String,
}

impl Default for ValidatorPromptConfig {
    fn default() -> Self {
        Self {
            requirements: String::new(),
            all_acceptance_criteria: Vec::new(),
            tasks_summary: Vec::new(),
            failed_tasks: Vec::new(),
            combined_output: String::new(),
            files_modified: Vec::new(),
            command_results: Vec::new(),
            provider_id: String::new(),
            format: ContextFormat::Auto,
            git_diff: None,
            supports_structured_output: false,
            modified_file_contents: Vec::new(),
        }
    }
}

/// Builds the prompt sent to the validator AI provider.
pub struct ValidatorPromptBuilder;

impl ValidatorPromptBuilder {
    /// Build the full validation prompt.
    pub fn build(config: &ValidatorPromptConfig) -> String {
        let effective_format = resolve_format(config.format, &config.provider_id);
        let mut prompt = String::with_capacity(8192);

        // System role
        prompt.push_str(
            "You are a requirement validation agent. Your task is to assess whether \
             an orchestration cycle's combined output fulfills the user's original \
             requirements. You evaluate holistically — not just whether individual \
             tasks passed, but whether the combination achieves the stated goal.",
        );
        prompt.push_str("\n\n");

        // Constraints (C7 appended when git diff is present)
        prompt.push_str(&format_section(
            "Validation Constraints",
            "validation_constraints",
            &Self::validation_constraints(config.git_diff.is_some()),
            effective_format,
        ));
        prompt.push_str("\n\n");

        // Original requirements
        if !config.requirements.is_empty() {
            prompt.push_str(&format_section(
                "Original Requirements",
                "original_requirements",
                &config.requirements,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Acceptance criteria
        if !config.all_acceptance_criteria.is_empty() {
            let criteria_text = config
                .all_acceptance_criteria
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{}. {}", i + 1, c))
                .collect::<Vec<_>>()
                .join("\n");

            prompt.push_str(&format_section(
                "Acceptance Criteria",
                "acceptance_criteria",
                &criteria_text,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Task summaries
        if !config.tasks_summary.is_empty() {
            let tasks_text = config
                .tasks_summary
                .iter()
                .map(|t| {
                    let status = if t.completed { "COMPLETED" } else { "FAILED" };
                    let mut entry = format!("- [{}] {}", status, t.title);
                    if !t.acceptance_criteria.is_empty() {
                        entry.push_str("\n  Criteria: ");
                        entry.push_str(&t.acceptance_criteria.join("; "));
                    }
                    if !t.output_excerpt.is_empty() {
                        entry.push_str("\n  Output: ");
                        entry.push_str(&t.output_excerpt);
                    }
                    entry
                })
                .collect::<Vec<_>>()
                .join("\n");

            prompt.push_str(&format_section(
                "Task Results",
                "task_results",
                &tasks_text,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Failed tasks callout
        if !config.failed_tasks.is_empty() {
            let failed_text = config
                .failed_tasks
                .iter()
                .map(|t| format!("- {}", t))
                .collect::<Vec<_>>()
                .join("\n");

            prompt.push_str(&format_section(
                "Failed Tasks",
                "failed_tasks",
                &failed_text,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Files modified
        if !config.files_modified.is_empty() {
            prompt.push_str(&format_section(
                "Files Modified",
                "files_modified",
                &config.files_modified.join("\n"),
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Code Changes (git diff) — primary evidence when provided (P1)
        if let Some(diff) = &config.git_diff {
            let diff_preview = if diff.len() > 51200 {
                format!(
                    "{}...\n[truncated — {} total bytes]",
                    &diff[..51200],
                    diff.len()
                )
            } else {
                diff.clone()
            };
            let diff_content = format!("```diff\n{diff_preview}\n```");
            prompt.push_str(&format_section(
                "Code Changes (git diff)",
                "git_diff",
                &diff_content,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Modified file contents (P2) — injected after git diff for deep structural checks
        if !config.modified_file_contents.is_empty() {
            let mut section = String::from(
                "## Modified File Contents\n\n\
                 Full contents of key files modified during this cycle. \
                 Use these to verify integration, imports, and overall structure.\n\n",
            );
            for entry in &config.modified_file_contents {
                if entry.truncated {
                    section.push_str(&format!(
                        "### `{}` (truncated from {} bytes)\n",
                        entry.path, entry.original_size
                    ));
                } else {
                    section.push_str(&format!("### `{}`\n", entry.path));
                }
                let ext = entry.path.rsplit('.').next().unwrap_or("").to_lowercase();
                let lang = match ext.as_str() {
                    "rs" => "rs",
                    "ts" | "tsx" => "ts",
                    "js" | "jsx" => "js",
                    "py" => "py",
                    "go" => "go",
                    "toml" => "toml",
                    "json" => "json",
                    _ => "",
                };
                section.push_str(&format!("```{lang}\n{}\n```\n\n", entry.content));
            }
            prompt.push_str(&section);
        }

        // Command results
        if !config.command_results.is_empty() {
            let cmd_text = config
                .command_results
                .iter()
                .map(|c| {
                    let status = if c.success { "PASS" } else { "FAIL" };
                    let mut entry = format!("[{}] $ {}", status, c.command);
                    if !c.output.is_empty() {
                        let preview = if c.output.len() > 2000 {
                            format!(
                                "{}...\n[truncated — {} total bytes]",
                                &c.output[..2000],
                                c.output.len()
                            )
                        } else {
                            c.output.clone()
                        };
                        entry.push_str(&format!("\n{}", preview));
                    }
                    entry
                })
                .collect::<Vec<_>>()
                .join("\n\n");

            prompt.push_str(&format_section(
                "Command Results",
                "command_results",
                &cmd_text,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Combined output (truncated)
        if !config.combined_output.is_empty() {
            let output_preview = if config.combined_output.len() > 12000 {
                format!(
                    "{}...\n\n[Output truncated — {} total bytes]",
                    &config.combined_output[..12000],
                    config.combined_output.len()
                )
            } else {
                config.combined_output.clone()
            };

            prompt.push_str(&format_section(
                "Combined Worker Output",
                "combined_output",
                &output_preview,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Output format (P6: structured output when supported)
        prompt.push_str(&Self::output_format(
            effective_format,
            config.supports_structured_output,
        ));

        prompt
    }

    /// Constraint-style validation instructions.
    ///
    /// When `has_git_diff` is true, appends constraint C7 instructing the model
    /// to treat the diff as primary evidence.
    fn validation_constraints(has_git_diff: bool) -> String {
        let mut constraints = "\
HARD CONSTRAINTS — Follow these exactly:

C1: Evaluate the COMBINATION of all task outputs against the original requirements.
    Individual task completion does NOT mean requirements are fulfilled.
    VIOLATION: Marking \"fulfilled\" because all tasks show as completed
    without verifying the combined output actually works.

C2: Set status=\"fulfilled\" ONLY when ALL requirements are demonstrably met
    and ALL acceptance criteria pass. If any critical gap exists, use
    \"partially_fulfilled\" or \"not_fulfilled\".
    VIOLATION: Marking fulfilled when any acceptance criterion is unmet.

C3: Every gap MUST reference a specific requirement and include a concrete
    suggested_fix that could be turned into a task.
    VIOLATION: Vague gaps like \"needs more work\" without specifics.

C4: For each acceptance criterion, provide specific evidence of whether it
    was met. Reference command output, file changes, or task results.
    VIOLATION: Marking a criterion as met without citing evidence.

C5: Set confidence between 0.0 and 1.0 based on how much evidence you have.
    High confidence (>0.8) requires command results or verifiable output.
    Low confidence (<0.5) when relying on task descriptions alone.
    VIOLATION: Confidence >0.9 without command verification evidence.

C6: Respond with ONLY the JSON structure specified below. No prose.
    VIOLATION: Any text outside the JSON block."
            .to_owned();

        if has_git_diff {
            constraints.push_str(
                "\n\nC7: When a `## Code Changes (git diff)` section is present, use it as PRIMARY evidence\n\
                 \x20    for your assessment. Terminal output is SECONDARY. Your verdict MUST cite specific\n\
                 \x20    diff hunks (file paths, line ranges) when evaluating whether requirements are met.\n\
                 \x20    VIOLATION: Ignoring the diff section when one is provided.",
            );
        }

        constraints
    }

    /// Output format specification for the validator.
    ///
    /// When `supports_structured_output` is true, replaces the markdown fence
    /// instruction with a bare-JSON directive (P6). When false, preserves the
    /// existing ` ```json ` fence format for backward compatibility.
    fn output_format(format: ContextFormat, supports_structured_output: bool) -> String {
        let json_example = r#"{
  "status": "partially_fulfilled",
  "summary": "Core authentication works but OAuth provider setup is missing.",
  "confidence": 0.75,
  "requirement_assessments": [
    {
      "requirement": "User login with email/password",
      "status": "fulfilled",
      "evidence": "POST /api/login endpoint exists and returns JWT",
      "contributing_tasks": ["Implement auth endpoints"]
    },
    {
      "requirement": "OAuth provider integration",
      "status": "not_fulfilled",
      "evidence": "No OAuth routes or provider configuration found in diff",
      "contributing_tasks": []
    }
  ],
  "gaps": [
    {
      "requirement": "OAuth provider integration",
      "expected": "Google and GitHub OAuth login flows",
      "actual": "No OAuth implementation found",
      "severity": "critical",
      "suggested_action": "Create OAuth routes, add provider configs, implement callback handlers",
      "affected_files": ["src/auth/oauth.rs", "src/config/oauth.rs"]
    }
  ],
  "criteria_results": [
    {
      "criterion": "cargo build succeeds",
      "task_title": "Implement auth endpoints",
      "passed": true,
      "evidence": "Build command exited with code 0, no errors"
    },
    {
      "criterion": "OAuth callback returns user profile",
      "task_title": "Implement OAuth flow",
      "passed": false,
      "evidence": "No OAuth callback route found"
    }
  ]
}"#;

        let field_constraints = "Field constraints:\n\
- \"status\": string, REQUIRED. One of: \"fulfilled\", \"partially_fulfilled\", \"not_fulfilled\".\n\
- \"summary\": string, REQUIRED. 1-3 sentence overview of the validation result.\n\
- \"confidence\": number 0.0-1.0, REQUIRED. How confident you are in this assessment.\n\
- \"requirement_assessments\": array, REQUIRED. One per identifiable requirement.\n\
  - \"requirement\": string, the requirement text.\n\
  - \"status\": \"fulfilled\" | \"partially_fulfilled\" | \"not_fulfilled\".\n\
  - \"evidence\": string, specific evidence from diff/report/commands.\n\
  - \"contributing_tasks\": array of task titles that addressed this requirement.\n\
- \"gaps\": array, REQUIRED (empty [] if no gaps).\n\
  - \"requirement\": string, the unmet requirement.\n\
  - \"expected\": string, specific expected behavior/output.\n\
  - \"actual\": string, what was actually produced.\n\
  - \"severity\": \"critical\" | \"major\" | \"minor\".\n\
  - \"suggested_action\": string, concrete action to close this gap.\n\
  - \"affected_files\": array of file paths.\n\
- \"criteria_results\": array, REQUIRED (empty [] if no criteria to evaluate).\n\
  - \"criterion\": string, the acceptance criterion text.\n\
  - \"task_title\": string, which task this criterion belongs to.\n\
  - \"passed\": boolean, whether it was satisfied.\n\
  - \"evidence\": string, specific evidence supporting the determination.";

        let structured_prefix = "Return ONLY valid JSON — no markdown fences, no commentary, \
             no preamble. Begin your response with `{` and end with `}`.";

        let instructions = if supports_structured_output {
            format!("{structured_prefix}\n\n{field_constraints}")
        } else {
            format!("OUTPUT CONSTRAINT: Respond with EXACTLY this JSON structure. No prose before or after.\n\n{field_constraints}")
        };

        if supports_structured_output {
            match format {
                ContextFormat::Xml | ContextFormat::Auto => format!(
                    "<output_format>\n{instructions}\n\nExample:\n{json_example}\n</output_format>"
                ),
                ContextFormat::Markdown => {
                    format!("## Output Format\n\n{instructions}\n\nExample:\n{json_example}")
                }
                ContextFormat::PlainText => format!(
                    "=== Output Format ===\n{instructions}\n\nExample:\n{json_example}\n---"
                ),
            }
        } else {
            match format {
                ContextFormat::Xml | ContextFormat::Auto => format!(
                    "<output_format>\n{instructions}\n\nExample:\n{json_example}\n</output_format>"
                ),
                ContextFormat::Markdown => format!(
                    "## Output Format\n\n{instructions}\n\nExample:\n```json\n{json_example}\n```"
                ),
                ContextFormat::PlainText => format!(
                    "=== Output Format ===\n{instructions}\n\nExample:\n{json_example}\n---"
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_config() -> ValidatorPromptConfig {
        ValidatorPromptConfig {
            requirements: "Implement user authentication".to_owned(),
            combined_output: "Worker output here".to_owned(),
            provider_id: String::new(),
            format: ContextFormat::Markdown,
            git_diff: None,
            supports_structured_output: false,
            ..ValidatorPromptConfig::default()
        }
    }

    #[test]
    fn build_without_diff_matches_baseline() {
        let prompt = ValidatorPromptBuilder::build(&baseline_config());
        assert!(
            !prompt.contains("Code Changes (git diff)"),
            "diff section must not appear when git_diff is None"
        );
        assert!(
            !prompt.contains("C7:"),
            "C7 constraint must not appear when git_diff is None"
        );
        assert!(
            prompt.contains("HARD CONSTRAINTS"),
            "base constraints must still be present"
        );
        assert!(
            prompt.contains("C6:"),
            "C6 must be the last constraint when no diff"
        );
    }

    #[test]
    fn build_with_diff_injects_diff_section() {
        let diff_text = "diff --git a/src/lib.rs b/src/lib.rs\n+pub fn new() {}";
        let config = ValidatorPromptConfig {
            git_diff: Some(diff_text.to_owned()),
            ..baseline_config()
        };
        let prompt = ValidatorPromptBuilder::build(&config);
        assert!(
            prompt.contains("## Code Changes (git diff)"),
            "diff section header must appear when git_diff is Some"
        );
        assert!(
            prompt.contains("```diff"),
            "diff must be wrapped in a diff code fence"
        );
        assert!(
            prompt.contains(diff_text),
            "original diff content must be present in the prompt"
        );
    }

    #[test]
    fn build_with_diff_includes_c7_constraint() {
        let config = ValidatorPromptConfig {
            git_diff: Some("+ fn added() {}".to_owned()),
            ..baseline_config()
        };
        let prompt = ValidatorPromptBuilder::build(&config);
        assert!(
            prompt.contains("C7:"),
            "C7 constraint must appear when git_diff is Some"
        );
        assert!(
            prompt.contains("PRIMARY evidence"),
            "C7 must mention PRIMARY evidence"
        );
        assert!(
            prompt.contains("diff hunks"),
            "C7 must instruct citation of diff hunks"
        );
    }

    #[test]
    fn build_structured_output_uses_bare_json_instruction() {
        let config = ValidatorPromptConfig {
            supports_structured_output: true,
            ..baseline_config()
        };
        let prompt = ValidatorPromptBuilder::build(&config);
        assert!(
            prompt.contains("Return ONLY valid JSON"),
            "structured output prompt must contain 'Return ONLY valid JSON'"
        );
        assert!(
            !prompt.contains("```json"),
            "structured output prompt must not contain backtick-json fence"
        );
    }

    #[test]
    fn build_non_structured_output_preserves_json_fence() {
        let prompt = ValidatorPromptBuilder::build(&baseline_config());
        assert!(
            prompt.contains("```json"),
            "non-structured output must preserve the backtick-json fence"
        );
        assert!(
            !prompt.contains("Return ONLY valid JSON"),
            "non-structured output must not contain the bare-JSON instruction"
        );
    }

    #[test]
    fn default_config_has_correct_new_field_defaults() {
        let config = ValidatorPromptConfig::default();
        assert!(config.git_diff.is_none(), "git_diff must default to None");
        assert!(
            !config.supports_structured_output,
            "supports_structured_output must default to false"
        );
        assert!(
            config.modified_file_contents.is_empty(),
            "modified_file_contents must default to empty"
        );
    }

    #[test]
    fn build_includes_modified_file_contents_section_when_non_empty() {
        let config = ValidatorPromptConfig {
            modified_file_contents: vec![FileContent {
                path: "src/lib.rs".to_owned(),
                content: "pub fn hello() {}".to_owned(),
                original_size: 18,
                truncated: false,
            }],
            ..baseline_config()
        };
        let prompt = ValidatorPromptBuilder::build(&config);
        assert!(
            prompt.contains("## Modified File Contents"),
            "heading must appear when modified_file_contents is non-empty"
        );
        assert!(
            prompt.contains("### `src/lib.rs`"),
            "file entry heading must appear"
        );
        assert!(
            prompt.contains("```rs"),
            "fenced code block with rs tag must appear for .rs extension"
        );
        assert!(
            prompt.contains("pub fn hello() {}"),
            "file content must be present in the prompt"
        );
    }

    #[test]
    fn build_omits_modified_file_contents_section_when_empty() {
        let config = ValidatorPromptConfig {
            modified_file_contents: Vec::new(),
            ..baseline_config()
        };
        let prompt = ValidatorPromptBuilder::build(&config);
        assert!(
            !prompt.contains("## Modified File Contents"),
            "section must not appear when modified_file_contents is empty"
        );
    }

    #[test]
    fn build_truncated_entry_shows_original_size_in_heading() {
        let config = ValidatorPromptConfig {
            modified_file_contents: vec![FileContent {
                path: "src/big_module.rs".to_owned(),
                content: "// truncated content".to_owned(),
                original_size: 102_400,
                truncated: true,
            }],
            ..baseline_config()
        };
        let prompt = ValidatorPromptBuilder::build(&config);
        assert!(
            prompt.contains("(truncated from 102400 bytes)"),
            "truncated entry must show original size: got prompt snippet: {}",
            &prompt[prompt.find("big_module").unwrap_or(0)..][..100.min(prompt.len())]
        );
        assert!(
            prompt.contains("### `src/big_module.rs` (truncated from 102400 bytes)"),
            "full truncated heading must match expected format"
        );
    }
}

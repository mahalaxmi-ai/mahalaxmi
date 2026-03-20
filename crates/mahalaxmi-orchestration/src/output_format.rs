//! Cycle output formatting — writes worker results per domain `OutputFormat`.
//!
//! `format_cycle_output` is the single entry point. It inspects the domain's
//! configured `OutputFormat` and either:
//! - returns `Ok(None)` when the format is `PullRequest` (existing PR flow owns output), or
//! - writes a file to `worktree_path` and returns `Ok(Some(relative_path))`.
//!
//! The caller (driver layer) is responsible for staging the returned file path
//! before the worktree commit is opened as a pull request.

use std::path::Path;

use mahalaxmi_core::domain::OutputFormat;

/// Errors produced during cycle output formatting.
#[derive(Debug, thiserror::Error)]
pub enum OutputFormatError {
    /// An I/O error occurred while writing the output file.
    #[error("I/O error writing output file: {0}")]
    Io(#[from] std::io::Error),
    /// The requested output format is not yet supported.
    #[error("Unsupported output format variant: {0}")]
    Unsupported(String),
}

/// Extract the content of a named markdown section from `output`.
///
/// Searches `output` case-insensitively for a heading line whose text equals
/// `section_name` (e.g. `## Executive Summary`, `### executive summary`, or
/// `# EXECUTIVE SUMMARY`).  If found, captures all lines from that heading
/// until the next heading of equal or higher level (i.e. the same number of
/// leading `#` characters or fewer).  Leading/trailing whitespace is stripped
/// from the result.
///
/// Returns `Some(content)` when the section is present and non-empty, or
/// `None` when the section heading is absent or the extracted content is
/// blank/whitespace-only.
pub fn extract_section(output: &str, section_name: &str) -> Option<String> {
    let section_lower = section_name.to_lowercase();
    let lines: Vec<&str> = output.lines().collect();

    // Find the index and heading level of the matching section heading.
    let mut start_idx: Option<usize> = None;
    let mut heading_level: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
        if hash_count == 0 {
            continue;
        }
        let after_hashes = &trimmed[hash_count..];
        // A valid markdown heading requires a space between `#` tokens and text.
        if !after_hashes.starts_with(' ') {
            continue;
        }
        let heading_text = after_hashes.trim();
        if heading_text.to_lowercase() == section_lower {
            start_idx = Some(i);
            heading_level = hash_count;
            break;
        }
    }

    let start = start_idx?;

    // Collect body lines until the next heading of equal or higher level.
    let mut content_lines: Vec<&str> = Vec::new();
    for line in &lines[start + 1..] {
        let trimmed = line.trim();
        let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
        if hash_count > 0 {
            let after_hashes = &trimmed[hash_count..];
            if after_hashes.starts_with(' ') && hash_count <= heading_level {
                break;
            }
        }
        content_lines.push(line);
    }

    let content = content_lines.join("\n");
    let trimmed_content = content.trim();
    if trimmed_content.is_empty() {
        None
    } else {
        Some(trimmed_content.to_owned())
    }
}

/// Format and write the cycle output per the domain `OutputFormat`, optionally
/// incorporating a synthesis manager result.
///
/// When `synthesis_result` is `Some` **and** the format is
/// `OutputFormat::StructuredReport`, the synthesis string is used as the
/// authoritative source for section extraction instead of aggregating
/// individual worker outputs.  Each declared section is extracted from
/// `synthesis_result` via [`extract_section`]; if a section heading is absent
/// the section body is written as `(No content extracted)`.  Sections appear
/// in the declaration order from the domain configuration.
///
/// When `synthesis_result` is `None` the function behaves identically to
/// [`format_cycle_output`] — existing multi-worker aggregation logic applies.
///
/// # Design note
///
/// The `synthesis_result` parameter was added as an explicit extra argument
/// (rather than embedding it in a wrapper struct) because the synthesis string
/// originates from a post-worker PTY session that lives outside the
/// `OutputFormat` enum.  The three-argument [`format_cycle_output`] overload
/// delegates here with `synthesis_result = None`, preserving backwards
/// compatibility for all existing call sites (service layer, integration tests).
///
/// # Arguments
///
/// * `format` — the domain's configured output format.
/// * `worker_outputs` — collected terminal output strings from all workers.
/// * `worktree_path` — absolute path to the worktree root where files should
///   be written.
/// * `synthesis_result` — optional output from a post-worker synthesis manager
///   session.
pub fn format_cycle_output_with_synthesis(
    format: &OutputFormat,
    worker_outputs: &[String],
    worktree_path: &Path,
    synthesis_result: Option<&str>,
) -> Result<Option<String>, OutputFormatError> {
    // When a synthesis result is available and the format is StructuredReport,
    // build the output document from the synthesis text rather than
    // aggregating worker outputs section-by-section.
    if let (
        Some(synthesis_text),
        OutputFormat::StructuredReport {
            sections,
            output_file,
        },
    ) = (synthesis_result, format)
    {
        let mut markdown = String::new();

        for section in sections {
            markdown.push_str(&format!("# {section}\n\n"));
            let content = extract_section(synthesis_text, section)
                .unwrap_or_else(|| "(No content extracted)".to_owned());
            markdown.push_str(&content);
            markdown.push_str("\n\n");
        }

        let dest = worktree_path.join(output_file);
        std::fs::write(&dest, &markdown)?;
        return Ok(Some(output_file.clone()));
    }

    // No synthesis result — delegate to the standard aggregation path.
    format_cycle_output(format, worker_outputs, worktree_path)
}

/// Format and write the cycle output per the domain `OutputFormat`.
///
/// Returns the relative output file path (relative to `worktree_path`) if a
/// file was written, or `None` for `PullRequest` (no file — the existing PR
/// flow handles the output).
///
/// For synthesis-domain cycles, use [`format_cycle_output_with_synthesis`]
/// instead so that the post-worker synthesis manager result is incorporated
/// into the output document.
///
/// # Arguments
///
/// * `format` — the domain's configured output format.
/// * `worker_outputs` — collected terminal output strings from all workers.
/// * `worktree_path` — absolute path to the worktree root where files should
///   be written.
pub fn format_cycle_output(
    format: &OutputFormat,
    worker_outputs: &[String],
    worktree_path: &Path,
) -> Result<Option<String>, OutputFormatError> {
    match format {
        OutputFormat::PullRequest => {
            // Existing PR flow handles the output; nothing to write here.
            Ok(None)
        }

        OutputFormat::StructuredReport {
            sections,
            output_file,
        } => {
            // Use a domain-specific report title when the configured output file
            // name matches the security report convention.
            let report_title = if output_file.ends_with("security-report.md") {
                "# Security Report"
            } else {
                "# Cycle Report"
            };

            let mut markdown = String::new();
            markdown.push_str(report_title);
            markdown.push_str("\n\n");

            for section in sections {
                markdown.push_str(&format!("## {section}\n\n"));

                // Collect contributions from all workers that contain this section.
                // Multiple workers may each flag findings under the same heading;
                // their extracted content is concatenated under a single heading.
                let mut section_content = String::new();
                for worker_output in worker_outputs {
                    if let Some(extracted) = extract_section(worker_output, section) {
                        if !section_content.is_empty() {
                            section_content.push_str("\n\n");
                        }
                        section_content.push_str(&extracted);
                    }
                }

                if section_content.is_empty() {
                    markdown.push_str("No findings in this category.\n\n");
                } else {
                    markdown.push_str(&section_content);
                    markdown.push_str("\n\n");
                }
            }

            let dest = worktree_path.join(output_file);
            std::fs::write(&dest, &markdown)?;
            Ok(Some(output_file.clone()))
        }

        OutputFormat::JsonExport {
            output_file,
            schema,
        } => {
            if let Some(schema_path) = schema {
                tracing::debug!(
                    schema_path = %schema_path,
                    "JSON schema validation is advisory; schema conformance is not enforced"
                );
            }

            let timestamp = chrono::Utc::now().to_rfc3339();
            let payload = serde_json::json!({
                "timestamp": timestamp,
                "worker_output_count": worker_outputs.len(),
                "outputs": worker_outputs,
            });

            let json_str = serde_json::to_string_pretty(&payload)
                .map_err(std::io::Error::other)?;

            let dest = worktree_path.join(output_file);
            std::fs::write(&dest, &json_str)?;
            Ok(Some(output_file.clone()))
        }

        OutputFormat::MarkdownFile {
            output_file,
            template,
        } => {
            let mut markdown = String::new();

            if let Some(template_path) = template {
                let template_full = worktree_path.join(template_path);
                match std::fs::read_to_string(&template_full) {
                    Ok(content) => {
                        markdown.push_str(&content);
                        markdown.push('\n');
                    }
                    Err(e) => {
                        tracing::warn!(
                            template_path = %template_path,
                            error = %e,
                            "Failed to read markdown template; using default formatting"
                        );
                    }
                }
            }

            if markdown.is_empty() {
                markdown.push_str("# Cycle Results\n\n");
            }

            for (idx, output) in worker_outputs.iter().enumerate() {
                markdown.push_str(&format!("## Worker {}\n\n", idx + 1));
                markdown.push_str(output.trim());
                markdown.push_str("\n\n");
            }

            let dest = worktree_path.join(output_file);
            std::fs::write(&dest, &markdown)?;
            Ok(Some(output_file.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── extract_section ──────────────────────────────────────────────────────

    #[test]
    fn extract_section_returns_content_when_present() {
        let output = "## Executive Summary\n\nAll systems nominal.\n\nNo critical issues.";
        let result = extract_section(output, "Executive Summary");
        assert_eq!(
            result,
            Some("All systems nominal.\n\nNo critical issues.".to_owned())
        );
    }

    #[test]
    fn extract_section_is_case_insensitive() {
        let output = "# EXECUTIVE SUMMARY\n\nFindings here.";
        let result = extract_section(output, "executive summary");
        assert_eq!(result, Some("Findings here.".to_owned()));
    }

    #[test]
    fn extract_section_returns_none_when_absent() {
        let output = "## Critical Findings\n\nSome issue found.";
        let result = extract_section(output, "Executive Summary");
        assert!(result.is_none(), "Should return None for a missing section");
    }

    #[test]
    fn extract_section_stops_at_next_same_level_heading() {
        let output = "\
## Critical Findings

SQL injection in auth module.

## Medium Findings

Outdated dependency.
";
        let result = extract_section(output, "Critical Findings");
        let content = result.expect("Critical Findings section should be found");
        assert!(
            content.contains("SQL injection"),
            "Should contain Critical Findings content"
        );
        assert!(
            !content.contains("Outdated dependency"),
            "Should not bleed into Medium Findings"
        );
        assert!(
            !content.contains("Medium Findings"),
            "Should not include the next section heading"
        );
    }

    #[test]
    fn extract_section_stops_at_higher_level_heading() {
        let output = "\
### Details

Some detail text.

## Top Level Section

Top level content.
";
        let result = extract_section(output, "Details");
        let content = result.expect("Details section should be found");
        assert!(content.contains("Some detail text."));
        assert!(
            !content.contains("Top Level Section"),
            "Should stop at a higher-level (fewer #) heading"
        );
    }

    #[test]
    fn extract_section_does_not_stop_at_lower_level_subheading() {
        let output = "\
## Critical Findings

Overview line.

### Sub-detail

Sub detail content.

## Medium Findings

Medium content.
";
        let result = extract_section(output, "Critical Findings");
        let content = result.expect("Critical Findings should be found");
        assert!(
            content.contains("Sub-detail"),
            "Sub-headings are part of the section and must be included"
        );
        assert!(
            content.contains("Sub detail content."),
            "Sub-heading body must be included"
        );
        assert!(
            !content.contains("Medium content."),
            "Should not bleed past the next same-level heading"
        );
    }

    #[test]
    fn extract_section_returns_none_for_blank_section_body() {
        let output = "## Executive Summary\n\n   \n\n## Findings\n\nSomething.";
        let result = extract_section(output, "Executive Summary");
        assert!(
            result.is_none(),
            "Blank section body should return None"
        );
    }

    // ── format_cycle_output — StructuredReport ───────────────────────────────

    #[test]
    fn pull_request_returns_none() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let result = format_cycle_output(&OutputFormat::PullRequest, &[], dir.path());
        assert!(result.is_ok(), "PullRequest should not error");
        assert!(
            result.unwrap().is_none(),
            "PullRequest should return None (no file written)"
        );
    }

    #[test]
    fn structured_report_creates_expected_file() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let format = OutputFormat::StructuredReport {
            sections: vec!["Summary".to_owned(), "Findings".to_owned()],
            output_file: "report.md".to_owned(),
        };
        let outputs = vec![
            "## Summary\nAll tests passed.".to_owned(),
            "## Findings\nNo issues found.".to_owned(),
        ];

        let result = format_cycle_output(&format, &outputs, dir.path());
        assert!(result.is_ok(), "StructuredReport should not error");

        let path = result.unwrap();
        assert_eq!(
            path.as_deref(),
            Some("report.md"),
            "Should return the relative output file path"
        );

        let written = std::fs::read_to_string(dir.path().join("report.md"))
            .expect("output file should exist");
        assert!(
            written.contains("## Summary"),
            "Report should contain Summary section"
        );
        assert!(
            written.contains("## Findings"),
            "Report should contain Findings section"
        );
        assert!(
            written.contains("All tests passed."),
            "Summary content should be present"
        );
        assert!(
            written.contains("No issues found."),
            "Findings content should be present"
        );
    }

    #[test]
    fn structured_report_fills_missing_sections() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let format = OutputFormat::StructuredReport {
            sections: vec!["Critical Issues".to_owned(), "Low Issues".to_owned()],
            output_file: "gaps.md".to_owned(),
        };
        // Neither worker output contains a proper markdown heading for either section.
        let outputs = vec!["Critical Issues: buffer overflow found at line 42.".to_owned()];

        let result = format_cycle_output(&format, &outputs, dir.path());
        assert!(result.is_ok());

        let written =
            std::fs::read_to_string(dir.path().join("gaps.md")).expect("file should exist");
        assert!(
            written.contains("No findings in this category."),
            "Missing sections should be filled with placeholder text"
        );
    }

    #[test]
    fn structured_report_concatenates_multiple_worker_contributions() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let format = OutputFormat::StructuredReport {
            sections: vec![
                "Critical Findings".to_owned(),
                "Medium Findings".to_owned(),
            ],
            output_file: "multi.md".to_owned(),
        };
        // Worker 1 contributes Critical Findings only.
        // Worker 2 contributes Medium Findings only.
        // Worker 3 contributes an additional Critical Findings entry.
        let outputs = vec![
            "## Critical Findings\n\nSQL injection in login form.".to_owned(),
            "## Medium Findings\n\nOutdated OpenSSL version.".to_owned(),
            "## Critical Findings\n\nRemote code execution via upload endpoint.".to_owned(),
        ];

        let result = format_cycle_output(&format, &outputs, dir.path());
        assert!(result.is_ok());

        let written =
            std::fs::read_to_string(dir.path().join("multi.md")).expect("file should exist");

        // Both critical findings from different workers must appear together.
        assert!(
            written.contains("SQL injection in login form."),
            "First worker's Critical Findings should be present"
        );
        assert!(
            written.contains("Remote code execution via upload endpoint."),
            "Third worker's Critical Findings should also be present"
        );
        // Medium findings from worker 2 must appear.
        assert!(
            written.contains("Outdated OpenSSL version."),
            "Worker 2's Medium Findings should be present"
        );
        // The placeholder should NOT appear for any section that has content.
        let critical_section_start = written
            .find("## Critical Findings")
            .expect("Critical Findings heading must be present");
        let medium_section_start = written
            .find("## Medium Findings")
            .expect("Medium Findings heading must be present");
        let critical_body = &written[critical_section_start..medium_section_start];
        assert!(
            !critical_body.contains("No findings in this category."),
            "Placeholder must not appear in a section that has content"
        );
    }

    #[test]
    fn structured_report_uses_security_report_title_for_security_report_md() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let format = OutputFormat::StructuredReport {
            sections: vec!["Executive Summary".to_owned()],
            output_file: "security-report.md".to_owned(),
        };
        let outputs = vec!["## Executive Summary\n\nAll clear.".to_owned()];

        let result = format_cycle_output(&format, &outputs, dir.path());
        assert!(result.is_ok());

        let written = std::fs::read_to_string(dir.path().join("security-report.md"))
            .expect("file should exist");
        assert!(
            written.starts_with("# Security Report"),
            "security-report.md must begin with '# Security Report', got: {written:?}"
        );
        assert!(
            !written.starts_with("# Cycle Report"),
            "security-report.md must NOT use the generic '# Cycle Report' title"
        );
    }

    #[test]
    fn structured_report_uses_cycle_report_title_for_other_files() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let format = OutputFormat::StructuredReport {
            sections: vec!["Summary".to_owned()],
            output_file: "report.md".to_owned(),
        };
        let outputs = vec!["## Summary\n\nDone.".to_owned()];

        let result = format_cycle_output(&format, &outputs, dir.path());
        assert!(result.is_ok());

        let written =
            std::fs::read_to_string(dir.path().join("report.md")).expect("file should exist");
        assert!(
            written.starts_with("# Cycle Report"),
            "Non-security output files must use the generic '# Cycle Report' title"
        );
    }

    #[test]
    fn json_export_creates_valid_json_file() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let format = OutputFormat::JsonExport {
            output_file: "results.json".to_owned(),
            schema: None,
        };
        let outputs = vec!["Worker output text".to_owned()];

        let result = format_cycle_output(&format, &outputs, dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_deref(), Some("results.json"));

        let raw =
            std::fs::read_to_string(dir.path().join("results.json")).expect("file should exist");
        let parsed: serde_json::Value = serde_json::from_str(&raw).expect("should be valid JSON");
        assert!(parsed.get("outputs").is_some(), "JSON should have outputs field");
    }

    #[test]
    fn markdown_file_creates_output_with_worker_sections() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let format = OutputFormat::MarkdownFile {
            output_file: "output.md".to_owned(),
            template: None,
        };
        let outputs = vec!["Fixed the login bug.".to_owned(), "Added tests.".to_owned()];

        let result = format_cycle_output(&format, &outputs, dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_deref(), Some("output.md"));

        let written =
            std::fs::read_to_string(dir.path().join("output.md")).expect("file should exist");
        assert!(written.contains("## Worker 1"), "Should have worker section headers");
        assert!(written.contains("Fixed the login bug."));
        assert!(written.contains("## Worker 2"));
        assert!(written.contains("Added tests."));
    }

    // ── format_cycle_output_with_synthesis ───────────────────────────────────

    /// Verify that all 6 legal-analysis sections are written in declaration
    /// order when a synthesis result containing each heading is provided.
    #[test]
    fn synthesis_legal_analysis_all_six_sections_present_in_order() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");

        // Section list mirrors domains/legal/config.yaml declaration order.
        let sections = vec![
            "Executive Summary".to_owned(),
            "Key Obligations".to_owned(),
            "Risk Areas".to_owned(),
            "Unusual Clauses".to_owned(),
            "Missing Protections".to_owned(),
            "Negotiation Recommendations".to_owned(),
        ];
        let format = OutputFormat::StructuredReport {
            sections: sections.clone(),
            output_file: "legal-analysis.md".to_owned(),
        };

        // Synthesis output contains all 6 sections with dummy content.
        let synthesis_text = "\
## Executive Summary

This agreement is generally acceptable.

## Key Obligations

Licensee must pay monthly fees by the 1st.

## Risk Areas

Indemnification clause is broadly written.

## Unusual Clauses

Automatic renewal without notice is atypical.

## Missing Protections

No limitation-of-liability cap specified.

## Negotiation Recommendations

Propose a 30-day cure period for material breaches.
";

        let result = format_cycle_output_with_synthesis(
            &format,
            &[],
            dir.path(),
            Some(synthesis_text),
        );
        assert!(result.is_ok(), "Should not error: {:?}", result.err());
        assert_eq!(
            result.unwrap().as_deref(),
            Some("legal-analysis.md"),
            "Must return relative path 'legal-analysis.md'"
        );

        let written = std::fs::read_to_string(dir.path().join("legal-analysis.md"))
            .expect("legal-analysis.md must be created");

        // Verify declaration order by checking positions are monotonically increasing.
        let positions: Vec<usize> = sections
            .iter()
            .map(|s| {
                written
                    .find(s.as_str())
                    .unwrap_or_else(|| panic!("Section '{}' must appear in output", s))
            })
            .collect();
        for window in positions.windows(2) {
            assert!(
                window[0] < window[1],
                "Sections must appear in declaration order"
            );
        }

        // Spot-check actual content extraction.
        assert!(
            written.contains("This agreement is generally acceptable."),
            "Executive Summary content must be present"
        );
        assert!(
            written.contains("Propose a 30-day cure period"),
            "Negotiation Recommendations content must be present"
        );
    }

    /// When a section is absent from the synthesis output it must be written as
    /// `(No content extracted)` rather than silently omitted.
    #[test]
    fn synthesis_missing_section_written_as_no_content_extracted() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let format = OutputFormat::StructuredReport {
            sections: vec![
                "Executive Summary".to_owned(),
                "Missing Protections".to_owned(),
            ],
            output_file: "legal-analysis.md".to_owned(),
        };

        // Synthesis output contains only Executive Summary — Missing Protections is absent.
        let synthesis_text = "## Executive Summary\n\nAll looks fine.\n";

        let result = format_cycle_output_with_synthesis(
            &format,
            &[],
            dir.path(),
            Some(synthesis_text),
        );
        assert!(result.is_ok(), "Should not error: {:?}", result.err());

        let written = std::fs::read_to_string(dir.path().join("legal-analysis.md"))
            .expect("legal-analysis.md must be created");

        assert!(
            written.contains("# Missing Protections"),
            "Absent section heading must still appear"
        );
        assert!(
            written.contains("(No content extracted)"),
            "Absent section body must read '(No content extracted)'"
        );
    }

    /// When synthesis_result is None, format_cycle_output_with_synthesis must
    /// behave identically to format_cycle_output (existing aggregation logic).
    #[test]
    fn synthesis_none_falls_through_to_existing_behaviour() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let format = OutputFormat::StructuredReport {
            sections: vec!["Summary".to_owned()],
            output_file: "report.md".to_owned(),
        };
        let outputs = vec!["## Summary\n\nDone.".to_owned()];

        let result =
            format_cycle_output_with_synthesis(&format, &outputs, dir.path(), None);
        assert!(result.is_ok());
        let written =
            std::fs::read_to_string(dir.path().join("report.md")).expect("file should exist");
        assert!(written.contains("Done."));
    }
}

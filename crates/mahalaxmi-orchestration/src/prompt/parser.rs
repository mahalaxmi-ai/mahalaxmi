//! Manager output parser — extracts JSON from raw AI terminal output.
//!
//! AI managers produce a mix of explanatory text and JSON. This parser
//! finds the JSON block containing the task proposal and delegates
//! to `ManagerProposal::from_json()` for structured parsing.

use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::ManagerId;
use mahalaxmi_core::MahalaxmiResult;

use crate::models::proposal::ManagerProposal;

/// Extracts a `ManagerProposal` from raw terminal output.
///
/// The parser tries multiple extraction strategies in order:
/// 1. JSON inside a code fence (` ```json ... ``` ` or ` ``` ... ``` `)
/// 2. Bare JSON object (outermost `{ ... }` containing `"tasks"`)
///
/// If no valid JSON is found, returns a descriptive error.
pub struct ManagerOutputParser;

impl ManagerOutputParser {
    /// Parse raw terminal output into a `ManagerProposal`.
    ///
    /// Scans the output for JSON containing a `"tasks"` array,
    /// then delegates to `ManagerProposal::from_json()`.
    pub fn parse(
        output: &str,
        manager_id: ManagerId,
        duration_ms: u64,
        i18n: &I18nService,
    ) -> MahalaxmiResult<ManagerProposal> {
        // Strip carriage returns before extraction. PTY output uses CRLF line
        // endings (\r\n). The \r chars are harmless between JSON tokens, but if
        // they appear inside a JSON string value (e.g. from a provider that
        // wraps lines mid-description) they make the JSON invalid per RFC 8259.
        // Stripping \r normalises to LF-only without changing JSON semantics.
        let clean = output.replace('\r', "");

        // Guard: detect echoed-prompt output. Providers like `echo` or any
        // misconfigured test binary reproduce the full prompt verbatim. The
        // prompt template ends with an example JSON block that the parser would
        // otherwise extract as a real manager response, yielding phantom tasks
        // with placeholder titles like "Short descriptive title". Detect this
        // by checking whether the output starts with the manager system role
        // text — a genuine AI response would never begin with those words.
        if clean.starts_with("You are a senior software engineering manager") {
            return Err(MahalaxmiError::orchestration(
                i18n,
                "error-manager-output-is-echoed-prompt",
                &[("manager", manager_id.as_str())],
            ));
        }

        let json = Self::extract_json(&clean).ok_or_else(|| {
            MahalaxmiError::orchestration(
                i18n,
                "error-no-json-in-output",
                &[("manager", manager_id.as_str())],
            )
        })?;

        ManagerProposal::from_json(&json, manager_id, duration_ms, i18n)
    }

    /// Extract JSON from raw output using multiple strategies.
    ///
    /// Returns the first valid, parseable JSON string found, or `None`.
    pub fn extract_json(output: &str) -> Option<String> {
        // Strategy 1: code fence (```json ... ``` or ``` ... ```)
        // Validate with serde_json before returning — descriptions may contain
        // literal backtick sequences that cause find("```") to find the wrong
        // closing fence and return truncated JSON that contains "tasks" but
        // fails to parse. If code fence extraction gives invalid JSON, fall
        // through to the bare JSON strategy.
        if let Some(json) = Self::extract_from_code_fence(output) {
            if json.contains("\"tasks\"")
                && serde_json::from_str::<serde_json::Value>(&json).is_ok()
            {
                return Some(json);
            }
        }

        // Strategy 2: bare JSON object containing "tasks"
        if let Some(json) = Self::extract_bare_json(output) {
            return Some(json);
        }

        // Strategy 3: anchor from "tasks": — finds the last "tasks" key and
        // walks backward to find the enclosing JSON object. Handles cases where
        // strategies 1 and 2 fail because the output has many small JSON
        // objects earlier in the text (tool call logs, prose examples, etc.)
        // that confuse the forward-scanning extractors.
        if let Some(json) = Self::extract_tasks_anchored(output) {
            return Some(json);
        }

        None
    }

    /// Extract JSON from within a code fence block.
    ///
    /// Handles both ` ```json ` and plain ` ``` ` fences. For each opening
    /// fence, tries every subsequent ` ``` ` as the potential closing fence
    /// so that ` ``` ` sequences inside string values (e.g. in description
    /// fields) don't permanently truncate the extracted content.
    fn extract_from_code_fence(output: &str) -> Option<String> {
        let fence_positions: Vec<usize> = output.match_indices("```").map(|(pos, _)| pos).collect();

        let mut i = 0;
        while i < fence_positions.len() {
            let start = fence_positions[i];
            let after_fence = start + 3;
            let line_end = output[after_fence..]
                .find('\n')
                .map(|p| after_fence + p + 1)
                .unwrap_or(output.len());

            let fence_suffix = output[after_fence..line_end].trim();
            if fence_suffix.is_empty()
                || fence_suffix.starts_with("json")
                || fence_suffix.starts_with("JSON")
            {
                // This is an opening fence. Try each subsequent ``` as a
                // potential closing fence, accepting the first that yields
                // content starting with '{'. Validation happens in
                // extract_json; here we just return the longest-first
                // candidate that looks like JSON.
                let mut j = i + 1;
                while j < fence_positions.len() {
                    let close_start = fence_positions[j];
                    if close_start <= line_end {
                        j += 1;
                        continue;
                    }
                    let content = output[line_end..close_start].trim();
                    if content.starts_with('{') {
                        return Some(content.to_string());
                    }
                    j += 1;
                }
            }
            i += 1;
        }

        None
    }

    /// Extract the outermost JSON object containing `"tasks"` from bare text.
    ///
    /// Uses brace counting to find the complete JSON object.
    fn extract_bare_json(output: &str) -> Option<String> {
        // Find positions where a `{` appears that could start our JSON
        for (start, _) in output.match_indices('{') {
            let remaining = &output[start..];

            // Brace-count to find the matching closing brace
            if let Some(end) = find_matching_brace(remaining) {
                let candidate = &remaining[..=end];
                // Validate the candidate itself contains "tasks" (not just the
                // remaining text after it) and is parseable JSON. Without the
                // candidate check, small valid JSON objects earlier in the
                // output (tool logs, prose examples) are returned prematurely.
                if candidate.contains("\"tasks\"")
                    && serde_json::from_str::<serde_json::Value>(candidate).is_ok()
                {
                    return Some(candidate.to_string());
                }
            }
        }

        None
    }

    /// Anchor-from-`"tasks":` extraction strategy.
    ///
    /// Finds the last `"tasks"` key in the output (the manager's plan is
    /// always at the end, after tool call logs and prose), then walks backward
    /// through `{` positions to find the enclosing JSON object.
    ///
    /// This handles output where strategies 1 and 2 fail because many small
    /// valid JSON objects appear earlier in the text and exhaust the forward
    /// scanners before reaching the actual proposal.
    fn extract_tasks_anchored(output: &str) -> Option<String> {
        let tasks_pos = output.rfind("\"tasks\"")?;

        // Walk backward through { positions; take the first one whose matched
        // range encloses tasks_pos and produces valid JSON.
        for (brace_start, _) in output[..tasks_pos].rmatch_indices('{') {
            let remaining = &output[brace_start..];
            if let Some(end) = find_matching_brace(remaining) {
                // end is relative to remaining; check it encloses tasks_pos
                if brace_start + end >= tasks_pos {
                    let candidate = &remaining[..=end];
                    if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
                        return Some(candidate.to_string());
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
    use mahalaxmi_core::types::ManagerId;

    fn manager() -> ManagerId {
        ManagerId::new("manager-0")
    }

    fn i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Empty-proposal anomaly — regression tests
    //
    // Production observation (2026-03-01, cycle 14b90837):
    //   manager-0 returned clean_output = `{"tasks": []}` (13 bytes) after a
    //   12-second run. The model received the full 59 923-byte prompt but
    //   exited without running any tools, presumably because CLAUDE.md
    //   "Completed" sections convinced it all work was done.
    //
    //   C0 in the quality mandate now explicitly forbids this response.
    //   These tests document the failure path so a regression is caught early.
    // ──────────────────────────────────────────────────────────────────────────

    /// The exact 13-byte output observed from manager-0 in production must be
    /// rejected as an empty proposal, not silently treated as success.
    #[test]
    fn parse_exact_production_empty_output_returns_error_empty_proposal() {
        let output = r#"{"tasks": []}"#; // 13 bytes — exactly what manager-0 produced
        let result = ManagerOutputParser::parse(output, manager(), 11_829, &i18n());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("error-empty-proposal"),
            "Expected error-empty-proposal, got: {err}"
        );
    }

    /// Variants of empty-tasks the model might produce.
    #[test]
    fn parse_empty_tasks_variants_all_return_error() {
        let variants = [
            r#"{"tasks":[]}"#,
            r#"{ "tasks": [] }"#,
            "{\n  \"tasks\": []\n}",
        ];
        for variant in variants {
            let result = ManagerOutputParser::parse(variant, manager(), 100, &i18n());
            assert!(
                result.is_err(),
                "Expected error for empty-tasks variant: {variant}"
            );
        }
    }

    /// When the model wraps its empty response in a code fence (also observed
    /// in some providers), the fence-extraction path must still surface the
    /// empty-tasks error — not silently succeed with zero tasks.
    #[test]
    fn parse_code_fenced_empty_tasks_returns_error() {
        let output = "```json\n{\"tasks\": []}\n```";
        let result = ManagerOutputParser::parse(output, manager(), 500, &i18n());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("error-empty-proposal"),
            "Fenced empty proposal should surface error-empty-proposal, got: {err}"
        );
    }

    /// A valid single-task proposal must succeed (baseline).
    #[test]
    fn parse_valid_single_task_succeeds() {
        let output = r#"{
            "tasks": [{
                "title": "Add feature X",
                "description": "Implement feature X. Acceptance: tests pass.",
                "complexity": 5,
                "priority": 1,
                "dependencies": [],
                "affected_files": ["src/lib.rs"]
            }]
        }"#;
        let result = ManagerOutputParser::parse(output, manager(), 5000, &i18n());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().tasks.len(), 1);
    }

    /// extract_json on the exact production string must return Some — i.e., the
    /// bare-JSON extractor does find the `{"tasks": []}` object.  This confirms
    /// the failure is in `from_json` (empty-tasks check), not in extraction.
    #[test]
    fn extract_json_finds_empty_tasks_object() {
        let result = ManagerOutputParser::extract_json(r#"{"tasks": []}"#);
        assert!(
            result.is_some(),
            "extract_json must find the tasks object even when empty"
        );
        assert_eq!(result.unwrap(), r#"{"tasks": []}"#);
    }

    /// Malformed JSON (e.g., truncated output) must not panic and must return
    /// an appropriate error.
    #[test]
    fn parse_truncated_json_returns_error() {
        let output = r#"{"tasks": [{"title": "Do something""#; // truncated
        let result = ManagerOutputParser::parse(output, manager(), 100, &i18n());
        assert!(result.is_err());
    }

    /// Output that is completely empty (zero bytes) must not panic.
    #[test]
    fn parse_empty_string_returns_error() {
        let result = ManagerOutputParser::parse("", manager(), 0, &i18n());
        assert!(result.is_err());
    }
}

/// Find the position of the closing brace that matches the opening brace at position 0.
///
/// Handles nested braces, strings (with escaped quotes), and ignores braces inside strings.
fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                escape_next = true;
            }
            '"' => {
                in_string = !in_string;
            }
            '{' if !in_string => {
                depth += 1;
            }
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

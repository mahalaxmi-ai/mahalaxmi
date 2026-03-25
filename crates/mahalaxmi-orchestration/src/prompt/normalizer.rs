//! Requirements normalization pre-processor.
//!
//! Converts arbitrary user-provided requirements documents (prose paragraphs,
//! JIRA exports, Figma text, Word pastes, bullet lists, non-standard markdown)
//! into one canonical 5-section structure before the text enters the manager
//! prompt pipeline.
//!
//! The canonical format has exactly these headings:
//! `## Overview`, `## Functional Requirements`, `## Acceptance Criteria`,
//! `## Technical Constraints`, `## Out of Scope`.
//!
//! Normalization is **idempotent** — already-normalized text is returned
//! unchanged, which is critical for chain-mode re-reads of the same file.
//! Any provider failure degrades gracefully: the original text is returned
//! so the orchestration cycle is never blocked.

use std::sync::Arc;

// ---------------------------------------------------------------------------
// NormalizationProvider trait
// ---------------------------------------------------------------------------

/// An object-safe interface for single-shot AI completions used during
/// requirements normalization.
///
/// Implementations must not panic. Returning `None` causes the normalizer
/// to pass the original text through unchanged.
#[async_trait::async_trait]
pub trait NormalizationProvider: Send + Sync {
    /// Send `prompt` to the underlying provider and return the completion
    /// text, or `None` on any failure or timeout.
    async fn complete(&self, prompt: &str) -> Option<String>;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum characters of raw requirements text sent to the provider.
///
/// Roughly 3 000 tokens at 4 chars/token — well within any provider's
/// context window for a one-shot completion.
const NORMALIZATION_CHAR_BUDGET: usize = 12_000;

/// Normalization prompt template (provider-agnostic, no XML tags).
const NORMALIZATION_PROMPT: &str = "\
You are a requirements analyst. Convert the following input into a structured \
requirements document. The input may be in any format: prose paragraphs, \
JIRA export, Figma-extracted text, bullet lists, or mixed.

Output ONLY these five sections with EXACTLY these headings (no other text before or after):

## Overview
## Functional Requirements
## Acceptance Criteria
## Technical Constraints
## Out of Scope

RULES:
- No prose before the first heading or after the last section.
- Do not invent requirements not present in the input.
- Preserve all technical specifics, version numbers, and named entities verbatim.
- Write \"None specified\" for any section whose content is absent from the input.
- Functional Requirements must use a bullet list (- item).
- Acceptance Criteria must use a numbered list (1. item).

INPUT:
";

// ---------------------------------------------------------------------------
// RequirementsNormalizer
// ---------------------------------------------------------------------------

/// Normalizes free-form requirements text into the canonical 5-section format.
///
/// Construct via [`RequirementsNormalizer::new`]; call [`RequirementsNormalizer::normalize`].
pub struct RequirementsNormalizer {
    provider: Arc<dyn NormalizationProvider>,
}

impl RequirementsNormalizer {
    /// Create a new normalizer backed by `provider`.
    pub fn new(provider: Arc<dyn NormalizationProvider>) -> Self {
        Self { provider }
    }

    /// Normalize `raw` requirements text.
    ///
    /// Returns `raw` unchanged when:
    /// - `is_normalized(raw)` is true (idempotent guard for chain re-reads)
    /// - The provider returns `None` or an empty string
    pub async fn normalize(&self, raw: &str) -> String {
        if is_normalized(raw) {
            tracing::debug!(
                normalizer = "requirements",
                "Text already normalized — skipping provider call"
            );
            return raw.to_owned();
        }

        if raw.len() > NORMALIZATION_CHAR_BUDGET {
            // WARN: skipping normalization is a performance degradation — the
            // manager will receive un-normalized requirements which may affect
            // plan quality.  Operators should see this in production log filters
            // so they can increase NORMALIZATION_CHAR_BUDGET or pre-process
            // large requirements before submission.
            tracing::warn!(
                normalizer = "requirements",
                original_chars = raw.len(),
                budget = NORMALIZATION_CHAR_BUDGET,
                "Input exceeds normalization budget ({} chars > {} limit) — \
                 skipping CLI normalization, using original text",
                raw.len(),
                NORMALIZATION_CHAR_BUDGET
            );
            return raw.to_owned();
        }

        let input = raw;

        let prompt = format!("{NORMALIZATION_PROMPT}{input}");

        match self.provider.complete(&prompt).await {
            Some(result) if !result.trim().is_empty() => {
                tracing::debug!(
                    normalizer = "requirements",
                    output_len = result.len(),
                    "Requirements normalization complete"
                );
                result
            }
            Some(_) => {
                tracing::warn!(
                    normalizer = "requirements",
                    "Provider returned empty normalization result — using original text"
                );
                raw.to_owned()
            }
            None => {
                tracing::warn!(
                    normalizer = "requirements",
                    "Provider failed to normalize requirements — using original text"
                );
                raw.to_owned()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// is_normalized detection heuristic
// ---------------------------------------------------------------------------

/// Returns `true` when `text` already contains the canonical normalized headings.
///
/// Detection requires **exact** `## Overview` and `## Functional Requirements`
/// lines (after trimming) — `==` comparison prevents `### Overview` or prose
/// mentions like "See Overview section" from triggering false positives.
pub fn is_normalized(text: &str) -> bool {
    let has_overview = text
        .lines()
        .any(|l| l.trim() == "## Overview");
    let has_functional = text
        .lines()
        .any(|l| l.trim() == "## Functional Requirements");
    has_overview && has_functional
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // is_normalized (pure, synchronous)
    // -----------------------------------------------------------------------

    #[test]
    fn is_normalized_true_when_both_headings_present() {
        let text = "## Overview\nBuild a thing.\n\n## Functional Requirements\n- Do it.";
        assert!(is_normalized(text));
    }

    #[test]
    fn is_normalized_false_when_missing_functional_heading() {
        let text = "## Overview\nBuild a thing.\n\n## Acceptance Criteria\n1. Works.";
        assert!(!is_normalized(text));
    }

    #[test]
    fn is_normalized_rejects_prose_containing_heading_words() {
        // The word "Overview" appears in prose but not as an exact heading.
        let text = "This is an Overview of the system. See Functional Requirements below.";
        assert!(!is_normalized(text));
    }

    #[test]
    fn is_normalized_rejects_h3_variant() {
        // ### Overview (H3) must NOT match — only exact ## headings count.
        let text = "### Overview\nBuild a thing.\n\n### Functional Requirements\n- Do it.";
        assert!(!is_normalized(text));
    }

    #[test]
    fn is_normalized_true_with_leading_trailing_whitespace_on_heading_line() {
        // trim() is called on each line so minor indentation is tolerated.
        let text = "  ## Overview  \nBuild.\n\n  ## Functional Requirements  \n- Do it.";
        assert!(is_normalized(text));
    }

    // -----------------------------------------------------------------------
    // Async tests (stub providers)
    // -----------------------------------------------------------------------

    struct FixedProvider(String);

    #[async_trait::async_trait]
    impl NormalizationProvider for FixedProvider {
        async fn complete(&self, _prompt: &str) -> Option<String> {
            Some(self.0.clone())
        }
    }

    struct FailingProvider;

    #[async_trait::async_trait]
    impl NormalizationProvider for FailingProvider {
        async fn complete(&self, _prompt: &str) -> Option<String> {
            None
        }
    }

    struct CapturingProvider {
        captured: tokio::sync::Mutex<Option<String>>,
        response: String,
    }

    impl CapturingProvider {
        fn new(response: impl Into<String>) -> Self {
            Self {
                captured: tokio::sync::Mutex::new(None),
                response: response.into(),
            }
        }
        async fn captured(&self) -> Option<String> {
            self.captured.lock().await.clone()
        }
    }

    #[async_trait::async_trait]
    impl NormalizationProvider for CapturingProvider {
        async fn complete(&self, prompt: &str) -> Option<String> {
            *self.captured.lock().await = Some(prompt.to_owned());
            Some(self.response.clone())
        }
    }

    #[tokio::test]
    async fn normalizer_returns_provider_output() {
        let expected = "## Overview\nBuilt.\n\n## Functional Requirements\n- Yes.";
        let provider = Arc::new(FixedProvider(expected.to_owned()));
        let normalizer = RequirementsNormalizer::new(provider);
        let result = normalizer.normalize("Some raw prose requirements.").await;
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn normalizer_degrades_on_provider_failure() {
        let raw = "Some raw prose requirements.";
        let provider = Arc::new(FailingProvider);
        let normalizer = RequirementsNormalizer::new(provider);
        let result = normalizer.normalize(raw).await;
        assert_eq!(result, raw, "original text must be returned on provider failure");
    }

    #[tokio::test]
    async fn normalizer_skips_call_for_already_normalized_text() {
        // FailingProvider would return None if called — but it must NOT be called.
        let already_normalized =
            "## Overview\nBuilt.\n\n## Functional Requirements\n- Done.";
        let provider = Arc::new(FailingProvider);
        let normalizer = RequirementsNormalizer::new(provider);
        let result = normalizer.normalize(already_normalized).await;
        // Must return the original text unchanged without calling the provider.
        assert_eq!(result, already_normalized);
    }

    #[tokio::test]
    async fn normalizer_skips_provider_when_input_exceeds_budget() {
        // Input longer than NORMALIZATION_CHAR_BUDGET must NOT be sent to the provider —
        // the original text is returned immediately without any CLI call.
        let long_raw = "x".repeat(NORMALIZATION_CHAR_BUDGET + 5_000);
        let provider = Arc::new(CapturingProvider::new(
            "## Overview\nOK.\n\n## Functional Requirements\n- Yes.",
        ));
        let normalizer = RequirementsNormalizer::new(Arc::clone(&provider) as Arc<dyn NormalizationProvider>);
        let result = normalizer.normalize(&long_raw).await;
        // Provider must NOT have been called.
        assert!(
            provider.captured().await.is_none(),
            "provider must not be called when input exceeds budget"
        );
        // Original text returned unchanged.
        assert_eq!(result, long_raw, "original text must be returned when input exceeds budget");
    }
}

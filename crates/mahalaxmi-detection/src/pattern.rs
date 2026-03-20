use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::MatchType;
use mahalaxmi_core::MahalaxmiResult;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// A detection pattern specification (serializable configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPattern {
    /// The pattern string to match against.
    pub pattern: String,
    /// How to interpret the pattern.
    pub match_type: MatchType,
    /// Whether matching is case-sensitive.
    pub case_sensitive: bool,
}

impl DetectionPattern {
    /// Create a new detection pattern.
    pub fn new(pattern: impl Into<String>, match_type: MatchType) -> Self {
        Self {
            pattern: pattern.into(),
            match_type,
            case_sensitive: false,
        }
    }

    /// Set case sensitivity.
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Compile this pattern into an efficient matcher.
    pub fn compile(&self, i18n: &I18nService) -> MahalaxmiResult<CompiledPattern> {
        CompiledPattern::compile(self, i18n)
    }
}

/// A compiled detection pattern ready for efficient matching.
///
/// For Regex patterns, the regex is pre-compiled. For string patterns,
/// the case-insensitive version of the pattern is pre-computed.
pub struct CompiledPattern {
    /// The original pattern configuration.
    source: DetectionPattern,
    /// Compiled regex (only for Regex match type).
    regex: Option<Regex>,
    /// Lowercased pattern (for case-insensitive non-regex matching).
    lower_pattern: String,
}

impl CompiledPattern {
    /// Compile a detection pattern.
    pub fn compile(pattern: &DetectionPattern, i18n: &I18nService) -> MahalaxmiResult<Self> {
        let regex = if pattern.match_type == MatchType::Regex {
            let regex_pattern = if pattern.case_sensitive {
                pattern.pattern.clone()
            } else {
                format!("(?i){}", pattern.pattern)
            };
            Some(Regex::new(&regex_pattern).map_err(|e| {
                MahalaxmiError::detection(
                    i18n,
                    keys::detection::RULE_COMPILE_FAILED,
                    &[("reason", &e.to_string())],
                )
            })?)
        } else {
            None
        };

        let lower_pattern = pattern.pattern.to_lowercase();

        Ok(Self {
            source: pattern.clone(),
            regex,
            lower_pattern,
        })
    }

    /// Test if this pattern matches the given text.
    pub fn matches(&self, text: &str) -> bool {
        match self.source.match_type {
            MatchType::Exact => {
                if self.source.case_sensitive {
                    text == self.source.pattern
                } else {
                    text.to_lowercase() == self.lower_pattern
                }
            }
            MatchType::Contains => {
                if self.source.case_sensitive {
                    text.contains(&self.source.pattern)
                } else {
                    text.to_lowercase().contains(&self.lower_pattern)
                }
            }
            MatchType::StartsWith => {
                if self.source.case_sensitive {
                    text.starts_with(&self.source.pattern)
                } else {
                    text.to_lowercase().starts_with(&self.lower_pattern)
                }
            }
            MatchType::EndsWith => {
                if self.source.case_sensitive {
                    text.ends_with(&self.source.pattern)
                } else {
                    text.to_lowercase().ends_with(&self.lower_pattern)
                }
            }
            MatchType::Regex => self
                .regex
                .as_ref()
                .map(|r| r.is_match(text))
                .unwrap_or(false),
        }
    }

    /// Get the source pattern configuration.
    pub fn source(&self) -> &DetectionPattern {
        &self.source
    }
}

impl std::fmt::Debug for CompiledPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledPattern")
            .field("source", &self.source)
            .field("has_regex", &self.regex.is_some())
            .finish()
    }
}

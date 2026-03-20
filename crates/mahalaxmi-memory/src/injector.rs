//! Memory injection into worker contexts.
//!
//! The injector selects relevant memories from a store, formats them, and produces
//! a string that can be prepended to worker prompts to share cross-agent knowledge.

use crate::entry::MemoryEntry;
use crate::query::{MemoryQuery, QueryOrder};
use crate::store::MemoryStore;
use crate::types::MemoryType;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Output format for injected memories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InjectionFormat {
    /// Markdown format with headers and lists.
    Markdown,
    /// Plain text with simple separators.
    PlainText,
    /// XML-tagged format.
    Xml,
}

impl InjectionFormat {
    /// Returns the format as a static string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::PlainText => "plaintext",
            Self::Xml => "xml",
        }
    }

    /// Parse from a string, defaulting to Markdown for unknown values.
    pub fn from_config_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "plaintext" | "plain_text" | "plain" | "text" => Self::PlainText,
            "xml" => Self::Xml,
            _ => Self::Markdown,
        }
    }
}

impl fmt::Display for InjectionFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Configuration for the memory injector.
#[derive(Debug, Clone)]
pub struct InjectorConfig {
    /// Maximum tokens to include in the injection.
    pub max_tokens: usize,
    /// Memory types to include.
    pub include_types: Vec<MemoryType>,
    /// Minimum confidence threshold.
    pub min_confidence: f64,
    /// Output format.
    pub format: InjectionFormat,
}

impl Default for InjectorConfig {
    fn default() -> Self {
        Self {
            max_tokens: 1024,
            include_types: vec![
                MemoryType::CodebaseFact,
                MemoryType::Convention,
                MemoryType::Decision,
                MemoryType::Warning,
            ],
            min_confidence: 0.5,
            format: InjectionFormat::Markdown,
        }
    }
}

/// Injects relevant memories from a store into a formatted string for worker contexts.
pub struct MemoryInjector {
    config: InjectorConfig,
}

impl MemoryInjector {
    /// Create a new injector with the given configuration.
    pub fn new(config: InjectorConfig) -> Self {
        Self { config }
    }

    /// Estimate the number of tokens in a string (approximately len/4, minimum 1).
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() / 4).max(1)
    }

    /// Format a single memory entry according to the configured format.
    pub fn format_entry(&self, entry: &MemoryEntry) -> String {
        match self.config.format {
            InjectionFormat::Markdown => {
                format!(
                    "### {} [{}] (confidence: {:.1})\n{}\n",
                    entry.title, entry.memory_type, entry.confidence, entry.content
                )
            }
            InjectionFormat::PlainText => {
                format!(
                    "[{}] {} (confidence: {:.1})\n{}\n---\n",
                    entry.memory_type, entry.title, entry.confidence, entry.content
                )
            }
            InjectionFormat::Xml => {
                format!(
                    "<memory type=\"{}\" confidence=\"{:.1}\">\n  <title>{}</title>\n  <content>{}</content>\n</memory>\n",
                    entry.memory_type, entry.confidence, entry.title, entry.content
                )
            }
        }
    }

    /// Inject memories from the store into a formatted string.
    ///
    /// If a query is provided, it's used as an additional filter.
    /// Entries are sorted by confidence (desc) then creation time (desc).
    /// Accumulation stops when the token budget is exhausted.
    pub fn inject(&self, store: &MemoryStore, query: Option<&MemoryQuery>) -> String {
        let base_query = MemoryQuery::new()
            .with_types(self.config.include_types.clone())
            .with_min_confidence(self.config.min_confidence)
            .order_by(QueryOrder::Relevance);

        let candidates = store.query(&base_query);

        let filtered: Vec<&&MemoryEntry> = if let Some(q) = query {
            candidates.iter().filter(|e| q.matches(e)).collect()
        } else {
            candidates.iter().collect()
        };

        let mut output = String::new();
        let mut token_count = 0;

        for entry in filtered {
            let formatted = self.format_entry(entry);
            let entry_tokens = Self::estimate_tokens(&formatted);

            if token_count + entry_tokens > self.config.max_tokens {
                break;
            }

            output.push_str(&formatted);
            token_count += entry_tokens;
        }

        output
    }

    /// Count how many entries would be injected (respecting filters and token budget).
    pub fn inject_count(&self, store: &MemoryStore, query: Option<&MemoryQuery>) -> usize {
        let base_query = MemoryQuery::new()
            .with_types(self.config.include_types.clone())
            .with_min_confidence(self.config.min_confidence)
            .order_by(QueryOrder::Relevance);

        let candidates = store.query(&base_query);

        let filtered: Vec<&&MemoryEntry> = if let Some(q) = query {
            candidates.iter().filter(|e| q.matches(e)).collect()
        } else {
            candidates.iter().collect()
        };

        let mut count = 0;
        let mut token_count = 0;

        for entry in filtered {
            let formatted = self.format_entry(entry);
            let entry_tokens = Self::estimate_tokens(&formatted);

            if token_count + entry_tokens > self.config.max_tokens {
                break;
            }

            count += 1;
            token_count += entry_tokens;
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injection_format_from_config_str() {
        assert_eq!(
            InjectionFormat::from_config_str("markdown"),
            InjectionFormat::Markdown
        );
        assert_eq!(
            InjectionFormat::from_config_str("plaintext"),
            InjectionFormat::PlainText
        );
        assert_eq!(
            InjectionFormat::from_config_str("xml"),
            InjectionFormat::Xml
        );
        assert_eq!(
            InjectionFormat::from_config_str("unknown"),
            InjectionFormat::Markdown
        );
    }

    #[test]
    fn estimate_tokens_basic() {
        assert_eq!(MemoryInjector::estimate_tokens(""), 1);
        assert_eq!(MemoryInjector::estimate_tokens("abcd"), 1);
        assert_eq!(MemoryInjector::estimate_tokens("abcdefgh"), 2);
    }

    #[test]
    fn injection_format_display() {
        assert_eq!(format!("{}", InjectionFormat::Markdown), "markdown");
        assert_eq!(format!("{}", InjectionFormat::PlainText), "plaintext");
        assert_eq!(format!("{}", InjectionFormat::Xml), "xml");
    }
}

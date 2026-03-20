//! Error type hierarchy for the Mahalaxmi system.
//!
//! All error messages are resolved through the i18n service. No hardcoded English strings.
//! Each domain-specific variant carries both the translated message and the i18n key.

use crate::i18n::I18nService;
use thiserror::Error;

/// Top-level error type for the Mahalaxmi system.
///
/// Each domain-specific variant carries:
/// - `message`: The resolved translated string (for display)
/// - `i18n_key`: The i18n key used to produce the message (for programmatic access)
#[derive(Debug, Error)]
pub enum MahalaxmiError {
    /// Configuration error.
    #[error("{message}")]
    Config {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// PTY/terminal error.
    #[error("{message}")]
    Pty {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// AI provider error.
    #[error("{message}")]
    Provider {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// Orchestration error.
    #[error("{message}")]
    Orchestration {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// Template error.
    #[error("{message}")]
    Template {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// License error.
    #[error("{message}")]
    License {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// Platform error.
    #[error("{message}")]
    Platform {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// Detection error.
    #[error("{message}")]
    Detection {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// Memory error.
    #[error("{message}")]
    Memory {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// Indexing error.
    #[error("{message}")]
    Indexing {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// I18n error.
    #[error("{message}")]
    I18n {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// MCP server error.
    #[error("{message}")]
    Mcp {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// Knowledge graph error.
    #[error("{message}")]
    Graph {
        /// Translated error message for display.
        message: String,
        /// The i18n key used to produce the message.
        i18n_key: String,
    },

    /// Standard I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// TOML parse error.
    #[error(transparent)]
    TomlParse(#[from] toml::de::Error),

    /// Cycle cost exceeded the configured budget threshold.
    #[error("cycle cost ${cost_usd:.4} exceeded budget ${limit_usd:.4}")]
    BudgetExceeded {
        /// Actual estimated cost at the time of the limit check.
        cost_usd: f64,
        /// Configured budget that was reached.
        limit_usd: f64,
    },

    /// Provider subscription session expired or not authenticated.
    ///
    /// Returned when a CLI-login provider's session cannot be verified
    /// before worker spawn. The UI should surface this as a dismissible
    /// banner with a "Re-authenticate" action, not a generic failure.
    #[error("Auth expired for {provider}: {message}")]
    AuthExpired {
        /// The provider whose session is expired (e.g. "Claude Code").
        provider: String,
        /// Human-readable explanation and recovery instruction.
        message: String,
    },
}

impl MahalaxmiError {
    /// Create a Config error with translated message.
    pub fn config(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Config {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create a Pty error with translated message.
    pub fn pty(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Pty {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create a Provider error with translated message.
    pub fn provider(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Provider {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create an Orchestration error with translated message.
    pub fn orchestration(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Orchestration {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create a Template error with translated message.
    pub fn template(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Template {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create a License error with translated message.
    pub fn license(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::License {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create a Platform error with translated message.
    pub fn platform(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Platform {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create a Detection error with translated message.
    pub fn detection(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Detection {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create a Memory error with translated message.
    pub fn memory(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Memory {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create an Indexing error with translated message.
    pub fn indexing(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Indexing {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create an I18n error with translated message.
    pub fn i18n(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::I18n {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create an MCP error with translated message.
    pub fn mcp(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Mcp {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Create a Graph error with translated message.
    pub fn graph(i18n: &I18nService, key: &str, args: &[(&str, &str)]) -> Self {
        Self::Graph {
            message: i18n.translate(key, args),
            i18n_key: key.to_owned(),
        }
    }

    /// Creates a threshold-failure validation error without requiring an i18n service.
    ///
    /// Used when a ValidationVerdict does not meet the configured AcceptanceThreshold.
    pub fn validation_threshold(message: String) -> Self {
        Self::Orchestration {
            message,
            i18n_key: "error.validation_threshold_failed".to_owned(),
        }
    }

    /// Create a `BudgetExceeded` error with the given cost and limit values.
    pub fn budget_exceeded(cost_usd: f64, limit_usd: f64) -> Self {
        Self::BudgetExceeded {
            cost_usd,
            limit_usd,
        }
    }

    /// Get the i18n key for this error, if it has one.
    ///
    /// Returns `None` for `Io` and `TomlParse` variants which wrap external errors.
    pub fn i18n_key(&self) -> Option<&str> {
        match self {
            Self::Config { i18n_key, .. }
            | Self::Pty { i18n_key, .. }
            | Self::Provider { i18n_key, .. }
            | Self::Orchestration { i18n_key, .. }
            | Self::Template { i18n_key, .. }
            | Self::License { i18n_key, .. }
            | Self::Platform { i18n_key, .. }
            | Self::Detection { i18n_key, .. }
            | Self::Memory { i18n_key, .. }
            | Self::Indexing { i18n_key, .. }
            | Self::I18n { i18n_key, .. }
            | Self::Mcp { i18n_key, .. }
            | Self::Graph { i18n_key, .. } => Some(i18n_key),
            Self::BudgetExceeded { .. }
            | Self::AuthExpired { .. }
            | Self::Io(_)
            | Self::TomlParse(_) => None,
        }
    }

    /// Returns true if this is a configuration-related error.
    pub fn is_config(&self) -> bool {
        matches!(self, Self::Config { .. })
    }

    /// Returns true if this is a memory-related error.
    pub fn is_memory(&self) -> bool {
        matches!(self, Self::Memory { .. })
    }

    /// Returns true if this is an indexing-related error.
    pub fn is_indexing(&self) -> bool {
        matches!(self, Self::Indexing { .. })
    }

    /// Returns true if this is an MCP-related error.
    pub fn is_mcp(&self) -> bool {
        matches!(self, Self::Mcp { .. })
    }

    /// Returns true if this is a graph-related error.
    pub fn is_graph(&self) -> bool {
        matches!(self, Self::Graph { .. })
    }

    /// Returns true if this is an I/O error.
    pub fn is_io(&self) -> bool {
        matches!(self, Self::Io(_))
    }

    /// Returns the error category as a static string.
    ///
    /// Useful for logging, UI display, and error reporting in higher layers.
    pub fn category(&self) -> &'static str {
        match self {
            Self::Config { .. } => "config",
            Self::Pty { .. } => "pty",
            Self::Provider { .. } => "provider",
            Self::Orchestration { .. } => "orchestration",
            Self::Template { .. } => "template",
            Self::License { .. } => "license",
            Self::Platform { .. } => "platform",
            Self::Detection { .. } => "detection",
            Self::Memory { .. } => "memory",
            Self::Indexing { .. } => "indexing",
            Self::I18n { .. } => "i18n",
            Self::Mcp { .. } => "mcp",
            Self::Graph { .. } => "graph",
            Self::BudgetExceeded { .. } => "orchestration",
            Self::AuthExpired { .. } => "provider",
            Self::Io(_) => "io",
            Self::TomlParse(_) => "toml-parse",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_exceeded_display_format() {
        let err = MahalaxmiError::budget_exceeded(0.05, 0.04);
        assert_eq!(
            err.to_string(),
            "cycle cost $0.0500 exceeded budget $0.0400"
        );
    }
}

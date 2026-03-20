use mahalaxmi_core::types::{ActionType, MatchType};
use serde::{Deserialize, Serialize};

use crate::pattern::DetectionPattern;

/// A detection rule that maps terminal output patterns to actions.
///
/// Rules are evaluated by priority (lower number = higher priority).
/// Multiple patterns per rule — any pattern match triggers the rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    /// Human-readable rule name.
    pub name: String,
    /// Patterns to match against (any match triggers the rule).
    pub patterns: Vec<DetectionPattern>,
    /// Priority (lower number = higher priority).
    pub priority: u32,
    /// Action to take when the rule matches.
    pub action: ActionType,
    /// Text to send for SendText/SendTextWithEnter actions.
    pub response_text: Option<String>,
    /// Only apply for specific provider IDs (None = all providers).
    pub provider_filter: Option<Vec<String>>,
    /// Only apply for a specific terminal role (None = all roles).
    pub role_filter: Option<String>,
    /// Cooldown in milliseconds to prevent rapid re-triggering.
    pub cooldown_ms: Option<u64>,
    /// Whether this rule is active.
    pub enabled: bool,
}

impl DetectionRule {
    /// Create a new detection rule with the given name and action.
    pub fn new(name: impl Into<String>, action: ActionType) -> Self {
        Self {
            name: name.into(),
            patterns: Vec::new(),
            priority: 100,
            action,
            response_text: None,
            provider_filter: None,
            role_filter: None,
            cooldown_ms: None,
            enabled: true,
        }
    }

    /// Add a pattern to this rule.
    pub fn with_pattern(mut self, pattern: DetectionPattern) -> Self {
        self.patterns.push(pattern);
        self
    }

    /// Add a Contains pattern (convenience helper).
    pub fn with_contains_pattern(self, pattern: impl Into<String>) -> Self {
        self.with_pattern(DetectionPattern::new(pattern, MatchType::Contains))
    }

    /// Add a Regex pattern (convenience helper).
    pub fn with_regex_pattern(self, pattern: impl Into<String>) -> Self {
        self.with_pattern(DetectionPattern::new(pattern, MatchType::Regex))
    }

    /// Set the priority.
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set the response text.
    pub fn with_response_text(mut self, text: impl Into<String>) -> Self {
        self.response_text = Some(text.into());
        self
    }

    /// Set the provider filter.
    pub fn with_provider_filter(mut self, providers: Vec<String>) -> Self {
        self.provider_filter = Some(providers);
        self
    }

    /// Set the role filter.
    pub fn with_role_filter(mut self, role: impl Into<String>) -> Self {
        self.role_filter = Some(role.into());
        self
    }

    /// Set the cooldown.
    pub fn with_cooldown_ms(mut self, ms: u64) -> Self {
        self.cooldown_ms = Some(ms);
        self
    }

    /// Enable or disable the rule.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

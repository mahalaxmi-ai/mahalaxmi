use mahalaxmi_core::types::ActionType;
use serde::{Deserialize, Serialize};

/// The result of evaluating detection rules against terminal output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Whether any rule matched.
    pub matched: bool,
    /// Name of the matched rule (if any).
    pub matched_rule_name: Option<String>,
    /// The text that was matched.
    pub matched_text: Option<String>,
    /// The action to take.
    pub action: ActionType,
    /// Response text to send (for SendText actions).
    pub response_text: Option<String>,
    /// Priority of the matched rule.
    pub priority: u32,
}

impl DetectionResult {
    /// Create a result indicating no rule matched.
    pub fn no_match() -> Self {
        Self {
            matched: false,
            matched_rule_name: None,
            matched_text: None,
            action: ActionType::ContinueProcessing,
            response_text: None,
            priority: u32::MAX,
        }
    }

    /// Create a result indicating a rule matched.
    pub fn matched(
        rule_name: impl Into<String>,
        matched_text: impl Into<String>,
        action: ActionType,
        response_text: Option<String>,
        priority: u32,
    ) -> Self {
        Self {
            matched: true,
            matched_rule_name: Some(rule_name.into()),
            matched_text: Some(matched_text.into()),
            action,
            response_text,
            priority,
        }
    }
}

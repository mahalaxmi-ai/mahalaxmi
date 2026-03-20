//! User-facing error messages — translates internal errors into actionable guidance.
//!
//! When something goes wrong, users should see messages that:
//! 1. Explain what happened in plain language
//! 2. Suggest what they can do to fix it
//! 3. Avoid exposing internal implementation details
//!
//! This module provides `UserMessage` — a structured error response for the frontend.
//! Internal errors are mapped to user-friendly messages with optional recovery actions.

use serde::{Deserialize, Serialize};

/// Severity level for a user-facing message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageSeverity {
    /// Informational — operation succeeded but with a note.
    Info,
    /// Warning — operation succeeded but something isn't ideal.
    Warning,
    /// Error — operation failed, user action needed.
    Error,
}

/// A user-facing message with optional recovery actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    /// Severity level.
    pub severity: MessageSeverity,
    /// Short, user-friendly title (e.g., "Provider not found").
    pub title: String,
    /// Detailed explanation of what happened.
    pub detail: String,
    /// Suggested actions the user can take to resolve the issue.
    pub actions: Vec<RecoveryAction>,
    /// Technical error code for support/debugging (e.g., "ERR_PROVIDER_NOT_FOUND").
    pub code: Option<String>,
}

/// A suggested action the user can take to resolve an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    /// Short label for the action (e.g., "Install provider").
    pub label: String,
    /// Detailed instruction or help text.
    pub description: String,
    /// Action type — helps the frontend render the right UI control.
    pub action_type: ActionType,
}

/// Type of recovery action — determines how the frontend renders it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Navigate to a page or section in the app.
    Navigate,
    /// Open an external URL (documentation, install page).
    OpenUrl,
    /// Retry the failed operation.
    Retry,
    /// Dismiss the message — no action needed.
    Dismiss,
}

impl UserMessage {
    /// Create an info message.
    pub fn info(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            severity: MessageSeverity::Info,
            title: title.into(),
            detail: detail.into(),
            actions: Vec::new(),
            code: None,
        }
    }

    /// Create a warning message.
    pub fn warning(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            severity: MessageSeverity::Warning,
            title: title.into(),
            detail: detail.into(),
            actions: Vec::new(),
            code: None,
        }
    }

    /// Create an error message.
    pub fn error(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            severity: MessageSeverity::Error,
            title: title.into(),
            detail: detail.into(),
            actions: Vec::new(),
            code: None,
        }
    }

    /// Add a recovery action to this message.
    pub fn with_action(mut self, action: RecoveryAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set the technical error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Common error message constructors
// ---------------------------------------------------------------------------

/// Provider is not installed on the system.
pub fn provider_not_installed(provider_name: &str, install_hint: &str) -> UserMessage {
    UserMessage::error(
        format!("{provider_name} is not installed"),
        format!(
            "Mahalaxmi needs {provider_name} to run AI tasks, but it wasn't found on your system."
        ),
    )
    .with_action(RecoveryAction {
        label: "Install it".to_string(),
        description: install_hint.to_string(),
        action_type: ActionType::OpenUrl,
    })
    .with_action(RecoveryAction {
        label: "Choose a different provider".to_string(),
        description: "Go to Settings to select another AI provider.".to_string(),
        action_type: ActionType::Navigate,
    })
    .with_code("ERR_PROVIDER_NOT_INSTALLED")
}

/// Provider credentials are missing or invalid.
pub fn provider_auth_failed(provider_name: &str) -> UserMessage {
    UserMessage::error(
        format!("{provider_name} authentication failed"),
        format!(
            "Your {provider_name} credentials are missing or expired. \
             Please re-authenticate to continue."
        ),
    )
    .with_action(RecoveryAction {
        label: "Configure credentials".to_string(),
        description: format!("Set up your {provider_name} API key or login."),
        action_type: ActionType::Navigate,
    })
    .with_code("ERR_PROVIDER_AUTH_FAILED")
}

/// No provider is configured at all.
pub fn no_provider_configured() -> UserMessage {
    UserMessage::error(
        "No AI provider configured",
        "Mahalaxmi needs at least one AI provider to work. \
         Set up Claude Code, OpenAI, or any supported provider to get started.",
    )
    .with_action(RecoveryAction {
        label: "Set up a provider".to_string(),
        description: "Go to the provider setup page.".to_string(),
        action_type: ActionType::Navigate,
    })
    .with_code("ERR_NO_PROVIDER")
}

/// License has expired.
pub fn license_expired() -> UserMessage {
    UserMessage::error(
        "License expired",
        "Your Mahalaxmi license has expired. Renew to continue using all features.",
    )
    .with_action(RecoveryAction {
        label: "Renew license".to_string(),
        description: "Visit the ThriveTech store to renew.".to_string(),
        action_type: ActionType::OpenUrl,
    })
    .with_code("ERR_LICENSE_EXPIRED")
}

/// License is required for a specific feature.
pub fn feature_requires_license(feature_name: &str) -> UserMessage {
    UserMessage::warning(
        format!("{feature_name} requires a license"),
        format!(
            "The {feature_name} feature is available on paid plans. \
             Start a free trial or purchase a license to access it."
        ),
    )
    .with_action(RecoveryAction {
        label: "Start free trial".to_string(),
        description: "Try all features free for 14 days.".to_string(),
        action_type: ActionType::Navigate,
    })
    .with_code("ERR_FEATURE_GATED")
}

/// Network/connectivity error when reaching the platform.
pub fn platform_unreachable() -> UserMessage {
    UserMessage::warning(
        "Can't reach the platform",
        "Mahalaxmi couldn't connect to the ThriveTech platform. \
         Your license is still valid offline, but some features may be limited.",
    )
    .with_action(RecoveryAction {
        label: "Retry".to_string(),
        description: "Try connecting again.".to_string(),
        action_type: ActionType::Retry,
    })
    .with_action(RecoveryAction {
        label: "Continue offline".to_string(),
        description: "Keep working — online features will resume when connected.".to_string(),
        action_type: ActionType::Dismiss,
    })
    .with_code("ERR_PLATFORM_UNREACHABLE")
}

/// Orchestration cycle failed to start.
pub fn cycle_start_failed(reason: &str) -> UserMessage {
    UserMessage::error(
        "Couldn't start orchestration",
        format!(
            "The orchestration cycle failed to start: {reason}. \
             Check your project settings and try again."
        ),
    )
    .with_action(RecoveryAction {
        label: "Check settings".to_string(),
        description: "Review your project and provider configuration.".to_string(),
        action_type: ActionType::Navigate,
    })
    .with_action(RecoveryAction {
        label: "Try again".to_string(),
        description: "Retry starting the orchestration cycle.".to_string(),
        action_type: ActionType::Retry,
    })
    .with_code("ERR_CYCLE_START_FAILED")
}

/// Terminal spawn failed.
pub fn terminal_spawn_failed(reason: &str) -> UserMessage {
    UserMessage::error(
        "Terminal failed to start",
        format!(
            "Couldn't open a terminal session: {reason}. \
             This may be a system configuration issue."
        ),
    )
    .with_action(RecoveryAction {
        label: "Check system settings".to_string(),
        description: "Verify your shell and terminal configuration.".to_string(),
        action_type: ActionType::Navigate,
    })
    .with_code("ERR_TERMINAL_SPAWN_FAILED")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_message_severity() {
        let msg = UserMessage::info("Test", "Detail");
        assert_eq!(msg.severity, MessageSeverity::Info);
        assert_eq!(msg.title, "Test");
    }

    #[test]
    fn warning_message_severity() {
        let msg = UserMessage::warning("Warn", "Detail");
        assert_eq!(msg.severity, MessageSeverity::Warning);
    }

    #[test]
    fn error_message_severity() {
        let msg = UserMessage::error("Err", "Detail");
        assert_eq!(msg.severity, MessageSeverity::Error);
    }

    #[test]
    fn with_action_adds_actions() {
        let msg = UserMessage::error("Test", "Detail")
            .with_action(RecoveryAction {
                label: "Fix it".to_string(),
                description: "Do the thing.".to_string(),
                action_type: ActionType::Retry,
            })
            .with_action(RecoveryAction {
                label: "Dismiss".to_string(),
                description: "Never mind.".to_string(),
                action_type: ActionType::Dismiss,
            });
        assert_eq!(msg.actions.len(), 2);
        assert_eq!(msg.actions[0].action_type, ActionType::Retry);
        assert_eq!(msg.actions[1].action_type, ActionType::Dismiss);
    }

    #[test]
    fn with_code_sets_error_code() {
        let msg = UserMessage::error("Test", "Detail").with_code("ERR_TEST");
        assert_eq!(msg.code, Some("ERR_TEST".to_string()));
    }

    #[test]
    fn provider_not_installed_message() {
        let msg = provider_not_installed("Claude Code", "npm install -g @anthropic-ai/claude-code");
        assert_eq!(msg.severity, MessageSeverity::Error);
        assert!(msg.title.contains("Claude Code"));
        assert!(msg.detail.contains("wasn't found"));
        assert_eq!(msg.actions.len(), 2);
        assert_eq!(msg.actions[0].action_type, ActionType::OpenUrl);
        assert_eq!(msg.code, Some("ERR_PROVIDER_NOT_INSTALLED".to_string()));
    }

    #[test]
    fn provider_auth_failed_message() {
        let msg = provider_auth_failed("OpenAI Foundry");
        assert!(msg.title.contains("authentication failed"));
        assert!(msg.detail.contains("credentials"));
        assert_eq!(msg.actions.len(), 1);
    }

    #[test]
    fn no_provider_message() {
        let msg = no_provider_configured();
        assert!(msg.title.contains("No AI provider"));
        assert_eq!(msg.code, Some("ERR_NO_PROVIDER".to_string()));
    }

    #[test]
    fn license_expired_message() {
        let msg = license_expired();
        assert_eq!(msg.severity, MessageSeverity::Error);
        assert!(msg.title.contains("expired"));
    }

    #[test]
    fn feature_requires_license_message() {
        let msg = feature_requires_license("GraphRAG");
        assert_eq!(msg.severity, MessageSeverity::Warning);
        assert!(msg.title.contains("GraphRAG"));
        assert!(msg.detail.contains("free trial"));
    }

    #[test]
    fn platform_unreachable_message() {
        let msg = platform_unreachable();
        assert_eq!(msg.severity, MessageSeverity::Warning);
        assert_eq!(msg.actions.len(), 2);
        assert!(msg.detail.contains("offline"));
    }

    #[test]
    fn cycle_start_failed_message() {
        let msg = cycle_start_failed("missing project root");
        assert!(msg.detail.contains("missing project root"));
        assert_eq!(msg.actions.len(), 2);
    }

    #[test]
    fn terminal_spawn_failed_message() {
        let msg = terminal_spawn_failed("shell not found");
        assert!(msg.detail.contains("shell not found"));
    }

    #[test]
    fn serialization_round_trip() {
        let msg = provider_not_installed("Test", "install test");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: UserMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, msg.title);
        assert_eq!(deserialized.actions.len(), 2);
        assert_eq!(deserialized.code, msg.code);
    }

    #[test]
    fn severity_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&MessageSeverity::Info).unwrap(),
            "\"info\""
        );
        assert_eq!(
            serde_json::to_string(&MessageSeverity::Warning).unwrap(),
            "\"warning\""
        );
        assert_eq!(
            serde_json::to_string(&MessageSeverity::Error).unwrap(),
            "\"error\""
        );
    }

    #[test]
    fn action_type_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&ActionType::Navigate).unwrap(),
            "\"navigate\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::OpenUrl).unwrap(),
            "\"open_url\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::Retry).unwrap(),
            "\"retry\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::Dismiss).unwrap(),
            "\"dismiss\""
        );
    }
}

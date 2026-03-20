//! Provider detection gate for the adversarial deliberation protocol.
//!
//! [`DeliberationGate::should_use_deliberation`] returns `true` only when all
//! of the following are satisfied:
//!
//! 1. `config.enabled == true`
//! 2. The provider name contains `"claude"` (case-insensitive)
//! 3. `ANTHROPIC_API_KEY` environment variable is set and non-empty
//! 4. `config.deliberation_turns >= 2`
//!
//! Any other combination causes graceful degradation to the standard PTY
//! manager path — no API calls are ever made.

use mahalaxmi_core::config::AdversarialDeliberationConfig;

/// Provider detection gate for the adversarial deliberation protocol.
///
/// All checks are pure (no I/O) except for reading `ANTHROPIC_API_KEY` from
/// the process environment.
pub struct DeliberationGate;

impl DeliberationGate {
    /// Return `true` when all four conditions are satisfied for deliberation.
    ///
    /// | Condition | Check |
    /// |-----------|-------|
    /// | `config.enabled` | must be `true` |
    /// | Provider is Claude | `provider_name` contains `"claude"` (case-insensitive) |
    /// | API key present | `ANTHROPIC_API_KEY` env var is non-empty |
    /// | Enough turns | `config.deliberation_turns >= 2` |
    ///
    /// `provider_name` is obtained by calling `provider.name()` at the call
    /// site, keeping the gate decoupled from the `AiProvider` trait object.
    /// This makes the gate extensible — callers add providers by passing
    /// different names without changing this signature.
    pub fn should_use_deliberation(
        provider_name: &str,
        config: &AdversarialDeliberationConfig,
    ) -> bool {
        if !config.enabled {
            return false;
        }

        // Allowlist check — case-insensitive so "Claude Code", "claude", etc. all match.
        if !provider_name.to_lowercase().contains("claude") {
            return false;
        }

        // API key must be present and non-empty.
        let key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        if key.trim().is_empty() {
            return false;
        }

        // Need at minimum Proposer + Synthesizer (turns >= 2).
        if config.deliberation_turns < 2 {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_config() -> AdversarialDeliberationConfig {
        AdversarialDeliberationConfig {
            enabled: true,
            deliberation_turns: 3,
            ..Default::default()
        }
    }

    #[test]
    fn test_gate_false_when_disabled() {
        let config = AdversarialDeliberationConfig {
            enabled: false,
            deliberation_turns: 3,
            ..Default::default()
        };
        assert!(!DeliberationGate::should_use_deliberation("Claude Code", &config));
    }

    #[test]
    fn test_gate_false_when_not_claude() {
        std::env::set_var("ANTHROPIC_API_KEY", "sk-test-key");
        let result = DeliberationGate::should_use_deliberation("Gemini", &enabled_config());
        std::env::remove_var("ANTHROPIC_API_KEY");
        assert!(!result, "Non-Claude provider should not trigger deliberation");
    }

    #[test]
    fn test_gate_false_when_no_api_key() {
        std::env::remove_var("ANTHROPIC_API_KEY");
        let result = DeliberationGate::should_use_deliberation("Claude Code", &enabled_config());
        assert!(!result, "Missing API key must prevent deliberation");
    }

    #[test]
    fn test_gate_true_when_all_conditions_met() {
        std::env::set_var("ANTHROPIC_API_KEY", "sk-ant-test-key-12345");
        let result = DeliberationGate::should_use_deliberation("Claude Code", &enabled_config());
        std::env::remove_var("ANTHROPIC_API_KEY");
        assert!(result, "All conditions met — gate should open");
    }

    #[test]
    fn test_gate_false_when_turns_less_than_2() {
        std::env::set_var("ANTHROPIC_API_KEY", "sk-ant-test-key");
        let config = AdversarialDeliberationConfig {
            enabled: true,
            deliberation_turns: 1,
            ..Default::default()
        };
        let result = DeliberationGate::should_use_deliberation("Claude Code", &config);
        std::env::remove_var("ANTHROPIC_API_KEY");
        assert!(!result, "turns < 2 must disable deliberation");
    }

    #[test]
    fn test_gate_case_insensitive_claude_match() {
        std::env::set_var("ANTHROPIC_API_KEY", "sk-ant-test");
        // "claude" lowercase should also match
        let result = DeliberationGate::should_use_deliberation("claude", &enabled_config());
        std::env::remove_var("ANTHROPIC_API_KEY");
        assert!(result, "lowercase 'claude' should match");
    }
}

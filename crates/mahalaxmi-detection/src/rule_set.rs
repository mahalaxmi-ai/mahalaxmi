use mahalaxmi_core::types::ActionType;

use crate::rule::DetectionRule;

/// Factory for built-in detection rule sets.
pub struct BuiltinRuleSets;

impl BuiltinRuleSets {
    /// Generic rules that apply to all AI providers.
    pub fn generic() -> Vec<DetectionRule> {
        vec![
            // Shell prompt detection (AI session ended, shell returned)
            DetectionRule::new("generic-shell-prompt", ActionType::CompleteWorkerCycle)
                .with_regex_pattern(r"\$\s*$")
                .with_priority(90)
                .with_cooldown_ms(5000),
            // Process crash detection
            DetectionRule::new("generic-segfault", ActionType::RestartSession)
                .with_contains_pattern("Segmentation fault")
                .with_priority(10),
            // Out of memory
            DetectionRule::new("generic-oom", ActionType::EscalateToManager)
                .with_contains_pattern("Out of memory")
                .with_contains_pattern("Cannot allocate memory")
                .with_priority(10),
            // Permission denied
            DetectionRule::new("generic-permission-denied", ActionType::EscalateToManager)
                .with_contains_pattern("Permission denied")
                .with_priority(50)
                .with_cooldown_ms(10000),
        ]
    }

    /// Rules specific to Claude Code.
    pub fn claude_code() -> Vec<DetectionRule> {
        vec![
            // Auto-confirm file write prompts
            DetectionRule::new("claude-auto-confirm", ActionType::SendTextWithEnter)
                .with_contains_pattern("Do you want to")
                .with_contains_pattern("Allow this action")
                .with_response_text("y")
                .with_priority(20)
                .with_provider_filter(vec!["claude-code".to_owned()])
                .with_cooldown_ms(1000),
            // Completion detection
            DetectionRule::new("claude-completion", ActionType::CompleteWorkerCycle)
                .with_regex_pattern(r"(?i)task\s+completed?")
                .with_priority(30)
                .with_provider_filter(vec!["claude-code".to_owned()]),
            // Error detection
            DetectionRule::new("claude-error", ActionType::EscalateToManager)
                .with_contains_pattern("Error:")
                .with_contains_pattern("error[E")
                .with_priority(40)
                .with_provider_filter(vec!["claude-code".to_owned()])
                .with_cooldown_ms(5000),
            // Cost warning
            DetectionRule::new("claude-cost-warning", ActionType::ContinueProcessing)
                .with_contains_pattern("token usage")
                .with_contains_pattern("cost warning")
                .with_priority(80)
                .with_provider_filter(vec!["claude-code".to_owned()])
                .with_cooldown_ms(30000),
        ]
    }

    /// Rules specific to OpenAI providers.
    pub fn openai() -> Vec<DetectionRule> {
        vec![
            // Completion detection
            DetectionRule::new("openai-completion", ActionType::CompleteWorkerCycle)
                .with_regex_pattern(r"(?i)completed?\s+successfully")
                .with_priority(30)
                .with_provider_filter(vec!["openai".to_owned()]),
            // Rate limit detection
            DetectionRule::new("openai-rate-limit", ActionType::WaitAndRetry)
                .with_contains_pattern("Rate limit exceeded")
                .with_contains_pattern("429")
                .with_priority(15)
                .with_provider_filter(vec!["openai".to_owned()])
                .with_cooldown_ms(60000),
            // Authentication error
            DetectionRule::new("openai-auth-error", ActionType::StopOrchestration)
                .with_contains_pattern("Invalid API key")
                .with_contains_pattern("Unauthorized")
                .with_priority(5)
                .with_provider_filter(vec!["openai".to_owned()]),
        ]
    }

    /// All default rule sets combined.
    pub fn all_defaults() -> Vec<DetectionRule> {
        let mut rules = Self::generic();
        rules.extend(Self::claude_code());
        rules.extend(Self::openai());
        rules
    }
}

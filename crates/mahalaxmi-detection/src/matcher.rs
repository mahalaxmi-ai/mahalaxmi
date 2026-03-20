use chrono::{DateTime, Utc};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use std::collections::HashMap;

use crate::pattern::CompiledPattern;
use crate::result::DetectionResult;
use crate::rule::DetectionRule;

/// Compiled rule with its pre-compiled patterns.
struct CompiledRule {
    rule: DetectionRule,
    patterns: Vec<CompiledPattern>,
}

/// The rule matcher evaluates terminal output against a set of compiled rules.
///
/// Features:
/// - Priority-based resolution (lower number = higher priority)
/// - Cooldown suppression (prevent rapid re-triggering)
/// - Provider and role filtering
pub struct RuleMatcher {
    /// Compiled rules sorted by priority.
    rules: Vec<CompiledRule>,
    /// Last trigger time per rule name (for cooldown tracking).
    last_triggered: HashMap<String, DateTime<Utc>>,
}

impl RuleMatcher {
    /// Create a new rule matcher from a set of detection rules.
    ///
    /// Compiles all patterns and sorts rules by priority.
    pub fn new(rules: Vec<DetectionRule>, i18n: &I18nService) -> MahalaxmiResult<Self> {
        let mut compiled_rules = Vec::new();

        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let mut patterns = Vec::new();
            for pattern in &rule.patterns {
                patterns.push(pattern.compile(i18n)?);
            }
            compiled_rules.push(CompiledRule { rule, patterns });
        }

        // Sort by priority (lower = higher priority)
        compiled_rules.sort_by_key(|r| r.rule.priority);

        Ok(Self {
            rules: compiled_rules,
            last_triggered: HashMap::new(),
        })
    }

    /// Evaluate text against all rules and return the highest-priority match.
    ///
    /// Filters by provider_id and role if specified. Respects cooldowns.
    pub fn evaluate(
        &mut self,
        text: &str,
        provider_id: Option<&str>,
        role: Option<&str>,
    ) -> Option<DetectionResult> {
        let now = Utc::now();

        for compiled in &self.rules {
            let rule = &compiled.rule;

            // Provider filter
            if let Some(ref providers) = rule.provider_filter {
                match provider_id {
                    Some(pid) => {
                        if !providers.iter().any(|p| p == pid) {
                            continue;
                        }
                    }
                    None => continue,
                }
            }

            // Role filter
            if let Some(ref required_role) = rule.role_filter {
                match role {
                    Some(r) => {
                        if r != required_role {
                            continue;
                        }
                    }
                    None => continue,
                }
            }

            // Cooldown check
            if let Some(cooldown_ms) = rule.cooldown_ms {
                if let Some(last) = self.last_triggered.get(&rule.name) {
                    let elapsed = now.signed_duration_since(*last).num_milliseconds();
                    if elapsed < cooldown_ms as i64 {
                        continue;
                    }
                }
            }

            // Pattern matching — any pattern match triggers the rule
            for pattern in &compiled.patterns {
                if pattern.matches(text) {
                    self.last_triggered.insert(rule.name.clone(), now);
                    return Some(DetectionResult::matched(
                        &rule.name,
                        text,
                        rule.action.clone(),
                        rule.response_text.clone(),
                        rule.priority,
                    ));
                }
            }
        }

        None
    }

    /// Get the number of active (compiled) rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Reset all cooldown timers.
    pub fn reset_cooldowns(&mut self) {
        self.last_triggered.clear();
    }
}

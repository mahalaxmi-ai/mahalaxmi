// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Custom detection rules
//
// Demonstrates how to define custom DetectionRules beyond the built-in sets.
// This is useful when integrating a provider whose prompts and completion
// markers are not covered by the built-in rules.
//
// Shows contains patterns, regex patterns, provider filters, and cooldowns.

use mahalaxmi_core::{i18n::{locale::SupportedLocale, I18nService}, types::ActionType};
use mahalaxmi_detection::{BuiltinRuleSets, DetectionRule, RuleMatcher};

fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);

    // Start with the generic built-in set as a base.
    let mut rules = BuiltinRuleSets::generic();

    // Rule 1: Detect a custom "READY" prompt from a fictional provider.
    // A contains pattern is the simplest and fastest option.
    let ready_rule = DetectionRule::new("my-provider-ready", ActionType::ContinueProcessing)
        .with_contains_pattern("READY>")
        .with_priority(20)                          // higher priority than generic (90)
        .with_provider_filter(vec!["my-provider".to_string()])
        .with_cooldown_ms(2000);

    // Rule 2: Detect a provider-specific completion marker using a regex.
    // The regex matches "== DONE ==" with any surrounding whitespace.
    let done_rule = DetectionRule::new("my-provider-done", ActionType::CompleteWorkerCycle)
        .with_regex_pattern(r"={2,}\s*DONE\s*={2,}")
        .with_priority(10)
        .with_provider_filter(vec!["my-provider".to_string()]);

    // Rule 3: Escalate on a custom budget-exceeded message.
    let budget_rule = DetectionRule::new("my-provider-budget", ActionType::EscalateToManager)
        .with_contains_pattern("BUDGET_EXCEEDED")
        .with_priority(5);

    rules.push(ready_rule);
    rules.push(done_rule);
    rules.push(budget_rule);

    let mut matcher = RuleMatcher::new(rules, &i18n).expect("failed to build rule matcher");

    let lines = vec![
        "Processing task...",
        "READY> waiting for input",
        "BUDGET_EXCEEDED: token limit reached",
        "== DONE ==",
        "user@host:~/project$ ",
    ];

    for line in &lines {
        match matcher.evaluate(line, None, None) {
            Some(result) => println!(
                "MATCH  {:?} => {:?}  (rule: {})",
                line,
                result.action,
                result.matched_rule_name.as_deref().unwrap_or("?"),
            ),
            None => println!("no match: {:?}", line),
        }
    }
}

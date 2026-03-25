// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Basic terminal state detection
//
// Demonstrates how to create a RuleMatcher from the built-in rule sets and
// match it against simulated terminal output lines. The matcher returns the
// highest-priority matching rule and the ActionType it triggers.

use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
use mahalaxmi_detection::{BuiltinRuleSets, RuleMatcher};

fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);

    // Load the generic built-in rules (shell prompt, segfault, OOM, permission denied).
    let mut rules = BuiltinRuleSets::generic();

    // Layer on Claude Code–specific rules (auto-confirm, completion, error).
    rules.extend(BuiltinRuleSets::claude_code());

    // Build the matcher — rules are sorted by priority internally.
    let mut matcher = RuleMatcher::new(rules, &i18n).expect("failed to build rule matcher");

    // Simulate lines of terminal output from an AI worker session.
    let lines = vec![
        "Analyzing the codebase...",
        "Error: cannot find module 'react'",
        "Do you want to proceed? (y/n)",
        "Task completed successfully.",
        "user@host:~/project$ ",            // shell prompt = cycle complete
        "Segmentation fault (core dumped)",  // crash
    ];

    for line in &lines {
        match matcher.evaluate(line, None, None) {
            Some(result) => {
                println!(
                    "MATCH  {:?} => {:?} (rule: {})",
                    line,
                    result.action,
                    result.matched_rule_name.as_deref().unwrap_or("?"),
                );
            }
            None => {
                println!("no match: {:?}", line);
            }
        }
    }
}

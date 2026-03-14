# mahalaxmi-detection

Pattern-matching rule engine for detecting terminal states, completion markers, error conditions, and provider prompts from PTY output streams.

## Overview

`mahalaxmi-detection` turns raw PTY output into actionable events. Each `DetectionRule` pairs one or more `DetectionPattern`s (substring or regex) with an `ActionType` (complete worker, restart session, escalate to manager, send text, etc.). Rules carry priorities, optional provider filters, role filters, and cooldown periods to prevent rapid re-triggering.

The `RuleMatcher` evaluates a set of rules against a line of terminal output and returns the highest-priority matching `DetectionResult`. `BuiltinRuleSets` provides pre-built rules for generic shell prompts, process crashes, OOM conditions, and provider-specific behavior (Claude Code auto-confirm, completion detection, cost warnings). These built-ins are composable — load the generic set, add provider-specific sets, then layer your own custom rules on top.

The `errors` sub-module provides a higher-level error analysis layer: `ErrorPatternAnalysis` clusters related errors into `ErrorCluster`s, identifies `RecurringError`s (same pattern seen multiple times), and produces `RootCauseHypothesis` candidates for escalation prompts. The `verification` sub-module holds output parsers for test runners (cargo, pytest, jest, go test) and linters (clippy, eslint, pylint, golint) used by the worker self-verification pipeline.

Orchestration engine developers use this crate to decide how to respond to worker output. Most end users interact with detection only through configuration.

## Key Types

| Type | Kind | Description |
|------|------|-------------|
| `DetectionRule` | Struct | A named rule with patterns, action, priority, and filters |
| `DetectionPattern` | Struct | A single pattern (string + `MatchType`) to test against output |
| `CompiledPattern` | Struct | Pre-compiled version of `DetectionPattern` for efficient matching |
| `RuleMatcher` | Struct | Evaluates a sorted rule set against terminal output lines |
| `DetectionResult` | Struct | The matching rule and the `ActionType` it triggered |
| `BuiltinRuleSets` | Struct | Factory for generic, Claude Code, and other provider-specific rule sets |
| `ErrorPatternAnalysis` | Struct | High-level error analysis over a sequence of output lines |
| `ErrorCluster` | Struct | Group of related error occurrences in output |
| `RootCauseHypothesis` | Struct | A hypothesis about what caused a cluster of errors |
| `RecurringError` | Struct | An error pattern seen multiple times with occurrence count |
| `TestResult` | Struct | Parsed result of a test runner invocation |
| `LintResult` | Struct | Parsed result of a linter invocation |

## Key Functions / Methods

| Function | Description |
|----------|-------------|
| `DetectionRule::new(name, action)` | Create a rule; chain `.with_pattern()`, `.with_priority()`, etc. |
| `DetectionRule::with_contains_pattern(text)` | Add a substring match pattern |
| `DetectionRule::with_regex_pattern(regex)` | Add a regex match pattern |
| `DetectionRule::with_cooldown_ms(ms)` | Prevent re-triggering within a cooldown window |
| `RuleMatcher::new(rules)` | Build a matcher from a list of rules; sorts by priority |
| `RuleMatcher::match_line(line)` | Test a single output line; returns highest-priority match or `None` |
| `BuiltinRuleSets::generic()` | Generic rules: shell prompt, segfault, OOM, permission denied |
| `BuiltinRuleSets::claude_code()` | Claude Code rules: auto-confirm, completion, error, cost warning |
| `ErrorPatternAnalysis::analyze(lines)` | Cluster errors and identify root cause hypotheses from output |

## Feature Flags

No feature flags.

## Dependencies

| Dependency | Why |
|-----------|-----|
| `regex` | Compiled regex matching for `DetectionPattern` |
| `serde` | Rule serialization (load/save rule sets from config) |
| `mahalaxmi-core` | Shared types (`ActionType`, `MatchType`), errors, i18n |

## Stability

**Unstable** — API may change in minor versions during the pre-1.0 period.

## License

MIT — Copyright 2026 ThriveTech Services LLC

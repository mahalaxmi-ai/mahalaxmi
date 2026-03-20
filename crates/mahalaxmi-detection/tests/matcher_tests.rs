use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
use mahalaxmi_core::types::{ActionType, MatchType};
use mahalaxmi_detection::{
    BuiltinRuleSets, DetectionPattern, DetectionResult, DetectionRule, RuleMatcher,
};
use std::thread::sleep;
use std::time::Duration;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

#[test]
fn matcher_evaluates_by_priority() {
    let rule_a = DetectionRule::new("rule-a", ActionType::ContinueProcessing)
        .with_pattern(DetectionPattern::new("hello", MatchType::Contains))
        .with_priority(10);
    let rule_b = DetectionRule::new("rule-b", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("hello", MatchType::Contains))
        .with_priority(5);

    let mut matcher = RuleMatcher::new(vec![rule_a, rule_b], &i18n()).unwrap();
    let result = matcher.evaluate("hello world", None, None).unwrap();

    // Lower priority number wins (rule_b with priority 5 beats rule_a with priority 10)
    assert_eq!(result.matched_rule_name.as_deref(), Some("rule-b"));
    assert_eq!(result.action, ActionType::SendEnter);
}

#[test]
fn matcher_returns_none_no_match() {
    let rule = DetectionRule::new("only-rule", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("hello", MatchType::Contains));

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();
    let result = matcher.evaluate("unmatched", None, None);
    assert!(result.is_none());
}

#[test]
fn matcher_first_pattern_match_triggers() {
    let rule = DetectionRule::new("multi-pattern", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("alpha", MatchType::Contains))
        .with_pattern(DetectionPattern::new("beta", MatchType::Contains));

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();
    let result = matcher.evaluate("alpha test", None, None).unwrap();
    assert_eq!(result.matched_rule_name.as_deref(), Some("multi-pattern"));
    assert!(result.matched);
}

#[test]
fn matcher_cooldown_suppresses_rapid_fire() {
    let rule = DetectionRule::new("cooldown-rule", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("ping", MatchType::Contains))
        .with_cooldown_ms(5000);

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();

    let first = matcher.evaluate("ping", None, None);
    assert!(first.is_some());

    let second = matcher.evaluate("ping", None, None);
    assert!(second.is_none());
}

#[test]
fn matcher_cooldown_allows_after_expiry() {
    let rule = DetectionRule::new("short-cooldown", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("ping", MatchType::Contains))
        .with_cooldown_ms(100);

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();

    let first = matcher.evaluate("ping", None, None);
    assert!(first.is_some());

    sleep(Duration::from_millis(150));

    let second = matcher.evaluate("ping", None, None);
    assert!(second.is_some());
}

#[test]
fn matcher_reset_cooldowns() {
    let rule = DetectionRule::new("reset-test", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("ping", MatchType::Contains))
        .with_cooldown_ms(60000);

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();

    let first = matcher.evaluate("ping", None, None);
    assert!(first.is_some());

    // Without reset, the cooldown would block this
    matcher.reset_cooldowns();

    let second = matcher.evaluate("ping", None, None);
    assert!(second.is_some());
}

#[test]
fn matcher_provider_filter_matches() {
    let rule = DetectionRule::new("claude-only", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("prompt", MatchType::Contains))
        .with_provider_filter(vec!["claude-code".to_owned()]);

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();
    let result = matcher.evaluate("prompt text", Some("claude-code"), None);
    assert!(result.is_some());
}

#[test]
fn matcher_provider_filter_excludes() {
    let rule = DetectionRule::new("claude-only", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("prompt", MatchType::Contains))
        .with_provider_filter(vec!["claude-code".to_owned()]);

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();
    let result = matcher.evaluate("prompt text", Some("openai"), None);
    assert!(result.is_none());
}

#[test]
fn matcher_provider_filter_none_provider() {
    let rule = DetectionRule::new("claude-only", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("prompt", MatchType::Contains))
        .with_provider_filter(vec!["claude-code".to_owned()]);

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();
    let result = matcher.evaluate("prompt text", None, None);
    assert!(result.is_none());
}

#[test]
fn matcher_role_filter_matches() {
    let rule = DetectionRule::new("worker-only", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("task", MatchType::Contains))
        .with_role_filter("Worker");

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();
    let result = matcher.evaluate("task complete", None, Some("Worker"));
    assert!(result.is_some());
}

#[test]
fn matcher_role_filter_excludes() {
    let rule = DetectionRule::new("worker-only", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("task", MatchType::Contains))
        .with_role_filter("Worker");

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();
    let result = matcher.evaluate("task complete", None, Some("Manager"));
    assert!(result.is_none());
}

#[test]
fn matcher_disabled_rules_excluded() {
    let rule = DetectionRule::new("disabled-rule", ActionType::SendEnter)
        .with_pattern(DetectionPattern::new("ping", MatchType::Contains))
        .with_enabled(false);

    let mut matcher = RuleMatcher::new(vec![rule], &i18n()).unwrap();
    let result = matcher.evaluate("ping", None, None);
    assert!(result.is_none());
}

#[test]
fn matcher_rule_count() {
    let rules = vec![
        DetectionRule::new("active-1", ActionType::SendEnter)
            .with_pattern(DetectionPattern::new("a", MatchType::Contains)),
        DetectionRule::new("active-2", ActionType::SendEnter)
            .with_pattern(DetectionPattern::new("b", MatchType::Contains)),
        DetectionRule::new("active-3", ActionType::SendEnter)
            .with_pattern(DetectionPattern::new("c", MatchType::Contains)),
        DetectionRule::new("disabled-1", ActionType::SendEnter)
            .with_pattern(DetectionPattern::new("d", MatchType::Contains))
            .with_enabled(false),
        DetectionRule::new("disabled-2", ActionType::SendEnter)
            .with_pattern(DetectionPattern::new("e", MatchType::Contains))
            .with_enabled(false),
    ];

    let matcher = RuleMatcher::new(rules, &i18n()).unwrap();
    assert_eq!(matcher.rule_count(), 3);
}

#[test]
fn matcher_detection_result_no_match() {
    let result = DetectionResult::no_match();
    assert!(!result.matched);
    assert!(result.matched_rule_name.is_none());
    assert!(result.matched_text.is_none());
    assert_eq!(result.action, ActionType::ContinueProcessing);
    assert_eq!(result.priority, u32::MAX);
}

#[test]
fn matcher_detection_result_matched() {
    let result = DetectionResult::matched(
        "test-rule",
        "some output",
        ActionType::SendEnter,
        Some("y".to_owned()),
        42,
    );
    assert!(result.matched);
    assert_eq!(result.matched_rule_name.as_deref(), Some("test-rule"));
    assert_eq!(result.matched_text.as_deref(), Some("some output"));
    assert_eq!(result.action, ActionType::SendEnter);
    assert_eq!(result.response_text.as_deref(), Some("y"));
    assert_eq!(result.priority, 42);
}

#[test]
fn matcher_builtin_rule_sets_compile() {
    let rules = BuiltinRuleSets::all_defaults();
    let matcher = RuleMatcher::new(rules, &i18n());
    assert!(matcher.is_ok());
    let matcher = matcher.unwrap();
    assert!(matcher.rule_count() > 0);
}

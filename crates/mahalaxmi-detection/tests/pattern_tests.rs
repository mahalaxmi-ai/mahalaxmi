use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
use mahalaxmi_core::types::MatchType;
use mahalaxmi_detection::DetectionPattern;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

// ---------- Contains ----------

#[test]
fn contains_match_found() {
    let pattern = DetectionPattern::new("hello", MatchType::Contains);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(compiled.matches("say hello world"));
}

#[test]
fn contains_match_not_found() {
    let pattern = DetectionPattern::new("goodbye", MatchType::Contains);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(!compiled.matches("say hello"));
}

#[test]
fn contains_case_insensitive() {
    let pattern = DetectionPattern::new("HELLO", MatchType::Contains).with_case_sensitive(false);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(compiled.matches("hello world"));
}

#[test]
fn contains_case_sensitive() {
    let pattern = DetectionPattern::new("HELLO", MatchType::Contains).with_case_sensitive(true);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(!compiled.matches("hello world"));
}

// ---------- Exact ----------

#[test]
fn exact_match() {
    let pattern = DetectionPattern::new("hello", MatchType::Exact);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(compiled.matches("hello"));
}

#[test]
fn exact_no_match_partial() {
    let pattern = DetectionPattern::new("hello", MatchType::Exact);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(!compiled.matches("hello world"));
}

#[test]
fn exact_case_insensitive() {
    let pattern = DetectionPattern::new("Hello", MatchType::Exact).with_case_sensitive(false);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(compiled.matches("hello"));
}

// ---------- StartsWith / EndsWith ----------

#[test]
fn starts_with_match() {
    let pattern = DetectionPattern::new("hello", MatchType::StartsWith);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(compiled.matches("hello world"));
}

#[test]
fn starts_with_no_match() {
    let pattern = DetectionPattern::new("world", MatchType::StartsWith);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(!compiled.matches("hello world"));
}

#[test]
fn ends_with_match() {
    let pattern = DetectionPattern::new("world", MatchType::EndsWith);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(compiled.matches("hello world"));
}

#[test]
fn ends_with_no_match() {
    let pattern = DetectionPattern::new("hello", MatchType::EndsWith);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(!compiled.matches("hello world"));
}

// ---------- Regex ----------

#[test]
fn regex_match() {
    let pattern = DetectionPattern::new(r"h\w+o", MatchType::Regex);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(compiled.matches("hello"));
}

#[test]
fn regex_no_match() {
    let pattern = DetectionPattern::new(r"^goodbye", MatchType::Regex);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(!compiled.matches("hello"));
}

#[test]
fn regex_case_insensitive() {
    let pattern = DetectionPattern::new("HELLO", MatchType::Regex).with_case_sensitive(false);
    let compiled = pattern.compile(&i18n()).unwrap();
    assert!(compiled.matches("hello"));
}

#[test]
fn regex_compile_failure() {
    let pattern = DetectionPattern::new("[invalid", MatchType::Regex);
    let result = pattern.compile(&i18n());
    assert!(result.is_err());
}

// ---------- Debug ----------

#[test]
fn compiled_pattern_debug_shows_structure() {
    let pattern = DetectionPattern::new("hello", MatchType::Contains);
    let compiled = pattern.compile(&i18n()).unwrap();
    let debug_output = format!("{:?}", compiled);
    assert!(debug_output.contains("CompiledPattern"));
    assert!(debug_output.contains("has_regex"));
    assert!(debug_output.contains("source"));
}

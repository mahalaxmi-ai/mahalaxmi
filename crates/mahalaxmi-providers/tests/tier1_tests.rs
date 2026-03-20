//! Tests for Step 11: Tier 1 Provider definitions.

use mahalaxmi_providers::tier1::*;
use mahalaxmi_providers::types::{CostTier, Proficiency, TaskType};
use mahalaxmi_providers::{AiProvider, ProviderRegistry};

#[test]
fn kiro_provider_id() {
    let p = kiro_provider();
    assert_eq!(p.id().as_str(), "kiro");
    assert_eq!(p.name(), "Kiro");
}

#[test]
fn kiro_provider_capabilities() {
    let p = kiro_provider();
    let caps = p.capabilities();
    assert_eq!(caps.cost_tier, CostTier::Medium);
    assert_eq!(
        caps.proficiency_for(TaskType::CodeGeneration),
        Proficiency::Excellent
    );
    assert_eq!(
        caps.proficiency_for(TaskType::Planning),
        Proficiency::Excellent
    );
}

#[test]
fn goose_provider_id() {
    let p = goose_provider();
    assert_eq!(p.id().as_str(), "goose");
    assert_eq!(p.name(), "Goose");
}

#[test]
fn goose_provider_is_free() {
    let p = goose_provider();
    assert_eq!(p.capabilities().cost_tier, CostTier::Free);
}

#[test]
fn deepseek_provider_id() {
    let p = deepseek_provider();
    assert_eq!(p.id().as_str(), "deepseek");
    assert_eq!(p.name(), "DeepSeek");
}

#[test]
fn deepseek_requires_api_key() {
    let p = deepseek_provider();
    let creds = p.credential_requirements();
    assert_eq!(creds.len(), 1);
    assert_eq!(creds[0].env_var_name.as_deref(), Some("DEEPSEEK_API_KEY"));
    assert!(creds[0].required);
}

#[test]
fn qwen_provider_id() {
    let p = qwen_provider();
    assert_eq!(p.id().as_str(), "qwen");
    assert_eq!(p.name(), "Qwen Coder");
}

#[test]
fn qwen_low_cost() {
    let p = qwen_provider();
    assert_eq!(p.capabilities().cost_tier, CostTier::Low);
}

#[test]
fn opencode_provider_id() {
    let p = opencode_provider();
    assert_eq!(p.id().as_str(), "opencode");
    assert_eq!(p.name(), "OpenCode");
}

#[test]
fn opencode_is_free() {
    let p = opencode_provider();
    assert_eq!(p.capabilities().cost_tier, CostTier::Free);
}

#[test]
fn opencode_no_credential_requirements() {
    let p = opencode_provider();
    assert!(p.credential_requirements().is_empty());
}

#[test]
fn cody_provider_id() {
    let p = cody_provider();
    assert_eq!(p.id().as_str(), "cody");
    assert_eq!(p.name(), "Cody");
}

#[test]
fn cody_excellent_code_review() {
    let p = cody_provider();
    assert_eq!(
        p.capabilities().proficiency_for(TaskType::CodeReview),
        Proficiency::Excellent
    );
}

#[test]
fn register_tier1_adds_six_providers() {
    let mut registry = ProviderRegistry::new();
    register_tier1_providers(&mut registry);
    assert_eq!(registry.len(), 6);
}

#[test]
fn register_tier1_all_retrievable() {
    let mut registry = ProviderRegistry::new();
    register_tier1_providers(&mut registry);

    for id in &["kiro", "goose", "deepseek", "qwen", "opencode", "cody"] {
        assert!(
            registry
                .get(&mahalaxmi_core::types::ProviderId::new(*id))
                .is_some(),
            "provider '{}' should be registered",
            id
        );
    }
}

#[test]
fn all_tier1_have_streaming() {
    let providers: Vec<Box<dyn AiProvider>> = vec![
        Box::new(kiro_provider()),
        Box::new(goose_provider()),
        Box::new(deepseek_provider()),
        Box::new(qwen_provider()),
        Box::new(opencode_provider()),
        Box::new(cody_provider()),
    ];

    for p in &providers {
        assert!(
            p.capabilities().supports_streaming,
            "{} should support streaming",
            p.name()
        );
    }
}

#[test]
fn all_tier1_have_tool_use() {
    let providers: Vec<Box<dyn AiProvider>> = vec![
        Box::new(kiro_provider()),
        Box::new(goose_provider()),
        Box::new(deepseek_provider()),
        Box::new(qwen_provider()),
        Box::new(opencode_provider()),
        Box::new(cody_provider()),
    ];

    for p in &providers {
        assert!(
            p.capabilities().supports_tool_use,
            "{} should support tool use",
            p.name()
        );
    }
}

#[test]
fn all_tier1_have_positive_latency() {
    let providers: Vec<Box<dyn AiProvider>> = vec![
        Box::new(kiro_provider()),
        Box::new(goose_provider()),
        Box::new(deepseek_provider()),
        Box::new(qwen_provider()),
        Box::new(opencode_provider()),
        Box::new(cody_provider()),
    ];

    for p in &providers {
        assert!(
            p.capabilities().avg_latency_ms > 0,
            "{} should have positive latency estimate",
            p.name()
        );
    }
}

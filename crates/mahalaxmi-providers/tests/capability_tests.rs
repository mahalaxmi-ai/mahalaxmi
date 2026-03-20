//! Tests for Step 09: Extended Provider Capabilities — TaskType, CostTier, Proficiency.

use mahalaxmi_providers::types::{CostTier, Proficiency, ProviderCapabilities, TaskType};
use mahalaxmi_providers::AiProvider;
use std::collections::HashMap;

// ===========================================================================
// TaskType
// ===========================================================================

#[test]
fn task_type_all_returns_eight_types() {
    assert_eq!(TaskType::all().len(), 8);
}

#[test]
fn task_type_display() {
    assert_eq!(TaskType::CodeGeneration.to_string(), "code_generation");
    assert_eq!(TaskType::CodeReview.to_string(), "code_review");
    assert_eq!(TaskType::Debugging.to_string(), "debugging");
    assert_eq!(TaskType::Refactoring.to_string(), "refactoring");
    assert_eq!(TaskType::Testing.to_string(), "testing");
    assert_eq!(TaskType::Documentation.to_string(), "documentation");
    assert_eq!(TaskType::Planning.to_string(), "planning");
    assert_eq!(TaskType::General.to_string(), "general");
}

#[test]
fn task_type_serialization_round_trip() {
    let task = TaskType::CodeGeneration;
    let json = serde_json::to_string(&task).unwrap();
    assert_eq!(json, "\"code_generation\"");
    let deserialized: TaskType = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, task);
}

#[test]
fn task_type_eq_and_hash() {
    let mut map = HashMap::new();
    map.insert(TaskType::Debugging, "debug");
    assert_eq!(map[&TaskType::Debugging], "debug");
}

// ===========================================================================
// CostTier
// ===========================================================================

#[test]
fn cost_tier_ordering() {
    assert!(CostTier::Free < CostTier::Low);
    assert!(CostTier::Low < CostTier::Medium);
    assert!(CostTier::Medium < CostTier::High);
    assert!(CostTier::High < CostTier::Premium);
}

#[test]
fn cost_tier_weight() {
    assert_eq!(CostTier::Free.weight(), 0);
    assert_eq!(CostTier::Low.weight(), 1);
    assert_eq!(CostTier::Medium.weight(), 2);
    assert_eq!(CostTier::High.weight(), 3);
    assert_eq!(CostTier::Premium.weight(), 4);
}

#[test]
fn cost_tier_display() {
    assert_eq!(CostTier::Free.to_string(), "free");
    assert_eq!(CostTier::Premium.to_string(), "premium");
}

#[test]
fn cost_tier_serialization() {
    let tier = CostTier::High;
    let json = serde_json::to_string(&tier).unwrap();
    assert_eq!(json, "\"high\"");
    let deserialized: CostTier = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, tier);
}

// ===========================================================================
// Proficiency
// ===========================================================================

#[test]
fn proficiency_ordering() {
    assert!(Proficiency::None < Proficiency::Basic);
    assert!(Proficiency::Basic < Proficiency::Good);
    assert!(Proficiency::Good < Proficiency::Excellent);
}

#[test]
fn proficiency_score() {
    assert_eq!(Proficiency::None.score(), 0);
    assert_eq!(Proficiency::Basic.score(), 1);
    assert_eq!(Proficiency::Good.score(), 2);
    assert_eq!(Proficiency::Excellent.score(), 3);
}

// ===========================================================================
// ProviderCapabilities
// ===========================================================================

#[test]
fn capabilities_default() {
    let caps = ProviderCapabilities::default();
    assert!(caps.supports_streaming);
    assert!(!caps.supports_agent_teams);
    assert!(!caps.supports_tool_use);
    assert_eq!(caps.max_context_tokens, 0);
    assert_eq!(caps.cost_tier, CostTier::Medium);
    assert_eq!(caps.avg_latency_ms, 0);
    assert!(caps.supports_concurrent_sessions);
    assert!(caps.task_proficiency.is_empty());
    assert!(!caps.supports_local_only);
    assert!(!caps.supports_web_search);
}

#[test]
fn capabilities_proficiency_default_is_good() {
    let caps = ProviderCapabilities::default();
    assert_eq!(caps.proficiency_for(TaskType::Debugging), Proficiency::Good);
}

#[test]
fn capabilities_proficiency_explicit() {
    let mut caps = ProviderCapabilities::default();
    caps.task_proficiency
        .insert(TaskType::CodeGeneration, Proficiency::Excellent);
    caps.task_proficiency
        .insert(TaskType::Documentation, Proficiency::None);

    assert_eq!(
        caps.proficiency_for(TaskType::CodeGeneration),
        Proficiency::Excellent
    );
    assert_eq!(
        caps.proficiency_for(TaskType::Documentation),
        Proficiency::None
    );
    assert_eq!(caps.proficiency_for(TaskType::Testing), Proficiency::Good);
}

#[test]
fn capabilities_supports_task() {
    let mut caps = ProviderCapabilities::default();
    caps.task_proficiency
        .insert(TaskType::Documentation, Proficiency::None);

    assert!(caps.supports_task(TaskType::CodeGeneration));
    assert!(!caps.supports_task(TaskType::Documentation));
}

#[test]
fn capabilities_routing_score_proficiency_dominant() {
    let mut caps_excellent = ProviderCapabilities::default();
    caps_excellent
        .task_proficiency
        .insert(TaskType::Debugging, Proficiency::Excellent);

    let mut caps_basic = ProviderCapabilities::default();
    caps_basic
        .task_proficiency
        .insert(TaskType::Debugging, Proficiency::Basic);

    assert!(
        caps_excellent.routing_score(TaskType::Debugging)
            > caps_basic.routing_score(TaskType::Debugging)
    );
}

#[test]
fn capabilities_routing_score_cost_bonus() {
    let mut free = ProviderCapabilities::default();
    free.cost_tier = CostTier::Free;

    let mut premium = ProviderCapabilities::default();
    premium.cost_tier = CostTier::Premium;

    assert!(free.routing_score(TaskType::General) > premium.routing_score(TaskType::General));
}

#[test]
fn capabilities_routing_score_context_bonus() {
    let mut large = ProviderCapabilities::default();
    large.max_context_tokens = 200_000;

    let mut small = ProviderCapabilities::default();
    small.max_context_tokens = 8_000;

    assert!(large.routing_score(TaskType::General) > small.routing_score(TaskType::General));
}

#[test]
fn capabilities_serialization_round_trip() {
    let mut caps = ProviderCapabilities::default();
    caps.cost_tier = CostTier::High;
    caps.avg_latency_ms = 3000;
    caps.task_proficiency
        .insert(TaskType::Debugging, Proficiency::Excellent);

    let json = serde_json::to_string(&caps).unwrap();
    let deserialized: ProviderCapabilities = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.cost_tier, CostTier::High);
    assert_eq!(deserialized.avg_latency_ms, 3000);
    assert_eq!(
        deserialized.proficiency_for(TaskType::Debugging),
        Proficiency::Excellent
    );
}

// ===========================================================================
// Provider integration
// ===========================================================================

#[test]
fn claude_provider_has_extended_capabilities() {
    let claude = mahalaxmi_providers::ClaudeCodeProvider::new();
    let caps = claude.capabilities();
    assert_eq!(caps.cost_tier, CostTier::High);
    assert!(caps.avg_latency_ms > 0);
    assert_eq!(
        caps.proficiency_for(TaskType::CodeGeneration),
        Proficiency::Excellent
    );
}

#[test]
fn mock_provider_has_default_capabilities() {
    let mock = mahalaxmi_providers::MockProvider::new();
    let caps = mock.capabilities();
    assert_eq!(caps.cost_tier, CostTier::Medium); // default
}

// ===========================================================================
// Local-only and web search capability fields
// ===========================================================================

#[test]
fn capabilities_local_only_field() {
    let mut caps = ProviderCapabilities::default();
    assert!(!caps.supports_local_only);
    caps.supports_local_only = true;
    assert!(caps.supports_local_only);
    assert!(!mahalaxmi_providers::is_local_by_capabilities(
        &ProviderCapabilities::default()
    ));
    assert!(mahalaxmi_providers::is_local_by_capabilities(&caps));
}

#[test]
fn capabilities_web_search_field() {
    let mut caps = ProviderCapabilities::default();
    assert!(!caps.supports_web_search);
    caps.supports_web_search = true;
    assert!(caps.supports_web_search);
}

#[test]
fn capabilities_local_and_web_search_serialization() {
    let mut caps = ProviderCapabilities::default();
    caps.supports_local_only = true;
    caps.supports_web_search = true;
    let json = serde_json::to_string(&caps).unwrap();
    let deserialized: ProviderCapabilities = serde_json::from_str(&json).unwrap();
    assert!(deserialized.supports_local_only);
    assert!(deserialized.supports_web_search);
}

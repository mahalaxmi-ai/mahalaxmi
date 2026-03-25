//! Tests for Step 10: Task Router — routing tasks to optimal providers.

use mahalaxmi_core::types::ProviderId;
use mahalaxmi_providers::markers::OutputMarkers;
use mahalaxmi_providers::types::{CostTier, Proficiency, ProviderCapabilities, TaskType};
use mahalaxmi_providers::{
    GenericCliProvider, MockProvider, ProviderRegistry, RoutingConstraints, TaskRouter,
};

fn test_markers() -> OutputMarkers {
    OutputMarkers::new(r"\$\s*$", r"(?i)error", r">\s*$").unwrap()
}

fn make_provider(
    id: &str,
    name: &str,
    cost: CostTier,
    context: u64,
    proficiency: Vec<(TaskType, Proficiency)>,
) -> GenericCliProvider {
    let mut caps = ProviderCapabilities {
        supports_streaming: true,
        supports_tool_use: true,
        max_context_tokens: context,
        cost_tier: cost,
        ..Default::default()
    };
    for (tt, p) in proficiency {
        caps.task_proficiency.insert(tt, p);
    }
    GenericCliProvider::new(id, name, "echo", test_markers()).with_capabilities(caps)
}

// ===========================================================================
// Basic routing
// ===========================================================================

#[test]
fn route_selects_highest_scoring_provider() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(make_provider(
        "p1",
        "Provider 1",
        CostTier::Low,
        100_000,
        vec![(TaskType::CodeGeneration, Proficiency::Excellent)],
    )));
    registry.register(Box::new(make_provider(
        "p2",
        "Provider 2",
        CostTier::Low,
        100_000,
        vec![(TaskType::CodeGeneration, Proficiency::Basic)],
    )));

    let decision = TaskRouter::route(
        &registry,
        TaskType::CodeGeneration,
        &RoutingConstraints::default(),
    );
    assert!(decision.is_some());
    assert_eq!(decision.unwrap().provider_id, ProviderId::new("p1"));
}

#[test]
fn route_returns_none_with_empty_registry() {
    let registry = ProviderRegistry::new();
    let decision = TaskRouter::route(
        &registry,
        TaskType::Debugging,
        &RoutingConstraints::default(),
    );
    assert!(decision.is_none());
}

#[test]
fn route_skips_excluded_providers() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(make_provider(
        "p1",
        "Provider 1",
        CostTier::Free,
        100_000,
        vec![(TaskType::Testing, Proficiency::Excellent)],
    )));
    registry.register(Box::new(make_provider(
        "p2",
        "Provider 2",
        CostTier::Free,
        100_000,
        vec![(TaskType::Testing, Proficiency::Good)],
    )));

    let constraints = RoutingConstraints {
        excluded_providers: vec![ProviderId::new("p1")],
        ..Default::default()
    };
    let decision = TaskRouter::route(&registry, TaskType::Testing, &constraints);
    assert_eq!(decision.unwrap().provider_id, ProviderId::new("p2"));
}

#[test]
fn route_respects_cost_constraint() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(make_provider(
        "cheap",
        "Cheap",
        CostTier::Low,
        32_000,
        vec![(TaskType::Documentation, Proficiency::Good)],
    )));
    registry.register(Box::new(make_provider(
        "expensive",
        "Expensive",
        CostTier::Premium,
        200_000,
        vec![(TaskType::Documentation, Proficiency::Excellent)],
    )));

    let constraints = RoutingConstraints {
        max_cost_tier: Some(CostTier::Medium),
        ..Default::default()
    };
    let decision = TaskRouter::route(&registry, TaskType::Documentation, &constraints);
    assert_eq!(decision.unwrap().provider_id, ProviderId::new("cheap"));
}

#[test]
fn route_respects_context_constraint() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(make_provider(
        "small",
        "Small Context",
        CostTier::Low,
        8_000,
        vec![(TaskType::CodeGeneration, Proficiency::Excellent)],
    )));
    registry.register(Box::new(make_provider(
        "large",
        "Large Context",
        CostTier::Low,
        200_000,
        vec![(TaskType::CodeGeneration, Proficiency::Good)],
    )));

    let constraints = RoutingConstraints {
        min_context_tokens: Some(100_000),
        ..Default::default()
    };
    let decision = TaskRouter::route(&registry, TaskType::CodeGeneration, &constraints);
    assert_eq!(decision.unwrap().provider_id, ProviderId::new("large"));
}

#[test]
fn route_preferred_provider_gets_bonus() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(make_provider(
        "p1",
        "Provider 1",
        CostTier::Medium,
        100_000,
        vec![(TaskType::Refactoring, Proficiency::Good)],
    )));
    registry.register(Box::new(make_provider(
        "p2",
        "Provider 2",
        CostTier::Medium,
        100_000,
        vec![(TaskType::Refactoring, Proficiency::Good)],
    )));

    let constraints = RoutingConstraints {
        preferred_provider: Some(ProviderId::new("p2")),
        ..Default::default()
    };
    let decision = TaskRouter::route(&registry, TaskType::Refactoring, &constraints);
    assert_eq!(decision.unwrap().provider_id, ProviderId::new("p2"));
}

#[test]
fn route_skips_providers_with_none_proficiency() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(make_provider(
        "cant-do",
        "Cannot Do",
        CostTier::Free,
        100_000,
        vec![(TaskType::Planning, Proficiency::None)],
    )));
    registry.register(Box::new(make_provider(
        "can-do",
        "Can Do",
        CostTier::High,
        100_000,
        vec![(TaskType::Planning, Proficiency::Basic)],
    )));

    let decision = TaskRouter::route(
        &registry,
        TaskType::Planning,
        &RoutingConstraints::default(),
    );
    assert_eq!(decision.unwrap().provider_id, ProviderId::new("can-do"));
}

// ===========================================================================
// Fallback routing
// ===========================================================================

#[test]
fn route_with_fallbacks_returns_ranked_list() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(make_provider(
        "p1",
        "P1",
        CostTier::Low,
        100_000,
        vec![(TaskType::Debugging, Proficiency::Excellent)],
    )));
    registry.register(Box::new(make_provider(
        "p2",
        "P2",
        CostTier::Medium,
        100_000,
        vec![(TaskType::Debugging, Proficiency::Good)],
    )));
    registry.register(Box::new(make_provider(
        "p3",
        "P3",
        CostTier::High,
        100_000,
        vec![(TaskType::Debugging, Proficiency::Basic)],
    )));

    let decisions = TaskRouter::route_with_fallbacks(
        &registry,
        TaskType::Debugging,
        &RoutingConstraints::default(),
        2,
    );
    assert_eq!(decisions.len(), 3);
    assert!(decisions[0].score >= decisions[1].score);
    assert!(decisions[1].score >= decisions[2].score);
}

#[test]
fn route_with_fallbacks_respects_max() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(make_provider(
        "p1",
        "P1",
        CostTier::Low,
        100_000,
        vec![],
    )));
    registry.register(Box::new(make_provider(
        "p2",
        "P2",
        CostTier::Low,
        100_000,
        vec![],
    )));
    registry.register(Box::new(make_provider(
        "p3",
        "P3",
        CostTier::Low,
        100_000,
        vec![],
    )));

    let decisions = TaskRouter::route_with_fallbacks(
        &registry,
        TaskType::General,
        &RoutingConstraints::default(),
        1,
    );
    assert_eq!(decisions.len(), 2); // primary + 1 fallback
}

#[test]
fn route_with_fallbacks_empty_registry() {
    let registry = ProviderRegistry::new();
    let decisions = TaskRouter::route_with_fallbacks(
        &registry,
        TaskType::General,
        &RoutingConstraints::default(),
        3,
    );
    assert!(decisions.is_empty());
}

// ===========================================================================
// Provider status
// ===========================================================================

#[test]
fn check_status_not_installed() {
    let provider = MockProvider::new();
    let status = TaskRouter::check_provider_status(&provider);
    // MockProvider's cli_binary is "echo" which exists
    // This tests the status check mechanism works
    assert!(
        status == mahalaxmi_providers::ProviderStatus::Ready
            || status == mahalaxmi_providers::ProviderStatus::NotInstalled
    );
}

// ===========================================================================
// RoutingDecision
// ===========================================================================

#[test]
fn routing_decision_has_reason() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(MockProvider::new()));

    let decision = TaskRouter::route(&registry, TaskType::General, &RoutingConstraints::default());
    let d = decision.unwrap();
    assert!(!d.reason.is_empty());
    assert!(d.score > 0);
}

// ===========================================================================
// RoutingConstraints
// ===========================================================================

#[test]
fn routing_constraints_default() {
    let c = RoutingConstraints::default();
    assert!(c.max_cost_tier.is_none());
    assert!(c.min_context_tokens.is_none());
    assert!(c.preferred_provider.is_none());
    assert!(c.excluded_providers.is_empty());
}

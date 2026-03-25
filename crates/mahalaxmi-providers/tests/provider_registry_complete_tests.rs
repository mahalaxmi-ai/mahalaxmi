//! Full registry validation tests.

use mahalaxmi_providers::tier1;
use mahalaxmi_providers::{
    ClaudeCodeProvider, GeminiProvider, MockProvider, ProviderId, ProviderRegistry,
};
use std::collections::HashSet;
use std::path::Path;

fn full_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(ClaudeCodeProvider::new()));
    registry.register(Box::new(GeminiProvider::new()));
    registry.register(Box::new(MockProvider::new()));
    tier1::register_tier1_providers(&mut registry);
    registry
}

// ===========================================================================
// Expected provider count
// ===========================================================================

#[test]
fn registry_has_expected_provider_count() {
    let registry = full_registry();
    // Claude, Gemini, Mock + 6 Tier1 = 9
    assert!(
        registry.len() >= 9,
        "Expected at least 9 providers, got {}",
        registry.len()
    );
}

// ===========================================================================
// Each known provider is registered
// ===========================================================================

#[test]
fn registry_contains_claude() {
    let registry = full_registry();
    assert!(registry.get(&ProviderId::new("claude-code")).is_some());
}

#[test]
fn registry_contains_gemini() {
    let registry = full_registry();
    assert!(registry.get(&ProviderId::new("google-gemini")).is_some());
}

#[test]
fn registry_contains_mock() {
    let registry = full_registry();
    assert!(registry.get(&ProviderId::new("mock")).is_some());
}

#[test]
fn registry_contains_kiro() {
    let registry = full_registry();
    assert!(registry.get(&ProviderId::new("kiro")).is_some());
}

#[test]
fn registry_contains_goose() {
    let registry = full_registry();
    assert!(registry.get(&ProviderId::new("goose")).is_some());
}

#[test]
fn registry_contains_deepseek() {
    let registry = full_registry();
    assert!(registry.get(&ProviderId::new("deepseek")).is_some());
}

#[test]
fn registry_contains_qwen() {
    let registry = full_registry();
    assert!(registry.get(&ProviderId::new("qwen")).is_some());
}

#[test]
fn registry_contains_opencode() {
    let registry = full_registry();
    assert!(registry.get(&ProviderId::new("opencode")).is_some());
}

#[test]
fn registry_contains_cody() {
    let registry = full_registry();
    assert!(registry.get(&ProviderId::new("cody")).is_some());
}

// ===========================================================================
// No duplicate IDs
// ===========================================================================

#[test]
fn registry_no_duplicate_ids() {
    let registry = full_registry();
    let ids: Vec<&ProviderId> = registry.list();
    let unique: HashSet<&ProviderId> = ids.iter().copied().collect();
    assert_eq!(
        ids.len(),
        unique.len(),
        "Registry has duplicate provider IDs"
    );
}

// ===========================================================================
// All providers callable via dyn AiProvider
// ===========================================================================

#[test]
fn all_providers_accessible_via_trait_dispatch() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        // Verify basic trait methods work
        assert!(
            !provider.name().is_empty(),
            "Provider {:?} has empty name",
            id
        );
        assert_eq!(provider.id(), id);
        assert!(
            !provider.cli_binary().is_empty(),
            "Provider {:?} has empty cli_binary",
            id
        );
    }
}

#[test]
fn all_providers_can_build_commands() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let result = provider.build_command(Path::new("/tmp"), "test");
        assert!(
            result.is_ok(),
            "Provider '{}' ({:?}) failed to build command: {:?}",
            provider.name(),
            id,
            result.err()
        );
    }
}

#[test]
fn all_providers_return_capabilities() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let caps = provider.capabilities();
        // max_context_tokens should be positive for all providers
        assert!(
            caps.max_context_tokens > 0,
            "Provider '{}' has zero max_context_tokens",
            provider.name()
        );
    }
}

#[test]
fn all_providers_return_markers() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let markers = provider.output_markers();
        // Markers should be non-trivial regexes
        assert!(
            !markers.completion_marker.as_str().is_empty(),
            "Provider '{}' has empty completion marker",
            provider.name()
        );
        assert!(
            !markers.error_marker.as_str().is_empty(),
            "Provider '{}' has empty error marker",
            provider.name()
        );
    }
}

#[test]
fn all_providers_return_metadata() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let meta = provider.metadata();
        assert!(
            !meta.install_hint.is_empty(),
            "Provider '{}' has empty install_hint",
            provider.name()
        );
    }
}

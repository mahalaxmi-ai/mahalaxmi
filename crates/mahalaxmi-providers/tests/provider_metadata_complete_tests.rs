//! Metadata completeness tests across all providers.

use mahalaxmi_providers::credentials::AuthMode;
use mahalaxmi_providers::tier1;
use mahalaxmi_providers::{
    AiProvider, ClaudeCodeProvider, GeminiProvider, MockProvider, ProviderRegistry,
};

fn full_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(ClaudeCodeProvider::new()));
    registry.register(Box::new(GeminiProvider::new()));
    registry.register(Box::new(MockProvider::new()));
    tier1::register_tier1_providers(&mut registry);
    registry
}

// ===========================================================================
// All providers have non-empty install hints
// ===========================================================================

#[test]
fn all_providers_have_non_empty_install_hint() {
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

// ===========================================================================
// All providers have test mechanism (test_args or connection_check)
// ===========================================================================

#[test]
fn all_providers_have_test_mechanism() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let meta = provider.metadata();
        let has_test = !meta.test_args.is_empty() || meta.connection_check.is_some();
        assert!(
            has_test,
            "Provider '{}' has neither test_args nor connection_check",
            provider.name()
        );
    }
}

// ===========================================================================
// All providers have at least one auth mode
// ===========================================================================

#[test]
fn all_providers_have_at_least_one_auth_mode() {
    let registry = full_registry();
    for id in registry.list() {
        let provider = registry.get(id).unwrap();
        let meta = provider.metadata();
        assert!(
            !meta.auth_modes.is_empty(),
            "Provider '{}' has no auth_modes",
            provider.name()
        );
    }
}

// ===========================================================================
// Claude-specific metadata
// ===========================================================================

#[test]
fn claude_has_platform_specific_install() {
    let provider = ClaudeCodeProvider::new();
    let meta = provider.metadata();
    assert!(
        meta.platform_install.linux.is_some(),
        "Claude should have Linux install command"
    );
    assert!(
        meta.platform_install.macos.is_some(),
        "Claude should have macOS install command"
    );
    assert!(
        meta.platform_install.windows.is_some(),
        "Claude should have Windows install command"
    );
}

#[test]
fn claude_has_install_url() {
    let provider = ClaudeCodeProvider::new();
    let meta = provider.metadata();
    assert!(meta.install_url.is_some(), "Claude should have install_url");
}

#[test]
fn claude_has_test_args() {
    let provider = ClaudeCodeProvider::new();
    let meta = provider.metadata();
    assert!(!meta.test_args.is_empty(), "Claude should have test_args");
    assert!(meta.test_args.contains(&"--print".to_string()));
}

#[test]
fn claude_has_cli_login_and_api_key_auth() {
    let provider = ClaudeCodeProvider::new();
    let meta = provider.metadata();
    let has_cli_login = meta
        .auth_modes
        .iter()
        .any(|m| matches!(m, AuthMode::CliLogin { .. }));
    let has_api_key = meta
        .auth_modes
        .iter()
        .any(|m| matches!(m, AuthMode::ApiKey { .. }));
    assert!(has_cli_login, "Claude should support CLI login auth");
    assert!(has_api_key, "Claude should support API key auth");
}

// ===========================================================================
// Gemini-specific metadata
// ===========================================================================

#[test]
fn gemini_has_platform_specific_install() {
    let provider = GeminiProvider::new();
    let meta = provider.metadata();
    assert!(
        meta.platform_install.linux.is_some(),
        "Gemini should have Linux install command"
    );
    assert!(
        meta.platform_install.macos.is_some(),
        "Gemini should have macOS install command"
    );
    assert!(
        meta.platform_install.windows.is_some(),
        "Gemini should have Windows install command"
    );
}

#[test]
fn gemini_has_install_url() {
    let provider = GeminiProvider::new();
    let meta = provider.metadata();
    assert!(meta.install_url.is_some(), "Gemini should have install_url");
}

#[test]
fn gemini_has_connection_check() {
    let provider = GeminiProvider::new();
    let meta = provider.metadata();
    assert!(
        meta.connection_check.is_some(),
        "Gemini should have connection_check"
    );
}

#[test]
fn gemini_has_service_account_and_api_key_auth() {
    let provider = GeminiProvider::new();
    let meta = provider.metadata();
    let has_service_account = meta
        .auth_modes
        .iter()
        .any(|m| matches!(m, AuthMode::ServiceAccount { .. }));
    let has_api_key = meta
        .auth_modes
        .iter()
        .any(|m| matches!(m, AuthMode::ApiKey { .. }));
    assert!(
        has_service_account,
        "Gemini should support service account auth"
    );
    assert!(has_api_key, "Gemini should support API key auth");
}

// ===========================================================================
// Credential specs consistent with auth modes
// ===========================================================================

#[test]
fn claude_credential_spec_matches_api_key_auth_mode() {
    let provider = ClaudeCodeProvider::new();
    let creds = provider.credential_requirements();
    let meta = provider.metadata();
    // Claude has API key auth mode with ANTHROPIC_API_KEY
    let api_key_mode = meta.auth_modes.iter().find_map(|m| {
        if let AuthMode::ApiKey { env_var } = m {
            Some(env_var.clone())
        } else {
            None
        }
    });
    let cred_env = creds.iter().find_map(|c| c.env_var_name.clone());
    assert_eq!(
        api_key_mode, cred_env,
        "Claude credential spec should match API key auth mode env var"
    );
}

#[test]
fn gemini_credential_specs_match_auth_modes() {
    let provider = GeminiProvider::new();
    let creds = provider.credential_requirements();
    let meta = provider.metadata();

    // Collect all env vars from auth modes
    let auth_env_vars: Vec<String> = meta
        .auth_modes
        .iter()
        .filter_map(|m| match m {
            AuthMode::ApiKey { env_var } => Some(env_var.clone()),
            AuthMode::ServiceAccount { env_var, .. } => Some(env_var.clone()),
            _ => None,
        })
        .collect();

    // Each credential spec env var should appear in auth modes
    for cred in &creds {
        if let Some(ref env_var) = cred.env_var_name {
            assert!(
                auth_env_vars.contains(env_var),
                "Gemini credential '{}' not in auth modes",
                env_var
            );
        }
    }
}

// ===========================================================================
// Tier 1 metadata
// ===========================================================================

#[test]
fn tier1_kiro_has_install_url() {
    let provider = tier1::kiro_provider();
    assert!(provider.metadata().install_url.is_some());
}

#[test]
fn tier1_goose_has_install_url() {
    let provider = tier1::goose_provider();
    assert!(provider.metadata().install_url.is_some());
}

#[test]
fn tier1_deepseek_has_install_url() {
    let provider = tier1::deepseek_provider();
    assert!(provider.metadata().install_url.is_some());
}

#[test]
fn tier1_all_have_test_args() {
    let providers: Vec<Box<dyn AiProvider>> = vec![
        Box::new(tier1::kiro_provider()),
        Box::new(tier1::goose_provider()),
        Box::new(tier1::deepseek_provider()),
        Box::new(tier1::qwen_provider()),
        Box::new(tier1::opencode_provider()),
        Box::new(tier1::cody_provider()),
    ];
    for provider in &providers {
        assert!(
            !provider.metadata().test_args.is_empty(),
            "Tier1 provider '{}' has no test_args",
            provider.name()
        );
    }
}

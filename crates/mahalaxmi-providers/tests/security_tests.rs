use mahalaxmi_core::types::ProviderId;
use mahalaxmi_providers::security::{
    apply_security_routing, classify_task, is_local_provider, SecurityClassification,
    SecurityRoutingMode,
};
use mahalaxmi_providers::RoutingConstraints;

// ---------------------------------------------------------------------------
// Classification tests
// ---------------------------------------------------------------------------

#[test]
fn test_env_file_auto_restricted() {
    let result = classify_task(
        "Update configuration",
        "Modify the .env file to add database settings",
        &[],
    );
    assert_eq!(result, SecurityClassification::Restricted);
}

#[test]
fn test_secrets_keyword_auto_restricted() {
    let result = classify_task(
        "Manage api_key rotation",
        "Implement automatic API key rotation for all services",
        &[],
    );
    assert_eq!(result, SecurityClassification::Restricted);
}

#[test]
fn test_affected_env_file_auto_restricted() {
    let result = classify_task(
        "Update config",
        "Change database URL",
        &["src/config.rs".to_string(), ".env.production".to_string()],
    );
    assert_eq!(result, SecurityClassification::Restricted);
}

#[test]
fn test_pem_file_auto_restricted() {
    let result = classify_task(
        "Update TLS",
        "Refresh certificate",
        &["certs/server.pem".to_string()],
    );
    assert_eq!(result, SecurityClassification::Restricted);
}

#[test]
fn test_credential_file_auto_restricted() {
    let result = classify_task(
        "Fix login",
        "Update credential handling",
        &["src/credential_manager.rs".to_string()],
    );
    assert_eq!(result, SecurityClassification::Restricted);
}

#[test]
fn test_password_keyword_auto_restricted() {
    let result = classify_task(
        "Fix password hashing",
        "The password storage needs updating",
        &[],
    );
    assert_eq!(result, SecurityClassification::Restricted);
}

#[test]
fn test_auth_file_auto_sensitive() {
    let result = classify_task(
        "Fix login flow",
        "Update the login page",
        &["src/auth/middleware.rs".to_string()],
    );
    assert_eq!(result, SecurityClassification::Sensitive);
}

#[test]
fn test_auth_keyword_auto_sensitive() {
    let result = classify_task(
        "Implement OAuth flow",
        "Add OAuth 2.0 support for third-party login",
        &[],
    );
    assert_eq!(result, SecurityClassification::Sensitive);
}

#[test]
fn test_jwt_keyword_auto_sensitive() {
    let result = classify_task(
        "Add JWT validation",
        "Validate JWT claims on protected routes",
        &[],
    );
    assert_eq!(result, SecurityClassification::Sensitive);
}

#[test]
fn test_session_file_auto_sensitive() {
    let result = classify_task(
        "Fix timeout",
        "Session expires too early",
        &["src/session_handler.rs".to_string()],
    );
    assert_eq!(result, SecurityClassification::Sensitive);
}

#[test]
fn test_normal_task_unclassified() {
    let result = classify_task(
        "Add pagination to user list",
        "Implement cursor-based pagination for the user listing endpoint",
        &[
            "src/routes/users.rs".to_string(),
            "src/utils.rs".to_string(),
        ],
    );
    assert_eq!(result, SecurityClassification::Normal);
}

#[test]
fn test_classification_case_insensitive() {
    let upper = classify_task("Fix API_KEY handling", "Update the API_KEY rotation", &[]);
    assert_eq!(upper, SecurityClassification::Restricted);

    let mixed = classify_task("Fix Api_Key handling", "Update the Api_Key rotation", &[]);
    assert_eq!(mixed, SecurityClassification::Restricted);
}

#[test]
fn test_restricted_takes_priority_over_sensitive() {
    // "auth" is Sensitive, but "api_key" is Restricted — Restricted wins
    let result = classify_task(
        "Fix auth api_key validation",
        "Update auth module to validate API keys",
        &[],
    );
    assert_eq!(result, SecurityClassification::Restricted);
}

// ---------------------------------------------------------------------------
// Local provider detection
// ---------------------------------------------------------------------------

#[test]
fn test_ollama_is_local() {
    assert!(is_local_provider(&ProviderId::new("ollama")));
}

#[test]
fn test_claude_is_not_local() {
    assert!(!is_local_provider(&ProviderId::new("claude-code")));
}

#[test]
fn test_custom_local_prefix() {
    assert!(is_local_provider(&ProviderId::new("local-llama")));
    assert!(is_local_provider(&ProviderId::new("ollama-custom")));
}

// ---------------------------------------------------------------------------
// Routing behavior tests
// ---------------------------------------------------------------------------

#[test]
fn test_restricted_routes_local_only() {
    let local_providers = vec![ProviderId::new("ollama")];
    let result = apply_security_routing(
        SecurityClassification::Restricted,
        SecurityRoutingMode::Automatic,
        &RoutingConstraints::default(),
        &local_providers,
    );
    assert!(!result.blocked);
    assert!(result.notification.is_none());
    assert!(result.constraints.preferred_provider.is_some());
    assert_eq!(
        result.constraints.preferred_provider.unwrap().as_str(),
        "ollama"
    );
}

#[test]
fn test_restricted_no_local_shows_message() {
    let result = apply_security_routing(
        SecurityClassification::Restricted,
        SecurityRoutingMode::Automatic,
        &RoutingConstraints::default(),
        &[], // No local providers
    );
    assert!(result.blocked);
    assert!(result.block_reason.is_some());
    let reason = result.block_reason.unwrap();
    assert!(reason.contains("local AI provider"));
}

#[test]
fn test_sensitive_prefers_local() {
    let local_providers = vec![ProviderId::new("ollama")];
    let result = apply_security_routing(
        SecurityClassification::Sensitive,
        SecurityRoutingMode::Automatic,
        &RoutingConstraints::default(),
        &local_providers,
    );
    assert!(!result.blocked);
    assert!(result.notification.is_none());
    assert_eq!(
        result.constraints.preferred_provider.unwrap().as_str(),
        "ollama"
    );
}

#[test]
fn test_sensitive_falls_back_to_cloud() {
    let result = apply_security_routing(
        SecurityClassification::Sensitive,
        SecurityRoutingMode::Automatic,
        &RoutingConstraints::default(),
        &[], // No local providers
    );
    assert!(!result.blocked);
    assert!(result.notification.is_some());
    let msg = result.notification.unwrap();
    assert!(msg.contains("cloud provider"));
}

#[test]
fn test_all_local_mode() {
    let local_providers = vec![ProviderId::new("ollama")];
    let result = apply_security_routing(
        SecurityClassification::Normal, // Even Normal tasks go to local
        SecurityRoutingMode::AllLocal,
        &RoutingConstraints::default(),
        &local_providers,
    );
    assert!(!result.blocked);
    assert_eq!(
        result.constraints.preferred_provider.unwrap().as_str(),
        "ollama"
    );
}

#[test]
fn test_all_local_no_provider_blocks() {
    let result = apply_security_routing(
        SecurityClassification::Normal,
        SecurityRoutingMode::AllLocal,
        &RoutingConstraints::default(),
        &[],
    );
    assert!(result.blocked);
    assert!(result.block_reason.unwrap().contains("All Local"));
}

#[test]
fn test_disabled_mode() {
    let result = apply_security_routing(
        SecurityClassification::Restricted, // Even Restricted is ignored
        SecurityRoutingMode::Disabled,
        &RoutingConstraints::default(),
        &[],
    );
    assert!(!result.blocked);
    assert!(result.notification.is_none());
    assert!(result.constraints.preferred_provider.is_none());
}

#[test]
fn test_normal_task_no_routing_changes() {
    let result = apply_security_routing(
        SecurityClassification::Normal,
        SecurityRoutingMode::Automatic,
        &RoutingConstraints::default(),
        &[ProviderId::new("ollama")],
    );
    assert!(!result.blocked);
    assert!(result.notification.is_none());
    assert!(result.constraints.preferred_provider.is_none());
}

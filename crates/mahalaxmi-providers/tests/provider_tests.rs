use std::path::Path;
use std::sync::Mutex;

use mahalaxmi_providers::{
    find_binary, AiProvider, AuthMethod, AuthMode, ClaudeCodeProvider, CredentialSpec,
    GenericCliProvider, I18nService, MockProvider, OutputMarkers, ProviderCapabilities, ProviderId,
    ProviderMetadata, ProviderRegistry, ProviderStatus, SupportedLocale,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ---------------------------------------------------------------------------
// MockProvider Tests
// ---------------------------------------------------------------------------

#[test]
fn mock_provider_name() {
    let provider = MockProvider::new();
    assert_eq!(provider.name(), "Mock Provider");
}

#[test]
fn mock_provider_id() {
    let provider = MockProvider::new();
    assert_eq!(provider.id(), &ProviderId::new("mock"));
}

#[test]
fn mock_provider_default_impl() {
    let p1 = MockProvider::new();
    let p2 = MockProvider::default();
    assert_eq!(p1.id(), p2.id());
    assert_eq!(p1.name(), p2.name());
}

#[test]
fn mock_provider_build_command() {
    let provider = MockProvider::new();
    let cmd = provider
        .build_command(Path::new("/tmp"), "hello world")
        .unwrap();
    assert_eq!(cmd.program, "echo");
    assert!(cmd.args.contains(&"hello world".to_string()));
    assert_eq!(cmd.working_dir, Some(std::path::PathBuf::from("/tmp")));
}

#[tokio::test]
async fn mock_provider_validate_credentials() {
    let provider = MockProvider::new();
    let result = provider.validate_credentials(&i18n()).await;
    assert!(result.is_ok());
}

#[test]
fn mock_provider_no_credential_requirements() {
    let provider = MockProvider::new();
    assert!(provider.credential_requirements().is_empty());
}

#[test]
fn mock_provider_capabilities() {
    let provider = MockProvider::new();
    let caps = provider.capabilities();
    assert!(caps.supports_streaming);
    assert!(!caps.supports_agent_teams);
    assert!(caps.supports_tool_use);
    assert_eq!(caps.max_context_tokens, 100_000);
}

#[test]
fn mock_provider_output_markers() {
    let provider = MockProvider::new();
    let markers = provider.output_markers();
    assert!(markers.completion_marker.is_match("$ "));
    assert!(markers.error_marker.is_match("Error: something"));
}

#[test]
fn mock_provider_object_safety() {
    let provider: Box<dyn AiProvider> = Box::new(MockProvider::new());
    assert_eq!(provider.name(), "Mock Provider");
    assert_eq!(provider.id(), &ProviderId::new("mock"));
}

// ---------------------------------------------------------------------------
// ClaudeCodeProvider Tests
// ---------------------------------------------------------------------------

#[test]
fn claude_provider_name() {
    let provider = ClaudeCodeProvider::new();
    assert_eq!(provider.name(), "Claude Code");
}

#[test]
fn claude_provider_id() {
    let provider = ClaudeCodeProvider::new();
    assert_eq!(provider.id(), &ProviderId::new("claude-code"));
}

#[test]
fn claude_provider_default_impl() {
    let p1 = ClaudeCodeProvider::new();
    let p2 = ClaudeCodeProvider::default();
    assert_eq!(p1.id(), p2.id());
    assert_eq!(p1.name(), p2.name());
}

#[test]
fn claude_provider_build_command_args() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "fix the bug")
        .unwrap();
    assert_eq!(cmd.program, "claude");
    assert!(cmd.args.contains(&"--print".to_string()));
    assert!(cmd
        .args
        .contains(&"--dangerously-skip-permissions".to_string()));
    // Prompt passed as positional arg, not via stdin redirect
    assert!(cmd.args.contains(&"fix the bug".to_string()));
    assert!(cmd.stdin_data.is_none());
}

#[test]
fn claude_provider_build_command_working_dir() {
    let provider = ClaudeCodeProvider::new();
    let cmd = provider
        .build_command(Path::new("/my/project"), "task")
        .unwrap();
    assert_eq!(
        cmd.working_dir,
        Some(std::path::PathBuf::from("/my/project"))
    );
}

#[test]
fn claude_provider_custom_binary() {
    let provider = ClaudeCodeProvider::with_binary("/usr/local/bin/claude-custom");
    let cmd = provider.build_command(Path::new("/tmp"), "task").unwrap();
    assert_eq!(cmd.program, "/usr/local/bin/claude-custom");
}

#[tokio::test]
async fn claude_provider_validate_with_no_api_key() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::remove_var("ANTHROPIC_API_KEY");
    let provider = ClaudeCodeProvider::new();
    let result = provider.validate_credentials(&i18n()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn claude_provider_localized_error() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::remove_var("ANTHROPIC_API_KEY");
    let provider = ClaudeCodeProvider::new();
    let i18n_service = I18nService::new(SupportedLocale::EnUs);
    let err = provider
        .validate_credentials(&i18n_service)
        .await
        .unwrap_err();
    let key = err.i18n_key().unwrap();
    assert_eq!(key, "error-provider-credentials-missing");
    let msg = err.to_string();
    assert!(
        msg.contains("ANTHROPIC_API_KEY"),
        "Error message should mention env var: {}",
        msg
    );
}

#[test]
fn claude_provider_credential_requirements() {
    let provider = ClaudeCodeProvider::new();
    let creds = provider.credential_requirements();
    assert_eq!(creds.len(), 1);
    assert_eq!(creds[0].method, AuthMethod::ApiKey);
    assert_eq!(creds[0].env_var_name, Some("ANTHROPIC_API_KEY".to_string()));
    assert!(creds[0].required);
}

#[test]
fn claude_provider_capabilities() {
    let provider = ClaudeCodeProvider::new();
    let caps = provider.capabilities();
    assert!(caps.supports_streaming);
    assert!(caps.supports_agent_teams);
    assert!(caps.supports_tool_use);
    assert_eq!(caps.max_context_tokens, 200_000);
}

#[test]
fn claude_provider_object_safety() {
    let provider: Box<dyn AiProvider> = Box::new(ClaudeCodeProvider::new());
    assert_eq!(provider.name(), "Claude Code");
}

// ---------------------------------------------------------------------------
// GenericCliProvider Tests
// ---------------------------------------------------------------------------

fn test_markers() -> OutputMarkers {
    OutputMarkers::new(r"\$\s*$", r"(?i)error:", r">\s*$").unwrap()
}

#[test]
fn generic_provider_custom_program() {
    let provider = GenericCliProvider::new("my-ai", "My AI Tool", "my-ai-cli", test_markers());
    let cmd = provider
        .build_command(Path::new("/tmp"), "do stuff")
        .unwrap();
    assert_eq!(cmd.program, "my-ai-cli");
}

#[test]
fn generic_provider_name_and_id() {
    let provider = GenericCliProvider::new("my-ai", "My AI Tool", "my-ai-cli", test_markers());
    assert_eq!(provider.name(), "My AI Tool");
    assert_eq!(provider.id(), &ProviderId::new("my-ai"));
}

#[test]
fn generic_provider_base_args() {
    let provider = GenericCliProvider::new("test", "Test", "test-cli", test_markers())
        .with_arg("--verbose")
        .with_arg("--json");
    let cmd = provider
        .build_command(Path::new("/tmp"), "run task")
        .unwrap();
    assert!(cmd.args.contains(&"--verbose".to_string()));
    assert!(cmd.args.contains(&"--json".to_string()));
    // Prompt is passed as final positional arg, not via stdin redirect
    assert!(cmd.args.contains(&"run task".to_string()));
    assert!(cmd.stdin_data.is_none());
}

#[test]
fn generic_provider_env_vars() {
    let provider = GenericCliProvider::new("test", "Test", "test-cli", test_markers())
        .with_env("MY_KEY", "my_value");
    let cmd = provider.build_command(Path::new("/tmp"), "task").unwrap();
    assert_eq!(cmd.env.get("MY_KEY"), Some(&"my_value".to_string()));
}

#[test]
fn generic_provider_capabilities() {
    let caps = ProviderCapabilities {
        supports_streaming: false,
        supports_agent_teams: true,
        supports_tool_use: false,
        max_context_tokens: 50_000,
        ..Default::default()
    };
    let provider =
        GenericCliProvider::new("test", "Test", "test-cli", test_markers()).with_capabilities(caps);
    let result = provider.capabilities();
    assert!(!result.supports_streaming);
    assert!(result.supports_agent_teams);
    assert!(!result.supports_tool_use);
    assert_eq!(result.max_context_tokens, 50_000);
}

#[test]
fn generic_provider_credentials() {
    let spec = CredentialSpec {
        method: AuthMethod::EnvironmentVariable,
        env_var_name: Some("MY_API_KEY".to_string()),
        description_key: "credential-generic-api-key".to_string(),
        required: true,
    };
    let provider =
        GenericCliProvider::new("test", "Test", "test-cli", test_markers()).with_credential(spec);
    let creds = provider.credential_requirements();
    assert_eq!(creds.len(), 1);
    assert_eq!(creds[0].method, AuthMethod::EnvironmentVariable);
}

#[tokio::test]
async fn generic_provider_validate_with_missing_env() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::remove_var("NONEXISTENT_GENERIC_KEY_FOR_TEST");
    let spec = CredentialSpec {
        method: AuthMethod::EnvironmentVariable,
        env_var_name: Some("NONEXISTENT_GENERIC_KEY_FOR_TEST".to_string()),
        description_key: "credential-generic-api-key".to_string(),
        required: true,
    };
    let provider = GenericCliProvider::new("test", "Test Provider", "test-cli", test_markers())
        .with_credential(spec);
    let result = provider.validate_credentials(&i18n()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn generic_provider_validate_with_no_requirements() {
    let provider = GenericCliProvider::new("test", "Test", "test-cli", test_markers());
    let result = provider.validate_credentials(&i18n()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn generic_provider_i18n_error() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::remove_var("NONEXISTENT_GENERIC_KEY_FOR_TEST_2");
    let spec = CredentialSpec {
        method: AuthMethod::EnvironmentVariable,
        env_var_name: Some("NONEXISTENT_GENERIC_KEY_FOR_TEST_2".to_string()),
        description_key: "credential-generic-api-key".to_string(),
        required: true,
    };
    let provider = GenericCliProvider::new("test", "Test Provider", "test-cli", test_markers())
        .with_credential(spec);
    let err = provider.validate_credentials(&i18n()).await.unwrap_err();
    assert_eq!(
        err.i18n_key().unwrap(),
        "error-provider-credentials-missing"
    );
}

#[test]
fn generic_provider_object_safety() {
    let provider: Box<dyn AiProvider> = Box::new(GenericCliProvider::new(
        "test",
        "Test",
        "test",
        test_markers(),
    ));
    assert_eq!(provider.name(), "Test");
}

// ---------------------------------------------------------------------------
// ProviderRegistry Tests
// ---------------------------------------------------------------------------

#[test]
fn registry_new_is_empty() {
    let registry = ProviderRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn registry_default_is_empty() {
    let registry = ProviderRegistry::default();
    assert!(registry.is_empty());
}

#[test]
fn registry_register_and_get() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(MockProvider::new()));
    let provider = registry.get(&ProviderId::new("mock"));
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().name(), "Mock Provider");
}

#[test]
fn registry_get_nonexistent() {
    let registry = ProviderRegistry::new();
    assert!(registry.get(&ProviderId::new("nonexistent")).is_none());
}

#[test]
fn registry_register_default() {
    let mut registry = ProviderRegistry::new();
    registry.register_default(Box::new(MockProvider::new()));
    let provider = registry.default_provider(&i18n()).unwrap();
    assert_eq!(provider.name(), "Mock Provider");
}

#[test]
fn registry_no_default_errors() {
    let registry = ProviderRegistry::new();
    let result = registry.default_provider(&i18n());
    assert!(result.is_err());
}

#[test]
fn registry_no_default_error_uses_i18n() {
    let registry = ProviderRegistry::new();
    let result = registry.default_provider(&i18n());
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("Expected error"),
    };
    assert_eq!(err.i18n_key().unwrap(), "error-provider-no-default");
    assert_eq!(err.category(), "provider");
}

#[test]
fn registry_list_providers() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(MockProvider::new()));
    registry.register(Box::new(ClaudeCodeProvider::new()));
    let list = registry.list();
    assert_eq!(list.len(), 2);
}

#[test]
fn registry_len_and_is_empty() {
    let mut registry = ProviderRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);

    registry.register(Box::new(MockProvider::new()));
    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 1);
}

#[test]
fn registry_replace_duplicate() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(MockProvider::new()));
    registry.register(Box::new(MockProvider::new()));
    assert_eq!(registry.len(), 1);
}

#[test]
fn registry_multiple_providers_coexist() {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(MockProvider::new()));
    registry.register(Box::new(ClaudeCodeProvider::new()));
    registry.register(Box::new(GenericCliProvider::new(
        "custom",
        "Custom",
        "custom-cli",
        test_markers(),
    )));
    assert_eq!(registry.len(), 3);
    assert!(registry.get(&ProviderId::new("mock")).is_some());
    assert!(registry.get(&ProviderId::new("claude-code")).is_some());
    assert!(registry.get(&ProviderId::new("custom")).is_some());
}

// ---------------------------------------------------------------------------
// ProviderStatus Tests
// ---------------------------------------------------------------------------

#[test]
fn provider_status_as_str() {
    assert_eq!(ProviderStatus::NotInstalled.as_str(), "not_installed");
    assert_eq!(ProviderStatus::NotConfigured.as_str(), "not_configured");
    assert_eq!(ProviderStatus::Ready.as_str(), "ready");
    assert_eq!(ProviderStatus::Verified.as_str(), "verified");
    assert_eq!(ProviderStatus::Error("bad key".into()).as_str(), "error");
}

#[test]
fn provider_status_display() {
    assert_eq!(format!("{}", ProviderStatus::NotInstalled), "not_installed");
    assert_eq!(format!("{}", ProviderStatus::Ready), "ready");
    let err = ProviderStatus::Error("timeout".into());
    assert!(format!("{err}").contains("timeout"));
}

#[test]
fn provider_status_serialization_round_trip() {
    let statuses = vec![
        ProviderStatus::NotInstalled,
        ProviderStatus::NotConfigured,
        ProviderStatus::Ready,
        ProviderStatus::Verified,
        ProviderStatus::Error("test error".to_string()),
    ];
    for status in &statuses {
        let json = serde_json::to_string(status).unwrap();
        let deserialized: ProviderStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, &deserialized);
    }
}

#[test]
fn auth_mode_serialization_round_trip() {
    let modes = vec![
        AuthMode::CliLogin {
            login_command: "claude auth login".into(),
            check_command: "claude auth status".into(),
        },
        AuthMode::ApiKey {
            env_var: "ANTHROPIC_API_KEY".into(),
        },
        AuthMode::None,
    ];
    for mode in &modes {
        let json = serde_json::to_string(mode).unwrap();
        let deserialized: AuthMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, &deserialized);
    }
}

// ---------------------------------------------------------------------------
// Metadata & Binary Detection Tests
// ---------------------------------------------------------------------------

#[test]
fn claude_provider_cli_binary() {
    let provider = ClaudeCodeProvider::new();
    assert_eq!(provider.cli_binary(), "claude");
}

#[test]
fn claude_provider_metadata() {
    let provider = ClaudeCodeProvider::new();
    let meta = provider.metadata();
    assert!(meta.install_hint.contains("npm install"));
    assert!(meta.install_url.is_some());
    assert!(!meta.test_args.is_empty());
    assert!(meta.auth_modes.len() >= 2);
}

#[test]
fn generic_provider_cli_binary() {
    let provider = GenericCliProvider::new("test", "Test", "my-binary", test_markers());
    assert_eq!(provider.cli_binary(), "my-binary");
}

#[test]
fn generic_provider_with_metadata() {
    let meta = ProviderMetadata::new("pip install test")
        .with_install_url("https://example.com")
        .with_test_args(vec!["--version".into()])
        .with_auth_mode(AuthMode::ApiKey {
            env_var: "TEST_KEY".into(),
        });
    let provider =
        GenericCliProvider::new("test", "Test", "test-cli", test_markers()).with_metadata(meta);
    let m = provider.metadata();
    assert_eq!(m.install_hint, "pip install test");
    assert_eq!(m.install_url.as_deref(), Some("https://example.com"));
    assert_eq!(m.test_args, vec!["--version"]);
    assert_eq!(m.auth_modes.len(), 1);
}

#[test]
fn mock_provider_cli_binary() {
    let provider = MockProvider::new();
    assert_eq!(provider.cli_binary(), "echo");
}

#[test]
fn mock_provider_metadata() {
    let provider = MockProvider::new();
    let meta = provider.metadata();
    assert!(!meta.install_hint.is_empty());
    assert!(!meta.test_args.is_empty());
}

#[test]
fn find_binary_for_known_binary() {
    // "echo" or "ls" should exist on any Unix system
    #[cfg(unix)]
    {
        let result = find_binary("ls");
        assert!(result.is_some(), "ls should be found on PATH");
    }
}

#[test]
fn find_binary_for_nonexistent() {
    let result = find_binary("nonexistent_binary_12345_xyz");
    assert!(result.is_none(), "nonexistent binary should not be found");
}

use std::path::Path;
use std::sync::Mutex;

use mahalaxmi_providers::{
    AiProvider, AuthMethod, GeminiProvider, I18nService, ProviderId, SupportedLocale,
};

// A mutex to ensure that environment variable tests don't interfere with each other.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ---------------------------------------------------------------------------
// GeminiProvider Tests
// ---------------------------------------------------------------------------

#[test]
fn gemini_provider_name() {
    let provider = GeminiProvider::new();
    assert_eq!(provider.name(), "Google Gemini");
}

#[test]
fn gemini_provider_id() {
    let provider = GeminiProvider::new();
    assert_eq!(provider.id(), &ProviderId::new("google-gemini"));
}

#[test]
fn gemini_provider_default_impl() {
    let p1 = GeminiProvider::new();
    let p2 = GeminiProvider::default();
    assert_eq!(p1.id(), p2.id());
    assert_eq!(p1.name(), p2.name());
}

#[test]
fn gemini_provider_build_command_args() {
    let provider = GeminiProvider::new();
    let cmd = provider
        .build_command(Path::new("/project"), "write a Rust function")
        .unwrap();
    assert_eq!(cmd.program, "gemini");
    assert!(cmd.args.contains(&"-p".to_string()));
    assert!(cmd.args.contains(&"write a Rust function".to_string()));
    assert_eq!(cmd.stdin_data, None);
}

#[test]
fn gemini_provider_build_command_working_dir() {
    let provider = GeminiProvider::new();
    let cmd = provider
        .build_command(Path::new("/my/awesome/project"), "task")
        .unwrap();
    assert_eq!(
        cmd.working_dir,
        Some(std::path::PathBuf::from("/my/awesome/project"))
    );
}

#[test]
fn gemini_provider_custom_binary() {
    let provider = GeminiProvider::with_binary("/usr/local/bin/my-gemini-cli");
    let cmd = provider.build_command(Path::new("/tmp"), "task").unwrap();
    assert_eq!(cmd.program, "/usr/local/bin/my-gemini-cli");
}

#[tokio::test]
async fn gemini_provider_validate_with_no_credentials() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("GEMINI_API_KEY");
    std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");

    let provider = GeminiProvider::new();
    let result = provider.validate_credentials(&i18n()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn gemini_provider_validate_with_api_key() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");
    std::env::set_var("GEMINI_API_KEY", "test_gemini_api_key");

    let provider = GeminiProvider::new();
    let result = provider.validate_credentials(&i18n()).await;
    assert!(result.is_ok());

    std::env::remove_var("GEMINI_API_KEY");
}

#[tokio::test]
async fn gemini_provider_validate_with_service_account_file() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("GEMINI_API_KEY");

    // Create a dummy service account file for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let sa_file_path = temp_dir.path().join("service_account.json");
    std::fs::write(&sa_file_path, "{}").unwrap(); // Empty JSON is enough for existence check

    std::env::set_var(
        "GOOGLE_APPLICATION_CREDENTIALS",
        sa_file_path.to_str().unwrap(),
    );

    let provider = GeminiProvider::new();
    let result = provider.validate_credentials(&i18n()).await;
    assert!(result.is_ok());

    std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");
}

#[tokio::test]
async fn gemini_provider_validate_with_nonexistent_service_account_file() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("GEMINI_API_KEY");
    std::env::set_var(
        "GOOGLE_APPLICATION_CREDENTIALS",
        "/nonexistent/path/to/sa.json",
    );

    let provider = GeminiProvider::new();
    let result = provider.validate_credentials(&i18n()).await;
    assert!(result.is_err());

    std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");
}

#[tokio::test]
async fn gemini_provider_localized_error() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("GEMINI_API_KEY");
    std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");

    let provider = GeminiProvider::new();
    let i18n_service = I18nService::new(SupportedLocale::EnUs);
    let err = provider
        .validate_credentials(&i18n_service)
        .await
        .unwrap_err();
    let key = err.i18n_key().unwrap();
    assert_eq!(key, "error-provider-credentials-missing");
    let msg = err.to_string();
    assert!(
        msg.contains("GEMINI_API_KEY") && msg.contains("GOOGLE_APPLICATION_CREDENTIALS"),
        "Error message should mention both env vars: {}",
        msg
    );
}

#[test]
fn gemini_provider_credential_requirements() {
    let provider = GeminiProvider::new();
    let creds = provider.credential_requirements();
    assert_eq!(creds.len(), 2);

    let api_key_cred = creds
        .iter()
        .find(|c| c.env_var_name == Some("GEMINI_API_KEY".to_string()))
        .unwrap();
    assert_eq!(api_key_cred.method, AuthMethod::ApiKey);
    assert!(!api_key_cred.required); // Optional because SA can be used

    let sa_cred = creds
        .iter()
        .find(|c| c.env_var_name == Some("GOOGLE_APPLICATION_CREDENTIALS".to_string()))
        .unwrap();
    assert_eq!(sa_cred.method, AuthMethod::ServiceAccount);
    assert!(!sa_cred.required); // Optional because API key can be used
}

#[test]
fn gemini_provider_capabilities() {
    let provider = GeminiProvider::new();
    let caps = provider.capabilities();
    assert!(caps.supports_streaming);
    assert!(caps.supports_agent_teams);
    assert!(caps.supports_tool_use);
    assert_eq!(caps.max_context_tokens, 32_768);
    assert!(caps.supports_web_search);
}

#[test]
fn gemini_provider_output_markers() {
    let provider = GeminiProvider::new();
    let markers = provider.output_markers();
    // These are placeholders from the implementation, actual regex might be more complex
    assert!(markers.completion_marker.is_match("GEMINI_COMPLETE"));
    assert!(markers.error_marker.is_match("Error: something"));
}

#[test]
fn gemini_provider_object_safety() {
    let provider: Box<dyn AiProvider> = Box::new(GeminiProvider::new());
    assert_eq!(provider.name(), "Google Gemini");
    assert_eq!(provider.id(), &ProviderId::new("google-gemini"));
}

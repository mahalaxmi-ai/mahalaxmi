use async_trait::async_trait;
use mahalaxmi_core::config::{GeminiConfig, MahalaxmiConfig, ModelConfig};
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ProcessCommand, ProviderId};
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

use crate::credentials::{AuthMethod, AuthMode, CredentialSpec};
use crate::markers::OutputMarkers;
use crate::metadata::ProviderMetadata;
use crate::traits::AiProvider;
use crate::types::ProviderCapabilities;

/// AI provider for Google Gemini.
#[derive(Clone)]
pub struct GeminiProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    /// The CLI binary name (default: "gemini").
    cli_binary: String,
    metadata: ProviderMetadata,
    /// The Gemini configuration from MahalaxmiConfig.
    config: GeminiConfig,
}

impl GeminiProvider {
    /// Create a new Gemini provider with default configuration.
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("google-gemini"),
            capabilities: ProviderCapabilities {
                supports_streaming: true,
                supports_agent_teams: true,
                supports_tool_use: true,
                max_context_tokens: 32_768,
                cost_tier: crate::types::CostTier::High,
                avg_latency_ms: 1000,
                supports_concurrent_sessions: true,
                task_proficiency: {
                    use crate::types::{TaskType, Proficiency};
                    let mut m = std::collections::HashMap::new();
                    m.insert(TaskType::CodeGeneration, Proficiency::Excellent);
                    m.insert(TaskType::CodeReview, Proficiency::Excellent);
                    m.insert(TaskType::Debugging, Proficiency::Excellent);
                    m.insert(TaskType::Refactoring, Proficiency::Excellent);
                    m.insert(TaskType::Testing, Proficiency::Excellent);
                    m.insert(TaskType::Documentation, Proficiency::Excellent);
                    m.insert(TaskType::Planning, Proficiency::Excellent);
                    m.insert(TaskType::General, Proficiency::Excellent);
                    m
                },
                supports_local_only: false,
                supports_web_search: true,
                supports_structured_output: true,
            },
            markers: OutputMarkers::new(
                r"GEMINI_COMPLETE",
                r"(?i)(error|fatal|failed)",
                r"(>\s*$|waiting for input)",
            )
            .expect("Gemini markers are valid regex"),
            metadata: ProviderMetadata::new("npm install -g @google/gemini-cli")
                .with_platform_install(
                    Some("npm install -g @google/gemini-cli"),
                    Some("npm install -g @google/gemini-cli"),
                    Some("npm install -g @google/gemini-cli"),
                )
                .with_install_url("https://github.com/google-gemini/gemini-cli")
                .with_test_args(vec![
                    "-p".to_string(),
                    "respond with OK".to_string(),
                ])
                .with_connection_check(
                    "if [ -n \"${GEMINI_API_KEY}\" ]; then \
                        curl -sf -H \"x-goog-api-key: ${GEMINI_API_KEY}\" --max-time 15 \
                        \"https://generativelanguage.googleapis.com/v1beta/models\" \
                        | grep -q '\"models\"'; \
                     elif [ -n \"${GOOGLE_APPLICATION_CREDENTIALS}\" ] && [ -f \"${GOOGLE_APPLICATION_CREDENTIALS}\" ]; then \
                        echo \"Service account file verified\"; \
                     else \
                        exit 1; \
                     fi",
                )
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "GEMINI_API_KEY".to_string(),
                })
                .with_auth_mode(AuthMode::ServiceAccount {
                    env_var: "GOOGLE_APPLICATION_CREDENTIALS".to_string(),
                    description: "Path to Google service account JSON file".to_string(),
                })
                .with_models(vec![
                    crate::metadata::ModelSpec {
                        id: "gemini-2.0-flash".to_string(),
                        name: "Gemini 2.0 Flash".to_string(),
                        description: "models.gemini_flash.description".to_string(),
                        is_default: true,
                    },
                    crate::metadata::ModelSpec {
                        id: "gemini-1.5-pro".to_string(),
                        name: "Gemini 1.5 Pro".to_string(),
                        description: "models.gemini_pro.description".to_string(),
                        is_default: false,
                    },
                ])
                .with_config_file("~/.gemini/config.json"),
            cli_binary: "gemini".to_string(),
            config: GeminiConfig::default(),
        }
    }

    /// Creates a provider with a custom CLI binary path.
    pub fn with_binary(cli_binary: &str) -> Self {
        let mut provider = Self::new();
        provider.cli_binary = cli_binary.to_string();
        provider
    }

    /// Test constructor — sets binary, model, and API key directly.
    #[cfg(test)]
    pub fn with_binary_and_model(cli_binary: &str, model: &str, api_key: Option<String>) -> Self {
        let mut provider = Self::new();
        provider.cli_binary = cli_binary.to_string();
        provider.config = GeminiConfig {
            api_key,
            selected_model: Some(model.to_string()),
            ..Default::default()
        };
        provider
    }

    /// Create a Gemini provider from the application configuration.
    pub fn from_mahalaxmi_config(mahalaxmi_config: &MahalaxmiConfig) -> Self {
        let mut provider = Self::new();
        provider.config = mahalaxmi_config.gemini.clone();
        provider
    }

    /// Select the best model based on manual selection or automatic scoring.
    fn select_best_model(&self) -> Option<ModelConfig> {
        if !self.config.auto_select.enabled {
            if let Some(ref selected_id) = self.config.selected_model {
                return self
                    .config
                    .models
                    .iter()
                    .find(|m| m.id == *selected_id && m.enabled)
                    .cloned();
            }
        }

        // Automatic scoring
        let mut best_model: Option<ModelConfig> = None;
        let mut highest_score = -1.0;

        for model in &self.config.models {
            if !model.enabled {
                continue;
            }

            // Score based on tier (cost), performance, and quality
            // tiers: Free (0), Tier1 (1.0), Tier1.5 (1.5), Tier3 (3.0)
            let cost_score = match model.tier {
                mahalaxmi_core::config::ModelTier::Free => 1.0,
                mahalaxmi_core::config::ModelTier::Tier1 => 0.7,
                mahalaxmi_core::config::ModelTier::Tier1_5 => 0.5,
                mahalaxmi_core::config::ModelTier::Tier3 => 0.2,
            };

            // In a real implementation, performance and quality would come from
            // provider metadata or historical data. Here we use heuristics.
            let performance_score = if model.id.contains("flash") { 0.9 } else { 0.6 };
            let quality_score = if model.id.contains("pro") { 0.95 } else { 0.75 };

            let score = (cost_score * self.config.auto_select.cost_weight)
                + (performance_score * self.config.auto_select.performance_weight)
                + (quality_score * self.config.auto_select.quality_weight);

            if score > highest_score {
                highest_score = score;
                best_model = Some(model.clone());
            }
        }

        best_model.or_else(|| {
            // Final fallback: just use the first enabled model or a hardcoded default if none configured
            self.config.models.iter().find(|m| m.enabled).cloned()
        })
    }
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for GeminiProvider {
    fn name(&self) -> &str {
        "Google Gemini"
    }

    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn cli_binary(&self) -> &str {
        &self.cli_binary
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    fn build_command(&self, working_dir: &Path, prompt: &str) -> MahalaxmiResult<ProcessCommand> {
        let model = self.select_best_model();
        let model_id = model
            .as_ref()
            .map(|m| m.id.clone())
            .unwrap_or_else(|| "gemini-1.5-flash".to_string());

        let mut command = ProcessCommand::new(&self.cli_binary)
            .arg("-m")
            .arg(&model_id)
            .arg("-p")
            .arg(prompt)
            .arg("--raw-output")
            .arg("--approval-mode")
            .arg("yolo")
            .working_dir(working_dir);

        // Apply optimal JSON configuration if present
        if let Some(ref m) = model {
            if let Some(ref opt_cfg) = m.optimal_config {
                command = command.arg("--config").arg(opt_cfg);
            }
        }

        if let Some(api_key) = &self.config.api_key {
            command = command.env_var("GEMINI_API_KEY", api_key);
        }

        Ok(command)
    }

    async fn validate_credentials(&self, i18n: &I18nService) -> MahalaxmiResult<()> {
        if self.config.api_key.is_some() {
            return Ok(());
        }

        let service_account_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS");
        if let Ok(path) = service_account_path {
            if Path::new(&path).exists() {
                return Ok(());
            }
        }

        let gemini_api_key_env = std::env::var("GEMINI_API_KEY");
        if let Ok(key) = gemini_api_key_env {
            if !key.is_empty() {
                return Ok(());
            }
        }

        Err(MahalaxmiError::provider(
            i18n,
            keys::provider::CREDENTIALS_MISSING,
            &[
                ("provider", "Google Gemini"),
                (
                    "env_var",
                    "MahalaxmiConfig, GOOGLE_APPLICATION_CREDENTIALS or GEMINI_API_KEY",
                ),
            ],
        ))
    }

    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        vec![
            CredentialSpec {
                method: AuthMethod::ApiKey,
                env_var_name: Some("GEMINI_API_KEY".to_string()),
                description_key: String::from("credential-gemini-api-key"),
                required: false, // Can also use Service Account
            },
            CredentialSpec {
                method: AuthMethod::ServiceAccount,
                env_var_name: Some("GOOGLE_APPLICATION_CREDENTIALS".to_string()),
                description_key: String::from("credential-google-service-account-path"),
                required: false, // Can also use API Key
            },
        ]
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    fn output_markers(&self) -> &OutputMarkers {
        &self.markers
    }

    fn apply_model_override(&mut self, model_id: Option<String>) {
        self.config.selected_model = model_id;
    }

    fn configure(&self, config: &MahalaxmiConfig) -> Box<dyn AiProvider> {
        Box::new(Self::from_mahalaxmi_config(config))
    }

    fn clone_box(&self) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }

    fn get_model_template(&self, model_id: &str) -> Option<String> {
        if model_id.contains("gemini") {
            Some(
                r#"{
  "temperature": 0.7,
  "top_p": 0.95,
  "top_k": 40,
  "max_output_tokens": 8192,
  "response_mime_type": "text/plain"
}"#
                .to_string(),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::AuthMethod;
    use mahalaxmi_core::config::{GeminiConfig, MahalaxmiConfig};
    use mahalaxmi_core::i18n::locale::SupportedLocale;
    use mahalaxmi_core::i18n::I18nService;
    use std::sync::Mutex;

    // A mutex to ensure that environment variable tests don't interfere with each other.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

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
        assert!(cmd.env.is_empty()); // No API key set
    }

    #[test]
    fn gemini_provider_build_command_args_with_api_key() {
        let provider = GeminiProvider::with_binary_and_model(
            "gemini",
            "gemini-1.5-flash",
            Some("test_api_key_from_config".to_string()),
        );
        let cmd = provider
            .build_command(Path::new("/project"), "write a Rust function")
            .unwrap();
        assert_eq!(cmd.program, "gemini");
        assert!(cmd.env.contains_key("GEMINI_API_KEY"));
        assert_eq!(
            cmd.env.get("GEMINI_API_KEY"),
            Some(&"test_api_key_from_config".to_string())
        );
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

        let provider = GeminiProvider::new(); // No API key from config
        let result = provider.validate_credentials(&i18n()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn gemini_provider_validate_with_api_key_from_config() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("GEMINI_API_KEY");
        std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");

        let provider = GeminiProvider::with_binary_and_model(
            "gemini",
            "gemini-1.5-flash",
            Some("config_api_key".to_string()),
        );
        let result = provider.validate_credentials(&i18n()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn gemini_provider_validate_with_api_key_from_env() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("GOOGLE_APPLICATION_CREDENTIALS");
        std::env::set_var("GEMINI_API_KEY", "test_gemini_api_key");

        let provider = GeminiProvider::new(); // No API key from config
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

        let provider = GeminiProvider::new(); // No API key from config
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

        let provider = GeminiProvider::new(); // No API key from config
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
            msg.contains("MahalaxmiConfig")
                && msg.contains("GEMINI_API_KEY")
                && msg.contains("GOOGLE_APPLICATION_CREDENTIALS"),
            "Error message should mention all credential sources: {}",
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
            .find(|c| c.method == AuthMethod::ApiKey)
            .unwrap();
        assert_eq!(
            api_key_cred.env_var_name,
            Some("GEMINI_API_KEY".to_string())
        );
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

    #[test]
    fn gemini_provider_from_mahalaxmi_config() {
        let mahalaxmi_config = MahalaxmiConfig {
            gemini: GeminiConfig {
                api_key: Some("gemini_config_key".to_string()),
                selected_model: Some("gemini-pro-test".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let provider = GeminiProvider::from_mahalaxmi_config(&mahalaxmi_config);
        assert_eq!(
            provider.config.api_key,
            Some("gemini_config_key".to_string())
        );
        assert_eq!(
            provider.config.selected_model,
            Some("gemini-pro-test".to_string())
        );
        assert_eq!(provider.cli_binary, "gemini"); // Default
    }
}

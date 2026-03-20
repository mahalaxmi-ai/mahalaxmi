use async_trait::async_trait;
use mahalaxmi_core::config::{CustomCliConfig, MahalaxmiConfig, ModelConfig};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ProcessCommand, ProviderId};
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

use crate::credentials::{AuthMode, CredentialSpec};
use crate::markers::OutputMarkers;
use crate::metadata::ProviderMetadata;
use crate::traits::AiProvider;
use crate::types::{CostTier, ProviderCapabilities};

#[derive(Clone)]
/// AI provider for custom CLI tools.
pub struct CustomCliProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    metadata: ProviderMetadata,
    config: CustomCliConfig,
}

impl CustomCliProvider {
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("custom-cli"),
            capabilities: ProviderCapabilities {
                supports_streaming: true,
                supports_agent_teams: false,
                supports_tool_use: false,
                max_context_tokens: 0,
                cost_tier: CostTier::Medium,
                avg_latency_ms: 0,
                supports_concurrent_sessions: true,
                task_proficiency: std::collections::HashMap::new(),
                supports_local_only: false,
                supports_web_search: false,
                supports_structured_output: false,
            },
            markers: OutputMarkers::new(
                r"CUSTOM_COMPLETE",
                r"(?i)(error|fatal|failed)",
                r"(>\s*$|waiting for input)",
            )
            .expect("Custom markers are valid regex"),
            metadata: ProviderMetadata::new("custom binary instructions")
                .with_auth_mode(AuthMode::None),
            config: CustomCliConfig::default(),
        }
    }

    pub fn from_mahalaxmi_config(mahalaxmi_config: &MahalaxmiConfig) -> Self {
        let mut provider = Self::new();
        provider.config = mahalaxmi_config.custom_cli.clone();
        provider
    }

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
        self.config.models.iter().find(|m| m.enabled).cloned()
    }
}

impl Default for CustomCliProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for CustomCliProvider {
    fn name(&self) -> &str {
        "Custom CLI"
    }

    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn cli_binary(&self) -> &str {
        // Return an empty string when no binary path has been configured.
        // The status computation treats an empty cli_binary as NotConfigured
        // rather than NotInstalled, since there is no tool to install —
        // the user simply hasn't filled in the binary path yet.
        self.config.binary_path.as_deref().unwrap_or("")
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    fn build_command(&self, working_dir: &Path, prompt: &str) -> MahalaxmiResult<ProcessCommand> {
        let binary = self.cli_binary();
        if binary.is_empty() {
            return Err(mahalaxmi_core::error::MahalaxmiError::Provider {
                message: "Custom CLI has no binary path configured. Set binary_path in provider settings.".to_string(),
                i18n_key: "provider.not_configured".to_string(),
            });
        }
        let mut command = ProcessCommand::new(binary);

        if let Some(ref args) = self.config.args {
            for arg in args {
                command = command.arg(arg);
            }
        }

        let model = self.select_best_model();
        if let Some(ref m) = model {
            command = command.arg("--model").arg(&m.id);
        }

        command = command.arg(prompt).working_dir(working_dir);

        if let Some(ref envs) = self.config.env_vars {
            for (k, v) in envs {
                command = command.env_var(k, v);
            }
        }

        Ok(command)
    }

    async fn validate_credentials(&self, _i18n: &I18nService) -> MahalaxmiResult<()> {
        Ok(())
    }

    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        vec![]
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

    fn configure(&self, config: &mahalaxmi_core::config::MahalaxmiConfig) -> Box<dyn AiProvider> {
        Box::new(Self::from_mahalaxmi_config(config))
    }

    fn clone_box(&self) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }
}

use async_trait::async_trait;
use mahalaxmi_core::config::{GrokConfig, MahalaxmiConfig, ModelConfig};
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
use crate::types::{CostTier, Proficiency, ProviderCapabilities, TaskType};

#[derive(Clone)]
/// AI provider for xAI's Grok CLI.
pub struct GrokProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    cli_binary: String,
    metadata: ProviderMetadata,
    config: GrokConfig,
}

impl GrokProvider {
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("xai-grok"),
            capabilities: ProviderCapabilities {
                supports_streaming: true,
                supports_agent_teams: true,
                supports_tool_use: true,
                max_context_tokens: 128_000,
                cost_tier: CostTier::High,
                avg_latency_ms: 2000,
                supports_concurrent_sessions: true,
                task_proficiency: {
                    let mut m = std::collections::HashMap::new();
                    m.insert(TaskType::CodeGeneration, Proficiency::Excellent);
                    m.insert(TaskType::Planning, Proficiency::Excellent);
                    m.insert(TaskType::General, Proficiency::Excellent);
                    m
                },
                supports_local_only: false,
                supports_web_search: true,
                supports_structured_output: true,
            },
            markers: OutputMarkers::new(
                r"GROK_COMPLETE",
                r"(?i)(error|fatal|failed)",
                r"(>\s*$|waiting for input)",
            )
            .expect("Grok markers are valid regex"),
            metadata: ProviderMetadata::new("npm install -g @xai/grok-cli")
                .with_platform_install(
                    Some("npm install -g @xai/grok-cli"),
                    Some("npm install -g @xai/grok-cli"),
                    Some("npm install -g @xai/grok-cli"),
                )
                .with_install_url("https://x.ai/grok")
                .with_test_args(vec!["-p".to_string(), "hi".to_string()])
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "XAI_API_KEY".to_string(),
                }),
            cli_binary: "grok".to_string(),
            config: GrokConfig::default(),
        }
    }

    pub fn from_mahalaxmi_config(mahalaxmi_config: &MahalaxmiConfig) -> Self {
        let mut provider = Self::new();
        provider.config = mahalaxmi_config.grok.clone();
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

        let mut best_model: Option<ModelConfig> = None;
        let mut highest_score = -1.0;

        for model in &self.config.models {
            if !model.enabled {
                continue;
            }

            let cost_score = match model.tier {
                mahalaxmi_core::config::ModelTier::Free => 1.0,
                mahalaxmi_core::config::ModelTier::Tier1 => 0.7,
                mahalaxmi_core::config::ModelTier::Tier1_5 => 0.5,
                mahalaxmi_core::config::ModelTier::Tier3 => 0.2,
            };

            let performance_score = if model.id.contains("fast") { 0.9 } else { 0.6 };
            let quality_score = if model.id.contains("pro") || model.id.contains("3") {
                0.95
            } else {
                0.8
            };

            let score = (cost_score * self.config.auto_select.cost_weight)
                + (performance_score * self.config.auto_select.performance_weight)
                + (quality_score * self.config.auto_select.quality_weight);

            if score > highest_score {
                highest_score = score;
                best_model = Some(model.clone());
            }
        }

        best_model.or_else(|| self.config.models.iter().find(|m| m.enabled).cloned())
    }
}

impl Default for GrokProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for GrokProvider {
    fn name(&self) -> &str {
        "xAI Grok"
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
        let mut command = ProcessCommand::new(&self.cli_binary)
            .arg("-p")
            .arg(prompt)
            .working_dir(working_dir);

        if let Some(ref m) = model {
            command = command.arg("-m").arg(&m.id);
            if let Some(ref opt_cfg) = m.optimal_config {
                command = command.arg("--config").arg(opt_cfg);
            }
        }

        if let Some(api_key) = &self.config.api_key {
            command = command.env_var("XAI_API_KEY", api_key);
        }

        Ok(command)
    }

    async fn validate_credentials(&self, i18n: &I18nService) -> MahalaxmiResult<()> {
        if self.config.api_key.is_some() {
            return Ok(());
        }

        if std::env::var("XAI_API_KEY")
            .map(|k| !k.is_empty())
            .unwrap_or(false)
        {
            return Ok(());
        }

        Err(MahalaxmiError::provider(
            i18n,
            keys::provider::CREDENTIALS_MISSING,
            &[("provider", "Grok"), ("env_var", "XAI_API_KEY")],
        ))
    }

    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        vec![CredentialSpec {
            method: AuthMethod::ApiKey,
            env_var_name: Some("XAI_API_KEY".to_string()),
            description_key: String::from("credential-xai-api-key"),
            required: true,
        }]
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

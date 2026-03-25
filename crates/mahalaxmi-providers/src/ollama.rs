use async_trait::async_trait;
use mahalaxmi_core::config::{MahalaxmiConfig, ModelConfig, OllamaConfig};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ProcessCommand, ProviderId};
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

use crate::credentials::{AuthMode, CredentialSpec};
use crate::markers::OutputMarkers;
use crate::metadata::{DeploymentConstraint, ProviderMetadata};
use crate::traits::AiProvider;
use crate::types::{CostTier, Proficiency, ProviderCapabilities, TaskType};

/// AI provider for local Ollama models.
#[derive(Clone)]
pub struct OllamaProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    cli_binary: String,
    metadata: ProviderMetadata,
    config: OllamaConfig,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("ollama"),
            capabilities: ProviderCapabilities {
                supports_streaming: true,
                supports_agent_teams: false,
                supports_tool_use: true,
                max_context_tokens: 32_000,
                cost_tier: CostTier::Free,
                avg_latency_ms: 100,
                supports_concurrent_sessions: true,
                task_proficiency: {
                    let mut m = std::collections::HashMap::new();
                    m.insert(TaskType::CodeGeneration, Proficiency::Good);
                    m.insert(TaskType::Planning, Proficiency::Basic);
                    m.insert(TaskType::General, Proficiency::Good);
                    m
                },
                supports_local_only: true,
                supports_web_search: false,
                supports_structured_output: true,
            },
            markers: OutputMarkers::new(
                r"OLLAMA_COMPLETE",
                r"(?i)(error|fatal|failed)",
                r"(>\s*$|waiting for input)",
            )
            .expect("Ollama markers are valid regex"),
            metadata: ProviderMetadata::new("curl -L https://ollama.com/download | sh")
                .with_platform_install(
                    Some("curl -L https://ollama.com/download | sh"),
                    Some("brew install ollama"),
                    Some("msiexec.exe /i https://ollama.com/download/OllamaSetup.exe"),
                )
                .with_install_url("https://ollama.com/")
                .with_test_args(vec!["--version".to_string()])
                .with_auth_mode(AuthMode::None)
                // Ollama connects to localhost:11434 — not viable in cloud deployments.
                .with_deployment_constraint(DeploymentConstraint::LocalOnly)
                .with_min_version("0.1.0"),
            cli_binary: "ollama".to_string(),
            config: OllamaConfig::default(),
        }
    }

    pub fn from_mahalaxmi_config(mahalaxmi_config: &MahalaxmiConfig) -> Self {
        let mut provider = Self::new();
        provider.config = mahalaxmi_config.ollama.clone();
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

            let cost_score = 1.0; // Local models are free

            let performance_score = if model.id.contains("small") || model.id.contains("7b") {
                0.95
            } else {
                0.5
            };
            let quality_score = if model.id.contains("coder") || model.id.contains("70b") {
                0.95
            } else {
                0.7
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

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn name(&self) -> &str {
        "Ollama (Local)"
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
            .unwrap_or_else(|| "llama3".to_string());

        let mut command = ProcessCommand::new(&self.cli_binary)
            .arg("run")
            .arg(&model_id)
            .arg(prompt)
            .working_dir(working_dir);

        if let Some(ref url) = self.config.api_url {
            command = command.env_var("OLLAMA_HOST", url);
        }

        Ok(command)
    }

    async fn validate_credentials(&self, _i18n: &I18nService) -> MahalaxmiResult<()> {
        Ok(()) // Ollama usually doesn't need credentials locally
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

    fn configure(&self, config: &MahalaxmiConfig) -> Box<dyn AiProvider> {
        Box::new(Self::from_mahalaxmi_config(config))
    }

    fn clone_box(&self) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }
}

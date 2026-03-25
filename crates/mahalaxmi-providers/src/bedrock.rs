use async_trait::async_trait;
use mahalaxmi_core::config::{AwsBedrockConfig, MahalaxmiConfig, ModelConfig};
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

/// AI provider for AWS Bedrock.
#[derive(Clone)]
pub struct BedrockProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    cli_binary: String,
    metadata: ProviderMetadata,
    config: AwsBedrockConfig,
}

impl BedrockProvider {
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("aws-bedrock"),
            capabilities: ProviderCapabilities {
                supports_streaming: true,
                supports_agent_teams: true,
                supports_tool_use: true,
                max_context_tokens: 200_000,
                cost_tier: CostTier::High,
                avg_latency_ms: 3000,
                supports_concurrent_sessions: true,
                task_proficiency: {
                    let mut m = std::collections::HashMap::new();
                    m.insert(TaskType::CodeGeneration, Proficiency::Excellent);
                    m.insert(TaskType::Planning, Proficiency::Excellent);
                    m.insert(TaskType::General, Proficiency::Excellent);
                    m
                },
                supports_local_only: false,
                supports_web_search: false,
                supports_structured_output: true,
            },
            markers: OutputMarkers::new(
                r"BEDROCK_COMPLETE",
                r"(?i)(error|fatal|failed)",
                r"(>\s*$|waiting for input)",
            )
            .expect("Bedrock markers are valid regex"),
            metadata: ProviderMetadata::new("aws bedrock instructions")
                .with_platform_install(
                    Some("curl \"https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip\" -o \"awscliv2.zip\""),
                    Some("curl \"https://awscli.amazonaws.com/AWSCLIV2.pkg\" -o \"AWSCLIV2.pkg\""),
                    Some("msiexec.exe /i https://awscli.amazonaws.com/AWSCLIV2.msi"),
                )
                .with_install_url("https://aws.amazon.com/bedrock/")
                .with_test_args(vec!["bedrock-runtime".to_string(), "list-foundation-models".to_string()])
                .with_auth_mode(AuthMode::ApiKey { env_var: "AWS_ACCESS_KEY_ID".to_string() })
                .with_models(vec![
                    crate::metadata::ModelSpec {
                        id: "anthropic.claude-sonnet-4-6".to_string(),
                        name: "Claude Sonnet 4.6 (Bedrock)".to_string(),
                        description: "models.bedrock_claude_sonnet.description".to_string(),
                        is_default: true,
                    },
                    crate::metadata::ModelSpec {
                        id: "anthropic.claude-opus-4-6".to_string(),
                        name: "Claude Opus 4.6 (Bedrock)".to_string(),
                        description: "models.bedrock_claude_opus.description".to_string(),
                        is_default: false,
                    },
                ]),
            cli_binary: "aws".to_string(),
            config: AwsBedrockConfig::default(),
        }
    }

    pub fn from_mahalaxmi_config(mahalaxmi_config: &MahalaxmiConfig) -> Self {
        let mut provider = Self::new();
        provider.config = mahalaxmi_config.aws_bedrock.clone();
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

            let performance_score = if model.id.contains("haiku") { 0.9 } else { 0.6 };
            let quality_score = if model.id.contains("opus") || model.id.contains("3.5") {
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

impl Default for BedrockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for BedrockProvider {
    fn name(&self) -> &str {
        "AWS Bedrock"
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

    fn build_command(&self, working_dir: &Path, _prompt: &str) -> MahalaxmiResult<ProcessCommand> {
        let model = self.select_best_model();
        let model_id = model
            .as_ref()
            .map(|m| m.id.clone())
            .unwrap_or_else(|| "anthropic.claude-3-sonnet-20240229-v1:0".to_string());

        let mut command = ProcessCommand::new(&self.cli_binary)
            .arg("bedrock-runtime")
            .arg("invoke-model")
            .arg("--model-id")
            .arg(&model_id)
            .working_dir(working_dir);

        if let Some(ref m) = model {
            if let Some(ref opt_cfg) = m.optimal_config {
                command = command.arg("--body").arg(opt_cfg);
            }
        }

        // Add prompt via body if not already in optimal_config
        // This is a simplification; actual Bedrock CLI usage requires a body JSON

        if let Some(ref region) = self.config.region {
            command = command.arg("--region").arg(region);
        }
        if let Some(ref profile) = self.config.profile {
            command = command.arg("--profile").arg(profile);
        }

        if let Some(key) = &self.config.access_key_id {
            command = command.env_var("AWS_ACCESS_KEY_ID", key);
        }
        if let Some(secret) = &self.config.secret_access_key {
            command = command.env_var("AWS_SECRET_ACCESS_KEY", secret);
        }

        Ok(command)
    }

    async fn validate_credentials(&self, i18n: &I18nService) -> MahalaxmiResult<()> {
        if self.config.access_key_id.is_some() && self.config.secret_access_key.is_some() {
            return Ok(());
        }

        if std::env::var("AWS_ACCESS_KEY_ID").is_ok()
            && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok()
        {
            return Ok(());
        }

        Err(MahalaxmiError::provider(
            i18n,
            keys::provider::CREDENTIALS_MISSING,
            &[
                ("provider", "AWS Bedrock"),
                ("env_var", "AWS_ACCESS_KEY_ID"),
            ],
        ))
    }

    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        vec![
            CredentialSpec {
                method: AuthMethod::ApiKey,
                env_var_name: Some("AWS_ACCESS_KEY_ID".to_string()),
                description_key: String::from("credential-aws-access-key-id"),
                required: true,
            },
            CredentialSpec {
                method: AuthMethod::ApiKey,
                env_var_name: Some("AWS_SECRET_ACCESS_KEY".to_string()),
                description_key: String::from("credential-aws-secret-access-key"),
                required: true,
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
}

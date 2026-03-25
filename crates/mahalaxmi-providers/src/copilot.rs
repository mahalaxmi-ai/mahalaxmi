use async_trait::async_trait;
use mahalaxmi_core::config::{CopilotConfig, MahalaxmiConfig, ModelConfig};
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ProcessCommand, ProviderId};
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

use crate::credentials::{AuthMethod, AuthMode, CredentialSpec};
use crate::markers::OutputMarkers;
use crate::metadata::{DeploymentConstraint, ProviderMetadata};
use crate::traits::AiProvider;
use crate::types::{CostTier, Proficiency, ProviderCapabilities, TaskType};

/// AI provider for GitHub Copilot CLI.
#[derive(Clone)]
pub struct CopilotProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    cli_binary: String,
    metadata: ProviderMetadata,
    config: CopilotConfig,
}

impl CopilotProvider {
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("github-copilot"),
            capabilities: ProviderCapabilities {
                supports_streaming: true,
                supports_agent_teams: false,
                supports_tool_use: true,
                max_context_tokens: 64_000,
                cost_tier: CostTier::Medium,
                avg_latency_ms: 1500,
                supports_concurrent_sessions: true,
                task_proficiency: {
                    let mut m = std::collections::HashMap::new();
                    m.insert(TaskType::CodeGeneration, Proficiency::Excellent);
                    m.insert(TaskType::Debugging, Proficiency::Good);
                    m.insert(TaskType::General, Proficiency::Good);
                    m
                },
                supports_local_only: false,
                supports_web_search: false,
                supports_structured_output: false,
            },
            markers: OutputMarkers::new(
                r"COPILOT_COMPLETE",
                r"(?i)(error|fatal|failed)",
                r"(>\s*$|waiting for input)",
            )
            .expect("Copilot markers are valid regex"),
            metadata: ProviderMetadata::new(
                "Install GitHub CLI: https://cli.github.com\n\
                 Then install the Copilot extension: gh extension install github/gh-copilot\n\
                 Then authenticate: gh auth login",
            )
            .with_install_url("https://github.com/features/copilot")
            // Check that the gh copilot extension is present, not just the gh binary.
            .with_install_check(
                "gh extension list 2>/dev/null | grep -q 'github/gh-copilot'",
            )
            .with_connection_check(
                "gh copilot --version 2>&1 | grep -qi 'copilot' && echo 'OK' || \
                 (echo 'gh copilot extension not installed. Run: gh extension install github/gh-copilot' && exit 1)",
            )
            .with_test_args(vec!["copilot".to_string(), "--version".to_string()])
            .with_auth_mode(AuthMode::CliLogin {
                login_command: "gh auth login".to_string(),
                check_command: "gh auth status".to_string(),
            })
            .with_deployment_constraint(DeploymentConstraint::LocalOnly),
            cli_binary: "gh".to_string(),
            config: CopilotConfig::default(),
        }
    }

    pub fn from_mahalaxmi_config(mahalaxmi_config: &MahalaxmiConfig) -> Self {
        let mut provider = Self::new();
        provider.config = mahalaxmi_config.copilot.clone();
        provider
    }

    #[allow(dead_code)]
    fn select_best_model(&self) -> Option<ModelConfig> {
        // GitHub Copilot CLI handles its own model selection usually,
        // but we allow manual override if configured.
        if !self.config.auto_select.enabled {
            // Manual selection logic
        }
        None
    }
}

impl Default for CopilotProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for CopilotProvider {
    fn name(&self) -> &str {
        "GitHub Copilot"
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
        let mut command = ProcessCommand::new(&self.cli_binary)
            .arg("copilot")
            .arg("suggest")
            .arg("-t")
            .arg("shell")
            .arg(prompt)
            .working_dir(working_dir);

        if let Some(token) = &self.config.github_token {
            command = command.env_var("GITHUB_TOKEN", token);
        }

        Ok(command)
    }

    async fn validate_credentials(&self, i18n: &I18nService) -> MahalaxmiResult<()> {
        if self.config.github_token.is_some() || self.config.gh_token.is_some() {
            return Ok(());
        }

        if std::env::var("GITHUB_TOKEN").is_ok() || std::env::var("GH_TOKEN").is_ok() {
            return Ok(());
        }

        // Also check gh CLI auth status if possible

        Err(MahalaxmiError::provider(
            i18n,
            keys::provider::CREDENTIALS_MISSING,
            &[("provider", "GitHub Copilot"), ("env_var", "GITHUB_TOKEN")],
        ))
    }

    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        vec![CredentialSpec {
            method: AuthMethod::ApiKey,
            env_var_name: Some("GITHUB_TOKEN".to_string()),
            description_key: String::from("credential-github-token"),
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

    fn configure(&self, config: &MahalaxmiConfig) -> Box<dyn AiProvider> {
        Box::new(Self::from_mahalaxmi_config(config))
    }

    fn clone_box(&self) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }
}

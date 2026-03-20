use async_trait::async_trait;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ProcessCommand, ProviderId};
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

use crate::credentials::{AuthMode, CredentialSpec};
use crate::markers::OutputMarkers;
use crate::metadata::ProviderMetadata;
use crate::traits::AiProvider;
use crate::types::ProviderCapabilities;

/// Mock AI provider for testing and development.
///
/// Returns canned responses and always validates successfully.
/// Never makes real API calls or spawns real AI processes.
#[derive(Clone)]
pub struct MockProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    metadata: ProviderMetadata,
}

impl MockProvider {
    /// Create a new mock provider.
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("mock"),
            capabilities: ProviderCapabilities {
                supports_streaming: true,
                supports_agent_teams: false,
                supports_tool_use: true,
                max_context_tokens: 100_000,
                ..Default::default()
            },
            markers: OutputMarkers::new(r"\$\s*$", r"(?i)error:", r">\s*$")
                .expect("mock provider markers are valid regex"),
            metadata: ProviderMetadata::new("No installation required (mock provider)")
                .with_test_args(vec!["mock-test".to_string()])
                .with_auth_mode(AuthMode::None),
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for MockProvider {
    fn name(&self) -> &str {
        "Mock Provider"
    }

    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn cli_binary(&self) -> &str {
        "echo"
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    fn build_command(&self, working_dir: &Path, prompt: &str) -> MahalaxmiResult<ProcessCommand> {
        Ok(ProcessCommand::new("echo")
            .arg(prompt) // Pass prompt as an argument
            .working_dir(working_dir))
    }

    async fn validate_credentials(&self, _i18n: &I18nService) -> MahalaxmiResult<()> {
        Ok(())
    }

    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        Vec::new()
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    fn output_markers(&self) -> &OutputMarkers {
        &self.markers
    }

    fn configure(&self, _config: &mahalaxmi_core::config::MahalaxmiConfig) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }

    fn clone_box(&self) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }
}

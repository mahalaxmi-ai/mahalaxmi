use async_trait::async_trait;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ProcessCommand, ProviderId};
use mahalaxmi_core::MahalaxmiResult;
use std::collections::HashMap;
use std::path::Path;

use crate::credentials::{AuthMode, CredentialSpec};
use crate::markers::OutputMarkers;
use crate::metadata::ProviderMetadata;
use crate::traits::AiProvider;
use crate::types::ProviderCapabilities;

/// A configurable provider for arbitrary CLI tools.
///
/// Allows users to integrate any AI CLI tool by specifying the binary,
/// arguments, environment variables, and output markers.
#[derive(Clone)]
pub struct GenericCliProvider {
    id: ProviderId,
    display_name: String,
    program: String,
    base_args: Vec<String>,
    env_vars: HashMap<String, String>,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    credentials: Vec<CredentialSpec>,
    metadata: ProviderMetadata,
}

impl GenericCliProvider {
    /// Create a new generic CLI provider.
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        program: impl Into<String>,
        markers: OutputMarkers,
    ) -> Self {
        let prog: String = program.into();
        let id_str: String = id.into();
        let name_str: String = display_name.into();
        // OpenAI-API-compatible providers support native structured JSON output.
        let supports_structured_output = id_str.to_ascii_lowercase().contains("openai")
            || name_str.to_ascii_lowercase().contains("openai");
        let capabilities = ProviderCapabilities {
            supports_structured_output,
            ..ProviderCapabilities::default()
        };
        Self {
            id: ProviderId::new(id_str),
            display_name: name_str,
            metadata: ProviderMetadata::new(format!("Install {}", &prog))
                .with_auth_mode(AuthMode::None),
            program: prog,
            base_args: Vec::new(),
            env_vars: HashMap::new(),
            capabilities,
            markers,
            credentials: Vec::new(),
        }
    }

    /// Add a base argument that is always included when building commands.
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.base_args.push(arg.into());
        self
    }

    /// Add an environment variable that is always set.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Set the capabilities for this provider.
    pub fn with_capabilities(mut self, capabilities: ProviderCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Add a credential requirement.
    pub fn with_credential(mut self, spec: CredentialSpec) -> Self {
        self.credentials.push(spec);
        self
    }

    /// Set the provider metadata (install hints, test commands, auth modes).
    pub fn with_metadata(mut self, metadata: ProviderMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[async_trait]
impl AiProvider for GenericCliProvider {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn id(&self) -> &ProviderId {
        &self.id
    }

    fn cli_binary(&self) -> &str {
        &self.program
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    fn build_command(&self, working_dir: &Path, prompt: &str) -> MahalaxmiResult<ProcessCommand> {
        let mut cmd = ProcessCommand::new(&self.program);
        for arg in &self.base_args {
            cmd = cmd.arg(arg);
        }
        // Pass the prompt as the final positional argument.
        // CLI AI tools (echo, ollama, any custom binary) receive it via argv
        // rather than stdin so the driver's sh wrapper doesn't need to redirect
        // a temp file, and the process can't silently ignore stdin.
        cmd = cmd.arg(prompt);
        for (key, value) in &self.env_vars {
            cmd = cmd.env_var(key, value);
        }
        cmd = cmd.working_dir(working_dir);
        Ok(cmd)
    }

    async fn validate_credentials(&self, i18n: &I18nService) -> MahalaxmiResult<()> {
        for spec in &self.credentials {
            if spec.required {
                if let Some(env_var) = &spec.env_var_name {
                    if std::env::var(env_var).unwrap_or_default().is_empty() {
                        return Err(MahalaxmiError::provider(
                            i18n,
                            keys::provider::CREDENTIALS_MISSING,
                            &[("provider", &self.display_name), ("env_var", env_var)],
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        self.credentials.clone()
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

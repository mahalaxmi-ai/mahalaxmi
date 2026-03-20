use async_trait::async_trait;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ProcessCommand, ProviderId};
use mahalaxmi_core::MahalaxmiResult;

use crate::credentials::CredentialSpec;
use crate::markers::OutputMarkers;
use crate::metadata::ProviderMetadata;
use crate::types::ProviderCapabilities;

/// Trait abstracting over AI CLI tools.
///
/// Every AI provider (Claude Code, OpenAI, Bedrock, etc.) implements this trait.
/// The orchestration engine interacts with providers exclusively through this interface.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Human-readable name for this provider (e.g., "Claude Code", "OpenAI Foundry").
    fn name(&self) -> &str;

    /// Unique identifier for this provider.
    fn id(&self) -> &ProviderId;

    /// The CLI binary name this provider requires (e.g., "claude", "ollama").
    fn cli_binary(&self) -> &str;

    /// Provider metadata: install hints, test commands, auth modes.
    fn metadata(&self) -> &ProviderMetadata;

    /// Build the shell command to launch this provider's CLI tool.
    ///
    /// The returned `ProcessCommand` is passed to the PTY engine for spawning.
    /// The `working_dir` parameter is the project directory to operate on.
    /// The `prompt` parameter is the task/instruction to send to the AI.
    fn build_command(
        &self,
        working_dir: &std::path::Path,
        prompt: &str,
    ) -> MahalaxmiResult<ProcessCommand>;

    /// Validate that this provider's credentials are available and valid.
    ///
    /// Returns Ok(()) if credentials are found and appear valid.
    /// Does NOT make network calls — only checks local availability (env vars, files, keyrings).
    /// Accepts `I18nService` so error messages are localized.
    async fn validate_credentials(&self, i18n: &I18nService) -> MahalaxmiResult<()>;

    /// Describe what credentials this provider needs.
    fn credential_requirements(&self) -> Vec<CredentialSpec>;

    /// Return this provider's capabilities.
    fn capabilities(&self) -> &ProviderCapabilities;

    /// Return the output markers for parsing this provider's output.
    fn output_markers(&self) -> &OutputMarkers;

    /// Additional CLI args to enable streaming output from this provider.
    /// Default: None (provider already streams or doesn't support it).
    fn streaming_args(&self) -> Option<Vec<String>> {
        None
    }

    /// Extract usable response text from raw terminal output.
    /// Providers with streaming formats (e.g., stream-json) override this
    /// to parse their format. Default: returns output as-is.
    fn extract_response(&self, output: &str) -> String {
        output.to_string()
    }

    /// Optional string marker that signals the provider's streaming output is complete.
    /// When this marker appears in the output, the driver can skip the idle timeout.
    /// Default: None (no early completion detection).
    fn stream_complete_marker(&self) -> Option<&str> {
        None
    }

    /// Validate that a line matching `stream_complete_marker()` truly signals
    /// session completion. Called by the driver after a marker match to prevent
    /// false positives from intermediate events that happen to contain the
    /// marker string. Default: `true` (marker presence alone is sufficient).
    fn validate_stream_completion(&self, marker_line: &str) -> bool {
        let _ = marker_line;
        true
    }

    /// Optional marker identifying the provider's initial handshake/init event.
    /// Output is not considered "real" until bytes arrive AFTER a line containing
    /// this marker. Prevents init metadata from triggering the idle timeout.
    /// Default: None (all output counts equally).
    fn stream_init_marker(&self) -> Option<&str> {
        None
    }

    /// Extract exact token usage from the provider's raw streaming output.
    ///
    /// Providers that embed usage metadata in their output (e.g., Claude Code's
    /// `"usage":{"inputTokens":X,"outputTokens":Y}` in the final `result` event)
    /// should override this to return exact counts. When the returned `TokenUsage`
    /// has `is_exact: true`, callers should prefer it over byte-based estimates.
    ///
    /// Returns `None` when no usage metadata is found; callers fall back to
    /// `TokenUsage::estimate_from_bytes`.
    fn extract_token_usage(&self, _raw_output: &str) -> Option<crate::cost::TokenUsage> {
        None
    }

    /// Override the model selection at runtime.
    ///
    /// Called by the orchestration driver immediately after [`configure`] to
    /// inject the user's UI selection stored in the credential store.  This
    /// takes precedence over any value in `config.toml` or the provider's
    /// automatic scoring logic.
    ///
    /// - `Some(model_id)` — use this model; an unknown ID is treated as a
    ///   free-text model name (valid for Ollama and similar open-ended providers).
    /// - `None` — clear any override; the provider falls back to its
    ///   `selected_model` config field and auto-selection scoring.
    ///
    /// Providers without model selection (e.g. `GenericCliProvider`,
    /// `MockProvider`) should leave this as the default no-op.
    fn apply_model_override(&mut self, model_id: Option<String>) {
        let _ = model_id;
    }

    /// Creates a new instance of the provider configured with the given MahalaxmiConfig.
    fn configure(&self, config: &mahalaxmi_core::config::MahalaxmiConfig) -> Box<dyn AiProvider>;

    /// Clones the provider into a new Boxed trait object.
    fn clone_box(&self) -> Box<dyn AiProvider>;

    /// Returns a pre-configured JSON template for the specific model.
    /// Returns `None` if no template is available for the given model ID.
    fn get_model_template(&self, _model_id: &str) -> Option<String> {
        None
    }
}

impl Clone for Box<dyn AiProvider> {
    fn clone(&self) -> Box<dyn AiProvider> {
        self.clone_box()
    }
}

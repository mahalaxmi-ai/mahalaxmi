// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Implementing the AiProvider trait from scratch
//
// This is the template every community contributor will copy when adding
// support for a new AI CLI tool. It implements a fictional "EchoProvider"
// that echoes its prompt back to stdout — useful for testing orchestration
// pipelines without a real AI backend.
//
// Every method is implemented with explanatory comments. Follow the same
// pattern for a real provider, substituting the actual CLI binary name,
// command arguments, credential requirements, and output markers.

use async_trait::async_trait;
use mahalaxmi_core::{
    config::MahalaxmiConfig,
    i18n::I18nService,
    types::{ProcessCommand, ProviderId},
    MahalaxmiResult,
};
use mahalaxmi_providers::{
    credentials::{AuthMethod, AuthMode, CredentialSpec},
    markers::OutputMarkers,
    metadata::ProviderMetadata,
    traits::AiProvider,
    types::{CostTier, ProviderCapabilities},
};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Provider struct
// ---------------------------------------------------------------------------

/// A fictional "Echo" provider that echoes the prompt back to stdout.
///
/// This struct holds any runtime state the provider needs — selected model,
/// resolved credentials, etc. For simple CLI wrappers this is often empty.
#[derive(Clone)]
pub struct EchoProvider {
    /// The provider's unique identifier. Must match what ProviderRegistry uses.
    id: ProviderId,
    /// Cached capabilities — computed once and returned by reference.
    capabilities: ProviderCapabilities,
    /// Cached metadata — install hints and auth mode description.
    metadata: ProviderMetadata,
    /// Output markers — regex patterns used to detect task completion.
    markers: OutputMarkers,
}

impl EchoProvider {
    /// Create a new EchoProvider.
    pub fn new() -> Self {
        // Declare what this provider is capable of.
        let capabilities = ProviderCapabilities {
            // Whether the provider emits streamed output.
            supports_streaming: false,
            // Whether the provider supports managing multiple agent teams.
            supports_agent_teams: false,
            // Whether the provider supports tool use / function calling.
            supports_tool_use: false,
            // Context window in tokens. Use 0 if unknown.
            max_context_tokens: 8_000,
            // Cost tier — Free means no API charges (runs locally).
            cost_tier: CostTier::Free,
            // Expected average latency in milliseconds (0 = unknown).
            avg_latency_ms: 10,
            // Whether the provider supports concurrent sessions.
            supports_concurrent_sessions: true,
            // Per-task proficiency map (empty = Good proficiency for all tasks).
            task_proficiency: HashMap::new(),
            // Whether this provider runs entirely locally (no cloud API calls).
            supports_local_only: true,
            // Whether this provider supports web search.
            supports_web_search: false,
            // Whether this provider supports native structured JSON output.
            supports_structured_output: false,
        };

        // Metadata: install instructions and auth modes shown in the UI.
        // Use ProviderMetadata::new(install_hint) then chain builder methods.
        let metadata = ProviderMetadata::new("echo is a built-in shell command — no install needed")
            .with_auth_mode(AuthMode::None);

        // Output markers: regex patterns the detection engine uses to decide
        // when this provider has completed a task, hit an error, or is waiting.
        //
        // For a real CLI: inspect what the tool prints at the end of a session.
        // Claude Code prints "Task completed" or a shell prompt; set those here.
        let markers = OutputMarkers::new(
            r"ECHO_DONE",      // completion marker
            r"ECHO_ERROR",     // error marker
            r"ECHO_PROMPT>",   // interactive prompt marker
        )
        .expect("invalid marker regex — fix the pattern");

        Self {
            id: ProviderId::new("echo-provider"),
            capabilities,
            metadata,
            markers,
        }
    }
}

impl Default for EchoProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// AiProvider implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl AiProvider for EchoProvider {
    /// Human-readable display name shown in the UI provider selector.
    fn name(&self) -> &str {
        "Echo Provider"
    }

    /// Unique provider ID. Must be stable across restarts — used as a map key.
    fn id(&self) -> &ProviderId {
        &self.id
    }

    /// The CLI binary name. The orchestration engine checks PATH for this binary.
    fn cli_binary(&self) -> &str {
        "echo"
    }

    /// Provider metadata: install hints, test commands, auth modes.
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    /// Build the shell command to launch this provider for a given task.
    ///
    /// The `working_dir` is the project root the AI should operate in.
    /// The `prompt` is the full task instruction to pass to the AI.
    ///
    /// For real CLIs: add authentication flags, model flags, non-interactive
    /// flags, and any provider-specific formatting to the args here.
    fn build_command(
        &self,
        working_dir: &std::path::Path,
        prompt: &str,
    ) -> MahalaxmiResult<ProcessCommand> {
        // Echo the prompt to stdout — our fictional "AI" just repeats it.
        // A real provider would be something like:
        //   vec!["claude", "--dangerously-skip-permissions", "-p", prompt]
        Ok(ProcessCommand {
            program: "echo".to_string(),
            args: vec![prompt.to_string()],
            // Environment variables needed for this provider.
            // Always inject only the credentials this provider needs — never
            // the full parent environment (PtySpawner does env_clear for isolation).
            env: HashMap::new(),
            // Set the working directory so the AI sees the right project context.
            working_dir: Some(working_dir.to_path_buf()),
            // Optional data to pipe to stdin after the process starts.
            stdin_data: None,
        })
    }

    /// Validate credentials without making network calls.
    ///
    /// Check env vars, keyring entries, or local files. Return Ok(()) if
    /// all required credentials are present and appear valid (not necessarily
    /// tested against the live API — that is the provider's job on first use).
    async fn validate_credentials(&self, _i18n: &I18nService) -> MahalaxmiResult<()> {
        // EchoProvider needs no credentials — always valid.
        // A real API-key provider would check:
        //   std::env::var("MY_API_KEY").map(|_| ()).map_err(|_| ...)
        Ok(())
    }

    /// Describe what credentials this provider needs.
    ///
    /// The UI uses this to show credential setup prompts and status indicators.
    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        // EchoProvider needs nothing.
        // A real provider would return:
        //   vec![CredentialSpec {
        //       method: AuthMethod::ApiKey,
        //       env_var_name: Some("MY_PROVIDER_API_KEY".to_string()),
        //       description_key: "credential-my-provider-api-key".to_string(),
        //       required: true,
        //   }]
        vec![]
    }

    /// Return provider capabilities — used by TaskRouter for routing decisions.
    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    /// Return output markers — used by the detection engine to recognize
    /// task completion, errors, and interactive prompts in the PTY stream.
    fn output_markers(&self) -> &OutputMarkers {
        &self.markers
    }

    /// Creates a new instance configured with the given MahalaxmiConfig.
    ///
    /// Called by the orchestration engine when building provider instances
    /// from user configuration. Return a new boxed provider that incorporates
    /// any relevant config fields (selected model, API key overrides, etc.).
    fn configure(&self, _config: &MahalaxmiConfig) -> Box<dyn AiProvider> {
        // EchoProvider has no config fields — return a fresh instance.
        Box::new(Self::new())
    }

    /// Clone the provider into a new boxed trait object.
    ///
    /// Required because `Box<dyn AiProvider>` needs to be cloneable
    /// for use across worker threads. The derive macro does not work
    /// through trait objects, so each provider implements this manually.
    fn clone_box(&self) -> Box<dyn AiProvider> {
        Box::new(self.clone())
    }

    // Optional overrides — only implement when the provider needs them:

    /// Streaming args: extra flags to enable streamed output.
    /// Omit if the provider already streams, or does not support it.
    fn streaming_args(&self) -> Option<Vec<String>> {
        None
    }

    /// Extract clean response text from raw terminal output.
    /// Override when the provider wraps output in a structured format (e.g., JSON events).
    fn extract_response(&self, output: &str) -> String {
        output.to_string()
    }
}

// ---------------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------------

fn main() {
    let provider = EchoProvider::new();

    println!("Provider:    {}", provider.name());
    println!("ID:          {}", provider.id().as_str());
    println!("CLI binary:  {}", provider.cli_binary());

    let cmd = provider
        .build_command(
            std::path::Path::new("/tmp/my-project"),
            "Add error handling to src/main.rs",
        )
        .expect("build_command failed");

    println!("Command:     {} {:?}", cmd.program, cmd.args);
    println!("Working dir: {:?}", cmd.working_dir);

    let caps = provider.capabilities();
    println!("Cost tier:   {:?}", caps.cost_tier);
    println!("Local only:  {}", caps.supports_local_only);
    println!("Streaming:   {}", caps.supports_streaming);
    println!("Max tokens:  {}", caps.max_context_tokens);

    // Demonstrate clone_box — used internally by the orchestration engine.
    let _cloned: Box<dyn AiProvider> = provider.clone_box();
    println!("clone_box:   ok");

    // Show credential requirements (empty for this provider).
    let creds = provider.credential_requirements();
    println!("Credentials required: {}", creds.len());

    // Show what a real API key credential spec looks like.
    let example_spec = CredentialSpec {
        method: AuthMethod::ApiKey,
        env_var_name: Some("MY_PROVIDER_API_KEY".to_string()),
        description_key: "credential-my-provider-api-key".to_string(),
        required: true,
    };
    println!("Example credential: method={:?}", example_spec.method);
}

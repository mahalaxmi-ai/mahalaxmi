use async_trait::async_trait;
use mahalaxmi_core::config::{ClaudeConfig, MahalaxmiConfig, ModelConfig};
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

#[derive(Clone)]
/// AI provider for Anthropic's Claude Code CLI tool.
///
/// Builds commands to invoke the `claude` CLI with appropriate arguments.
/// Supports both subscription login (`claude auth login`) and API key (`ANTHROPIC_API_KEY`).
pub struct ClaudeCodeProvider {
    id: ProviderId,
    capabilities: ProviderCapabilities,
    markers: OutputMarkers,
    /// The CLI binary name (default: "claude").
    cli_binary: String,
    metadata: ProviderMetadata,
    /// The Claude configuration from MahalaxmiConfig.
    config: ClaudeConfig,
}

impl ClaudeCodeProvider {
    /// Create a new Claude Code provider with default configuration.
    pub fn new() -> Self {
        Self {
            id: ProviderId::new("claude-code"),
            capabilities: ProviderCapabilities {
                supports_streaming: true,
                supports_agent_teams: true,
                supports_tool_use: true,
                max_context_tokens: 200_000,
                cost_tier: crate::types::CostTier::High,
                avg_latency_ms: 5000,
                supports_concurrent_sessions: true,
                task_proficiency: {
                    use crate::types::{Proficiency, TaskType};
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
                supports_web_search: false,
                supports_structured_output: false,
            },
            markers: OutputMarkers::new(
                r"\$\s*$",
                r"(?i)(error|fatal|failed)",
                r"(>\s*$|waiting for input)",
            )
            .expect("claude code markers are valid regex"),
            metadata: ProviderMetadata::new("npm install -g @anthropic-ai/claude-code")
                .with_platform_install(
                    Some("npm install -g @anthropic-ai/claude-code"),
                    Some("npm install -g @anthropic-ai/claude-code"),
                    Some("npm install -g @anthropic-ai/claude-code"),
                )
                .with_install_url("https://docs.anthropic.com/en/docs/claude-code")
                .with_test_args(vec!["--print".to_string(), "respond with OK".to_string()])
                .with_auth_mode(AuthMode::CliLogin {
                    login_command: "claude auth login".to_string(),
                    check_command: "claude auth status".to_string(),
                })
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "ANTHROPIC_API_KEY".to_string(),
                })
                .with_models(vec![
                    crate::metadata::ModelSpec {
                        id: "claude-opus-4-6".to_string(),
                        name: "Claude Opus 4.6".to_string(),
                        description: "models.claude_opus.description".to_string(),
                        is_default: false,
                    },
                    crate::metadata::ModelSpec {
                        id: "claude-sonnet-4-6".to_string(),
                        name: "Claude Sonnet 4.6".to_string(),
                        description: "models.claude_sonnet.description".to_string(),
                        is_default: true,
                    },
                    crate::metadata::ModelSpec {
                        id: "claude-haiku-4-5-20251001".to_string(),
                        name: "Claude Haiku 4.5".to_string(),
                        description: "models.claude_haiku.description".to_string(),
                        is_default: false,
                    },
                ])
                .with_config_file("~/.claude/settings.json")
                .with_one_shot_args(vec!["--print".to_string()]),
            cli_binary: "claude".to_string(),
            config: ClaudeConfig::default(),
        }
    }

    /// Probe the Claude subscription session immediately before worker spawn.
    ///
    /// When the provider is configured without an API key (subscription-login
    /// mode), this runs `claude auth status` to confirm the session is still
    /// active. If the session has expired the caller receives `AuthExpired`
    /// so the UI can surface a targeted "Re-authenticate" action rather than
    /// a generic orchestration failure.
    ///
    /// No-ops when an API key is configured — key auth never expires
    /// between app restarts.
    pub async fn validate_for_invocation(&self) -> MahalaxmiResult<()> {
        if self.config.api_key.is_some() {
            return Ok(());
        }

        let output = tokio::process::Command::new(&self.cli_binary)
            .args(["auth", "status"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .await
            .map_err(|e| MahalaxmiError::AuthExpired {
                provider: "Claude Code".into(),
                message: format!(
                    "Could not run 'claude auth status' to verify session: {e}. \
                     Please re-authenticate: run 'claude auth login'"
                ),
            })?;

        if !output.status.success() {
            return Err(MahalaxmiError::AuthExpired {
                provider: "Claude Code".into(),
                message: "Your Claude subscription session has expired. \
                           Please re-authenticate: run 'claude auth login'"
                    .into(),
            });
        }

        Ok(())
    }

    /// Creates a provider with a custom CLI binary path.
    pub fn with_binary(cli_binary: &str) -> Self {
        let mut provider = Self::new();
        provider.cli_binary = cli_binary.to_string();
        provider
    }

    /// Test constructor — sets binary and optional API key directly.
    #[cfg(test)]
    pub fn with_binary_and_key(cli_binary: &str, api_key: Option<String>) -> Self {
        let mut provider = Self::new();
        provider.cli_binary = cli_binary.to_string();
        provider.config = ClaudeConfig {
            api_key,
            ..Default::default()
        };
        provider
    }

    /// Create a Claude provider from the application configuration.
    pub fn from_mahalaxmi_config(mahalaxmi_config: &MahalaxmiConfig) -> Self {
        let mut provider = Self::new();
        provider.config = mahalaxmi_config.claude.clone();
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

            let cost_score = match model.tier {
                mahalaxmi_core::config::ModelTier::Free => 1.0,
                mahalaxmi_core::config::ModelTier::Tier1 => 0.7,
                mahalaxmi_core::config::ModelTier::Tier1_5 => 0.5,
                mahalaxmi_core::config::ModelTier::Tier3 => 0.2,
            };

            let performance_score = if model.id.contains("haiku") {
                0.95
            } else {
                0.6
            };
            let quality_score = if model.id.contains("opus") {
                0.98
            } else if model.id.contains("sonnet") {
                0.92
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

impl Default for ClaudeCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for ClaudeCodeProvider {
    fn name(&self) -> &str {
        "Claude Code"
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
            .arg("--print")
            .arg("--dangerously-skip-permissions")
            .working_dir(working_dir);

        if let Some(ref m) = model {
            command = command.arg("--model").arg(&m.id);
            if let Some(ref opt_cfg) = m.optimal_config {
                // Claude CLI might have different ways to pass JSON config,
                // assuming a generic --config for now as per user request for highly usable JSON config.
                command = command.arg("--config").arg(opt_cfg);
            }
        }

        command = command.arg(prompt);

        if let Some(api_key) = &self.config.api_key {
            command = command.env_var("ANTHROPIC_API_KEY", api_key);
        }

        Ok(command)
    }

    async fn validate_credentials(&self, i18n: &I18nService) -> MahalaxmiResult<()> {
        if self.config.api_key.is_some() {
            return Ok(());
        }

        match std::env::var("ANTHROPIC_API_KEY") {
            Ok(key) if !key.is_empty() => Ok(()),
            _ => Err(MahalaxmiError::provider(
                i18n,
                keys::provider::CREDENTIALS_MISSING,
                &[
                    ("provider", "Claude Code"),
                    ("env_var", "MahalaxmiConfig or ANTHROPIC_API_KEY"),
                ],
            )),
        }
    }

    fn credential_requirements(&self) -> Vec<CredentialSpec> {
        vec![CredentialSpec {
            method: AuthMethod::ApiKey,
            env_var_name: Some("ANTHROPIC_API_KEY".to_string()),
            description_key: String::from("credential-anthropic-api-key"),
            required: true,
        }]
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    fn output_markers(&self) -> &OutputMarkers {
        &self.markers
    }

    fn streaming_args(&self) -> Option<Vec<String>> {
        // Claude Code requires --verbose when combining --print with
        // --output-format=stream-json (otherwise exits with error).
        Some(vec![
            "--verbose".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
        ])
    }

    fn extract_response(&self, output: &str) -> String {
        // Tier 1: Non-agentic result event (--print without --verbose)
        for line in output.lines().rev() {
            let trimmed = line.trim();
            if trimmed.is_empty() || !trimmed.starts_with('{') {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if value.get("type").and_then(|t| t.as_str()) == Some("result") {
                    if let Some(result_text) = value.get("result").and_then(|r| r.as_str()) {
                        return result_text.to_string();
                    }
                }
            }
        }

        // Tier 2: Agentic format — extract text from assistant message content blocks
        let mut text_parts: Vec<String> = Vec::new();
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || !trimmed.starts_with('{') {
                continue;
            }
            let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
                continue;
            };
            if value.get("type").and_then(|t| t.as_str()) != Some("assistant") {
                continue;
            }
            // Navigate: message.content[] → blocks with type=="text"
            if let Some(content) = value
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array())
            {
                for block in content {
                    if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            if !text.is_empty() {
                                text_parts.push(text.to_string());
                            }
                        }
                    }
                }
            }
        }
        if !text_parts.is_empty() {
            return text_parts.join("\n\n");
        }

        // Tier 3: Fallback — return raw output
        output.to_string()
    }

    fn stream_complete_marker(&self) -> Option<&str> {
        // Claude Code's stream-json emits session stats (totalDurationMs, totalTokens,
        // usage) on the final event. This field only appears once the session is fully
        // complete, making it a reliable completion signal.
        Some("\"totalDurationMs\":")
    }

    fn validate_stream_completion(&self, marker_line: &str) -> bool {
        // In agentic mode, "totalDurationMs": can appear as a top-level field
        // on intermediate type=user events (tool results). Only the final
        // type=result event is a true completion signal.
        marker_line.contains("\"type\":\"result\"")
    }

    fn stream_init_marker(&self) -> Option<&str> {
        // Claude Code's --verbose --output-format stream-json emits a system init
        // event (~1KB) immediately, then goes silent for minutes while thinking.
        // This marker prevents that init event from triggering idle detection.
        Some("\"subtype\":\"init\"")
    }

    fn extract_token_usage(&self, raw_output: &str) -> Option<crate::cost::TokenUsage> {
        // Claude Code's stream-json format embeds real token counts in the final
        // result event: {"type":"result",...,"usage":{"inputTokens":X,"outputTokens":Y}}
        // The model ID is in the init event: {"type":"system","subtype":"init","model":"..."}
        // We scan the entire output once to collect both.
        let mut input_tokens: Option<u64> = None;
        let mut output_tokens: Option<u64> = None;
        let mut model_id: Option<String> = None;

        for line in raw_output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || !trimmed.starts_with('{') {
                continue;
            }
            let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
                continue;
            };

            match value.get("type").and_then(|t| t.as_str()) {
                Some("system") if value.get("subtype").and_then(|s| s.as_str()) == Some("init") => {
                    // Capture model from the first init event we see.
                    if model_id.is_none() {
                        if let Some(m) = value.get("model").and_then(|m| m.as_str()) {
                            model_id = Some(m.to_lowercase());
                        }
                    }
                }
                Some("result") => {
                    // Use the last result event in the stream (scan continues to EOF).
                    if let Some(usage) = value.get("usage") {
                        if let (Some(inp), Some(out)) = (
                            usage.get("inputTokens").and_then(|v| v.as_u64()),
                            usage.get("outputTokens").and_then(|v| v.as_u64()),
                        ) {
                            input_tokens = Some(inp);
                            output_tokens = Some(out);
                        }
                    }
                    // Some Claude CLI versions echo the model on the result event too.
                    if model_id.is_none() {
                        if let Some(m) = value.get("model").and_then(|m| m.as_str()) {
                            model_id = Some(m.to_lowercase());
                        }
                    }
                }
                _ => {}
            }
        }

        match (input_tokens, output_tokens) {
            (Some(inp), Some(out)) => Some(crate::cost::TokenUsage {
                input_tokens: inp,
                output_tokens: out,
                is_exact: true,
                model_id: model_id.unwrap_or_default(),
            }),
            _ => None,
        }
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
        if model_id.contains("claude") {
            Some(
                r#"{
  "temperature": 0.0,
  "max_tokens": 4096,
  "top_p": 1.0,
  "top_k": 0
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
    use mahalaxmi_core::config::{ClaudeConfig, MahalaxmiConfig};
    use mahalaxmi_core::i18n::locale::SupportedLocale;
    use mahalaxmi_core::i18n::I18nService;
    use std::sync::Mutex;

    // A mutex to ensure that environment variable tests don't interfere with each other.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    #[test]
    fn claude_provider_name() {
        let provider = ClaudeCodeProvider::new();
        assert_eq!(provider.name(), "Claude Code");
    }

    #[test]
    fn claude_provider_id() {
        let provider = ClaudeCodeProvider::new();
        assert_eq!(provider.id(), &ProviderId::new("claude-code"));
    }

    #[test]
    fn claude_provider_default_impl() {
        let p1 = ClaudeCodeProvider::new();
        let p2 = ClaudeCodeProvider::default();
        assert_eq!(p1.id(), p2.id());
        assert_eq!(p1.name(), p2.name());
    }

    #[test]
    fn claude_provider_build_command_args() {
        let provider = ClaudeCodeProvider::new();
        let cmd = provider
            .build_command(Path::new("/project"), "write a Rust function")
            .unwrap();
        assert_eq!(cmd.program, "claude");
        assert!(cmd.args.contains(&"--print".to_string()));
        assert!(cmd.args.contains(&"write a Rust function".to_string()));
        assert_eq!(cmd.stdin_data, None);
        assert!(cmd.env.is_empty()); // No API key set
    }

    #[test]
    fn claude_provider_build_command_args_with_api_key() {
        let provider = ClaudeCodeProvider::with_binary_and_key(
            "claude",
            Some("test_api_key_from_config".to_string()),
        );
        let cmd = provider
            .build_command(Path::new("/project"), "write a Rust function")
            .unwrap();
        assert_eq!(cmd.program, "claude");
        assert!(cmd.env.contains_key("ANTHROPIC_API_KEY"));
        assert_eq!(
            cmd.env.get("ANTHROPIC_API_KEY"),
            Some(&"test_api_key_from_config".to_string())
        );
    }

    #[test]
    fn claude_provider_build_command_working_dir() {
        let provider = ClaudeCodeProvider::new();
        let cmd = provider
            .build_command(Path::new("/my/awesome/project"), "task")
            .unwrap();
        assert_eq!(
            cmd.working_dir,
            Some(std::path::PathBuf::from("/my/awesome/project"))
        );
    }

    #[tokio::test]
    async fn claude_provider_validate_with_no_credentials() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("ANTHROPIC_API_KEY");

        let provider = ClaudeCodeProvider::new(); // No API key from config
        let result = provider.validate_credentials(&i18n()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn claude_provider_validate_with_api_key_from_config() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("ANTHROPIC_API_KEY");

        let provider =
            ClaudeCodeProvider::with_binary_and_key("claude", Some("config_api_key".to_string()));
        let result = provider.validate_credentials(&i18n()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn claude_provider_validate_with_api_key_from_env() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("ANTHROPIC_API_KEY", "test_anthropic_api_key");

        let provider = ClaudeCodeProvider::new(); // No API key from config
        let result = provider.validate_credentials(&i18n()).await;
        assert!(result.is_ok());

        std::env::remove_var("ANTHROPIC_API_KEY");
    }

    #[tokio::test]
    async fn claude_provider_localized_error() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("ANTHROPIC_API_KEY");

        let provider = ClaudeCodeProvider::new();
        let i18n_service = I18nService::new(SupportedLocale::EnUs);
        let err = provider
            .validate_credentials(&i18n_service)
            .await
            .unwrap_err();
        let key = err.i18n_key().unwrap();
        assert_eq!(key, "error-provider-credentials-missing");
        let msg = err.to_string();
        assert!(
            msg.contains("MahalaxmiConfig") && msg.contains("ANTHROPIC_API_KEY"),
            "Error message should mention all credential sources: {}",
            msg
        );
    }

    #[test]
    fn claude_provider_credential_requirements() {
        let provider = ClaudeCodeProvider::new();
        let creds = provider.credential_requirements();
        assert_eq!(creds.len(), 1);

        let api_key_cred = creds
            .iter()
            .find(|c| c.method == AuthMethod::ApiKey)
            .unwrap();
        assert_eq!(
            api_key_cred.env_var_name,
            Some("ANTHROPIC_API_KEY".to_string())
        );
        assert!(api_key_cred.required);
    }

    #[test]
    fn claude_provider_capabilities() {
        let provider = ClaudeCodeProvider::new();
        let caps = provider.capabilities();
        assert!(caps.supports_streaming);
        assert!(caps.supports_agent_teams);
        assert!(caps.supports_tool_use);
        assert_eq!(caps.max_context_tokens, 200_000);
    }

    #[test]
    fn claude_provider_output_markers() {
        let provider = ClaudeCodeProvider::new();
        let markers = provider.output_markers();
        assert!(markers.completion_marker.is_match("$ "));
        assert!(markers.error_marker.is_match("Error: something"));
    }

    #[test]
    fn claude_provider_object_safety() {
        let provider: Box<dyn AiProvider> = Box::new(ClaudeCodeProvider::new());
        assert_eq!(provider.name(), "Claude Code");
        assert_eq!(provider.id(), &ProviderId::new("claude-code"));
    }

    #[test]
    fn claude_provider_from_mahalaxmi_config() {
        let mahalaxmi_config = MahalaxmiConfig {
            claude: ClaudeConfig {
                api_key: Some("claude_config_key".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let provider = ClaudeCodeProvider::from_mahalaxmi_config(&mahalaxmi_config);
        assert_eq!(
            provider.config.api_key,
            Some("claude_config_key".to_string())
        );
        assert_eq!(provider.cli_binary, "claude"); // Default
    }

    // ── Tier 1: result-event extraction ───────────────────────────────────────
    //
    // Production anomaly (2026-03-01, cycle 14b90837, manager-0):
    //   raw_len: 6122 bytes  →  vt_len: 6075  →  clean_len: 13
    //   clean output: `{"tasks": []}`
    //
    // Tier 1 of extract_response() scans backwards for a `"type":"result"` JSON
    // line and returns its `result` field.  This test confirms that path works
    // correctly for both the empty-tasks anomaly and the normal success case.

    /// When Claude Code's result event contains `{"tasks": []}` as the model's
    /// response text, Tier 1 must extract exactly that string.  This is the
    /// exact scenario that triggers the empty-proposal anomaly.
    #[test]
    fn extract_response_tier1_empty_tasks_result_event() {
        // Minimal stream-json output that manager-0 effectively produced:
        // init event + result event with empty tasks JSON.
        let output = concat!(
            r#"{"type":"system","subtype":"init","cwd":"/project","tools":[],"model":"claude-sonnet-4-6","permissionMode":"bypassPermissions"}"#,
            "\n",
            r#"{"type":"result","subtype":"success","result":"{\"tasks\": []}","totalDurationMs":8823,"usage":{"inputTokens":1200,"outputTokens":5}}"#,
        );
        let extracted = ClaudeCodeProvider::new().extract_response(output);
        assert_eq!(
            extracted, r#"{"tasks": []}"#,
            "Tier 1 must return the result field verbatim, even when it is an empty-tasks JSON"
        );
    }

    /// A normal successful result event extracts the model's task JSON.
    #[test]
    fn extract_response_tier1_normal_result_event() {
        let tasks_json = r#"{"tasks":[{"title":"Add feature","description":"...","complexity":5,"priority":1,"dependencies":[],"affected_files":["src/lib.rs"]}]}"#;
        // Escape for JSON string embedding
        let escaped = tasks_json.replace('"', "\\\"");
        let output = format!(
            r#"{{"type":"result","subtype":"success","result":"{escaped}","totalDurationMs":300000}}"#
        );
        let extracted = ClaudeCodeProvider::new().extract_response(&output);
        assert_eq!(extracted, tasks_json);
    }

    /// Tier 1 must scan from the END of the output so the last result event
    /// wins, not an intermediate one.
    #[test]
    fn extract_response_tier1_uses_last_result_event() {
        let first = r#"{"type":"result","subtype":"success","result":"first-response","totalDurationMs":1}"#;
        let last =
            r#"{"type":"result","subtype":"success","result":"last-response","totalDurationMs":2}"#;
        let output = format!("{first}\n{last}");
        let extracted = ClaudeCodeProvider::new().extract_response(&output);
        assert_eq!(extracted, "last-response");
    }

    // ── Tier 2: assistant content block extraction ────────────────────────────

    /// When there is no result event (non-agentic or truncated output), Tier 2
    /// assembles text from assistant message content blocks.
    #[test]
    fn extract_response_tier2_assembles_assistant_text_blocks() {
        let output = r#"{"type":"assistant","message":{"model":"claude-sonnet-4-6","content":[{"type":"text","text":"Hello from Claude"}]}}"#;
        let extracted = ClaudeCodeProvider::new().extract_response(output);
        assert_eq!(extracted, "Hello from Claude");
    }

    /// Multiple assistant text blocks are joined with double newlines.
    #[test]
    fn extract_response_tier2_joins_multiple_blocks() {
        let output = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Part A"}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Part B"}]}}"#,
        );
        let extracted = ClaudeCodeProvider::new().extract_response(output);
        assert_eq!(extracted, "Part A\n\nPart B");
    }

    // ── Tier 3: fallback ──────────────────────────────────────────────────────

    /// When the output contains no JSON events at all, Tier 3 returns it raw.
    #[test]
    fn extract_response_tier3_returns_raw_for_plain_text() {
        let output = "plain text output with no json";
        let extracted = ClaudeCodeProvider::new().extract_response(output);
        assert_eq!(extracted, output);
    }

    // ── extract_token_usage ───────────────────────────────────────────────────

    /// A complete stream-json session with both init and result events yields
    /// exact token counts and the real model ID.
    #[test]
    fn extract_token_usage_returns_exact_counts_from_result_event() {
        let output = concat!(
            r#"{"type":"system","subtype":"init","cwd":"/project","tools":[],"model":"claude-sonnet-4-6","permissionMode":"bypassPermissions"}"#,
            "\n",
            r#"{"type":"result","subtype":"success","result":"done","totalDurationMs":8823,"usage":{"inputTokens":1200,"outputTokens":512}}"#,
        );
        let usage = ClaudeCodeProvider::new()
            .extract_token_usage(output)
            .expect("usage present");
        assert_eq!(usage.input_tokens, 1200);
        assert_eq!(usage.output_tokens, 512);
        assert!(usage.is_exact);
        assert_eq!(usage.model_id, "claude-sonnet-4-6");
    }

    /// When the result event is present but there is no init event, token counts
    /// are still returned but the model_id is empty (pricing falls back to defaults).
    #[test]
    fn extract_token_usage_works_without_init_event() {
        let output = r#"{"type":"result","subtype":"success","result":"done","totalDurationMs":1234,"usage":{"inputTokens":300,"outputTokens":50}}"#;
        let usage = ClaudeCodeProvider::new()
            .extract_token_usage(output)
            .expect("usage present");
        assert_eq!(usage.input_tokens, 300);
        assert_eq!(usage.output_tokens, 50);
        assert!(usage.is_exact);
        assert!(usage.model_id.is_empty());
    }

    /// When there is no result event, None is returned (caller falls back to byte estimation).
    #[test]
    fn extract_token_usage_returns_none_when_no_result_event() {
        let output = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"hi"}]}}"#;
        assert!(ClaudeCodeProvider::new()
            .extract_token_usage(output)
            .is_none());
    }

    /// When multiple result events appear (e.g., retry), the last one wins.
    #[test]
    fn extract_token_usage_uses_last_result_event() {
        let output = concat!(
            r#"{"type":"result","subtype":"success","result":"first","usage":{"inputTokens":100,"outputTokens":10}}"#,
            "\n",
            r#"{"type":"result","subtype":"success","result":"last","usage":{"inputTokens":5000,"outputTokens":2000}}"#,
        );
        let usage = ClaudeCodeProvider::new()
            .extract_token_usage(output)
            .expect("usage present");
        assert_eq!(usage.input_tokens, 5000);
        assert_eq!(usage.output_tokens, 2000);
    }

    /// Plain-text or non-JSON output returns None without panicking.
    #[test]
    fn extract_token_usage_handles_plain_text_output() {
        assert!(ClaudeCodeProvider::new()
            .extract_token_usage("plain text no json")
            .is_none());
        assert!(ClaudeCodeProvider::new().extract_token_usage("").is_none());
    }

    // ── stream_complete_marker / validate_stream_completion ───────────────────

    /// The completion marker is `"totalDurationMs":`.
    #[test]
    fn stream_complete_marker_is_total_duration() {
        assert_eq!(
            ClaudeCodeProvider::new().stream_complete_marker(),
            Some("\"totalDurationMs\":")
        );
    }

    /// Only a line containing both `"totalDurationMs":` AND `"type":"result"`
    /// is a true completion signal — not an intermediate tool-result event.
    #[test]
    fn validate_stream_completion_requires_type_result() {
        let p = ClaudeCodeProvider::new();
        // Final result event — valid completion
        assert!(p.validate_stream_completion(
            r#"{"type":"result","subtype":"success","totalDurationMs":1234}"#
        ));
        // Intermediate tool result with totalDurationMs — not a completion
        assert!(!p.validate_stream_completion(
            r#"{"type":"user","content":[{"type":"tool_result","totalDurationMs":50}]}"#
        ));
    }
}

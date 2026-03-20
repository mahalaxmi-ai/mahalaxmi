pub mod loader;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use toml;

use crate::security::encryption::EncryptedString;
use crate::types::VerificationCheck;

/// Global application configuration for AI providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawMahalaxmiConfig {
    #[serde(default)]
    pub general: RawGeneralConfig,
    #[serde(default)]
    pub providers: RawProvidersConfig,
    #[serde(default)]
    pub ui: RawUiConfig,
    #[serde(default)]
    pub licensing: RawLicensingConfig,
    #[serde(default)]
    pub indexing: RawIndexingConfig,
    #[serde(default)]
    pub verification: RawVerificationConfig,
    #[serde(default)]
    pub gemini: RawGeminiConfig,
    #[serde(default)]
    pub claude: RawClaudeConfig,
    #[serde(default)]
    pub ollama: RawOllamaConfig,
    #[serde(default)]
    pub aws_bedrock: RawAwsBedrockConfig,
    #[serde(default)]
    pub grok: RawGrokConfig,
    #[serde(default)]
    pub chatgpt: RawChatGptConfig,
    #[serde(default)]
    pub copilot: RawCopilotConfig,
    #[serde(default)]
    pub aider: RawAiderConfig,
    #[serde(default)]
    pub custom_cli: RawCustomCliConfig,
    #[serde(default)]
    pub team: Option<TeamConfig>,
    #[serde(default)]
    pub logging: RawLoggingConfig,
    #[serde(default)]
    pub orchestration: RawOrchestrationConfig,
    #[serde(default)]
    pub deliberation: AdversarialDeliberationConfig,
    #[serde(default)]
    pub git: RawGitConfig,
}

/// Global application configuration for AI providers (runtime, decrypted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MahalaxmiConfig {
    pub general: GeneralConfig,
    pub providers: ProvidersConfig,
    pub ui: UiConfig,
    pub licensing: LicensingConfig,
    pub indexing: IndexingConfig,
    pub verification: VerificationConfig,
    pub templates: TemplatesConfig,
    pub gemini: GeminiConfig,
    pub claude: ClaudeConfig,
    pub ollama: OllamaConfig,
    pub aws_bedrock: AwsBedrockConfig,
    pub grok: GrokConfig,
    pub chatgpt: ChatGptConfig,
    pub copilot: CopilotConfig,
    pub aider: AiderConfig,
    pub custom_cli: CustomCliConfig,
    pub team: Option<TeamConfig>,
    pub logging: LoggingConfig,
    pub orchestration: OrchestrationConfig,
    pub service: RemoteServiceConfig,
    pub graph: GraphConfig,
    pub mcp: McpConfig,
    pub memory: MemoryConfig,
    pub deliberation: AdversarialDeliberationConfig,
    pub git: GitConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RawGeneralConfig {
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub data_directory: Option<PathBuf>,
    #[serde(default)]
    pub terminal_verbose_logging: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub locale: String,
    pub log_level: String,
    pub data_directory: PathBuf,
    pub terminal_verbose_logging: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            locale: default_locale(),
            log_level: default_log_level(),
            data_directory: dirs::home_dir().unwrap_or_default().join(".mahalaxmi"),
            terminal_verbose_logging: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RawProvidersConfig {
    #[serde(default = "default_provider")]
    pub default_provider: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub default_provider: String,
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
        }
    }
}

fn default_provider() -> String {
    "claude-code".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawUiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_size")]
    pub terminal_font_size: u32,
}

impl Default for RawUiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            terminal_font_size: default_font_size(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub terminal_font_size: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            terminal_font_size: default_font_size(),
        }
    }
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_font_size() -> u32 {
    14
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RawLicensingConfig {
    #[serde(default = "default_offline_grace")]
    pub offline_grace_days: u32,
    pub license_file: Option<PathBuf>,
    pub verification_endpoint: Option<String>,
    pub platform_base_url: Option<String>,
    pub channel_api_key: Option<String>,
    pub product_id: Option<String>,
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_seconds: u64,
    #[serde(default = "default_message_poll_interval")]
    pub message_poll_interval_seconds: u64,
    pub cache_file: Option<PathBuf>,
    pub platform_api_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LicensingConfig {
    pub license_file: PathBuf,
    pub verification_endpoint: Option<String>,
    pub offline_grace_days: u32,
    pub platform_base_url: Option<String>,
    pub channel_api_key: Option<String>,
    pub product_id: Option<String>,
    pub heartbeat_interval_seconds: u64,
    pub message_poll_interval_seconds: u64,
    pub cache_file: PathBuf,
    pub platform_api_url: Option<String>,
    /// Whether offline validation (cache fallback) is permitted.
    /// Set from the embedded PAK's `offlineValidationEnabled` claim.
    /// Defaults to `true` to preserve compatibility with non-PAK deployments.
    #[serde(default = "default_true")]
    pub offline_validation_enabled: bool,
}

impl Default for LicensingConfig {
    fn default() -> Self {
        Self {
            license_file: dirs::home_dir()
                .unwrap_or_default()
                .join(".mahalaxmi")
                .join("license.key"),
            verification_endpoint: None,
            offline_grace_days: default_offline_grace(),
            platform_base_url: None,
            channel_api_key: None,
            product_id: None,
            heartbeat_interval_seconds: default_heartbeat_interval(),
            message_poll_interval_seconds: default_message_poll_interval(),
            cache_file: dirs::home_dir()
                .unwrap_or_default()
                .join(".mahalaxmi")
                .join("license_cache.bin"),
            platform_api_url: None,
            offline_validation_enabled: true,
        }
    }
}

fn default_offline_grace() -> u32 {
    7
}

fn default_heartbeat_interval() -> u64 {
    300
}

fn default_message_poll_interval() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawIndexingConfig {
    #[serde(default = "default_excluded_dirs")]
    pub excluded_dirs: Vec<String>,
    #[serde(default = "default_excluded_extensions")]
    pub excluded_extensions: Vec<String>,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_bytes: u64,
    #[serde(default = "default_damping")]
    pub ranking_damping_factor: f64,
    #[serde(default = "default_iterations")]
    pub ranking_max_iterations: usize,
    #[serde(default = "default_tokens")]
    pub max_tokens: usize,
    #[serde(default = "default_true")]
    pub include_signatures: bool,
    #[serde(default = "default_true")]
    pub include_doc_comments: bool,
    #[serde(default = "default_group_by")]
    pub group_by: String,
    pub enabled_languages: Option<Vec<String>>,
    pub root_path: Option<PathBuf>,
}

impl Default for RawIndexingConfig {
    fn default() -> Self {
        Self {
            excluded_dirs: default_excluded_dirs(),
            excluded_extensions: default_excluded_extensions(),
            max_file_size_bytes: default_max_file_size(),
            ranking_damping_factor: default_damping(),
            ranking_max_iterations: default_iterations(),
            max_tokens: default_tokens(),
            include_signatures: true,
            include_doc_comments: false,
            group_by: default_group_by(),
            enabled_languages: None,
            root_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub excluded_dirs: Vec<String>,
    pub excluded_extensions: Vec<String>,
    pub max_file_size_bytes: u64,
    pub ranking_damping_factor: f64,
    pub ranking_max_iterations: usize,
    pub max_tokens: usize,
    pub include_signatures: bool,
    pub include_doc_comments: bool,
    pub group_by: String,
    pub enabled_languages: Option<Vec<String>>,
    pub root_path: Option<PathBuf>,
}

impl Default for IndexingConfig {
    fn default() -> Self {
        let raw = RawIndexingConfig::default();
        Self {
            excluded_dirs: raw.excluded_dirs,
            excluded_extensions: raw.excluded_extensions,
            max_file_size_bytes: raw.max_file_size_bytes,
            ranking_damping_factor: raw.ranking_damping_factor,
            ranking_max_iterations: raw.ranking_max_iterations,
            max_tokens: raw.max_tokens,
            include_signatures: raw.include_signatures,
            include_doc_comments: raw.include_doc_comments,
            group_by: raw.group_by,
            enabled_languages: None,
            root_path: None,
        }
    }
}

fn default_excluded_dirs() -> Vec<String> {
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        "dist".to_string(),
        "build".to_string(),
        "__pycache__".to_string(),
    ]
}

fn default_excluded_extensions() -> Vec<String> {
    vec![
        ".min.js".to_string(),
        ".map".to_string(),
        ".lock".to_string(),
        "log".to_string(),
        "tmp".to_string(),
        "exe".to_string(),
        "dll".to_string(),
        "so".to_string(),
        "dylib".to_string(),
    ]
}

fn default_max_file_size() -> u64 {
    1024 * 1024
}

fn default_damping() -> f64 {
    0.85
}

fn default_iterations() -> usize {
    20
}

fn default_tokens() -> usize {
    2048
}

fn default_group_by() -> String {
    "file".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RawVerificationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub checks: Vec<VerificationCheck>,
    #[serde(default = "default_true")]
    pub auto_fix: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default)]
    pub fail_fast: bool,
    #[serde(default = "default_verification_timeout")]
    pub timeout_seconds: u64,
    /// Parse VERIFICATION: → FAIL lines from worker output and block completion.
    /// Default: true — safe change, only fires when workers explicitly output FAIL.
    #[serde(default = "default_true")]
    pub parse_verification_failures: bool,
    /// Require at least one VERIFICATION: line before TASK COMPLETE is accepted.
    /// Default: false — preserves existing behavior for workers that don't emit lines.
    #[serde(default)]
    pub require_verification_output: bool,
    /// Run a per-worker build check (TypeScript/Rust/Python) after commit and before PR (REQ-006).
    /// Default: false — opt-in to avoid breaking existing workflows.
    #[serde(default)]
    pub run_build_check_per_worker: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub enabled: bool,
    pub checks: Vec<VerificationCheck>,
    pub auto_fix: bool,
    pub max_retries: u32,
    pub fail_fast: bool,
    pub timeout_seconds: u64,
    /// When true, VERIFICATION: → FAIL lines block worker completion (REQ-002).
    pub parse_verification_failures: bool,
    /// When true, workers without any VERIFICATION: lines are re-queued (REQ-002).
    pub require_verification_output: bool,
    /// When true, runs a per-worker build check (TypeScript/Rust/Python) (REQ-006).
    pub run_build_check_per_worker: bool,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            checks: Vec::new(),
            auto_fix: true,
            max_retries: default_max_retries(),
            fail_fast: false,
            timeout_seconds: default_verification_timeout(),
            parse_verification_failures: true,
            require_verification_output: false,
            run_build_check_per_worker: false,
        }
    }
}

fn default_verification_timeout() -> u64 {
    300
}

/// Token format for context preambles sent to AI providers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContextFormat {
    #[default]
    Auto,
    Xml,
    Markdown,
    PlainText,
}

/// Budget allocation percentages across context sections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BudgetAllocations {
    /// Fraction of token budget allocated to the repo map (0.0–1.0).
    pub repo_map_pct: f64,
    /// Fraction of token budget allocated to relevant file content (0.0–1.0).
    pub relevant_files_pct: f64,
    /// Fraction of token budget allocated to shared memory entries (0.0–1.0).
    pub memory_pct: f64,
    /// Fraction of token budget allocated to the task description (0.0–1.0).
    pub task_description_pct: f64,
}

impl Default for BudgetAllocations {
    fn default() -> Self {
        Self {
            repo_map_pct: 0.15,
            relevant_files_pct: 0.60,
            memory_pct: 0.10,
            task_description_pct: 0.15,
        }
    }
}

/// Configuration for the context builder that prepares AI worker prompts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextConfig {
    /// Whether context preparation is enabled.
    pub enabled: bool,
    /// Whether to include a repo map in the context preamble.
    pub include_repo_map: bool,
    /// Maximum number of relevant files to include.
    pub max_files: usize,
    /// Whether to include shared memory entries in the context preamble.
    pub include_memory: bool,
    /// Output format for context preambles.
    pub format: ContextFormat,
    /// Budget allocations across context sections.
    pub budget_allocations: BudgetAllocations,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_repo_map: true,
            max_files: 20,
            include_memory: true,
            format: ContextFormat::Auto,
            budget_allocations: BudgetAllocations::default(),
        }
    }
}

/// Threshold configuration for acceptance validation verdicts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AcceptanceThreshold {
    /// Minimum confidence score (0.0–1.0) required to pass.
    pub min_confidence: f64,
    /// Minimum fulfillment status string required to pass (e.g. "fulfilled").
    pub min_status: String,
    /// If true, a failing threshold is a hard gate that stops the cycle.
    pub hard_gate: bool,
}

impl Default for AcceptanceThreshold {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            min_status: "partially_fulfilled".to_owned(),
            hard_gate: false,
        }
    }
}

fn default_max_retries() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplatesConfig {
    pub catalog_path: PathBuf,
    pub custom_directories: Vec<PathBuf>,
}

impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            catalog_path: dirs::home_dir()
                .unwrap_or_default()
                .join(".mahalaxmi")
                .join("templates"),
            custom_directories: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HotkeyConfig {
    pub enabled: bool,
    pub focus_next: String,
    pub focus_previous: String,
    pub toggle_orchestration: String,
    pub emergency_shutdown: String,
    pub cycle_layout: String,
    pub open_settings: String,
    pub quick_search: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            focus_next: "Ctrl+Shift+J".to_owned(),
            focus_previous: "Ctrl+Shift+K".to_owned(),
            toggle_orchestration: "Ctrl+Shift+O".to_owned(),
            emergency_shutdown: "Ctrl+Shift+Q".to_owned(),
            cycle_layout: "Ctrl+Shift+L".to_owned(),
            open_settings: "Ctrl+Shift+P".to_owned(),
            quick_search: "Ctrl+Shift+F".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutConfig {
    pub min_panel_width: f64,
    pub min_panel_height: f64,
    pub panel_padding: f64,
    pub preferred_aspect_ratio: f64,
    pub warn_panel_count: u32,
    pub warn_aspect_ratio_max: f64,
    pub warn_aspect_ratio_min: f64,
    pub score_weight_utilization: f64,
    pub score_weight_aspect_ratio: f64,
    pub score_weight_efficiency: f64,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            min_panel_width: 320.0,
            min_panel_height: 240.0,
            panel_padding: 4.0,
            preferred_aspect_ratio: 1.6,
            warn_panel_count: 25,
            warn_aspect_ratio_max: 4.0,
            warn_aspect_ratio_min: 0.25,
            score_weight_utilization: 0.4,
            score_weight_aspect_ratio: 0.3,
            score_weight_efficiency: 0.3,
        }
    }
}

/// Configuration for connecting the desktop app to a remote mahalaxmi-service instance.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RemoteServiceConfig {
    /// Base URL of the remote mahalaxmi-service (e.g. "http://localhost:17421").
    pub service_url: Option<String>,
    /// Optional API token for authenticating against the remote service.
    pub api_token: Option<String>,
    /// When true, the desktop app auto-starts the mahalaxmi-service binary on launch.
    #[serde(default)]
    pub auto_start: bool,
}

/// Transport mode for the MCP server.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "PascalCase")]
pub enum McpTransportType {
    #[default]
    Stdio,
    Sse,
}

/// Configuration for the MCP (Model Context Protocol) server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpConfig {
    /// Whether the MCP server is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Transport mode (Stdio or Sse).
    #[serde(default)]
    pub transport: McpTransportType,
    /// Host address for the SSE transport (default: "127.0.0.1").
    #[serde(default = "default_mcp_sse_host")]
    pub sse_host: String,
    /// Port to listen on for the SSE transport (default: 3001).
    #[serde(default = "default_mcp_sse_port")]
    pub sse_port: u16,
    /// List of tool names to enable (empty = all tools).
    #[serde(default)]
    pub enabled_tools: Vec<String>,
    /// Optional API token for authenticating MCP clients.
    #[serde(default)]
    pub api_token: Option<String>,
}

fn default_mcp_sse_host() -> String {
    "127.0.0.1".to_string()
}
fn default_mcp_sse_port() -> u16 {
    3001
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            transport: McpTransportType::Stdio,
            sse_host: default_mcp_sse_host(),
            sse_port: default_mcp_sse_port(),
            enabled_tools: Vec::new(),
            api_token: None,
        }
    }
}

/// Configuration for a single intake source (Jira, Slack, GitHub, REST endpoint).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntakeSourceConfig {
    /// Adapter type identifier: "jira", "slack", "github", "rest_endpoint".
    pub source_type: String,
    /// Stable identifier for this adapter instance.
    pub id: String,
    /// REST endpoint URL (used by rest/github adapters).
    pub url: Option<String>,
    /// Base URL for API calls (e.g. Jira instance root).
    pub base_url: Option<String>,
    /// Slack channel ID to listen on.
    pub channel: Option<String>,
    /// GitHub repository (owner/repo) to poll.
    pub repo: Option<String>,
    /// GitHub issue label filter.
    pub label: Option<String>,
    /// JQL filter string for Jira searches.
    pub jql_filter: Option<String>,
    /// Message prefix that triggers intake (e.g. "@mahalaxmi").
    pub trigger_prefix: Option<String>,
    /// Env var name containing an API key.
    pub api_key_env: Option<String>,
    /// Env var name containing an API token (used for Jira Basic Auth).
    pub api_token_env: Option<String>,
    /// Email address for Jira Basic Auth.
    pub email: Option<String>,
    /// Env var name containing a bot token (Slack).
    pub bot_token_env: Option<String>,
    /// How often to poll this source, in seconds.
    pub poll_interval_seconds: u64,
    /// Absolute path to the project root for generated work items.
    pub project_root: Option<PathBuf>,
}

impl Default for IntakeSourceConfig {
    fn default() -> Self {
        Self {
            source_type: String::new(),
            id: String::new(),
            url: None,
            base_url: None,
            channel: None,
            repo: None,
            label: None,
            jql_filter: None,
            trigger_prefix: None,
            api_key_env: None,
            api_token_env: None,
            email: None,
            bot_token_env: None,
            poll_interval_seconds: 60,
            project_root: None,
        }
    }
}

/// Configuration for the GraphRAG knowledge graph builder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphConfig {
    pub max_entities: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            max_entities: 100_000,
        }
    }
}

/// Configuration for application logging.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct RawLoggingConfig {
    #[serde(default)]
    pub session: bool,
    #[serde(default)]
    pub orchestration_phases: bool,
    #[serde(default)]
    pub manager: bool,
    #[serde(default)]
    pub worker: bool,
    #[serde(default)]
    pub consensus: bool,
    #[serde(default)]
    pub provider_routing: bool,
    #[serde(default)]
    pub pty: bool,
    #[serde(default)]
    pub detection: bool,
    #[serde(default)]
    pub git: bool,
    #[serde(default = "default_locale")]
    pub locale: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub session: bool,
    pub orchestration_phases: bool,
    pub manager: bool,
    pub worker: bool,
    pub consensus: bool,
    pub provider_routing: bool,
    pub pty: bool,
    pub detection: bool,
    pub git: bool,
    pub locale: String,
}

/// Configuration for a team of developers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TeamConfig {
    #[serde(default)]
    pub developers: Vec<DeveloperConfig>,
}

/// Configuration for an individual developer.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DeveloperConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(default = "default_developer_weight")]
    pub weight: f32,
    #[serde(default = "default_max_managers")]
    pub max_managers: u8,
    pub seat_id: Option<String>,
}

fn default_developer_weight() -> f32 {
    1.0
}

fn default_max_managers() -> u8 {
    2
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_locale() -> String {
    "en-US".to_string()
}

// =============================================================================
// Memory config
// =============================================================================

fn default_memory_max_entries() -> usize {
    1000
}
fn default_memory_confidence() -> f64 {
    0.5
}
fn default_memory_decay_rate() -> f64 {
    0.1
}
fn default_injector_max_tokens() -> usize {
    1024
}
fn default_injector_min_confidence() -> f64 {
    0.5
}
fn default_injector_format() -> String {
    "markdown".to_string()
}
fn default_injector_include_types() -> Vec<String> {
    vec![
        "convention".to_string(),
        "warning".to_string(),
        "discovery".to_string(),
        "pattern".to_string(),
    ]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryInjectorConfig {
    #[serde(default = "default_injector_max_tokens")]
    pub max_tokens: usize,
    #[serde(default = "default_injector_min_confidence")]
    pub min_confidence: f64,
    #[serde(default = "default_injector_format")]
    pub format: String,
    #[serde(default = "default_injector_include_types")]
    pub include_types: Vec<String>,
}

impl Default for MemoryInjectorConfig {
    fn default() -> Self {
        Self {
            max_tokens: default_injector_max_tokens(),
            min_confidence: default_injector_min_confidence(),
            format: default_injector_format(),
            include_types: default_injector_include_types(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub persistence_enabled: bool,
    #[serde(default = "default_memory_max_entries")]
    pub max_entries_per_session: usize,
    #[serde(default = "default_memory_confidence")]
    pub default_confidence: f64,
    #[serde(default = "default_memory_decay_rate")]
    pub confidence_decay_rate: f64,
    #[serde(default)]
    pub injector: MemoryInjectorConfig,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            persistence_enabled: false,
            max_entries_per_session: default_memory_max_entries(),
            default_confidence: default_memory_confidence(),
            confidence_decay_rate: default_memory_decay_rate(),
            injector: MemoryInjectorConfig::default(),
        }
    }
}

impl RawMahalaxmiConfig {
    /// Loads MahalaxmiConfig from the specified TOML file.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let toml_str = fs::read_to_string(path).with_context(|| {
            format!("Failed to read RawMahalaxmiConfig from {}", path.display())
        })?;
        toml::from_str(&toml_str)
            .with_context(|| format!("Failed to parse RawMahalaxmiConfig from {}", path.display()))
    }

    /// Checks if any fields contain encrypted data.
    pub fn has_encrypted_fields(&self) -> bool {
        self.gemini.api_key.is_some()
            || self.claude.api_key.is_some()
            || self.aws_bedrock.access_key_id.is_some()
            || self.aws_bedrock.secret_access_key.is_some()
            || self.grok.api_key.is_some()
            || self.chatgpt.openai_api_key.is_some()
            || self.chatgpt.codex_api_key.is_some()
            || self.copilot.github_token.is_some()
            || self.copilot.gh_token.is_some()
            || self.aider.anthropic_api_key.is_some()
            || self.aider.openai_api_key.is_some()
            || self.aider.gemini_api_key.is_some()
            || self.aider.xai_api_key.is_some()
            || self.aider.deepseek_api_key.is_some()
            || self.aider.openrouter_api_key.is_some()
            || self.aider.groq_api_key.is_some()
            || self.aider.mistral_api_key.is_some()
            || self.custom_cli.api_key.is_some()
            || self
                .custom_cli
                .env_vars
                .as_ref()
                .is_some_and(|m| m.values().any(|v| v.is_encrypted()))
            || self.orchestration.github_token.is_some()
    }
}

impl MahalaxmiConfig {
    pub fn from_raw(raw: RawMahalaxmiConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            general: GeneralConfig {
                locale: crate::i18n::locale::SupportedLocale::from_code(&raw.general.locale)
                    .map(|l| l.as_code().to_string())
                    .unwrap_or(raw.general.locale),
                log_level: raw.general.log_level,
                data_directory: raw
                    .general
                    .data_directory
                    .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".mahalaxmi")),
                terminal_verbose_logging: raw.general.terminal_verbose_logging,
            },
            providers: ProvidersConfig {
                default_provider: raw.providers.default_provider,
            },
            ui: UiConfig {
                theme: raw.ui.theme,
                terminal_font_size: raw.ui.terminal_font_size,
            },
            licensing: LicensingConfig {
                license_file: raw.licensing.license_file.unwrap_or_else(|| {
                    dirs::home_dir()
                        .unwrap_or_default()
                        .join(".mahalaxmi")
                        .join("license.key")
                }),
                verification_endpoint: raw.licensing.verification_endpoint,
                offline_grace_days: raw.licensing.offline_grace_days,
                platform_base_url: raw.licensing.platform_base_url,
                channel_api_key: raw.licensing.channel_api_key,
                product_id: raw.licensing.product_id,
                heartbeat_interval_seconds: raw.licensing.heartbeat_interval_seconds,
                message_poll_interval_seconds: raw.licensing.message_poll_interval_seconds,
                cache_file: raw.licensing.cache_file.unwrap_or_else(|| {
                    dirs::home_dir()
                        .unwrap_or_default()
                        .join(".mahalaxmi")
                        .join("license_cache.bin")
                }),
                platform_api_url: raw.licensing.platform_api_url,
                offline_validation_enabled: true,
            },
            indexing: IndexingConfig {
                excluded_dirs: raw.indexing.excluded_dirs,
                excluded_extensions: raw.indexing.excluded_extensions,
                max_file_size_bytes: raw.indexing.max_file_size_bytes,
                ranking_damping_factor: raw.indexing.ranking_damping_factor,
                ranking_max_iterations: raw.indexing.ranking_max_iterations,
                max_tokens: raw.indexing.max_tokens,
                include_signatures: raw.indexing.include_signatures,
                include_doc_comments: raw.indexing.include_doc_comments,
                group_by: raw.indexing.group_by,
                enabled_languages: raw.indexing.enabled_languages,
                root_path: raw.indexing.root_path,
            },
            templates: TemplatesConfig::default(),
            verification: VerificationConfig {
                enabled: raw.verification.enabled,
                checks: raw.verification.checks,
                auto_fix: raw.verification.auto_fix,
                max_retries: raw.verification.max_retries,
                fail_fast: raw.verification.fail_fast,
                timeout_seconds: raw.verification.timeout_seconds,
                parse_verification_failures: raw.verification.parse_verification_failures,
                require_verification_output: raw.verification.require_verification_output,
                run_build_check_per_worker: raw.verification.run_build_check_per_worker,
            },
            gemini: GeminiConfig::from_raw(raw.gemini, encryption_key),
            claude: ClaudeConfig::from_raw(raw.claude, encryption_key),
            ollama: OllamaConfig::from_raw(raw.ollama, encryption_key),
            aws_bedrock: AwsBedrockConfig::from_raw(raw.aws_bedrock, encryption_key),
            grok: GrokConfig::from_raw(raw.grok, encryption_key),
            chatgpt: ChatGptConfig::from_raw(raw.chatgpt, encryption_key),
            copilot: CopilotConfig::from_raw(raw.copilot, encryption_key),
            aider: AiderConfig::from_raw(raw.aider, encryption_key),
            custom_cli: CustomCliConfig::from_raw(raw.custom_cli, encryption_key),
            team: raw.team,
            logging: LoggingConfig {
                session: raw.logging.session,
                orchestration_phases: raw.logging.orchestration_phases,
                manager: raw.logging.manager,
                worker: raw.logging.worker,
                consensus: raw.logging.consensus,
                provider_routing: raw.logging.provider_routing,
                pty: raw.logging.pty,
                detection: raw.logging.detection,
                git: raw.logging.git,
                locale: raw.logging.locale,
            },
            orchestration: OrchestrationConfig::from_raw(raw.orchestration, encryption_key),
            service: RemoteServiceConfig::default(),
            graph: GraphConfig::default(),
            mcp: McpConfig::default(),
            memory: MemoryConfig::default(),
            deliberation: raw.deliberation,
            git: GitConfig::from_raw(raw.git),
        }
    }

    /// Merges project-specific overrides into this global MahalaxmiConfig.
    pub fn merge_with_project_overrides(
        mut self,
        project_overrides: Option<ProjectMahalaxmiConfig>,
    ) -> Self {
        if let Some(project_cfg) = project_overrides {
            if let Some(gemini) = project_cfg.gemini {
                self.gemini = self.gemini.merge(gemini);
            }
            if let Some(claude) = project_cfg.claude {
                self.claude = self.claude.merge(claude);
            }
            if let Some(ollama) = project_cfg.ollama {
                self.ollama = self.ollama.merge(ollama);
            }
            if let Some(aws_bedrock) = project_cfg.aws_bedrock {
                self.aws_bedrock = self.aws_bedrock.merge(aws_bedrock);
            }
            if let Some(grok) = project_cfg.grok {
                self.grok = self.grok.merge(grok);
            }
            if let Some(chatgpt) = project_cfg.chatgpt {
                self.chatgpt = self.chatgpt.merge(chatgpt);
            }
            if let Some(copilot) = project_cfg.copilot {
                self.copilot = self.copilot.merge(copilot);
            }
            if let Some(aider) = project_cfg.aider {
                self.aider = self.aider.merge(aider);
            }
            if let Some(custom_cli) = project_cfg.custom_cli {
                self.custom_cli = self.custom_cli.merge(custom_cli);
            }
        }
        self
    }
}

// =============================================================================
// Orchestration Config
// =============================================================================

/// Orchestration execution mode that controls review and precision behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OrchestrationMode {
    /// Standard orchestration: balanced speed and quality.
    #[serde(rename = "standard")]
    #[default]
    Standard,
    /// Review mode: each worker output is reviewed before merge.
    #[serde(rename = "review")]
    Review,
    /// Precise mode: stricter consensus thresholds and slower, higher-quality runs.
    #[serde(rename = "precise")]
    Precise,
}

impl std::fmt::Display for OrchestrationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestrationMode::Standard => write!(f, "standard"),
            OrchestrationMode::Review => write!(f, "review"),
            OrchestrationMode::Precise => write!(f, "precise"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawOrchestrationConfig {
    #[serde(default)]
    pub github_token: Option<EncryptedString>,
    #[serde(default)]
    pub autonomous_mode: bool,
    #[serde(default = "default_max_workers")]
    pub max_concurrent_workers: u32,
    #[serde(default = "default_consensus_strategy")]
    pub consensus_strategy: String,
    #[serde(default = "default_timeout")]
    pub manager_timeout_seconds: u64,
    #[serde(default = "default_worker_timeout")]
    pub worker_timeout_seconds: u64,
    #[serde(default)]
    pub enable_plan_review: bool,
    #[serde(default = "default_plan_review_timeout")]
    pub plan_review_timeout_secs: u64,
    #[serde(default)]
    pub security_strict_mode: bool,
    #[serde(default)]
    pub allowed_licenses: Vec<String>,
    #[serde(default)]
    pub max_cycle_cost_usd: Option<f64>,
    #[serde(default)]
    pub acceptance_commands: Vec<String>,
    #[serde(default = "default_max_parallel_commands")]
    pub max_parallel_commands: usize,
    #[serde(default = "default_command_timeout")]
    pub per_command_timeout_seconds: u64,
    #[serde(default = "default_validator_timeout")]
    pub validator_timeout_seconds: u64,
    #[serde(default)]
    pub acceptance_threshold: AcceptanceThreshold,
    /// Default number of manager agents per cycle when the caller sends `manager_count == 0`.
    /// Configurable via `[orchestration] default_manager_count` in `~/.mahalaxmi/config.toml`.
    #[serde(default = "default_manager_count")]
    pub default_manager_count: u32,
    /// Orchestration execution mode (standard, review, or precise).
    #[serde(default)]
    pub mode: OrchestrationMode,
    /// Maximum number of retries for a single batch before it is marked failed.
    #[serde(default = "default_max_batch_retries")]
    pub max_batch_retries: usize,
    /// Maximum total number of batches allowed in a single orchestration cycle.
    #[serde(default = "default_max_total_batches")]
    pub max_total_batches: usize,
    /// Minimum consecutive identical-state observations before stall is declared.
    #[serde(default = "default_stall_detection_threshold")]
    pub stall_detection_threshold: usize,
    /// Minimum number of manager proposals required before consensus runs (REQ-008).
    /// Default: 1 — preserves existing behavior (any single proposal proceeds).
    /// Set to 2 for majority quorum, or match manager_count for unanimity.
    #[serde(default = "default_min_manager_quorum")]
    pub min_manager_quorum: u32,
    /// Hard timeout (seconds) for all manager agents combined (REQ-007).
    /// When this wall-clock deadline elapses, remaining managers are aborted and
    /// consensus runs with whatever proposals arrived. Default: 600 s (10 min).
    #[serde(default = "default_manager_hard_timeout")]
    pub manager_hard_timeout_seconds: u64,
}

fn default_min_manager_quorum() -> u32 {
    1
}

fn default_manager_hard_timeout() -> u64 {
    600
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    pub github_token: Option<String>,
    pub autonomous_mode: bool,
    pub max_concurrent_workers: u32,
    pub consensus_strategy: String,
    pub manager_timeout_seconds: u64,
    pub worker_timeout_seconds: u64,
    pub enable_plan_review: bool,
    pub plan_review_timeout_secs: u64,
    pub security_strict_mode: bool,
    pub allowed_licenses: Vec<String>,
    pub max_cycle_cost_usd: Option<f64>,
    pub acceptance_commands: Vec<String>,
    pub max_parallel_commands: usize,
    pub per_command_timeout_seconds: u64,
    pub validator_timeout_seconds: u64,
    pub acceptance_threshold: AcceptanceThreshold,
    /// Default number of manager agents per cycle when the caller sends `manager_count == 0`.
    /// Configurable via `[orchestration] default_manager_count` in `~/.mahalaxmi/config.toml`.
    pub default_manager_count: u32,
    /// Orchestration execution mode (standard, review, or precise).
    pub mode: OrchestrationMode,
    /// Maximum number of retries for a single batch before it is marked failed.
    pub max_batch_retries: usize,
    /// Maximum total number of batches allowed in a single orchestration cycle.
    pub max_total_batches: usize,
    /// Minimum consecutive identical-state observations before stall is declared.
    pub stall_detection_threshold: usize,
    /// Minimum number of manager proposals required before consensus runs (REQ-008).
    pub min_manager_quorum: u32,
    /// Hard timeout (seconds) for all manager agents combined (REQ-007).
    pub manager_hard_timeout_seconds: u64,
}

impl OrchestrationConfig {
    /// Validate the orchestration configuration for obviously invalid values.
    ///
    /// Returns `Ok(())` when the configuration is sane, or `Err(String)` with
    /// a human-readable description of the first problem found.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_batch_retries == 0 {
            return Err(
                "orchestration.max_batch_retries must be at least 1".to_string(),
            );
        }
        if self.max_total_batches == 0 {
            return Err(
                "orchestration.max_total_batches must be at least 1".to_string(),
            );
        }
        if self.stall_detection_threshold == 0 {
            return Err(
                "orchestration.stall_detection_threshold must be at least 1".to_string(),
            );
        }
        Ok(())
    }
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            github_token: None,
            autonomous_mode: false,
            max_concurrent_workers: default_max_workers(),
            consensus_strategy: default_consensus_strategy(),
            manager_timeout_seconds: default_timeout(),
            worker_timeout_seconds: default_worker_timeout(),
            enable_plan_review: false,
            plan_review_timeout_secs: default_plan_review_timeout(),
            security_strict_mode: false,
            allowed_licenses: Vec::new(),
            max_cycle_cost_usd: None,
            acceptance_commands: Vec::new(),
            max_parallel_commands: default_max_parallel_commands(),
            per_command_timeout_seconds: default_command_timeout(),
            validator_timeout_seconds: default_validator_timeout(),
            acceptance_threshold: AcceptanceThreshold::default(),
            default_manager_count: default_manager_count(),
            mode: OrchestrationMode::default(),
            max_batch_retries: default_max_batch_retries(),
            max_total_batches: default_max_total_batches(),
            stall_detection_threshold: default_stall_detection_threshold(),
            min_manager_quorum: default_min_manager_quorum(),
            manager_hard_timeout_seconds: default_manager_hard_timeout(),
        }
    }
}

fn default_max_workers() -> u32 {
    0
}
fn default_manager_count() -> u32 {
    3
}
fn default_consensus_strategy() -> String {
    "weighted_voting".to_string()
}
fn default_timeout() -> u64 {
    300
}
fn default_worker_timeout() -> u64 {
    600
}
fn default_plan_review_timeout() -> u64 {
    300
}
fn default_max_parallel_commands() -> usize {
    4
}
fn default_command_timeout() -> u64 {
    120
}
fn default_validator_timeout() -> u64 {
    300
}
fn default_max_batch_retries() -> usize {
    3
}
fn default_max_total_batches() -> usize {
    20
}
fn default_stall_detection_threshold() -> usize {
    2
}

fn default_sequential_merge_threshold() -> u32 {
    2
}

/// Git-specific global configuration (raw, from TOML).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawGitConfig {
    /// When true, Mahalaxmi falls back to sequential PR merging after
    /// `sequential_merge_threshold` auto-merge failures in a single cycle.
    #[serde(default)]
    pub sequential_merge_fallback: bool,
    /// Number of auto-merge failures before sequential fallback activates.
    #[serde(default = "default_sequential_merge_threshold")]
    pub sequential_merge_threshold: u32,
}

/// Git-specific global configuration (runtime).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub sequential_merge_fallback: bool,
    pub sequential_merge_threshold: u32,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            sequential_merge_fallback: false,
            sequential_merge_threshold: default_sequential_merge_threshold(),
        }
    }
}

impl GitConfig {
    pub fn from_raw(raw: RawGitConfig) -> Self {
        Self {
            sequential_merge_fallback: raw.sequential_merge_fallback,
            sequential_merge_threshold: raw.sequential_merge_threshold,
        }
    }
}

impl OrchestrationConfig {
    pub fn from_raw(raw: RawOrchestrationConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            github_token: raw
                .github_token
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            autonomous_mode: raw.autonomous_mode,
            max_concurrent_workers: raw.max_concurrent_workers,
            consensus_strategy: raw.consensus_strategy,
            manager_timeout_seconds: raw.manager_timeout_seconds,
            worker_timeout_seconds: raw.worker_timeout_seconds,
            enable_plan_review: raw.enable_plan_review,
            plan_review_timeout_secs: raw.plan_review_timeout_secs,
            security_strict_mode: raw.security_strict_mode,
            allowed_licenses: raw.allowed_licenses,
            max_cycle_cost_usd: raw.max_cycle_cost_usd,
            acceptance_commands: raw.acceptance_commands,
            max_parallel_commands: raw.max_parallel_commands,
            per_command_timeout_seconds: raw.per_command_timeout_seconds,
            validator_timeout_seconds: raw.validator_timeout_seconds,
            acceptance_threshold: raw.acceptance_threshold,
            default_manager_count: raw.default_manager_count,
            mode: raw.mode,
            max_batch_retries: raw.max_batch_retries,
            max_total_batches: raw.max_total_batches,
            stall_detection_threshold: raw.stall_detection_threshold,
            min_manager_quorum: raw.min_manager_quorum,
            manager_hard_timeout_seconds: raw.manager_hard_timeout_seconds,
        }
    }
}

// =============================================================================
// Provider-specific config structs
// =============================================================================

/// Usage tier for a model, affecting cost and performance weighting.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelTier {
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "1x")]
    #[default]
    Tier1,
    #[serde(rename = "1.5x")]
    Tier1_5,
    #[serde(rename = "3x")]
    Tier3,
}

/// Configuration for a specific model within a provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tier: ModelTier,
    #[serde(default)]
    pub enabled: bool,
    /// Optimal JSON configuration for this model (e.g. temperature, top_p).
    pub optimal_config: Option<String>,
}

/// Configuration for automatic model selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoSelectionConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Priority weight for performance (0.0 to 1.0)
    #[serde(default = "default_auto_weight")]
    pub performance_weight: f32,
    /// Priority weight for cost (0.0 to 1.0)
    #[serde(default = "default_auto_weight")]
    pub cost_weight: f32,
    /// Priority weight for completion quality (0.0 to 1.0)
    #[serde(default = "default_auto_weight")]
    pub quality_weight: f32,
}

impl Default for AutoSelectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            performance_weight: 0.33,
            cost_weight: 0.33,
            quality_weight: 0.34,
        }
    }
}

fn default_auto_weight() -> f32 {
    0.33
}

/// Gemini-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawGeminiConfig {
    pub api_key: Option<EncryptedString>,
    pub selected_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub auto_select: AutoSelectionConfig,
}

/// Gemini-specific configuration (runtime, decrypted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: Option<String>,
    pub selected_model: Option<String>,
    pub models: Vec<ModelConfig>,
    pub auto_select: AutoSelectionConfig,
}

impl GeminiConfig {
    pub fn merge(mut self, project_cfg: ProjectGeminiConfig) -> Self {
        if let Some(model) = project_cfg.selected_model.filter(|s| !s.is_empty()) {
            self.selected_model = Some(model);
        }
        if let Some(models) = project_cfg.models {
            self.models = models;
        }
        if let Some(auto) = project_cfg.auto_select {
            self.auto_select = auto;
        }
        self
    }

    pub fn from_raw(raw: RawGeminiConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            api_key: raw
                .api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            selected_model: raw.selected_model,
            models: raw.models,
            auto_select: raw.auto_select,
        }
    }
}

/// Claude-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawClaudeConfig {
    pub api_key: Option<EncryptedString>,
    pub selected_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub auto_select: AutoSelectionConfig,
}

/// Claude-specific configuration (runtime, decrypted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub api_key: Option<String>,
    pub selected_model: Option<String>,
    pub models: Vec<ModelConfig>,
    pub auto_select: AutoSelectionConfig,
}

impl ClaudeConfig {
    pub fn merge(mut self, project_cfg: ProjectClaudeConfig) -> Self {
        if let Some(model) = project_cfg.selected_model.filter(|s| !s.is_empty()) {
            self.selected_model = Some(model);
        }
        if let Some(models) = project_cfg.models {
            self.models = models;
        }
        if let Some(auto) = project_cfg.auto_select {
            self.auto_select = auto;
        }
        self
    }

    pub fn from_raw(raw: RawClaudeConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            api_key: raw
                .api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            selected_model: raw.selected_model,
            models: raw.models,
            auto_select: raw.auto_select,
        }
    }
}

/// Ollama-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawOllamaConfig {
    pub api_url: Option<String>,
    pub selected_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub auto_select: AutoSelectionConfig,
}

/// Ollama-specific configuration (runtime).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub api_url: Option<String>,
    pub selected_model: Option<String>,
    pub models: Vec<ModelConfig>,
    pub auto_select: AutoSelectionConfig,
}

impl OllamaConfig {
    pub fn merge(mut self, project_cfg: ProjectOllamaConfig) -> Self {
        if let Some(api_url) = project_cfg.api_url.filter(|s| !s.is_empty()) {
            self.api_url = Some(api_url);
        }
        if let Some(model) = project_cfg.selected_model.filter(|s| !s.is_empty()) {
            self.selected_model = Some(model);
        }
        if let Some(models) = project_cfg.models {
            self.models = models;
        }
        if let Some(auto) = project_cfg.auto_select {
            self.auto_select = auto;
        }
        self
    }

    pub fn from_raw(raw: RawOllamaConfig, _encryption_key: Option<&[u8]>) -> Self {
        Self {
            api_url: raw.api_url,
            selected_model: raw.selected_model,
            models: raw.models,
            auto_select: raw.auto_select,
        }
    }
}

/// AWS Bedrock-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawAwsBedrockConfig {
    pub access_key_id: Option<EncryptedString>,
    pub secret_access_key: Option<EncryptedString>,
    pub region: Option<String>,
    pub profile: Option<String>,
    pub selected_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub auto_select: AutoSelectionConfig,
}

/// AWS Bedrock-specific configuration (runtime, decrypted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AwsBedrockConfig {
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub region: Option<String>,
    pub profile: Option<String>,
    pub selected_model: Option<String>,
    pub models: Vec<ModelConfig>,
    pub auto_select: AutoSelectionConfig,
}

impl AwsBedrockConfig {
    pub fn merge(mut self, project_cfg: ProjectAwsBedrockConfig) -> Self {
        if let Some(region) = project_cfg.region.filter(|s| !s.is_empty()) {
            self.region = Some(region);
        }
        if let Some(profile) = project_cfg.profile.filter(|s| !s.is_empty()) {
            self.profile = Some(profile);
        }
        if let Some(model) = project_cfg.selected_model.filter(|s| !s.is_empty()) {
            self.selected_model = Some(model);
        }
        if let Some(models) = project_cfg.models {
            self.models = models;
        }
        if let Some(auto) = project_cfg.auto_select {
            self.auto_select = auto;
        }
        self
    }

    pub fn from_raw(raw: RawAwsBedrockConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            access_key_id: raw
                .access_key_id
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            secret_access_key: raw
                .secret_access_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            region: raw.region,
            profile: raw.profile,
            selected_model: raw.selected_model,
            models: raw.models,
            auto_select: raw.auto_select,
        }
    }
}

/// Grok-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawGrokConfig {
    pub api_key: Option<EncryptedString>,
    pub selected_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub auto_select: AutoSelectionConfig,
}

/// Grok-specific configuration (runtime, decrypted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrokConfig {
    pub api_key: Option<String>,
    pub selected_model: Option<String>,
    pub models: Vec<ModelConfig>,
    pub auto_select: AutoSelectionConfig,
}

impl GrokConfig {
    pub fn merge(mut self, project_cfg: ProjectGrokConfig) -> Self {
        if let Some(model) = project_cfg.selected_model.filter(|s| !s.is_empty()) {
            self.selected_model = Some(model);
        }
        if let Some(models) = project_cfg.models {
            self.models = models;
        }
        if let Some(auto) = project_cfg.auto_select {
            self.auto_select = auto;
        }
        self
    }

    pub fn from_raw(raw: RawGrokConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            api_key: raw
                .api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            selected_model: raw.selected_model,
            models: raw.models,
            auto_select: raw.auto_select,
        }
    }
}

/// ChatGPT/Codex CLI-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawChatGptConfig {
    pub openai_api_key: Option<EncryptedString>,
    pub codex_api_key: Option<EncryptedString>,
    pub selected_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub auto_select: AutoSelectionConfig,
}

/// ChatGPT/Codex CLI-specific configuration (runtime, decrypted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatGptConfig {
    pub openai_api_key: Option<String>,
    pub codex_api_key: Option<String>,
    pub selected_model: Option<String>,
    pub models: Vec<ModelConfig>,
    pub auto_select: AutoSelectionConfig,
}

impl ChatGptConfig {
    pub fn merge(mut self, project_cfg: ProjectChatGptConfig) -> Self {
        if let Some(model) = project_cfg.selected_model.filter(|s| !s.is_empty()) {
            self.selected_model = Some(model);
        }
        if let Some(models) = project_cfg.models {
            self.models = models;
        }
        if let Some(auto) = project_cfg.auto_select {
            self.auto_select = auto;
        }
        self
    }

    pub fn from_raw(raw: RawChatGptConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            openai_api_key: raw
                .openai_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            codex_api_key: raw
                .codex_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            selected_model: raw.selected_model,
            models: raw.models,
            auto_select: raw.auto_select,
        }
    }
}

/// GitHub Copilot CLI-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawCopilotConfig {
    pub github_token: Option<EncryptedString>,
    pub gh_token: Option<EncryptedString>,
    pub selected_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub auto_select: AutoSelectionConfig,
}

/// GitHub Copilot CLI-specific configuration (runtime, decrypted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CopilotConfig {
    pub github_token: Option<String>,
    pub gh_token: Option<String>,
    pub selected_model: Option<String>,
    pub models: Vec<ModelConfig>,
    pub auto_select: AutoSelectionConfig,
}

impl CopilotConfig {
    pub fn merge(mut self, project_cfg: ProjectCopilotConfig) -> Self {
        if let Some(model) = project_cfg.selected_model.filter(|s| !s.is_empty()) {
            self.selected_model = Some(model);
        }
        if let Some(models) = project_cfg.models {
            self.models = models;
        }
        if let Some(auto) = project_cfg.auto_select {
            self.auto_select = auto;
        }
        self
    }

    pub fn from_raw(raw: RawCopilotConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            github_token: raw
                .github_token
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            gh_token: raw
                .gh_token
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            selected_model: raw.selected_model,
            models: raw.models,
            auto_select: raw.auto_select,
        }
    }
}

/// Aider-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawAiderConfig {
    pub anthropic_api_key: Option<EncryptedString>,
    pub openai_api_key: Option<EncryptedString>,
    pub gemini_api_key: Option<EncryptedString>,
    pub xai_api_key: Option<EncryptedString>,
    pub deepseek_api_key: Option<EncryptedString>,
    pub openrouter_api_key: Option<EncryptedString>,
    pub groq_api_key: Option<EncryptedString>,
    pub mistral_api_key: Option<EncryptedString>,
    pub selected_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub auto_select: AutoSelectionConfig,
}

/// Aider-specific configuration (runtime, decrypted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiderConfig {
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub xai_api_key: Option<String>,
    pub deepseek_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    pub groq_api_key: Option<String>,
    pub mistral_api_key: Option<String>,
    pub selected_model: Option<String>,
    pub models: Vec<ModelConfig>,
    pub auto_select: AutoSelectionConfig,
}

impl AiderConfig {
    pub fn merge(mut self, project_cfg: ProjectAiderConfig) -> Self {
        if let Some(model) = project_cfg.selected_model.filter(|s| !s.is_empty()) {
            self.selected_model = Some(model);
        }
        if let Some(models) = project_cfg.models {
            self.models = models;
        }
        if let Some(auto) = project_cfg.auto_select {
            self.auto_select = auto;
        }
        self
    }

    pub fn from_raw(raw: RawAiderConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            anthropic_api_key: raw
                .anthropic_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            openai_api_key: raw
                .openai_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            gemini_api_key: raw
                .gemini_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            xai_api_key: raw
                .xai_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            deepseek_api_key: raw
                .deepseek_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            openrouter_api_key: raw
                .openrouter_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            groq_api_key: raw
                .groq_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            mistral_api_key: raw
                .mistral_api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            selected_model: raw.selected_model,
            models: raw.models,
            auto_select: raw.auto_select,
        }
    }
}

/// Custom CLI provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawCustomCliConfig {
    pub binary_path: Option<String>,
    pub selected_model: Option<String>,
    pub api_key: Option<EncryptedString>,
    pub args: Option<Vec<String>>,
    pub env_vars: Option<HashMap<String, EncryptedString>>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub auto_select: AutoSelectionConfig,
}

/// Custom CLI provider configuration (runtime, decrypted).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomCliConfig {
    pub binary_path: Option<String>,
    pub selected_model: Option<String>,
    pub api_key: Option<String>,
    pub args: Option<Vec<String>>,
    pub env_vars: Option<HashMap<String, String>>,
    pub models: Vec<ModelConfig>,
    pub auto_select: AutoSelectionConfig,
}

impl CustomCliConfig {
    pub fn merge(mut self, project_cfg: ProjectCustomCliConfig) -> Self {
        if let Some(binary_path) = project_cfg.binary_path.filter(|s| !s.is_empty()) {
            self.binary_path = Some(binary_path);
        }
        if let Some(model) = project_cfg.selected_model.filter(|s| !s.is_empty()) {
            self.selected_model = Some(model);
        }
        if let Some(args) = project_cfg.args {
            self.args = Some(args);
        }
        if let Some(models) = project_cfg.models {
            self.models = models;
        }
        if let Some(auto) = project_cfg.auto_select {
            self.auto_select = auto;
        }
        self
    }

    pub fn from_raw(raw: RawCustomCliConfig, encryption_key: Option<&[u8]>) -> Self {
        Self {
            binary_path: raw.binary_path,
            selected_model: raw.selected_model,
            api_key: raw
                .api_key
                .and_then(|key| encryption_key.and_then(|k| key.decrypt_no_warn(k))),
            args: raw.args,
            env_vars: raw.env_vars.and_then(|env_map| {
                encryption_key.map(|k| {
                    env_map
                        .into_iter()
                        .filter_map(|(name, enc_val)| {
                            enc_val.decrypt_no_warn(k).map(|val| (name, val))
                        })
                        .collect()
                })
            }),
            models: raw.models,
            auto_select: raw.auto_select,
        }
    }
}

// =============================================================================
// Adversarial deliberation configuration
// =============================================================================

/// Configuration for adversarial multi-agent manager deliberation (Phase 27).
///
/// When enabled, the manager phase spawns domain-specific teams of three agents
/// (Proposer, Challenger, Synthesizer) that debate task plans before consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AdversarialDeliberationConfig {
    /// Enable adversarial multi-agent deliberation for the manager phase.
    pub enabled: bool,
    /// Number of deliberation turns per domain team (minimum 2).
    pub deliberation_turns: u32,
    /// Maximum number of domain areas to discover (2–5).
    pub max_domains: usize,
    /// Anthropic model used for Proposer and Challenger roles.
    pub proposer_model: String,
    /// Anthropic model used for the Synthesizer role.
    pub synthesizer_model: String,
    /// Anthropic model used for domain discovery and the cross-team dependency pass.
    pub discovery_model: String,
    /// Whether to run the cross-team dependency pass after all domain teams complete.
    ///
    /// When `true`, a single Haiku-class API call reviews all teams' proposals
    /// for conflicts, seam gaps, and missing dependencies.  Gracefully degrades
    /// on API failure.
    pub cross_team_pass: bool,
}

impl Default for AdversarialDeliberationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            deliberation_turns: 3,
            max_domains: 5,
            proposer_model: "claude-sonnet-4-6".to_string(),
            synthesizer_model: "claude-sonnet-4-6".to_string(),
            discovery_model: "claude-haiku-4-5-20251001".to_string(),
            cross_team_pass: true,
        }
    }
}

// =============================================================================
// Project-specific MahalaxmiConfig overrides
// =============================================================================

/// Project-specific configuration overrides for AI providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMahalaxmiConfig {
    #[serde(default)]
    pub gemini: Option<ProjectGeminiConfig>,
    #[serde(default)]
    pub claude: Option<ProjectClaudeConfig>,
    #[serde(default)]
    pub ollama: Option<ProjectOllamaConfig>,
    #[serde(default)]
    pub aws_bedrock: Option<ProjectAwsBedrockConfig>,
    #[serde(default)]
    pub grok: Option<ProjectGrokConfig>,
    #[serde(default)]
    pub chatgpt: Option<ProjectChatGptConfig>,
    #[serde(default)]
    pub copilot: Option<ProjectCopilotConfig>,
    #[serde(default)]
    pub aider: Option<ProjectAiderConfig>,
    #[serde(default)]
    pub custom_cli: Option<ProjectCustomCliConfig>,
}

/// Project-specific overrides for Gemini provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectGeminiConfig {
    pub selected_model: Option<String>,
    pub models: Option<Vec<ModelConfig>>,
    pub auto_select: Option<AutoSelectionConfig>,
}

/// Project-specific overrides for Claude provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectClaudeConfig {
    pub selected_model: Option<String>,
    pub models: Option<Vec<ModelConfig>>,
    pub auto_select: Option<AutoSelectionConfig>,
}

/// Project-specific overrides for Ollama provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectOllamaConfig {
    pub api_url: Option<String>,
    pub selected_model: Option<String>,
    pub models: Option<Vec<ModelConfig>>,
    pub auto_select: Option<AutoSelectionConfig>,
}

/// Project-specific overrides for AWS Bedrock provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectAwsBedrockConfig {
    pub region: Option<String>,
    pub profile: Option<String>,
    pub selected_model: Option<String>,
    pub models: Option<Vec<ModelConfig>>,
    pub auto_select: Option<AutoSelectionConfig>,
}

/// Project-specific overrides for Grok provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectGrokConfig {
    pub selected_model: Option<String>,
    pub models: Option<Vec<ModelConfig>>,
    pub auto_select: Option<AutoSelectionConfig>,
}

/// Project-specific overrides for ChatGPT/Codex CLI provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectChatGptConfig {
    pub selected_model: Option<String>,
    pub models: Option<Vec<ModelConfig>>,
    pub auto_select: Option<AutoSelectionConfig>,
}

/// Project-specific overrides for GitHub Copilot CLI provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectCopilotConfig {
    pub selected_model: Option<String>,
    pub models: Option<Vec<ModelConfig>>,
    pub auto_select: Option<AutoSelectionConfig>,
}

/// Project-specific overrides for Aider provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectAiderConfig {
    pub selected_model: Option<String>,
    pub models: Option<Vec<ModelConfig>>,
    pub auto_select: Option<AutoSelectionConfig>,
}

/// Project-specific overrides for Custom CLI provider configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectCustomCliConfig {
    pub binary_path: Option<String>,
    pub selected_model: Option<String>,
    pub args: Option<Vec<String>>,
    pub models: Option<Vec<ModelConfig>>,
    pub auto_select: Option<AutoSelectionConfig>,
}

/// Policy for handling uncommitted changes in the working tree before a cycle starts.
///
/// Configured per-project in the Git Config step of the setup wizard and stored
/// in `StartCycleRequest.dirty_branch_policy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DirtyBranchPolicy {
    /// Fail fast: emit `CycleFailed` with the list of dirty files, make no git changes.
    #[default]
    Abort,
    /// Auto-stash before the cycle and auto-pop at cycle end (success or failure).
    Stash,
    /// Proceed with a `warn!` log — for advanced users who know what they are doing.
    Ignore,
}

impl std::str::FromStr for DirtyBranchPolicy {
    type Err = std::convert::Infallible;

    /// Parse from a string value as sent from the frontend (`"abort"`, `"stash"`, `"ignore"`).
    /// Falls back to `Abort` for unrecognised values (never returns `Err`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "stash" => Self::Stash,
            "ignore" => Self::Ignore,
            _ => Self::Abort,
        })
    }
}

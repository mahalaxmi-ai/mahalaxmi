//! Orchestration service — drives the full cycle lifecycle.
//!
//! `OrchestrationService` coordinates the state machine, manager prompt dispatch,
//! consensus evaluation, plan creation, and worker queue management into a
//! single coherent engine. It accepts commands (pause, resume, stop) via an
//! `mpsc` channel and broadcasts events via a `broadcast` channel.

use std::sync::Arc;

use chrono::Utc;
use mahalaxmi_core::domain::{ConsensusAlgorithm, DomainConfig};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{
    ConsensusStrategy, CycleId, DeveloperId, DeveloperRegistry, DeveloperSession, GitMergeStrategy,
    GitPrPlatform, ManagerId, OrchestrationCycleState, ProviderId, WorkerId, WorkerStatus,
};
use mahalaxmi_core::MahalaxmiResult;
use tokio::sync::{broadcast, mpsc, watch};

use crate::agent::AgentRegistry;
use crate::consensus::engine::ConsensusEngine;
use crate::context::{ContextRouterConfig, DefaultContextRouter};
use crate::models::events::OrchestrationEvent;
use crate::models::plan::ExecutionPlan;
use crate::models::proposal::ManagerProposal;
use crate::models::report::{
    AgentPerformanceSummary, CycleCostSummary, CycleOutcome, CycleReport, TaskFailureSummary,
    TaskSummary, VerificationSummary,
};
use crate::models::validation::ValidationVerdict;
use crate::models::{ConsensusConfiguration, ConsensusResult};
use crate::prompt::builder::{ManagerPromptBuilder, ManagerPromptConfig};
use crate::prompt::parser::ManagerOutputParser;
use crate::queue::WorkerQueue;
use crate::state_machine::CycleStateMachine;
use mahalaxmi_core::config::VerificationConfig;

/// Default synthesis manager session timeout in seconds.
const SYNTHESIS_TIMEOUT_SECS: u64 = 120;

/// Error returned when a synthesis manager session fails.
///
/// Returned by [`OrchestrationService::run_synthesis_manager`] when the AI
/// provider times out or returns an error. Propagates without panicking — no
/// `unwrap()` calls exist in the synthesis code paths.
#[derive(Debug)]
pub enum SynthesisManagerError {
    /// The synthesis AI session timed out or the provider returned an error.
    Failed { message: String },
}

impl std::fmt::Display for SynthesisManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SynthesisManagerError::Failed { message } => {
                write!(f, "Synthesis manager session failed: {message}")
            }
        }
    }
}

impl std::error::Error for SynthesisManagerError {}

/// Locate the `claude` CLI binary for synthesis invocation.
///
/// Checks Claude Code's default install path first, then falls back to
/// a PATH lookup via `which`. Returns `None` when no usable binary is found.
fn find_synthesis_claude_binary() -> Option<String> {
    if let Ok(home) = std::env::var("HOME") {
        let default = format!("{home}/.claude/local/claude");
        if std::path::Path::new(&default).exists() {
            return Some(default);
        }
    }
    let output = std::process::Command::new("which")
        .arg("claude")
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8(output.stdout).ok()?;
        let path = path.trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

/// Call the Anthropic REST API with the synthesis prompt and return the text response.
async fn call_synthesis_api(api_key: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 8192,
        "messages": [{ "role": "user", "content": prompt }]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Anthropic API request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Anthropic API returned {status}: {text}"));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Anthropic API response: {e}"))?;

    json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| "Anthropic API response missing content[0].text".to_owned())
        .map(|s| s.to_owned())
}

/// Invoke the `claude` CLI in print mode with the synthesis prompt.
async fn call_synthesis_cli(binary: &str, prompt: &str) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;

    let mut child = tokio::process::Command::new(binary)
        .arg("--print")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn claude CLI ({binary}): {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .await
            .map_err(|e| format!("Failed to write to claude CLI stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Failed to wait for claude CLI: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "claude CLI exited with status {}",
            output.status
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Invoke the AI provider with the synthesis prompt.
///
/// Priority order:
/// 1. `ANTHROPIC_API_KEY` env var → Anthropic REST API.
/// 2. `claude` CLI binary → `claude --print` pipe mode.
/// 3. Neither → immediate error.
async fn run_synthesis_ai_call(prompt: &str) -> Result<String, String> {
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        if !api_key.trim().is_empty() {
            return call_synthesis_api(&api_key, prompt).await;
        }
    }
    if let Some(binary) = find_synthesis_claude_binary() {
        return call_synthesis_cli(&binary, prompt).await;
    }
    Err(
        "No AI provider available for synthesis: set ANTHROPIC_API_KEY or install the claude CLI"
            .to_owned(),
    )
}

/// Signal emitted when all workers in a batch have resolved (success or permanent failure).
///
/// Delivered via the `watch` channel returned by
/// [`OrchestrationService::subscribe_batch_complete`]. The initial channel
/// value is `None`; after each batch completes the value becomes
/// `Some(BatchCompleteSignal)`. Subscribers should call
/// `watch::Receiver::changed()` (async) or `watch::Receiver::borrow()` to
/// observe new signals.
#[derive(Debug, Clone, Default)]
pub struct BatchCompleteSignal {
    /// The batch number that just completed (zero-based, increments per batch).
    pub batch_number: usize,
    /// Number of workers that completed successfully.
    pub success_count: usize,
    /// Number of workers that failed permanently (exhausted fallback chain).
    pub permanent_failure_count: usize,
    /// Combined git diff output from all worker commits in this batch.
    ///
    /// The orchestration service leaves this empty; callers that have
    /// access to worktree diffs (e.g., the driver) should use
    /// [`OrchestrationService::notify_batch_complete`] to emit a richer
    /// signal with the diff content populated.
    pub combined_diff: String,
    /// Exit codes reported by workers in this batch.
    ///
    /// Same caveat as `combined_diff`: the service itself has no access
    /// to process exit codes, so this defaults to an empty `Vec`. Drivers
    /// can populate it via `notify_batch_complete`.
    pub exit_codes: Vec<i32>,
}

impl BatchCompleteSignal {
    /// Returns `true` if all workers succeeded and no failures occurred.
    ///
    /// Specifically, returns `true` when `permanent_failure_count == 0`
    /// **and** every element of `exit_codes` is `0`.
    pub fn all_succeeded(&self) -> bool {
        self.permanent_failure_count == 0 && self.exit_codes.iter().all(|&c| c == 0)
    }
}

/// Commands that can be sent to the orchestration service loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrchestrationCommand {
    /// Pause the cycle (workers keep running but no new ones are dispatched).
    Pause,
    /// Resume a paused cycle.
    Resume,
    /// Stop the cycle immediately.
    Stop,
}

/// Configuration for starting an orchestration cycle.
#[derive(Clone)]
pub struct CycleConfig {
    /// Project root directory.
    pub project_root: String,
    /// AI provider ID (e.g., "claude-code", "openai-foundry").
    pub provider_id: String,
    /// Number of manager agents to spawn.
    pub manager_count: u32,
    /// Number of concurrent worker slots.
    pub worker_count: u32,
    /// Maximum retries for failed workers.
    pub max_retries: u32,
    /// Consensus engine configuration.
    pub consensus_config: ConsensusConfiguration,
    /// Template requirements text.
    pub requirements: String,
    /// Repository map (file tree).
    pub repo_map: String,
    /// Cross-agent shared memory context.
    pub shared_memory: String,
    /// IDs of all providers for multi-provider routing.
    pub provider_ids: Vec<String>,
    /// Routing strategy: "quality_first", "cost_optimized", "speed_first".
    pub routing_strategy: String,
    /// Provider to use for manager agents. If None, uses `provider_id`.
    pub manager_provider_id: Option<String>,
    /// If true, completed worker output is reviewed by a different provider before acceptance.
    pub enable_review_chain: bool,
    /// Provider to use for reviews. If None, auto-select a different provider than the worker's.
    pub review_provider_id: Option<String>,
    /// If true, partial work from failed workers is recorded and can be merged.
    pub accept_partial_progress: bool,
    /// Git merge strategy for worker worktrees.
    pub git_strategy: GitMergeStrategy,
    /// Target branch for merges / PR base. Empty = use current HEAD.
    pub git_target_branch: String,
    /// When BranchAndPr, auto-merge the PR after creation.
    pub git_auto_merge_pr: bool,
    /// PR platform (GitHub or GitLab).
    pub git_pr_platform: GitPrPlatform,
    /// Whether post-cycle requirement validation is enabled.
    pub enable_validation: bool,
    /// Provider to use for validation. If None, uses `provider_id`.
    pub validator_provider_id: Option<String>,
    /// Active domain configuration for this cycle.
    ///
    /// Looked up from `AppState::domain_registry` at cycle start using the key
    /// `"coding"`. When `Some`, the domain's prompt strings are injected into
    /// both `ManagerPromptConfig::domain` and `WorkerPromptConfig::domain`,
    /// overriding the built-in hardcoded strings. When `None` (registry empty
    /// or domain not found), the built-in defaults are used unchanged.
    pub active_domain: Option<Arc<dyn DomainConfig>>,
}

impl std::fmt::Debug for CycleConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CycleConfig")
            .field("project_root", &self.project_root)
            .field("provider_id", &self.provider_id)
            .field("manager_count", &self.manager_count)
            .field("worker_count", &self.worker_count)
            .field("max_retries", &self.max_retries)
            .field("requirements_len", &self.requirements.len())
            .field("active_domain", &self.active_domain.as_ref().map(|d| d.id()))
            .finish_non_exhaustive()
    }
}

/// Describes a multi-developer orchestration cycle.
///
/// Extends the core `CycleConfig` with optional multi-developer workspace
/// sessions. When `developer_sessions` is non-empty, `build_manager_prompts()`
/// generates `sum(developer.max_managers)` prompts — one slot per developer
/// per their configured manager quota — each tagged with the originating
/// `DeveloperId` so that the `WeightedVotingStrategy` can apply per-developer
/// weights during consensus.
///
/// When `developer_sessions` is empty the cycle behaves identically to a
/// single-developer cycle (backward compatible).
#[derive(Clone)]
pub struct StartCycleRequest {
    /// Core cycle configuration shared by all developers in the session.
    pub config: CycleConfig,
    /// Developer sessions contributing to this cycle.
    ///
    /// Each entry maps to one or more manager prompt slots according to the
    /// developer's `max_managers` setting in the `developer_registry`.
    #[allow(dead_code)]
    pub developer_sessions: Vec<DeveloperSession>,
    /// Registry of developer metadata (weight, max_managers) looked up during
    /// prompt generation. Typically built from `MahalaxmiConfig` via
    /// `DeveloperRegistry::from_config`.
    #[allow(dead_code)]
    pub developer_registry: DeveloperRegistry,
}

impl std::fmt::Debug for StartCycleRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StartCycleRequest")
            .field("config", &self.config)
            .field("developer_sessions", &self.developer_sessions)
            .field("developer_registry", &self.developer_registry)
            .finish()
    }
}

/// Handle returned when a cycle is started, for sending commands and receiving events.
pub struct CycleHandle {
    /// Send commands (pause, resume, stop) to the running cycle.
    pub command_tx: mpsc::Sender<OrchestrationCommand>,
    /// Subscribe to orchestration events emitted by the cycle.
    pub event_rx: broadcast::Receiver<OrchestrationEvent>,
    /// Cycle ID for this execution.
    pub cycle_id: CycleId,
}

/// Snapshot of the orchestration service's current state.
#[derive(Debug, Clone)]
pub struct CycleSnapshot {
    /// The cycle ID.
    pub cycle_id: CycleId,
    /// Current state machine state.
    pub state: OrchestrationCycleState,
    /// Whether the cycle is paused.
    pub paused: bool,
    /// Manager proposals collected so far.
    pub proposals_collected: u32,
    /// Total manager count.
    pub managers_total: u32,
    /// Execution plan phase count (0 if not yet created).
    pub plan_phases: u32,
    /// Total workers in the plan (0 if not yet created).
    pub total_workers: u32,
    /// Completed workers.
    pub completed_workers: u32,
    /// Failed workers.
    pub failed_workers: u32,
    /// Active workers.
    pub active_workers: u32,
    /// Consensus strategy configured for this cycle.
    pub consensus_strategy: ConsensusStrategy,
}

/// The orchestration service drives the full cycle lifecycle.
///
/// It is intentionally **synchronous in its core logic** — the async boundary
/// lives in the Tauri command layer. This makes the service testable without
/// spawning actual PTY sessions or AI providers.
///
/// The service operates on "steps" that can be driven externally:
/// 1. `initialize()` — create state machine, consensus engine
/// 2. `build_manager_prompts()` — generate prompts for N managers
/// 3. `submit_manager_output()` — feed raw manager output for parsing
/// 4. `run_consensus()` — evaluate proposals, create plan, create queue
/// 5. `ready_workers()` — get worker IDs ready for dispatch
/// 6. `activate_worker()` / `complete_worker()` / `fail_worker()` — manage worker lifecycle
/// 7. `check_completion()` — check if all workers are done
///
/// State transitions happen automatically at each step boundary.
pub struct OrchestrationService {
    /// Cycle state machine.
    state_machine: CycleStateMachine,
    /// Consensus engine.
    consensus_engine: ConsensusEngine,
    /// Event broadcaster.
    event_tx: broadcast::Sender<OrchestrationEvent>,
    /// I18n service for error messages.
    i18n: I18nService,
    /// Cycle configuration.
    config: CycleConfig,
    /// Collected manager proposals.
    proposals: Vec<ManagerProposal>,
    /// Execution plan (populated after consensus).
    execution_plan: Option<ExecutionPlan>,
    /// Worker queue (populated after plan creation).
    worker_queue: Option<WorkerQueue>,
    /// Whether the cycle is paused.
    paused: bool,
    /// Agent spec registry for specialist personas and accuracy tracking.
    agent_registry: AgentRegistry,
    /// Report from the last completed cycle (for handoff to next cycle).
    last_cycle_report: Option<CycleReport>,
    /// Timestamp when the current cycle was started (for report generation).
    cycle_started_at: Option<chrono::DateTime<Utc>>,
    /// If true, this is a fix cycle that skips the manager/consensus phase
    /// and goes directly to worker execution with pre-generated fix tasks.
    is_fix_cycle: bool,
    /// Pre-generated fix tasks for a fix cycle (populated by `generate_fix_tasks()`).
    fix_tasks: Vec<crate::models::plan::WorkerTask>,
    /// Verification pipeline configuration for worker output validation.
    verification_config: VerificationConfig,
    /// Validation verdict from the previous cycle (injected into next cycle's manager prompt).
    last_validation_verdict: Option<ValidationVerdict>,
    /// Optional codebase index for intelligent context routing.
    ///
    /// When set, `prepare_worker_prompts()` uses `DefaultContextRouter` to score
    /// files by relevance to each task and injects the routed context into the
    /// worker's `context_preamble`. Existing cycles without an index fall back
    /// to the repo-map preamble.
    codebase_index: Option<std::sync::Arc<mahalaxmi_indexing::CodebaseIndex>>,
    /// Configuration for the context router (weights, token budget).
    context_router_config: ContextRouterConfig,
    /// Path to the Living Skill Registry SQLite database (Phase 28).
    ///
    /// When `Some` (and the `living-skills` feature is compiled in), the
    /// `SkillMatcher` pre-loads domain context before `DefaultContextRouter`.
    #[cfg(feature = "living-skills")]
    skill_db_path: Option<std::path::PathBuf>,
    /// Project UUID scoping skill profile lookups (Phase 28).
    #[cfg(feature = "living-skills")]
    skill_project_id: Option<uuid::Uuid>,
    /// Active developer sessions for multi-developer workspace cycles.
    ///
    /// Empty for single-developer cycles (backward compatible). Populated via
    /// `configure_developer_sessions()` before calling `build_manager_prompts()`.
    developer_sessions: Vec<DeveloperSession>,
    /// Registry of developer metadata for the current cycle.
    ///
    /// Used to look up `max_managers` and `weight` when generating per-developer
    /// manager prompts. Empty for single-developer cycles.
    developer_registry: DeveloperRegistry,
    /// Maps each manager ID to the developer that spawned it.
    ///
    /// Built during `build_manager_prompts()` when `developer_sessions` is
    /// non-empty. Used in `submit_manager_output()` to tag the resulting
    /// `ManagerProposal` with the correct `developer_id`.
    manager_developer_ids: std::collections::HashMap<ManagerId, DeveloperId>,
    /// Maps each manager ID to its developer's consensus weight.
    ///
    /// Built alongside `manager_developer_ids`. Used in `submit_manager_output()`
    /// to cache `developer_weight` on the `ManagerProposal` so the
    /// `WeightedVotingStrategy` can score without registry access.
    manager_developer_weights: std::collections::HashMap<ManagerId, f32>,
    /// Watch channel sender for batch-level completion signals.
    ///
    /// Sends `Some(BatchCompleteSignal)` each time all dispatched workers in
    /// the current batch resolve (success or permanent failure). Subscribers
    /// obtain a receiver via [`OrchestrationService::subscribe_batch_complete`].
    batch_complete_tx: watch::Sender<Option<BatchCompleteSignal>>,
    /// Collected worker terminal output strings for the current cycle.
    ///
    /// Populated via [`record_worker_output`] as workers complete. Used by
    /// [`synthesis_context`] to build the synthesis prompt when the domain
    /// algorithm is `ConsensusAlgorithm::Synthesis`, and by
    /// [`apply_output_format`] to format the cycle output file.
    worker_outputs: Vec<String>,
    /// Monotonically increasing batch counter; incremented each time a batch
    /// completes and populates the `batch_number` field of `BatchCompleteSignal`.
    batch_number: usize,
    /// Count of workers dispatched (via `activate_worker`) in the current batch
    /// that have not yet resolved. When it reaches zero the batch is complete.
    batch_active_count: usize,
    /// Textual output from the synthesis manager session.
    ///
    /// Populated by [`run_synthesis_manager`] after a successful AI session.
    /// Exposed via [`synthesis_result`] so that `apply_output_format()` and
    /// the output formatter can write `legal-analysis.md` or equivalent.
    synthesis_result: Option<String>,
}

impl OrchestrationService {
    /// Create a new orchestration service.
    ///
    /// The service starts in the `Idle` state and must be initialized
    /// before any operations can be performed.
    pub fn new(
        config: CycleConfig,
        i18n: I18nService,
        event_tx: broadcast::Sender<OrchestrationEvent>,
        verification_config: VerificationConfig,
    ) -> Self {
        tracing::debug!(
            consensus_strategy = ?config.consensus_config.strategy,
            manager_count = config.manager_count,
            worker_count = config.worker_count,
            "OrchestrationService created"
        );
        let consensus_engine = ConsensusEngine::new(config.consensus_config.clone());
        let (batch_complete_tx, _) = watch::channel::<Option<BatchCompleteSignal>>(None);
        Self {
            state_machine: CycleStateMachine::new(),
            consensus_engine,
            event_tx,
            i18n,
            config,
            proposals: Vec::new(),
            execution_plan: None,
            worker_queue: None,
            paused: false,
            agent_registry: AgentRegistry::new(),
            last_cycle_report: None,
            cycle_started_at: None,
            is_fix_cycle: false,
            fix_tasks: Vec::new(),
            verification_config,
            last_validation_verdict: None,
            codebase_index: None,
            context_router_config: ContextRouterConfig::default(),
            #[cfg(feature = "living-skills")]
            skill_db_path: None,
            #[cfg(feature = "living-skills")]
            skill_project_id: None,
            developer_sessions: Vec::new(),
            developer_registry: DeveloperRegistry::new(),
            manager_developer_ids: std::collections::HashMap::new(),
            manager_developer_weights: std::collections::HashMap::new(),
            batch_complete_tx,
            batch_number: 0,
            batch_active_count: 0,
            worker_outputs: Vec::new(),
            synthesis_result: None,
        }
    }

    /// Attach a codebase index for intelligent context routing.
    ///
    /// When set, `prepare_worker_prompts()` will score and rank files for
    /// each worker task using the three-signal router. Call this before
    /// `prepare_worker_prompts()` to enable routing; omit to use the repo-map
    /// fallback.
    pub fn set_codebase_index(
        &mut self,
        index: Option<std::sync::Arc<mahalaxmi_indexing::CodebaseIndex>>,
    ) {
        self.codebase_index = index;
    }

    /// Configure the context router weights and token budget.
    ///
    /// Must be called before `prepare_worker_prompts()` to take effect.
    pub fn set_context_router_config(&mut self, config: ContextRouterConfig) {
        self.context_router_config = config;
    }

    /// Record a worker's terminal output for post-cycle processing.
    ///
    /// Call this as each worker completes. The collected outputs are used by
    /// [`synthesis_context`] (for `ConsensusAlgorithm::Synthesis`) and by
    /// [`apply_output_format`] (for non-`PullRequest` domain output formats).
    pub fn record_worker_output(&mut self, output: String) {
        self.worker_outputs.push(output);
    }

    /// Build the full synthesis prompt if the active domain uses
    /// `ConsensusAlgorithm::Synthesis`, or return `None` otherwise.
    ///
    /// When `Some`, the returned string is the complete manager prompt that
    /// the driver should use to spawn a synthesis manager session. The
    /// synthesis manager session output replaces the standard cycle output.
    pub fn synthesis_context(&self) -> Option<String> {
        let domain = self.config.active_domain.as_ref()?;
        let synthesis_prompt = match domain.consensus_algorithm() {
            ConsensusAlgorithm::Synthesis { synthesis_prompt } => synthesis_prompt,
            _ => return None,
        };

        let mut prompt = format!("{synthesis_prompt}\n\n");
        for (idx, output) in self.worker_outputs.iter().enumerate() {
            prompt.push_str(&format!(
                "## Worker {} Output\n\n{}\n\n",
                idx + 1,
                output.trim()
            ));
        }
        Some(prompt)
    }

    /// Get the synthesis manager's textual output, if a session has completed.
    ///
    /// Returns `None` when no synthesis session has run yet, or when the active
    /// domain does not use `ConsensusAlgorithm::Synthesis`. The driver and
    /// `apply_output_format()` read this to produce `legal-analysis.md` or
    /// equivalent structured output files.
    pub fn synthesis_result(&self) -> Option<&str> {
        self.synthesis_result.as_deref()
    }

    /// Store the output from a completed synthesis manager session.
    ///
    /// Called automatically by [`run_synthesis_manager`] on success. May also
    /// be called by the driver when the synthesis session was managed
    /// externally (e.g., via a PTY agent session).
    pub fn set_synthesis_result(&mut self, output: String) {
        self.synthesis_result = Some(output);
    }

    /// Spawn a synthesis manager AI session and store the result.
    ///
    /// Should be called by the driver after **all workers have completed**,
    /// when the active domain uses `ConsensusAlgorithm::Synthesis`.
    ///
    /// The method:
    /// 1. Calls `synthesis_context()` — returns `Ok(())` immediately when the
    ///    domain does not use `Synthesis` (no active domain, or a different
    ///    algorithm).
    /// 2. Selects the best available AI backend (`ANTHROPIC_API_KEY` env var
    ///    → direct REST API; `claude` CLI binary → print mode; otherwise error).
    /// 3. Invokes the backend with a [`SYNTHESIS_TIMEOUT_SECS`]-second timeout.
    /// 4. On success, stores the textual output via `set_synthesis_result()` and
    ///    emits a structured `tracing::info!` log with `synthesis_manager_spawned`
    ///    and `synthesis_output_len` fields.
    /// 5. On timeout or provider error, returns
    ///    `Err(SynthesisManagerError::Failed)` — no `unwrap()` anywhere in this
    ///    path.
    pub async fn run_synthesis_manager(&mut self) -> Result<(), SynthesisManagerError> {
        let prompt = match self.synthesis_context() {
            Some(p) => p,
            None => return Ok(()),
        };

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(SYNTHESIS_TIMEOUT_SECS),
            run_synthesis_ai_call(&prompt),
        )
        .await
        .map_err(|_| SynthesisManagerError::Failed {
            message: format!(
                "Synthesis manager session timed out after {SYNTHESIS_TIMEOUT_SECS}s"
            ),
        })?
        .map_err(|e| SynthesisManagerError::Failed { message: e })?;

        let output_len = result.len();
        tracing::info!(
            synthesis_manager_spawned = true,
            synthesis_output_len = output_len,
            "Synthesis manager session completed successfully"
        );

        self.set_synthesis_result(result);
        Ok(())
    }

    /// Write the cycle output file per the domain's `OutputFormat` configuration.
    ///
    /// Returns the relative output file path when a file was written, or `None`
    /// for `OutputFormat::PullRequest` (the existing PR flow owns the output).
    /// The driver should stage the returned path before opening the PR.
    ///
    /// When no domain is active, returns `Ok(None)` — all existing behaviour
    /// is unchanged.
    pub fn apply_output_format(
        &self,
        worktree_path: &std::path::Path,
    ) -> Result<Option<String>, crate::output_format::OutputFormatError> {
        let domain = match &self.config.active_domain {
            Some(d) => d,
            None => return Ok(None),
        };
        crate::output_format::format_cycle_output(
            &domain.output_format(),
            &self.worker_outputs,
            worktree_path,
        )
    }

    /// Load and chunk document input based on the active domain's `InputFormat`.
    ///
    /// Returns `Vec<String>` of text chunks. An empty `Vec` signals the driver
    /// to fall back to the default codebase repository-map scan.
    ///
    /// When no domain is active or the format is `Codebase`, returns an empty
    /// `Vec` — all existing behaviour is unchanged.
    pub fn load_input_document(
        &self,
        file_path: Option<&str>,
        text_input: Option<&str>,
    ) -> Result<Vec<String>, crate::input_adapter::InputAdapterError> {
        let domain = match &self.config.active_domain {
            Some(d) => d,
            None => return Ok(vec![]),
        };
        crate::input_adapter::load_document(&domain.input_format(), file_path, text_input)
    }

    /// Configure the Living Skill Registry for Phase 28 skill pre-loading.
    ///
    /// When called with a valid `db_path` and `project_id`, each call to
    /// `prepare_worker_prompts()` will attempt to match each task's domain
    /// using the `SkillMatcher` and prepend a token-budgeted context block
    /// before `DefaultContextRouter` runs. Requires the `living-skills` feature.
    #[cfg(feature = "living-skills")]
    pub fn set_skill_registry(
        &mut self,
        db_path: std::path::PathBuf,
        project_id: uuid::Uuid,
    ) {
        self.skill_db_path = Some(db_path);
        self.skill_project_id = Some(project_id);
    }

    /// Configure multi-developer workspace sessions for this cycle.
    ///
    /// When set before `build_manager_prompts()`, the service generates
    /// `sum(developer.max_managers)` prompt configs — one per developer-slot —
    /// each tagged with the originating `DeveloperId`. The `DeveloperRegistry`
    /// is used to look up `max_managers` and `weight` for each session.
    ///
    /// Calling this with empty `sessions` reverts to single-developer mode.
    pub fn configure_developer_sessions(
        &mut self,
        sessions: Vec<DeveloperSession>,
        registry: DeveloperRegistry,
    ) {
        self.developer_sessions = sessions;
        self.developer_registry = registry;
        self.manager_developer_ids.clear();
        self.manager_developer_weights.clear();
    }

    /// Get the cycle ID.
    pub fn cycle_id(&self) -> CycleId {
        self.state_machine.cycle_id()
    }

    /// Get the current state.
    pub fn current_state(&self) -> OrchestrationCycleState {
        self.state_machine.current_state()
    }

    /// Set the validation verdict from a previous cycle for injection
    /// into the next cycle's manager prompt.
    pub fn set_last_validation_verdict(&mut self, verdict: Option<ValidationVerdict>) {
        self.last_validation_verdict = verdict;
    }

    /// Set the cycle report from a previous cycle for injection into the next
    /// cycle's manager prompt, enabling cross-cycle deduplication.
    pub fn set_last_cycle_report(&mut self, report: Option<CycleReport>) {
        self.last_cycle_report = report;
    }

    /// Check if the cycle is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Check if the cycle is finished (stopped or completed).
    pub fn is_finished(&self) -> bool {
        self.state_machine.is_finished()
    }

    /// Get a snapshot of the current state.
    pub fn snapshot(&self) -> CycleSnapshot {
        let (total_workers, completed, failed, active) = self
            .worker_queue
            .as_ref()
            .map(|q| {
                let stats = q.statistics();
                (stats.total, stats.completed, stats.failed, stats.active)
            })
            .unwrap_or((0, 0, 0, 0));

        CycleSnapshot {
            cycle_id: self.state_machine.cycle_id(),
            state: self.state_machine.current_state(),
            paused: self.paused,
            proposals_collected: self.proposals.len() as u32,
            managers_total: self.config.manager_count,
            plan_phases: self
                .execution_plan
                .as_ref()
                .map(|p| p.phase_count() as u32)
                .unwrap_or(0),
            total_workers,
            completed_workers: completed,
            failed_workers: failed,
            active_workers: active,
            consensus_strategy: self.config.consensus_config.strategy,
        }
    }

    // =========================================================================
    // Phase 1: Initialize
    // =========================================================================

    /// Initialize the cycle — transitions from Idle to InitializingManager.
    ///
    /// Emits `CycleStarted` and `StateChanged` events.
    pub fn initialize(&mut self) -> MahalaxmiResult<()> {
        let event = self
            .state_machine
            .transition_to(OrchestrationCycleState::InitializingManager, &self.i18n)?;
        tracing::info!(
            cycle_id = %self.state_machine.cycle_id(),
            "Cycle initialized, transitioning to InitializingManager"
        );
        self.cycle_started_at = Some(Utc::now());
        self.emit(OrchestrationEvent::CycleStarted {
            cycle_id: self.state_machine.cycle_id(),
            timestamp: Utc::now(),
        });
        self.emit(event);
        Ok(())
    }

    // =========================================================================
    // Phase 2: Manager Processing
    // =========================================================================

    /// Build prompts for all managers.
    ///
    /// Transitions to ManagerProcessing and returns a list of (ManagerId, prompt) pairs
    /// that should be sent to AI manager terminals.
    ///
    /// When `configure_developer_sessions()` has been called with a non-empty session
    /// list, this method generates `sum(developer.max_managers)` prompts — one per
    /// developer manager slot — and records the manager→developer mapping internally.
    /// The requirements text used for each prompt is the session's `requirements`
    /// field when present, falling back to the cycle config requirements.
    ///
    /// When no developer sessions are configured (the default), uses the existing
    /// single-developer logic unchanged (backward compatible).
    pub fn build_manager_prompts(&mut self) -> MahalaxmiResult<Vec<(ManagerId, String)>> {
        let event = self
            .state_machine
            .transition_to(OrchestrationCycleState::ManagerProcessing, &self.i18n)?;
        self.emit(event);

        if self.developer_sessions.is_empty() {
            // Single-developer path: unchanged original logic.
            let mut prompts = Vec::with_capacity(self.config.manager_count as usize);
            for i in 0..self.config.manager_count {
                let manager_id = ManagerId::new(format!("manager-{}", i));
                let prompt_config = ManagerPromptConfig {
                    requirements: self.config.requirements.clone(),
                    repo_map: self.config.repo_map.clone(),
                    shared_memory: self.config.shared_memory.clone(),
                    provider_id: self
                        .config
                        .manager_provider_id
                        .as_deref()
                        .unwrap_or(&self.config.provider_id)
                        .to_string(),
                    worker_count: self.config.worker_count,
                    previous_cycle_report: self
                        .last_cycle_report
                        .as_ref()
                        .map(|r| r.to_prompt_summary()),
                    previous_validation_verdict: self
                        .last_validation_verdict
                        .as_ref()
                        .map(|v| v.to_prompt_summary()),
                    domain: self.config.active_domain.clone(),
                    ..Default::default()
                };
                let prompt = ManagerPromptBuilder::build(&prompt_config);
                prompts.push((manager_id, prompt));
            }
            return Ok(prompts);
        }

        // Multi-developer path: generate prompts per developer session.
        let sessions = self.developer_sessions.clone();
        let mut prompts = Vec::new();
        let mut global_index: u32 = 0;

        for session in &sessions {
            let developer = self.developer_registry.get(&session.developer_id);

            let max_managers = developer.map(|d| d.max_managers as u32).unwrap_or(1);
            let weight = developer.map(|d| d.weight).unwrap_or(1.0);

            if developer.is_none() {
                tracing::warn!(
                    developer_id = %session.developer_id,
                    "Developer not found in registry; using max_managers=1, weight=1.0"
                );
            }

            // Use session-specific requirements when provided (non-empty),
            // otherwise fall back to the cycle's shared requirements.
            let requirements = if session.requirements.trim().is_empty() {
                self.config.requirements.clone()
            } else {
                session.requirements.clone()
            };

            for slot in 0..max_managers {
                let manager_id =
                    ManagerId::new(format!("{}-manager-{}", session.developer_id, slot));

                // Record the manager→developer mapping for tagging proposals.
                self.manager_developer_ids
                    .insert(manager_id.clone(), session.developer_id.clone());
                self.manager_developer_weights
                    .insert(manager_id.clone(), weight);

                let prompt_config = ManagerPromptConfig {
                    requirements: requirements.clone(),
                    repo_map: self.config.repo_map.clone(),
                    shared_memory: self.config.shared_memory.clone(),
                    provider_id: self
                        .config
                        .manager_provider_id
                        .as_deref()
                        .unwrap_or(&self.config.provider_id)
                        .to_string(),
                    worker_count: self.config.worker_count,
                    previous_cycle_report: self
                        .last_cycle_report
                        .as_ref()
                        .map(|r| r.to_prompt_summary()),
                    previous_validation_verdict: self
                        .last_validation_verdict
                        .as_ref()
                        .map(|v| v.to_prompt_summary()),
                    domain: self.config.active_domain.clone(),
                    ..Default::default()
                };
                let prompt = ManagerPromptBuilder::build(&prompt_config);
                prompts.push((manager_id, prompt));
                global_index += 1;
            }
        }

        tracing::info!(
            developer_count = sessions.len(),
            manager_count = global_index,
            "Built multi-developer manager prompts"
        );

        Ok(prompts)
    }

    /// Submit raw terminal output from a manager for parsing.
    ///
    /// The output is parsed to extract a `ManagerProposal`. If parsing fails,
    /// a failed proposal is recorded. Emits `ManagerCompleted` event.
    ///
    /// When multi-developer sessions are active, the resulting proposal is
    /// tagged with the `developer_id` and `developer_weight` looked up from the
    /// internal manager→developer map built during `build_manager_prompts()`.
    ///
    /// Returns `true` if all manager proposals have been collected.
    pub fn submit_manager_output(
        &mut self,
        manager_id: ManagerId,
        output: &str,
        duration_ms: u64,
    ) -> MahalaxmiResult<bool> {
        let mut proposal =
            match ManagerOutputParser::parse(output, manager_id.clone(), duration_ms, &self.i18n) {
                Ok(p) => {
                    self.emit(OrchestrationEvent::ManagerCompleted {
                        cycle_id: self.state_machine.cycle_id(),
                        manager_id: manager_id.clone(),
                        task_count: p.tasks.len() as u32,
                        duration_ms,
                        timestamp: Utc::now(),
                    });
                    p
                }
                Err(e) => {
                    // Show the LAST 500 chars — the JSON proposal is always at
                    // the end of Claude Code output, after tool call logs and
                    // prose. Showing the tail is far more useful for debugging
                    // than showing the head (which is just startup messages).
                    let tail_preview = if output.len() > 500 {
                        format!("…{}", &output[output.len() - 500..])
                    } else {
                        output.to_string()
                    };
                    tracing::error!(
                        manager_id = %manager_id,
                        output_len = output.len(),
                        error = %e,
                        output_tail = %tail_preview,
                        "Failed to parse manager output — no tasks extracted"
                    );
                    let failed = ManagerProposal::failed(
                        manager_id.clone(),
                        format!("Failed to parse manager output: {e}"),
                        duration_ms,
                    );
                    self.emit(OrchestrationEvent::ManagerCompleted {
                        cycle_id: self.state_machine.cycle_id(),
                        manager_id: manager_id.clone(),
                        task_count: 0,
                        duration_ms,
                        timestamp: Utc::now(),
                    });
                    failed
                }
            };

        // Tag with developer identity when multi-developer sessions are active.
        if let Some(dev_id) = self.manager_developer_ids.get(&manager_id).cloned() {
            let weight = self
                .manager_developer_weights
                .get(&manager_id)
                .copied()
                .unwrap_or(1.0);
            proposal.developer_id = Some(dev_id);
            proposal.developer_weight = weight;
        }

        self.proposals.push(proposal);

        // In multi-developer mode the effective manager count is the number of
        // manager slots created (stored in manager_developer_ids). Fall back to
        // the config manager_count for single-developer cycles.
        let expected_count = if self.manager_developer_ids.is_empty() {
            self.config.manager_count
        } else {
            self.manager_developer_ids.len() as u32
        };
        Ok(self.proposals.len() as u32 >= expected_count)
    }

    /// Get the number of proposals collected so far.
    pub fn proposals_collected(&self) -> u32 {
        self.proposals.len() as u32
    }

    // =========================================================================
    // Phase 3: Consensus & Planning
    // =========================================================================

    /// Run consensus on collected proposals and create the execution plan.
    ///
    /// Transitions through: AnalyzingManagerResults → DiscoveringWorkerRequirements
    ///
    /// Emits `ConsensusReached` and `PlanCreated` events.
    /// Returns the consensus result for inspection.
    pub fn run_consensus(&mut self) -> MahalaxmiResult<ConsensusResult> {
        // Transition to AnalyzingManagerResults
        let event = self
            .state_machine
            .transition_to(OrchestrationCycleState::AnalyzingManagerResults, &self.i18n)?;
        self.emit(event);

        // Determine domain-level consensus algorithm (opt-in; None = existing behaviour).
        let domain_algorithm = self
            .config
            .active_domain
            .as_ref()
            .map(|d| d.consensus_algorithm());

        // When the domain uses Synthesis, workers still execute normally here.
        // After all workers complete the driver must call
        // `run_synthesis_manager()` which invokes the AI provider with the
        // combined worker-output prompt and stores the result for the output
        // formatter to write (e.g. `legal-analysis.md`).
        if let Some(ConsensusAlgorithm::Synthesis {
            ref synthesis_prompt,
        }) = domain_algorithm
        {
            tracing::info!(
                synthesis_prompt_len = synthesis_prompt.len(),
                synthesis_timeout_secs = SYNTHESIS_TIMEOUT_SECS,
                "Synthesis domain configured; driver must call run_synthesis_manager() \
                 after all workers complete"
            );
        }

        // Filter to successful proposals only
        let successful: Vec<&ManagerProposal> =
            self.proposals.iter().filter(|p| p.completed).collect();

        let result = if successful.is_empty() {
            ConsensusResult::no_consensus(self.config.consensus_config.strategy)
        } else {
            let proposals_vec: Vec<ManagerProposal> =
                successful.iter().copied().cloned().collect();
            match &domain_algorithm {
                Some(algo @ ConsensusAlgorithm::Majority { .. }) => {
                    self.consensus_engine
                        .evaluate_with_domain_algorithm(&proposals_vec, &self.i18n, Some(algo))?
                }
                _ => self.consensus_engine.evaluate(&proposals_vec, &self.i18n)?,
            }
        };

        self.emit(OrchestrationEvent::ConsensusReached {
            cycle_id: self.state_machine.cycle_id(),
            agreed_count: result.agreed_tasks.len() as u32,
            dissenting_count: result.dissenting_tasks.len() as u32,
            timestamp: Utc::now(),
        });

        // Transition to DiscoveringWorkerRequirements
        let event = self.state_machine.transition_to(
            OrchestrationCycleState::DiscoveringWorkerRequirements,
            &self.i18n,
        )?;
        self.emit(event);

        // Build execution plan from consensus result
        let mut plan = ExecutionPlan::from_consensus_result(&result, &self.i18n)?;

        // Enforce worktree scope: strip tasks whose affected_files live entirely
        // in the platform product tree (services/, db/migrations/, etc.).  Those
        // paths belong to a separate repository, need platform credentials, and
        // must never be assigned to workers operating in this worktree.
        let filtered = plan.filter_platform_scoped_tasks();
        if filtered > 0 {
            tracing::warn!(
                filtered_tasks = filtered,
                "Routing constraint removed {} task(s) with platform-only affected_files \
                 (services/, db/migrations/, database/migrations/). \
                 These tasks must be executed by the platform session, not a Mahalaxmi worker.",
                filtered
            );
        }

        // Populate contributing_developers: collect unique DeveloperIds from
        // proposals whose manager appeared in at least one agreed task's
        // `proposed_by` list.
        if !self.manager_developer_ids.is_empty() {
            let agreed_manager_ids: std::collections::HashSet<&ManagerId> = result
                .agreed_tasks
                .iter()
                .flat_map(|t| &t.proposed_by)
                .collect();

            let mut contributors: std::collections::HashSet<DeveloperId> =
                std::collections::HashSet::new();
            for p in &self.proposals {
                if p.completed && agreed_manager_ids.contains(&p.manager_id) {
                    if let Some(ref dev_id) = p.developer_id {
                        contributors.insert(dev_id.clone());
                    }
                }
            }
            plan.contributing_developers = contributors.into_iter().collect();
            plan.contributing_developers.sort_by(|a, b| {
                let a_str: &str = a.as_ref();
                let b_str: &str = b.as_ref();
                a_str.cmp(b_str)
            });
        }

        self.emit(OrchestrationEvent::PlanCreated {
            cycle_id: self.state_machine.cycle_id(),
            phase_count: plan.phase_count() as u32,
            worker_count: plan.all_workers().len() as u32,
            timestamp: Utc::now(),
        });

        // Create worker queue from plan
        let queue = WorkerQueue::from_plan_with_verification(
            &plan,
            self.config.worker_count,
            self.config.max_retries,
            self.verification_config.clone(),
        );

        self.execution_plan = Some(plan);
        self.worker_queue = Some(queue);

        Ok(result)
    }

    // =========================================================================
    // Phase 3b-pre: Agent Spec Resolution
    // =========================================================================

    /// Resolve agent specs from manager proposals and assign them to workers.
    ///
    /// Iterates proposals looking for `ProposedAgentSpec` decisions. For "create"
    /// actions, registers the new agent in the registry. For "reuse" actions, looks
    /// up the existing agent. Matched agents are stored on the corresponding
    /// `QueuedWorker` for later prompt injection.
    ///
    /// Must be called after `run_consensus()` which populates the execution plan
    /// and worker queue. Has no effect if no proposals contain agent specs.
    pub fn process_agent_specs(&mut self) {
        use crate::agent::AgentSpec;

        let plan = match &self.execution_plan {
            Some(p) => p.clone(),
            None => return,
        };

        // Build map: normalized_key → ProposedAgentSpec from all proposals
        let mut key_to_spec: std::collections::HashMap<
            String,
            crate::models::proposal::ProposedAgentSpec,
        > = std::collections::HashMap::new();

        for proposal in &self.proposals {
            if !proposal.completed {
                continue;
            }
            for task in &proposal.tasks {
                if let Some(ref spec) = task.agent_spec {
                    let key = task.normalized_key();
                    key_to_spec.entry(key).or_insert_with(|| spec.clone());
                }
            }
        }

        if key_to_spec.is_empty() {
            return;
        }

        // Match plan tasks to agent specs by normalized title
        for phase in &plan.phases {
            for task in &phase.tasks {
                let normalized = normalize_title(&task.title);
                if let Some(proposed_spec) = key_to_spec.get(&normalized) {
                    let resolved = match proposed_spec.action.as_str() {
                        "create" => {
                            let spec = AgentSpec {
                                id: proposed_spec.id.clone(),
                                name: proposed_spec
                                    .name
                                    .clone()
                                    .unwrap_or_else(|| proposed_spec.id.clone()),
                                system_preamble: proposed_spec
                                    .system_preamble
                                    .clone()
                                    .unwrap_or_default(),
                                preferred_provider: None,
                                domain_tags: proposed_spec.domain_tags.clone(),
                            };
                            self.agent_registry.register(spec.clone());
                            Some(spec)
                        }
                        "reuse" => self.agent_registry.get(&proposed_spec.id).cloned(),
                        _ => None,
                    };

                    if let Some(spec) = resolved {
                        if let Some(queue) = &mut self.worker_queue {
                            if let Some(worker) = queue.get_worker_mut(&task.worker_id) {
                                worker.agent_spec = Some(spec);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get a reference to the agent registry.
    pub fn agent_registry(&self) -> &AgentRegistry {
        &self.agent_registry
    }

    /// Get a mutable reference to the agent registry.
    pub fn agent_registry_mut(&mut self) -> &mut AgentRegistry {
        &mut self.agent_registry
    }

    // =========================================================================
    // Phase 3b: Multi-Provider Task Routing
    // =========================================================================

    /// Route workers to providers based on task classification and capabilities.
    ///
    /// **Fast path**: If `config.provider_ids` is empty or has only one entry,
    /// returns immediately — zero overhead for single-provider mode.
    ///
    /// For multi-provider mode:
    /// 1. Classifies each task's type via `classifier::classify_task()`
    /// 2. Applies security routing constraints
    /// 3. Filters unhealthy providers via the health tracker
    /// 4. Routes to the best provider via `TaskRouter::route_with_fallbacks()`
    /// 5. Enforces review chain: CodeReview tasks exclude the provider that
    ///    generated the code being reviewed
    pub fn route_workers(
        &mut self,
        registry: &mahalaxmi_providers::ProviderRegistry,
        health_tracker: &mahalaxmi_providers::ProviderHealthTracker,
    ) -> MahalaxmiResult<()> {
        use mahalaxmi_core::types::ProviderId;
        use mahalaxmi_providers::{
            security as sec, RoutingConstraints, RoutingStrategy, TaskRouter,
        };

        // Fast path: skip if single-provider mode
        if self.config.provider_ids.len() <= 1 {
            return Ok(());
        }

        let plan = match &self.execution_plan {
            Some(p) => p.clone(),
            None => return Ok(()),
        };
        let queue = match &mut self.worker_queue {
            Some(q) => q,
            None => return Ok(()),
        };

        let strategy = match self.config.routing_strategy.as_str() {
            "cost_optimized" => RoutingStrategy::CostOptimized,
            "speed_first" => RoutingStrategy::SpeedFirst,
            _ => RoutingStrategy::QualityFirst,
        };

        // Identify local providers from capabilities
        let local_ids: Vec<ProviderId> = self
            .config
            .provider_ids
            .iter()
            .filter_map(|id| {
                let pid = ProviderId::new(id);
                registry.get(&pid).and_then(|p| {
                    if sec::is_local_by_capabilities(p.capabilities())
                        || sec::is_local_provider(&pid)
                    {
                        Some(pid)
                    } else {
                        None
                    }
                })
            })
            .collect();

        let security_mode = sec::SecurityRoutingMode::Automatic;

        for phase in &plan.phases {
            for task in &phase.tasks {
                // 1. Classify task type
                let task_type = crate::classifier::classify_task(
                    &task.title,
                    &task.description,
                    None, // ProposedTask.task_type is consumed during plan creation
                );

                // 2. Security classification and constraints
                let sec_class =
                    sec::classify_task(&task.title, &task.description, &task.affected_files);
                let constraints = RoutingConstraints {
                    complexity: Some(task.complexity.min(255) as u8),
                    strategy,
                    ..RoutingConstraints::default()
                };

                let sec_result =
                    sec::apply_security_routing(sec_class, security_mode, &constraints, &local_ids);

                if sec_result.blocked {
                    continue; // Skip blocked tasks
                }

                let mut final_constraints = sec_result.constraints;

                // 3. Filter unhealthy providers
                for id_str in &self.config.provider_ids {
                    let pid = ProviderId::new(id_str);
                    if !health_tracker.is_available(&pid) {
                        final_constraints.excluded_providers.push(pid);
                    }
                }

                // 4. Review chain: if this is a CodeReview, exclude the provider
                //    that produced the dependency tasks (immutable borrow first)
                if task_type == mahalaxmi_providers::TaskType::CodeReview {
                    let dep_providers: Vec<ProviderId> = task
                        .dependencies
                        .iter()
                        .filter_map(|dep_id| {
                            queue
                                .find_worker_by_task(dep_id)
                                .and_then(|w| w.provider_id.clone())
                        })
                        .collect();
                    for dep_provider in dep_providers {
                        if !final_constraints.excluded_providers.contains(&dep_provider) {
                            final_constraints.excluded_providers.push(dep_provider);
                        }
                    }
                }

                // 5. Route with fallbacks
                let decisions = TaskRouter::route_with_fallbacks(
                    registry,
                    task_type,
                    &final_constraints,
                    2, // max fallbacks
                );

                // 6. Assign provider to worker (mutable borrow)
                if let Some(worker) = queue.get_worker_mut(&task.worker_id) {
                    if let Some(primary) = decisions.first() {
                        worker.provider_id = Some(primary.provider_id.clone());
                        worker.fallback_provider_ids = decisions
                            .iter()
                            .skip(1)
                            .map(|d| d.provider_id.clone())
                            .collect();
                    }
                }
            }
        }

        Ok(())
    }

    // =========================================================================
    // Phase 3c: Worker Prompt Preparation
    // =========================================================================

    /// Assemble per-task prompts for all workers in the execution plan.
    ///
    /// For each task, builds a `WorkerPromptConfig` with the task details,
    /// template requirements, context preamble, and provider-specific formatting.
    /// Stores the assembled prompt on the corresponding `QueuedWorker`.
    ///
    /// Must be called after `run_consensus()` (and optionally `route_workers()`).
    /// Returns the number of prompts prepared.
    pub fn prepare_worker_prompts(&mut self) -> MahalaxmiResult<u32> {
        use crate::prompt::worker_builder::{WorkerPromptBuilder, WorkerPromptConfig};

        let plan = match &self.execution_plan {
            Some(p) => p.clone(),
            None => return Ok(0),
        };
        let queue = match &mut self.worker_queue {
            Some(q) => q,
            None => return Ok(0),
        };

        let mut prompt_count = 0u32;

        for phase in &plan.phases {
            // Collect titles of all tasks in this phase for parallel work notes
            let phase_task_titles: Vec<&str> =
                phase.tasks.iter().map(|t| t.title.as_str()).collect();

            for task in &phase.tasks {
                // Determine provider for this worker (from routing or default)
                let provider_id = queue
                    .get_worker(&task.worker_id)
                    .and_then(|w| w.provider_id.as_ref())
                    .map(|p| p.as_str().to_string())
                    .unwrap_or_else(|| self.config.provider_id.clone());

                // Build parallel work notes (other tasks in same phase)
                let parallel_notes: Vec<String> = phase_task_titles
                    .iter()
                    .filter(|t| **t != task.title)
                    .map(|t| format!("Another worker is simultaneously handling: {t}"))
                    .collect();

                // Build context preamble: prefer routed context when index available.
                let context_preamble = if let Some(ref index) = self.codebase_index {
                    use crate::context::ContextRouter;
                    let router = DefaultContextRouter;
                    let scored = router.route(
                        task,
                        index,
                        self.last_cycle_report.as_ref(),
                        &self.context_router_config,
                    );
                    tracing::info!(
                        worker_id = %task.worker_id,
                        file_count = scored.files.len(),
                        estimated_tokens = scored.estimated_tokens,
                        "Context router selected files for worker"
                    );
                    if scored.files.is_empty() {
                        // Fall back to existing preamble when router returns nothing.
                        queue
                            .worker_context(&task.worker_id)
                            .unwrap_or("")
                            .to_string()
                    } else {
                        let mut preamble = String::new();
                        for fs in &scored.files {
                            preamble.push_str(&format!(
                                "```{}  (score: {:.2})\n",
                                fs.path.display(),
                                fs.score
                            ));
                            if let Ok(raw) = std::fs::read_to_string(&fs.path) {
                                let excerpt: String =
                                    raw.lines().take(200).collect::<Vec<_>>().join("\n");
                                preamble.push_str(&excerpt);
                                preamble.push('\n');
                            }
                            preamble.push_str("```\n\n");
                        }
                        preamble
                    }
                } else {
                    queue
                        .worker_context(&task.worker_id)
                        .unwrap_or("")
                        .to_string()
                };

                // Phase 28: prepend skill context when living-skills feature is active
                // and a registry is configured. Runs BEFORE DefaultContextRouter so the
                // snapshot occupies the 30% skill budget and leaves 50% for reactive files.
                #[cfg(feature = "living-skills")]
                let context_preamble = {
                    let mut preamble = context_preamble;
                    if let (Some(db_path), Some(project_id)) =
                        (&self.skill_db_path, &self.skill_project_id)
                    {
                        use mahalaxmi_skills::matching::matcher::SkillMatcher;
                        use mahalaxmi_skills::matching::injector::SkillContextInjector;
                        use mahalaxmi_skills::profile::compressor::ContextCompressor;
                        use mahalaxmi_skills::registry::store::SkillRegistry;
                        use std::sync::Arc;

                        // Default provider context window for budget calculation (200K for Claude).
                        const DEFAULT_CONTEXT_WINDOW: u32 = 200_000;
                        match SkillRegistry::new(db_path) {
                            Ok(registry) => {
                                let matcher = SkillMatcher::new(
                                    Arc::new(registry),
                                    *project_id,
                                );
                                let hint_files: Vec<std::path::PathBuf> = task
                                    .affected_files
                                    .iter()
                                    .map(std::path::PathBuf::from)
                                    .collect();
                                match matcher.match_task_sync(&task.description, &hint_files) {
                                    Ok(Some(m)) => {
                                        tracing::info!(
                                            task_id = %task.task_id,
                                            domain = %m.matched_domain.name,
                                            confidence = m.confidence,
                                            token_count = m.profile.token_count,
                                            "Skill context pre-loaded for worker"
                                        );
                                        let injector = SkillContextInjector::new(
                                            ContextCompressor::new(DEFAULT_CONTEXT_WINDOW),
                                        );
                                        let skill_block = injector.inject(&m.profile);
                                        preamble = format!("{skill_block}\n\n{preamble}");
                                    }
                                    Ok(None) => {
                                        tracing::debug!(
                                            task_id = %task.task_id,
                                            "No skill profile matched — using default context"
                                        );
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            task_id = %task.task_id,
                                            error = %e,
                                            "SkillMatcher error — skipping skill injection"
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    "Failed to open SkillRegistry — skipping skill injection"
                                );
                            }
                        }
                    }
                    preamble
                };

                // Get retry context if this is a retry
                let retry_context = queue
                    .get_worker(&task.worker_id)
                    .and_then(|w| w.retry_context.clone());

                // Classify task type
                let task_type = task.task_type.map(|tt| tt.to_string()).unwrap_or_else(|| {
                    crate::classifier::classify_task(&task.title, &task.description, None)
                        .to_string()
                });

                // Get agent spec if assigned
                let worker_agent = queue
                    .get_worker(&task.worker_id)
                    .and_then(|w| w.agent_spec.clone());

                let (agent_success_rate, agent_total_tasks) = worker_agent
                    .as_ref()
                    .and_then(|spec| {
                        self.agent_registry
                            .get_record(&spec.id)
                            .map(|r| (Some(r.success_rate()), Some(r.total_tasks)))
                    })
                    .unwrap_or((None, None));

                let context_router: Option<Arc<dyn crate::error::ContextRouter>> =
                    Some(Arc::new(crate::error::DefaultContextRouter));
                let router_config = crate::error::ContextRouterConfig::default();

                let config = WorkerPromptConfig {
                    task_id: task.task_id.to_string(),
                    task_title: task.title.clone(),
                    task_description: task.description.clone(),
                    task_type,
                    complexity: task.complexity,
                    affected_files: task.affected_files.clone(),
                    requirements: self.config.requirements.clone(),
                    context_preamble,
                    provider_id,
                    retry_context,
                    parallel_work_notes: parallel_notes,
                    completion_criteria: task.acceptance_criteria.clone(),
                    agent_spec: worker_agent,
                    agent_success_rate,
                    agent_total_tasks,
                    git_strategy: self.config.git_strategy,
                    context_router,
                    context_router_config: Some(router_config),
                    codebase_index: None,    // Phase 5 sets this
                    last_cycle_report: None, // Phase 5 sets this
                    domain: self.config.active_domain.clone(),
                };

                let prompt = WorkerPromptBuilder::build(&config);
                queue.set_worker_prompt(task.worker_id, prompt, &self.i18n)?;
                prompt_count += 1;
            }
        }

        Ok(prompt_count)
    }

    /// Get the assembled prompt for a worker.
    ///
    /// Returns the full prompt from `WorkerPromptBuilder` if available,
    /// falling back to the raw task description for backward compatibility.
    pub fn worker_prompt(&self, worker_id: &WorkerId) -> Option<String> {
        self.worker_queue
            .as_ref()
            .and_then(|q| q.worker_prompt(worker_id))
            .map(String::from)
            .or_else(|| self.worker_task_description(worker_id))
    }

    /// Get the original requirements text for this cycle.
    pub fn requirements(&self) -> &str {
        &self.config.requirements
    }

    /// Collect all acceptance criteria from all worker tasks in the execution plan.
    pub fn all_acceptance_criteria(&self) -> Vec<String> {
        let Some(plan) = self.execution_plan.as_ref() else {
            return Vec::new();
        };
        let mut criteria: Vec<String> = plan
            .all_workers()
            .iter()
            .flat_map(|w| w.acceptance_criteria.iter().cloned())
            .collect();
        criteria.sort();
        criteria.dedup();
        criteria
    }

    /// Get a snapshot of all workers' statuses.
    ///
    /// Returns tuples of `(WorkerId, WorkerStatus, task_title, assigned_provider_id)`
    /// for every worker in the queue. Returns an empty vec if no queue exists yet.
    pub fn worker_statuses(&self) -> Vec<(WorkerId, WorkerStatus, String, Option<ProviderId>)> {
        let Some(queue) = self.worker_queue.as_ref() else {
            return vec![];
        };
        // Collect worker data first to release the queue borrow before calling
        // worker_task_title() which needs to borrow self.execution_plan.
        let worker_data: Vec<(WorkerId, WorkerStatus, Option<ProviderId>)> = queue
            .workers()
            .map(|w| (w.worker_id, w.status, w.provider_id.clone()))
            .collect();
        worker_data
            .into_iter()
            .map(|(id, status, provider_id)| {
                let task_title = self.worker_task_title(&id).unwrap_or_default();
                (id, status, task_title, provider_id)
            })
            .collect()
    }

    /// Get the task title for a worker identified by its string key ("worker-N").
    ///
    /// Used by the stall detector in the driver to populate the `task_name` field
    /// of `WorkerStalledEvent` without holding a lock on the execution plan.
    pub fn worker_task_title_by_key(&self, worker_key: &str) -> Option<String> {
        let n: u32 = worker_key.strip_prefix("worker-")?.parse().ok()?;
        let wid = mahalaxmi_core::types::WorkerId::new(n);
        self.worker_task_title(&wid)
    }

    /// Get the assigned provider for a worker.
    ///
    /// Returns the routed provider if available, falling back to the
    /// cycle's default provider for backward compatibility.
    pub fn worker_provider(&self, worker_id: &WorkerId) -> Option<String> {
        self.worker_queue
            .as_ref()
            .and_then(|q| q.get_worker(worker_id))
            .and_then(|w| w.provider_id.as_ref())
            .map(|p| p.as_str().to_string())
            .or_else(|| Some(self.config.provider_id.clone()))
    }

    /// Update `provider_id` on a queued worker to reflect the provider that
    /// was actually selected at spawn time (may differ from the plan assignment
    /// when fallback routing kicks in).
    ///
    /// Called from the driver immediately after a provider is resolved so that
    /// `generate_cycle_report()` and the Dashboard "What was accomplished"
    /// panel show the correct provider name rather than the planned one.
    pub fn set_worker_runtime_provider(
        &mut self,
        worker_id: &WorkerId,
        provider_id: ProviderId,
    ) {
        if let Some(queue) = self.worker_queue.as_mut() {
            if let Some(worker) = queue.get_worker_mut(worker_id) {
                worker.provider_id = Some(provider_id);
            }
        }
    }

    // =========================================================================
    // Phase 4: Worker Execution
    // =========================================================================

    /// Initialize workers — transitions to InitializingWorkers then WorkersProcessing.
    ///
    /// When `config.worker_count == 0` (auto mode), the effective concurrency
    /// limit is set to the number of tasks produced by consensus, with a minimum
    /// of 1. The existing worker queue (created by `run_consensus()`) has its
    /// `max_concurrent` updated in-place so that provider assignments and
    /// assembled prompts set by earlier phases are preserved.
    ///
    /// Must be called after `run_consensus()`.
    pub fn begin_worker_execution(&mut self) -> MahalaxmiResult<()> {
        let event = self
            .state_machine
            .transition_to(OrchestrationCycleState::InitializingWorkers, &self.i18n)?;
        self.emit(event);

        // P7: Auto-scale the concurrency limit when worker_count == 0.
        if self.config.worker_count == 0 {
            let task_count = self
                .execution_plan
                .as_ref()
                .map(|p| p.all_workers().len() as u32)
                .unwrap_or(0);
            let effective = task_count.max(1);
            tracing::info!(
                auto_scaled = true,
                task_count = task_count,
                effective = effective,
                "Auto-scaling worker count to task count"
            );
            if let Some(queue) = self.worker_queue.as_mut() {
                queue.set_max_concurrent(effective);
            }
        }

        let event = self
            .state_machine
            .transition_to(OrchestrationCycleState::WorkersProcessing, &self.i18n)?;
        self.emit(event);

        Ok(())
    }

    /// Get the effective worker concurrency limit.
    ///
    /// Returns the `max_concurrent` value from the active worker queue, which
    /// reflects auto-scaling. Falls back to `config.worker_count` when no queue
    /// has been created yet.
    pub fn effective_worker_count(&self) -> u32 {
        self.worker_queue
            .as_ref()
            .map(|q| q.max_concurrent())
            .unwrap_or(self.config.worker_count)
    }

    /// Returns the worker count mandated by the active domain's decomposition
    /// strategy when it is `RoleBased`, or `None` for all other strategies.
    ///
    /// Used by the driver before `build_manager_prompts()` to override the
    /// caller-supplied `worker_count` so that `ManagerPromptConfig.worker_count`
    /// correctly reflects the number of specialist roles.  Returns `None` when
    /// `active_domain` is absent (e.g. heal cycles that use
    /// `CycleMetadata::into_cycle_config()` which hardcodes `active_domain: None`).
    pub fn effective_worker_count_for_domain(&self) -> Option<u32> {
        match self.config.active_domain.as_ref()?.decomposition_strategy() {
            mahalaxmi_core::domain::DecompositionStrategy::RoleBased { roles } => {
                Some(roles.len() as u32)
            }
            _ => None,
        }
    }

    /// Override the worker count stored in the cycle config.
    ///
    /// Called by the driver immediately after
    /// `effective_worker_count_for_domain()` returns `Some(n)` so that
    /// `build_manager_prompts()` uses the domain-mandated count rather than
    /// the caller-supplied value.
    pub fn set_config_worker_count(&mut self, n: u32) {
        self.config.worker_count = n;
    }

    /// Get worker IDs ready for dispatch (respecting concurrency limits and dependencies).
    ///
    /// Returns empty if paused.
    pub fn ready_workers(&self) -> Vec<WorkerId> {
        if self.paused {
            return Vec::new();
        }
        self.worker_queue
            .as_ref()
            .map(|q| q.ready_worker_ids())
            .unwrap_or_default()
    }

    /// Get the affected files list for a worker (REQ-003 file scope enforcement).
    pub fn worker_affected_files(&self, worker_id: &WorkerId) -> Option<Vec<String>> {
        self.execution_plan.as_ref().and_then(|plan| {
            plan.all_workers()
                .into_iter()
                .find(|t| t.worker_id == *worker_id)
                .map(|t| t.affected_files.clone())
        })
    }

    /// Get the task description for a worker (the prompt to send to the AI worker).
    pub fn worker_task_description(&self, worker_id: &WorkerId) -> Option<String> {
        self.execution_plan.as_ref().and_then(|plan| {
            plan.all_workers()
                .into_iter()
                .find(|t| t.worker_id == *worker_id)
                .map(|t| t.description.clone())
        })
    }

    /// Get the task title for a worker.
    pub fn worker_task_title(&self, worker_id: &WorkerId) -> Option<String> {
        self.execution_plan.as_ref().and_then(|plan| {
            plan.all_workers()
                .into_iter()
                .find(|t| t.worker_id == *worker_id)
                .map(|t| t.title.clone())
        })
    }

    /// Get the task ID for a worker (used by the driver for worktree branch naming).
    pub fn worker_task_id(&self, worker_id: &WorkerId) -> Option<mahalaxmi_core::types::TaskId> {
        self.execution_plan.as_ref().and_then(|plan| {
            plan.all_workers()
                .into_iter()
                .find(|t| t.worker_id == *worker_id)
                .map(|t| t.task_id.clone())
        })
    }

    /// Activate a worker (mark as Active in the queue).
    ///
    /// Emits `WorkerStarted` event and increments the internal batch-active
    /// counter used to detect when all dispatched workers in a batch have
    /// resolved (see [`subscribe_batch_complete`]).
    pub fn activate_worker(&mut self, worker_id: WorkerId) -> MahalaxmiResult<()> {
        let queue = self.worker_queue.as_mut().ok_or_else(|| {
            crate::MahalaxmiError::orchestration(&self.i18n, "error-no-worker-queue", &[])
        })?;
        queue.activate_worker(worker_id, &self.i18n)?;

        let title = self
            .worker_task_title(&worker_id)
            .unwrap_or_else(|| format!("Worker {}", worker_id));

        self.batch_active_count += 1;

        self.emit(OrchestrationEvent::WorkerStarted {
            cycle_id: self.state_machine.cycle_id(),
            worker_id,
            task_summary: title,
            timestamp: Utc::now(),
        });
        Ok(())
    }

    /// Mark a worker as completed.
    ///
    /// Emits `WorkerCompleted` event. Automatically unblocks dependent workers.
    pub fn complete_worker(
        &mut self,
        worker_id: WorkerId,
        duration_ms: u64,
    ) -> MahalaxmiResult<()> {
        // Record agent accuracy before completing
        if let Some(queue) = &self.worker_queue {
            if let Some(worker) = queue.get_worker(&worker_id) {
                if let Some(ref spec) = worker.agent_spec {
                    let score = worker
                        .last_verification
                        .as_ref()
                        .map(|v| {
                            let total = v.checks_run.len();
                            if total == 0 {
                                1.0
                            } else {
                                v.passed_count() as f64 / total as f64
                            }
                        })
                        .unwrap_or(1.0);
                    self.agent_registry.record_result(&spec.id, true, score);
                }
            }
        }

        let queue = self.worker_queue.as_mut().ok_or_else(|| {
            crate::MahalaxmiError::orchestration(&self.i18n, "error-no-worker-queue", &[])
        })?;
        queue.complete_worker(worker_id, &self.i18n)?;

        self.emit(OrchestrationEvent::WorkerCompleted {
            cycle_id: self.state_machine.cycle_id(),
            worker_id,
            success: true,
            duration_ms,
            timestamp: Utc::now(),
        });

        if self.batch_active_count > 0 {
            self.batch_active_count -= 1;
            if self.batch_active_count == 0 {
                self.emit_batch_complete_signal();
            }
        }
        Ok(())
    }

    /// Mark a worker as completed, bypassing the verification gate.
    ///
    /// Call this instead of `complete_worker` when the worker produced no code
    /// changes (e.g. "task already done — skipping PR"). Verification requires
    /// terminal output to analyse; when there is none the worker would otherwise
    /// stall in `WorkerStatus::Verifying` indefinitely.
    pub fn force_complete_worker(
        &mut self,
        worker_id: WorkerId,
        duration_ms: u64,
    ) -> MahalaxmiResult<()> {
        // Record agent accuracy (treat no-changes as a perfect pass)
        if let Some(queue) = &self.worker_queue {
            if let Some(worker) = queue.get_worker(&worker_id) {
                if let Some(ref spec) = worker.agent_spec {
                    self.agent_registry.record_result(&spec.id, true, 1.0);
                }
            }
        }

        let queue = self.worker_queue.as_mut().ok_or_else(|| {
            crate::MahalaxmiError::orchestration(&self.i18n, "error-no-worker-queue", &[])
        })?;
        queue.force_complete_worker(worker_id, &self.i18n)?;

        self.emit(OrchestrationEvent::WorkerCompleted {
            cycle_id: self.state_machine.cycle_id(),
            worker_id,
            success: true,
            duration_ms,
            timestamp: Utc::now(),
        });

        if self.batch_active_count > 0 {
            self.batch_active_count -= 1;
            if self.batch_active_count == 0 {
                self.emit_batch_complete_signal();
            }
        }
        Ok(())
    }

    /// Mark a worker as failed.
    ///
    /// Emits `WorkerFailed` event.
    pub fn fail_worker(
        &mut self,
        worker_id: WorkerId,
        error_summary: String,
    ) -> MahalaxmiResult<()> {
        // Record agent failure
        if let Some(queue) = &self.worker_queue {
            if let Some(worker) = queue.get_worker(&worker_id) {
                if let Some(ref spec) = worker.agent_spec {
                    self.agent_registry.record_result(&spec.id, false, 0.0);
                }
            }
        }

        let queue = self.worker_queue.as_mut().ok_or_else(|| {
            crate::MahalaxmiError::orchestration(&self.i18n, "error-no-worker-queue", &[])
        })?;

        let retry_count = queue
            .get_worker(&worker_id)
            .map(|w| w.failure_count)
            .unwrap_or(0);

        // Store error summary on the worker for fix task generation (P7)
        if let Some(worker) = queue.get_worker_mut(&worker_id) {
            worker.retry_context = Some(error_summary.clone());
        }

        queue.fail_worker(worker_id, &self.i18n)?;

        self.emit(OrchestrationEvent::WorkerFailed {
            cycle_id: self.state_machine.cycle_id(),
            worker_id,
            error_summary,
            retry_count,
            timestamp: Utc::now(),
        });

        if self.batch_active_count > 0 {
            self.batch_active_count -= 1;
            if self.batch_active_count == 0 {
                self.emit_batch_complete_signal();
            }
        }
        Ok(())
    }

    /// Attempt to fail a worker with provider fallback.
    ///
    /// If the worker has fallback providers available, switches to the next
    /// fallback and resets to Pending. Emits `ProviderSwitched` event.
    /// Returns `true` if fallback was used, `false` if no fallback available
    /// (delegates to regular `fail_worker`).
    pub fn fail_worker_with_fallback(
        &mut self,
        worker_id: WorkerId,
        failed_provider_id: ProviderId,
        error_summary: String,
    ) -> MahalaxmiResult<bool> {
        // Check if fallbacks are available using scoped immutable borrow
        let has_fallbacks = {
            let queue = self.worker_queue.as_ref().ok_or_else(|| {
                crate::MahalaxmiError::orchestration(&self.i18n, "error-no-worker-queue", &[])
            })?;
            queue
                .get_worker(&worker_id)
                .map(|w| !w.fallback_provider_ids.is_empty())
                .unwrap_or(false)
        };

        if !has_fallbacks {
            self.fail_worker(worker_id, error_summary)?;
            return Ok(false);
        }

        // Perform the fallback switch
        let queue = self.worker_queue.as_mut().ok_or_else(|| {
            crate::MahalaxmiError::orchestration(&self.i18n, "error-no-worker-queue", &[])
        })?;

        let worker = queue.get_worker_mut(&worker_id).ok_or_else(|| {
            crate::MahalaxmiError::orchestration(
                &self.i18n,
                "error-worker-not-found",
                &[("worker_id", &worker_id.to_string())],
            )
        })?;

        let from_provider_id = failed_provider_id.clone(); // Use the failed_provider_id
        let next_provider = worker.fallback_provider_ids.remove(0);
        let to_provider_id = next_provider.clone(); // Use the actual ProviderId
        worker.provider_id = Some(next_provider);
        worker.status = mahalaxmi_core::types::WorkerStatus::Pending;
        worker.started_at = None;
        worker.finished_at = None;

        self.emit(OrchestrationEvent::ProviderSwitched {
            cycle_id: self.state_machine.cycle_id(),
            worker_id,
            from_provider: from_provider_id.to_string(), // Convert to String for the event
            to_provider: to_provider_id.to_string(),     // Convert to String for the event
            reason: error_summary,
            timestamp: Utc::now(),
        });

        Ok(true)
    }

    // =========================================================================
    // Phase 5: Completion
    // =========================================================================

    /// Check if all workers have finished (completed or failed).
    ///
    /// Returns `true` if there are no workers or all workers are done.
    pub fn all_workers_finished(&self) -> bool {
        self.worker_queue
            .as_ref()
            .map(|q| {
                let stats = q.statistics();
                // Empty queue (no agreed tasks) counts as finished
                stats.total == 0
                    || (stats.active == 0
                        && stats.pending == 0
                        && stats.blocked == 0
                        && stats.verifying == 0)
            })
            .unwrap_or(true)
    }

    /// Complete the cycle — transitions to CompletingCycle then Idle.
    ///
    /// Emits `CycleCompleted` event with final statistics.
    pub fn complete_cycle(&mut self, total_duration_ms: u64) -> MahalaxmiResult<()> {
        let event = self
            .state_machine
            .transition_to(OrchestrationCycleState::CompletingCycle, &self.i18n)?;
        self.emit(event);

        let stats = self
            .worker_queue
            .as_ref()
            .map(|q| q.statistics())
            .unwrap_or_default();

        let success_rate = if stats.total > 0 {
            stats.completed as f64 / stats.total as f64
        } else {
            1.0
        };

        self.emit(OrchestrationEvent::CycleCompleted {
            cycle_id: self.state_machine.cycle_id(),
            total_duration_ms,
            success_rate,
            completed_workers: stats.completed,
            failed_workers: stats.failed,
            total_workers: stats.total,
            timestamp: Utc::now(),
        });

        let event = self
            .state_machine
            .transition_to(OrchestrationCycleState::Idle, &self.i18n)?;
        self.emit(event);

        Ok(())
    }

    /// Generate a structured cycle report from the current cycle state.
    ///
    /// Collects completion/failure data from the worker queue, verification
    /// results, and agent performance metrics. Stores the report internally
    /// for injection into the next cycle's manager prompt via
    /// `last_cycle_report()`.
    ///
    /// Should be called after all workers have finished and before or during
    /// `complete_cycle()`.
    pub fn generate_cycle_report(&mut self, total_duration_ms: u64) -> CycleReport {
        let now = Utc::now();
        let cycle_id = self.state_machine.cycle_id();
        let started_at = self.cycle_started_at.unwrap_or(now);

        let mut tasks_completed = Vec::new();
        let mut tasks_failed = Vec::new();
        let mut all_files_modified = Vec::new();
        let mut verification_results = Vec::new();
        let mut recommendations = Vec::new();

        // Collect per-agent stats for this cycle
        let mut agent_completed: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        let mut agent_failed: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        if let (Some(queue), Some(plan)) = (&self.worker_queue, &self.execution_plan) {
            for worker in queue.workers() {
                // Look up the task in the plan
                let task = plan
                    .all_workers()
                    .into_iter()
                    .find(|t| t.worker_id == worker.worker_id);
                let title = task.map(|t| t.title.clone()).unwrap_or_default();
                let affected_files = task.map(|t| t.affected_files.clone()).unwrap_or_default();
                let provider = worker
                    .provider_id
                    .as_ref()
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| self.config.provider_id.clone());

                let duration_ms = match (worker.started_at, worker.finished_at) {
                    (Some(start), Some(end)) => {
                        end.signed_duration_since(start).num_milliseconds().max(0) as u64
                    }
                    _ => 0,
                };

                match worker.status {
                    mahalaxmi_core::types::WorkerStatus::Completed => {
                        all_files_modified.extend(affected_files.iter().cloned());
                        tasks_completed.push(TaskSummary {
                            title: title.clone(),
                            provider_id: provider,
                            duration_ms,
                            files_modified: affected_files,
                        });

                        if let Some(ref spec) = worker.agent_spec {
                            *agent_completed.entry(spec.id.clone()).or_insert(0) += 1;
                        }
                    }
                    mahalaxmi_core::types::WorkerStatus::Failed => {
                        let error = worker
                            .retry_context
                            .clone()
                            .unwrap_or_else(|| "Unknown error".to_owned());

                        let recommendation = if worker.failure_count >= worker.max_retries {
                            format!(
                                "Task failed after {} retries. Consider splitting into smaller subtasks or using a different provider.",
                                worker.failure_count
                            )
                        } else {
                            "Retry with additional context or a different approach.".to_owned()
                        };

                        tasks_failed.push(TaskFailureSummary {
                            title: title.clone(),
                            provider_id: provider,
                            error,
                            retry_count: worker.failure_count,
                            recommendation,
                        });

                        if let Some(ref spec) = worker.agent_spec {
                            *agent_failed.entry(spec.id.clone()).or_insert(0) += 1;
                        }
                    }
                    _ => {}
                }

                // Verification results
                if let Some(ref verification) = worker.last_verification {
                    let checks_passed = verification.passed_count();
                    let checks_total = verification.checks_run.len();
                    let failure_details: Vec<String> = verification
                        .checks_run
                        .iter()
                        .filter(|c| !c.passed)
                        .map(|c| format!("{}: {}", c.check, c.details))
                        .collect();

                    verification_results.push(VerificationSummary {
                        title: title.clone(),
                        passed: verification.status.is_success(),
                        checks_passed,
                        checks_total,
                        failure_details,
                    });
                }
            }
        }

        // Deduplicate files
        all_files_modified.sort();
        all_files_modified.dedup();

        // Build agent performance summaries
        let mut agent_performance = Vec::new();
        let agent_ids: std::collections::HashSet<String> = agent_completed
            .keys()
            .chain(agent_failed.keys())
            .cloned()
            .collect();
        for agent_id in agent_ids {
            let completed = agent_completed.get(&agent_id).copied().unwrap_or(0);
            let failed = agent_failed.get(&agent_id).copied().unwrap_or(0);
            let (name, lifetime_rate) = self
                .agent_registry
                .get(&agent_id)
                .map(|spec| {
                    let rate = self
                        .agent_registry
                        .get_record(&agent_id)
                        .map(|r| r.success_rate())
                        .unwrap_or(0.0);
                    (spec.name.clone(), rate)
                })
                .unwrap_or_else(|| (agent_id.clone(), 0.0));

            agent_performance.push(AgentPerformanceSummary {
                agent_id: agent_id.clone(),
                agent_name: name,
                tasks_completed: completed,
                tasks_failed: failed,
                lifetime_success_rate: lifetime_rate,
            });
        }

        // Generate recommendations
        let total_tasks = tasks_completed.len() + tasks_failed.len();
        if !tasks_failed.is_empty() {
            recommendations.push(format!(
                "{} of {} tasks failed — address these failures before adding new work.",
                tasks_failed.len(),
                total_tasks,
            ));
        }
        for fail in &tasks_failed {
            if fail.retry_count >= 2 {
                recommendations.push(format!(
                    "Task \"{}\" failed {} times — consider splitting it into smaller subtasks.",
                    fail.title, fail.retry_count,
                ));
            }
        }

        let completed_count = tasks_completed.len() as u32;
        let total = total_tasks as u32;
        let status = if tasks_failed.is_empty() && total > 0 {
            CycleOutcome::AllTasksCompleted
        } else if completed_count > 0 {
            CycleOutcome::PartialCompletion {
                completed: completed_count,
                total,
            }
        } else if total == 0 {
            CycleOutcome::Failed {
                reason: "No tasks were executed".to_owned(),
            }
        } else {
            CycleOutcome::Failed {
                reason: format!("All {} tasks failed", total),
            }
        };

        let report = CycleReport {
            cycle_id,
            started_at,
            completed_at: now,
            duration_ms: total_duration_ms,
            status,
            tasks_completed,
            tasks_failed,
            files_modified: all_files_modified,
            verification_results,
            agent_performance,
            recommendations,
            cost_summary: CycleCostSummary::default(),
            plan_audit: Vec::new(),
        };

        self.last_cycle_report = Some(report.clone());
        report
    }

    /// Get the last cycle report, if one was generated.
    pub fn last_cycle_report(&self) -> Option<&CycleReport> {
        self.last_cycle_report.as_ref()
    }

    // =========================================================================
    // Phase 5b: Self-Healing / Fix Cycle Support
    // =========================================================================

    /// Record partial progress on a failed worker.
    ///
    /// Called by the driver after extracting partial progress from a worktree
    /// before the worktree is cleaned up. Stores the list of files modified
    /// and an optional output snapshot on the worker for inclusion in the
    /// cycle report and fix task generation.
    pub fn record_partial_progress(
        &mut self,
        worker_id: WorkerId,
        files_modified: Vec<String>,
        partial_output: Option<String>,
    ) {
        if let Some(queue) = &mut self.worker_queue {
            if let Some(worker) = queue.get_worker_mut(&worker_id) {
                worker.partial_files_modified = files_modified;
                worker.partial_output = partial_output;
            }
        }
    }

    /// Generate fix tasks from failed workers in the current cycle.
    ///
    /// For each failed worker, builds a new `WorkerTask` that includes:
    /// - The original task description
    /// - The failure reason (error context)
    /// - Partial progress (what was already accomplished)
    /// - Specific guidance to complete the remaining work
    ///
    /// Returns the fix tasks. If no workers failed, returns an empty vec.
    pub fn generate_fix_tasks(&self) -> Vec<crate::models::plan::WorkerTask> {
        use crate::models::plan::WorkerTask;
        use mahalaxmi_core::types::{TaskId, WorkerId as WId};

        let queue = match &self.worker_queue {
            Some(q) => q,
            None => return Vec::new(),
        };
        let plan = match &self.execution_plan {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut fix_tasks = Vec::new();
        let mut fix_index = 0u32;

        for worker in queue.workers() {
            if worker.status != mahalaxmi_core::types::WorkerStatus::Failed {
                continue;
            }
            // Workers with failure_count == 0 were cascade-failed because a
            // dependency failed; they never actually ran so there is nothing to
            // fix or retry — skip them.
            if worker.failure_count == 0 {
                continue;
            }

            // Look up the original task in the plan
            let original_task = plan
                .all_workers()
                .into_iter()
                .find(|t| t.worker_id == worker.worker_id);

            let (title, description, affected_files, complexity) = match original_task {
                Some(t) => (
                    t.title.clone(),
                    t.description.clone(),
                    t.affected_files.clone(),
                    t.complexity,
                ),
                None => continue,
            };

            let error_context = worker
                .retry_context
                .clone()
                .unwrap_or_else(|| "Unknown error".to_owned());

            let partial_progress_note = if !worker.partial_files_modified.is_empty() {
                format!(
                    "\n\nPartial progress from previous attempt:\n\
                     Files already modified: {}\n\
                     These changes may already be merged into the codebase.",
                    worker.partial_files_modified.join(", ")
                )
            } else {
                String::new()
            };

            let fix_description = format!(
                "FIX TASK: Complete the remaining work for \"{title}\".\n\n\
                 Original task description:\n{description}\n\n\
                 Previous attempt failed with error:\n{error_context}\
                 {partial_progress_note}\n\n\
                 Instructions:\n\
                 1. Review what was already done (if any partial progress was merged).\n\
                 2. Fix the issues that caused the previous failure.\n\
                 3. Complete any remaining work from the original description.\n\
                 4. Ensure all acceptance criteria from the original task are met."
            );

            let task_id = TaskId::new(format!("fix-task-{fix_index}"));
            let worker_id = WId::new(fix_index);

            let mut fix_task =
                WorkerTask::new(task_id, worker_id, format!("Fix: {title}"), fix_description);
            fix_task.complexity = complexity;
            fix_task.affected_files = affected_files;

            fix_tasks.push(fix_task);
            fix_index += 1;
        }

        fix_tasks
    }

    /// Set up a fix cycle with pre-generated fix tasks.
    ///
    /// When called before `initialize()`, the service enters "fix cycle" mode:
    /// the driver should skip manager prompting and consensus, and go directly
    /// to worker execution with the provided fix tasks.
    ///
    /// Returns `true` if fix tasks were set (fix cycle will be used),
    /// `false` if the list was empty (normal cycle should proceed).
    pub fn setup_fix_cycle(&mut self, fix_tasks: Vec<crate::models::plan::WorkerTask>) -> bool {
        if fix_tasks.is_empty() {
            self.is_fix_cycle = false;
            return false;
        }
        self.fix_tasks = fix_tasks;
        self.is_fix_cycle = true;
        true
    }

    /// Check if this is a fix cycle (skips manager/consensus phases).
    pub fn is_fix_cycle(&self) -> bool {
        self.is_fix_cycle
    }

    /// Build the execution plan directly from fix tasks, bypassing consensus.
    ///
    /// Only valid when `is_fix_cycle()` returns true. All fix tasks are placed
    /// in a single phase (no dependencies between fix tasks) since each fixes
    /// an independent failure.
    pub fn build_fix_plan(&mut self) -> MahalaxmiResult<()> {
        use crate::models::plan::{ExecutionPhase, ExecutionPlan};

        if self.fix_tasks.is_empty() {
            return Ok(());
        }

        let tasks = std::mem::take(&mut self.fix_tasks);
        let plan = ExecutionPlan::from_phases(vec![ExecutionPhase {
            phase_number: 0,
            tasks,
        }]);

        let queue = crate::queue::WorkerQueue::from_plan_with_verification(
            &plan,
            self.config.worker_count,
            self.config.max_retries,
            self.verification_config.clone(),
        );

        self.execution_plan = Some(plan);
        self.worker_queue = Some(queue);
        Ok(())
    }

    // =========================================================================
    // Control: Pause / Resume / Stop
    // =========================================================================

    /// Pause the cycle. Workers continue but no new ones are dispatched.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume a paused cycle.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Force stop the cycle.
    ///
    /// Emits `StateChanged` event with transition to Stopped.
    pub fn stop(&mut self) {
        let event = self.state_machine.force_stop();
        self.emit(event);
    }

    /// Force error the cycle.
    ///
    /// Emits `StateChanged` event with transition to Error.
    pub fn force_error(&mut self) {
        let event = self.state_machine.force_error();
        self.emit(event);
    }

    // =========================================================================
    // Command processing
    // =========================================================================

    /// Process a command received from the command channel.
    pub fn handle_command(&mut self, cmd: OrchestrationCommand) {
        match cmd {
            OrchestrationCommand::Pause => self.pause(),
            OrchestrationCommand::Resume => self.resume(),
            OrchestrationCommand::Stop => self.stop(),
        }
    }

    /// Subscribe to orchestration events.
    pub fn subscribe(&self) -> broadcast::Receiver<OrchestrationEvent> {
        self.event_tx.subscribe()
    }

    /// Subscribe to batch-level completion signals.
    ///
    /// Returns a `watch::Receiver` whose current value is `None` at
    /// construction time. Each time all dispatched workers in a batch resolve
    /// (either complete or permanently fail), the value is updated to
    /// `Some(BatchCompleteSignal)`.
    ///
    /// Subscribers should use `changed()` (async) to wait for the next
    /// signal, or `borrow()` (sync) to inspect the latest value.
    ///
    /// Review-mode and precise-mode drivers subscribe here to coordinate
    /// cross-batch analysis and retry decisions.
    pub fn subscribe_batch_complete(&self) -> watch::Receiver<Option<BatchCompleteSignal>> {
        self.batch_complete_tx.subscribe()
    }

    /// Emit a batch-complete signal to all subscribers.
    ///
    /// This method is provided for drivers that have richer data than the
    /// orchestration service (e.g., combined git diffs, process exit codes).
    /// Drivers may call this directly to override the auto-emitted signal with
    /// one that has `combined_diff` and `exit_codes` populated.
    ///
    /// The service also emits automatically via `emit_batch_complete_signal`
    /// after the batch-active counter reaches zero.
    pub fn notify_batch_complete(&self, signal: BatchCompleteSignal) {
        let _ = self.batch_complete_tx.send(Some(signal));
    }

    /// Return the most recent batch-complete signal, or `None` if no batch has
    /// completed yet.
    ///
    /// Unlike [`subscribe_batch_complete`], this is a synchronous snapshot that
    /// does not block. Useful in the driver dispatch loop where the caller
    /// already knows a batch just finished.
    pub fn batch_complete_signal(&self) -> Option<BatchCompleteSignal> {
        self.batch_complete_tx.borrow().clone()
    }

    /// Replace the execution plan's task list with a new set of task
    /// descriptions and rebuild the worker queue for the next batch.
    ///
    /// Called by the driver after a review/precise mode pass returns
    /// [`CompletionVerdict::Continue`] with an updated task list.
    /// Each description string is converted into a [`WorkerTask`] with a
    /// generated `task_id` and `worker_id`.
    ///
    /// Returns an error if there is no active execution plan.
    pub fn set_next_batch_tasks(
        &mut self,
        task_descriptions: Vec<String>,
    ) -> crate::MahalaxmiResult<()> {
        use mahalaxmi_core::error::MahalaxmiError;
        use mahalaxmi_core::types::{TaskId, WorkerId};
        use crate::models::plan::{ExecutionPhase, WorkerTask};

        let plan = self
            .execution_plan
            .as_mut()
            .ok_or_else(|| MahalaxmiError::Config {
                message: "No active execution plan".to_string(),
                i18n_key: "error.no_execution_plan".to_string(),
            })?;

        let queue = self
            .worker_queue
            .as_ref()
            .ok_or_else(|| MahalaxmiError::Config {
                message: "No active worker queue".to_string(),
                i18n_key: "error.no_worker_queue".to_string(),
            })?;

        let max_concurrent = queue.max_concurrent();
        let max_retries = self.config.max_retries;
        let verification_config = self.verification_config.clone();

        let tasks: Vec<WorkerTask> = task_descriptions
            .into_iter()
            .enumerate()
            .map(|(i, desc)| {
                WorkerTask::new(
                    TaskId::new(format!("review-task-{i}")),
                    WorkerId::new(i as u32),
                    desc.as_str(),
                    desc.as_str(),
                )
            })
            .collect();

        // Replace the plan's first (or only) phase with the new tasks.
        if let Some(phase) = plan.phases.first_mut() {
            phase.tasks = tasks;
        } else {
            plan.phases.push(ExecutionPhase {
                phase_number: 0,
                tasks,
            });
        }

        self.worker_queue = Some(crate::queue::WorkerQueue::from_plan_with_verification(
            plan,
            max_concurrent,
            max_retries,
            verification_config,
        ));

        Ok(())
    }

    // =========================================================================
    // Internal
    // =========================================================================

    /// Get a reference to the execution plan (if created).
    pub fn execution_plan(&self) -> Option<&ExecutionPlan> {
        self.execution_plan.as_ref()
    }

    /// Replace the current execution plan with a modified version and rebuild the worker queue.
    ///
    /// Called after interactive plan review to apply developer modifications.
    /// The worker queue is rebuilt from the new plan, preserving the current
    /// `max_concurrent` slot limit, retry budget, and verification config.
    ///
    /// Has no effect when no plan/queue has been created yet.
    pub fn replace_execution_plan(&mut self, new_plan: ExecutionPlan) {
        let Some(current_queue) = self.worker_queue.as_ref() else {
            return;
        };
        let max_concurrent = current_queue.max_concurrent();
        let max_retries = self.config.max_retries;
        let verification_config = self.verification_config.clone();

        let new_queue = WorkerQueue::from_plan_with_verification(
            &new_plan,
            max_concurrent,
            max_retries,
            verification_config,
        );
        self.execution_plan = Some(new_plan);
        self.worker_queue = Some(new_queue);
    }

    /// Get a reference to the worker queue (if created).
    pub fn worker_queue(&self) -> Option<&WorkerQueue> {
        self.worker_queue.as_ref()
    }

    /// Get a mutable reference to the worker queue (if created).
    pub fn worker_queue_mut(&mut self) -> Option<&mut WorkerQueue> {
        self.worker_queue.as_mut()
    }

    /// Get the verification configuration.
    pub fn verification_config(&self) -> &VerificationConfig {
        &self.verification_config
    }

    /// Get a reference to the consensus engine.
    pub fn consensus_engine(&self) -> &ConsensusEngine {
        &self.consensus_engine
    }

    /// Return a snapshot of all proposals (including incomplete ones).
    ///
    /// Used by the driver to extract proposal data for async LLM arbitration
    /// outside the mutex-locked region.
    pub fn proposals(&self) -> &[ManagerProposal] {
        &self.proposals
    }

    /// Inject pre-built proposals directly into the orchestration service.
    ///
    /// Called by the Phase 27 adversarial deliberation path after all domain
    /// teams have synthesized their final task lists.  Bypasses PTY manager
    /// terminals entirely — the injected proposals are treated identically to
    /// proposals produced by `submit_manager_output`.
    ///
    /// Replaces any existing proposals.  Must be called while the service is
    /// in the `CollectingManagerOutput` state (i.e. after `build_manager_prompts`).
    pub fn inject_proposals(&mut self, proposals: Vec<ManagerProposal>) {
        self.proposals = proposals;
        tracing::info!(
            count = self.proposals.len(),
            "Adversarial deliberation proposals injected into orchestration service"
        );
    }

    /// Run consensus on pre-grouped (and optionally arbitrated) task groups.
    ///
    /// Identical to [`run_consensus`] except it skips the initial grouping step
    /// and uses the provided groups directly.  Called by the driver when LLM
    /// arbitration has already been applied to the groups.
    pub fn run_consensus_with_pre_grouped(
        &mut self,
        groups: Vec<crate::consensus::normalizer::TaskGroup>,
    ) -> MahalaxmiResult<ConsensusResult> {
        // Transition to AnalyzingManagerResults
        let event = self
            .state_machine
            .transition_to(OrchestrationCycleState::AnalyzingManagerResults, &self.i18n)?;
        self.emit(event);

        let result = if groups.is_empty() {
            ConsensusResult::no_consensus(self.config.consensus_config.strategy)
        } else {
            self.consensus_engine.evaluate_from_groups(
                groups,
                &self.proposals.clone(),
                &self.i18n,
            )?
        };

        self.emit(OrchestrationEvent::ConsensusReached {
            cycle_id: self.state_machine.cycle_id(),
            agreed_count: result.agreed_tasks.len() as u32,
            dissenting_count: result.dissenting_tasks.len() as u32,
            timestamp: Utc::now(),
        });

        // Transition to DiscoveringWorkerRequirements
        let event = self.state_machine.transition_to(
            OrchestrationCycleState::DiscoveringWorkerRequirements,
            &self.i18n,
        )?;
        self.emit(event);

        // Build execution plan from consensus result
        let mut plan = ExecutionPlan::from_consensus_result(&result, &self.i18n)?;

        // Enforce worktree scope (same as primary consensus path above).
        let filtered = plan.filter_platform_scoped_tasks();
        if filtered > 0 {
            tracing::warn!(
                filtered_tasks = filtered,
                "Routing constraint removed {} platform-scoped task(s) from deliberation plan.",
                filtered
            );
        }

        // Populate contributing_developers
        if !self.manager_developer_ids.is_empty() {
            let agreed_manager_ids: std::collections::HashSet<&ManagerId> = result
                .agreed_tasks
                .iter()
                .flat_map(|t| &t.proposed_by)
                .collect();

            let mut contributors: std::collections::HashSet<DeveloperId> =
                std::collections::HashSet::new();
            for p in &self.proposals {
                if p.completed && agreed_manager_ids.contains(&p.manager_id) {
                    if let Some(ref dev_id) = p.developer_id {
                        contributors.insert(dev_id.clone());
                    }
                }
            }
            plan.contributing_developers = contributors.into_iter().collect();
            plan.contributing_developers.sort_by(|a, b| {
                let a_str: &str = a.as_ref();
                let b_str: &str = b.as_ref();
                a_str.cmp(b_str)
            });
        }

        self.emit(OrchestrationEvent::PlanCreated {
            cycle_id: self.state_machine.cycle_id(),
            phase_count: plan.phase_count() as u32,
            worker_count: plan.all_workers().len() as u32,
            timestamp: Utc::now(),
        });

        let queue = WorkerQueue::from_plan_with_verification(
            &plan,
            self.config.worker_count,
            self.config.max_retries,
            self.verification_config.clone(),
        );

        self.execution_plan = Some(plan);
        self.worker_queue = Some(queue);

        Ok(result)
    }

    /// Emit an event to all subscribers.
    fn emit(&self, event: OrchestrationEvent) {
        // Ignore send errors (no receivers connected yet is fine)
        let _ = self.event_tx.send(event);
    }

    /// Construct and send a `BatchCompleteSignal` on the watch channel.
    ///
    /// Called automatically when `batch_active_count` reaches zero (i.e.,
    /// every worker dispatched in the current batch has resolved). Populates
    /// aggregate counts from the current queue statistics.
    ///
    /// `combined_diff` and `exit_codes` are left empty; drivers that have
    /// access to git output and process exit codes can enrich the signal by
    /// calling [`notify_batch_complete`] directly after this fires.
    fn emit_batch_complete_signal(&mut self) {
        let stats = self
            .worker_queue
            .as_ref()
            .map(|q| q.statistics())
            .unwrap_or_default();

        let signal = BatchCompleteSignal {
            batch_number: self.batch_number,
            success_count: stats.completed as usize,
            permanent_failure_count: stats.failed as usize,
            combined_diff: String::new(),
            exit_codes: Vec::new(),
        };

        tracing::debug!(
            batch_number = self.batch_number,
            success_count = signal.success_count,
            failure_count = signal.permanent_failure_count,
            "Batch complete signal emitted"
        );

        self.batch_number += 1;
        let _ = self.batch_complete_tx.send(Some(signal));
    }
}

/// Normalize a task title to a key for matching against ProposedTask.normalized_key().
fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .trim()
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahalaxmi_core::config::VerificationConfig;
    use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
    use mahalaxmi_core::types::{
        ConsensusStrategy, Developer, DeveloperId, DeveloperRegistry, DeveloperSession,
        DeveloperSessionStatus, GitMergeStrategy, GitPrPlatform,
    };
    use tokio::sync::broadcast;

    fn test_i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    /// Minimal `CycleConfig` for tests — uses WeightedVoting so that developer
    /// weights are exercised through the full consensus path.
    fn weighted_cycle_config(threshold: f64) -> CycleConfig {
        CycleConfig {
            project_root: "/tmp/test".into(),
            provider_id: "claude-code".into(),
            manager_count: 0, // overridden by developer sessions
            worker_count: 2,
            max_retries: 0,
            consensus_config: crate::models::ConsensusConfiguration {
                strategy: ConsensusStrategy::WeightedVoting,
                minimum_agreement_threshold: threshold,
                ..crate::models::ConsensusConfiguration::default()
            },
            requirements: "shared requirements".into(),
            repo_map: String::new(),
            shared_memory: String::new(),
            provider_ids: Vec::new(),
            routing_strategy: String::new(),
            manager_provider_id: None,
            enable_review_chain: false,
            review_provider_id: None,
            accept_partial_progress: false,
            git_strategy: GitMergeStrategy::DirectMerge,
            git_target_branch: String::new(),
            git_auto_merge_pr: false,
            git_pr_platform: GitPrPlatform::GitHub,
            enable_validation: false,
            validator_provider_id: None,
            active_domain: None,
        }
    }

    fn make_service(threshold: f64) -> OrchestrationService {
        let (tx, _rx) = broadcast::channel(16);
        OrchestrationService::new(
            weighted_cycle_config(threshold),
            test_i18n(),
            tx,
            VerificationConfig {
                enabled: false,
                ..VerificationConfig::default()
            },
        )
    }

    /// JSON for a single-task manager output.
    fn single_task_json(title: &str, description: &str) -> String {
        format!(
            r#"{{"tasks":[{{"title":"{}","description":"{}","complexity":5,"priority":1,"dependencies":[],"affected_files":[]}}]}}"#,
            title, description
        )
    }

    /// Two developers contribute distinct manager proposals. After consensus the
    /// `ExecutionPlan.contributing_developers` must contain both `DeveloperIds`.
    #[test]
    fn test_team_consensus_two_developers() {
        let alice_id = DeveloperId::from("alice");
        let bob_id = DeveloperId::from("bob");

        // Registry: alice weight 1.0 (max_managers 1), bob weight 1.5 (max_managers 1).
        let mut registry = DeveloperRegistry::new();
        registry.add(Developer {
            id: alice_id.clone(),
            name: "Alice".into(),
            provider: "claude-code".into(),
            weight: 1.0,
            max_managers: 1,
            seat_id: None,
        });
        registry.add(Developer {
            id: bob_id.clone(),
            name: "Bob".into(),
            provider: "claude-code".into(),
            weight: 1.5,
            max_managers: 1,
            seat_id: None,
        });

        // Sessions carry per-developer requirements (distinct task domains).
        let sessions = vec![
            DeveloperSession {
                developer_id: alice_id.clone(),
                requirements: "Alice needs authentication tasks".into(),
                started_at: chrono::Utc::now(),
                status: DeveloperSessionStatus::Active,
            },
            DeveloperSession {
                developer_id: bob_id.clone(),
                requirements: "Bob needs database migration tasks".into(),
                started_at: chrono::Utc::now(),
                status: DeveloperSessionStatus::Active,
            },
        ];

        // threshold 0.3: alice's task scores 1.0/2.5=0.40 >= 0.30 ✓
        //                bob's task scores   1.5/2.5=0.60 >= 0.30 ✓
        let mut svc = make_service(0.3);
        svc.configure_developer_sessions(sessions, registry);
        svc.initialize().unwrap();

        let prompts = svc.build_manager_prompts().unwrap();
        // Expect one prompt slot per developer (max_managers = 1 each).
        assert_eq!(
            prompts.len(),
            2,
            "Should generate 2 prompts (one per developer)"
        );

        let (alice_manager_id, _) = prompts
            .iter()
            .find(|(mid, _)| mid.as_str().starts_with("alice"))
            .expect("alice manager prompt must exist");
        let (bob_manager_id, _) = prompts
            .iter()
            .find(|(mid, _)| mid.as_str().starts_with("bob"))
            .expect("bob manager prompt must exist");

        let alice_mid = alice_manager_id.clone();
        let bob_mid = bob_manager_id.clone();

        // Submit distinct manager outputs so proposals don't overlap.
        svc.submit_manager_output(
            alice_mid,
            &single_task_json("Implement auth module", "JWT-based authentication"),
            500,
        )
        .unwrap();

        svc.submit_manager_output(
            bob_mid,
            &single_task_json("Migrate user table", "Add columns to users table"),
            600,
        )
        .unwrap();

        svc.run_consensus().unwrap();

        let plan = svc.execution_plan().expect("execution plan must exist");

        // Both developer IDs must appear in contributing_developers.
        assert!(
            plan.contributing_developers.contains(&alice_id),
            "alice must be in contributing_developers; got: {:?}",
            plan.contributing_developers
        );
        assert!(
            plan.contributing_developers.contains(&bob_id),
            "bob must be in contributing_developers; got: {:?}",
            plan.contributing_developers
        );
    }

    /// `configure_developer_sessions()` with an empty session list leaves the
    /// service in single-developer mode and uses the config `manager_count`.
    #[test]
    fn test_team_consensus_empty_sessions_uses_config_manager_count() {
        let mut svc = make_service(0.5);
        // No configure_developer_sessions call — single-developer mode.
        svc.initialize().unwrap();

        // config.manager_count = 0 in weighted_cycle_config; the test verifies
        // that the single-developer path uses config.manager_count (0 prompts).
        let prompts = svc.build_manager_prompts().unwrap();
        assert_eq!(prompts.len(), 0);
    }

    /// Subscribe then notify delivers the expected signal to the receiver.
    #[test]
    fn test_subscribe_and_notify_batch_complete() {
        let svc = make_service(0.5);

        let mut rx = svc.subscribe_batch_complete();

        // Initial value is None.
        assert!(rx.borrow().is_none(), "initial watch value must be None");

        // Emit a signal with a known payload.
        let signal = BatchCompleteSignal {
            batch_number: 3,
            success_count: 5,
            permanent_failure_count: 1,
            combined_diff: "diff --git a/foo.rs b/foo.rs".to_string(),
            exit_codes: vec![0, 0, 0, 0, 1],
        };
        svc.notify_batch_complete(signal);

        // Receiver must observe the change.
        assert!(
            rx.has_changed().unwrap(),
            "has_changed() must be true after notify_batch_complete"
        );

        let received = rx.borrow_and_update();
        let s = received.as_ref().expect("signal must be Some after notify");
        assert_eq!(s.batch_number, 3);
        assert_eq!(s.success_count, 5);
        assert_eq!(s.permanent_failure_count, 1);
        assert_eq!(s.combined_diff, "diff --git a/foo.rs b/foo.rs");
        assert_eq!(s.exit_codes, vec![0, 0, 0, 0, 1]);
    }

    /// `BatchCompleteSignal::all_succeeded` returns true only when
    /// `permanent_failure_count == 0` AND all exit codes are 0.
    #[test]
    fn test_batch_complete_signal_all_succeeded() {
        let all_good = BatchCompleteSignal {
            batch_number: 0,
            success_count: 3,
            permanent_failure_count: 0,
            combined_diff: String::new(),
            exit_codes: vec![0, 0, 0],
        };
        assert!(all_good.all_succeeded());

        // Non-zero exit code → not all succeeded.
        let bad_exit = BatchCompleteSignal {
            batch_number: 0,
            success_count: 2,
            permanent_failure_count: 0,
            combined_diff: String::new(),
            exit_codes: vec![0, 1, 0],
        };
        assert!(!bad_exit.all_succeeded());

        // Permanent failure → not all succeeded.
        let perm_fail = BatchCompleteSignal {
            batch_number: 0,
            success_count: 2,
            permanent_failure_count: 1,
            combined_diff: String::new(),
            exit_codes: vec![0, 0],
        };
        assert!(!perm_fail.all_succeeded());

        // Empty exit_codes with no permanent failures → all_succeeded is true.
        let empty_exits = BatchCompleteSignal {
            batch_number: 0,
            success_count: 0,
            permanent_failure_count: 0,
            combined_diff: String::new(),
            exit_codes: Vec::new(),
        };
        assert!(empty_exits.all_succeeded());
    }

    /// Multiple subscribers all receive the same signal.
    #[test]
    fn test_multiple_subscribers_receive_signal() {
        let svc = make_service(0.5);

        let mut rx1 = svc.subscribe_batch_complete();
        let mut rx2 = svc.subscribe_batch_complete();

        let signal = BatchCompleteSignal {
            batch_number: 7,
            success_count: 2,
            permanent_failure_count: 0,
            combined_diff: String::new(),
            exit_codes: vec![0, 0],
        };
        svc.notify_batch_complete(signal);

        assert!(rx1.has_changed().unwrap());
        assert!(rx2.has_changed().unwrap());

        let v1 = rx1.borrow_and_update();
        let v2 = rx2.borrow_and_update();
        assert_eq!(v1.as_ref().unwrap().batch_number, 7);
        assert_eq!(v2.as_ref().unwrap().batch_number, 7);
    }
}

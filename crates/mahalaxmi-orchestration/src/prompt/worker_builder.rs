//! Worker prompt builder — assembles targeted prompts for individual AI worker agents.
//!
//! Each worker receives a focused prompt containing only the context relevant to
//! its assigned task. The builder is provider-aware: Claude-like providers receive
//! XML-formatted sections, while others receive Markdown.

use std::sync::Arc;

use mahalaxmi_core::config::ContextFormat;
use mahalaxmi_core::domain::DomainConfig;
use mahalaxmi_core::types::GitMergeStrategy;

use super::builder::{format_section, resolve_format};
use crate::agent::AgentSpec;
use crate::error::{ContextRouter, ContextRouterConfig};
use crate::models::report::CycleReport;

/// Configuration for building a worker prompt.
pub struct WorkerPromptConfig {
    /// Unique task identifier.
    pub task_id: String,
    /// Short descriptive title of the task.
    pub task_title: String,
    /// Detailed description of what the worker should do.
    pub task_description: String,
    /// Classified task type (e.g., "code_generation", "code_review").
    pub task_type: String,
    /// Task complexity on a 1-10 scale.
    pub complexity: u32,
    /// Files this task is expected to modify.
    pub affected_files: Vec<String>,
    /// Activated template requirements text.
    pub requirements: String,
    /// Context preamble from ContextBuilder (repo map, files, memory).
    pub context_preamble: String,
    /// Provider identifier (used to select formatting style).
    pub provider_id: String,
    /// Error context from a previous failed attempt (for retries).
    pub retry_context: Option<String>,
    /// Descriptions of what other workers in the same phase are doing.
    pub parallel_work_notes: Vec<String>,
    /// Verifiable completion criteria for this task.
    pub completion_criteria: Vec<String>,
    /// Optional agent spec (specialist persona) for this worker.
    pub agent_spec: Option<AgentSpec>,
    /// Success rate from agent registry (0.0-1.0), populated when agent_spec is present.
    pub agent_success_rate: Option<f64>,
    /// Total tasks completed by this agent, populated when agent_spec is present.
    pub agent_total_tasks: Option<u32>,
    /// Git merge strategy configured for this cycle.
    pub git_strategy: GitMergeStrategy,
    /// Optional Phase 5 context router. When Some (along with codebase_index),
    /// the router ranks project files by relevance and appends them as code blocks.
    pub context_router: Option<Arc<dyn ContextRouter>>,
    /// Signal weights and token budget for the context router.
    pub context_router_config: Option<ContextRouterConfig>,
    /// Codebase index for context routing. None falls back to the context_preamble path.
    pub codebase_index: Option<Arc<mahalaxmi_indexing::CodebaseIndex>>,
    /// Previous cycle report for historical co-occurrence scoring. None skips the signal.
    pub last_cycle_report: Option<Arc<CycleReport>>,
    /// Optional domain configuration. When `Some`, the domain's worker system
    /// role overrides the built-in hardcoded string.
    pub domain: Option<Arc<dyn DomainConfig>>,
}

/// Builds targeted, provider-formatted prompts for individual AI worker agents.
///
/// This is a deterministic builder (no AI call). For each task in the
/// execution plan, it assembles a focused prompt with only the context
/// that specific worker needs.
pub struct WorkerPromptBuilder;

impl WorkerPromptBuilder {
    /// Resolve the output format for a given provider.
    pub fn resolve_format(provider_id: &str) -> ContextFormat {
        resolve_format(ContextFormat::Auto, provider_id)
    }

    /// Build the full worker prompt from the given configuration.
    ///
    /// The returned string is ready to be passed to the AI worker's terminal
    /// session as the prompt argument.
    pub fn build(config: &WorkerPromptConfig) -> String {
        let format = resolve_format(ContextFormat::Auto, &config.provider_id);
        let mut prompt = String::with_capacity(4096);

        // 1. System role — prefer domain override, fall back to hardcoded
        let system_role_text = config
            .domain
            .as_ref()
            .map(|d| d.worker_system_role())
            .unwrap_or_else(Self::system_role);
        prompt.push_str(&system_role_text);
        prompt.push_str("\n\n");

        // 1a. Agent identity (if specialist persona assigned)
        if let Some(ref spec) = config.agent_spec {
            prompt.push_str(&Self::agent_identity(config, spec, format));
            prompt.push_str("\n\n");
        }

        // 1b. Hard constraints
        prompt.push_str(&format_section(
            "Constraints",
            "constraints",
            &Self::worker_constraints(config),
            format,
        ));
        prompt.push_str("\n\n");

        // 2. Task assignment
        prompt.push_str(&Self::task_assignment(config, format));
        prompt.push_str("\n\n");

        // 3. Requirements (from activated template)
        if !config.requirements.is_empty() {
            let req_content = Self::truncate_requirements(&config.requirements, &config.task_description);
            prompt.push_str(&format_section(
                "Project Requirements",
                "requirements",
                &req_content,
                format,
            ));
            prompt.push_str("\n\n");
        }

        // 4. Context preamble (repo map, relevant files, shared memory)
        if !config.context_preamble.is_empty() {
            prompt.push_str(&format_section(
                "Codebase Context",
                "context",
                &config.context_preamble,
                format,
            ));
            prompt.push_str("\n\n");
        }

        // 4a. Intelligently routed context (Phase 5) — ranked file contents
        if let (Some(router), Some(index)) = (&config.context_router, &config.codebase_index) {
            let router_config = config.context_router_config.clone().unwrap_or_default();
            let task = Self::worker_task_for_routing(config);
            let last_report = config.last_cycle_report.as_deref();
            let scored = router.route(&task, index, last_report, &router_config);
            if !scored.files.is_empty() {
                let mut context_block = String::new();
                for file_score in &scored.files {
                    if let Ok(contents) = std::fs::read_to_string(&file_score.path) {
                        context_block.push_str(&format!(
                            "// {path} (relevance: {score:.2})\n```\n{contents}\n```\n\n",
                            path = file_score.path,
                            score = file_score.score,
                            contents = contents,
                        ));
                    }
                }
                if !context_block.is_empty() {
                    prompt.push_str(&format_section(
                        "Intelligently Routed Context",
                        "routed_context",
                        &context_block,
                        format,
                    ));
                    prompt.push_str("\n\n");
                }
            }
        }

        // 5. Files in scope
        if !config.affected_files.is_empty() {
            prompt.push_str(&Self::files_in_scope(config, format));
            prompt.push_str("\n\n");
        }

        // 6. Parallel work awareness
        if !config.parallel_work_notes.is_empty() {
            let notes = config.parallel_work_notes.join("\n");
            prompt.push_str(&format_section(
                "Parallel Work",
                "parallel_work",
                &format!(
                    "Other workers are executing simultaneously. Be aware of potential file conflicts:\n{}",
                    notes
                ),
                format,
            ));
            prompt.push_str("\n\n");
        }

        // 7. Retry context
        if let Some(ref retry) = config.retry_context {
            prompt.push_str(&format_section(
                "Previous Attempt Feedback",
                "retry_context",
                &format!(
                    "A previous attempt at this task failed. Learn from the feedback below and adjust your approach:\n{}",
                    retry
                ),
                format,
            ));
            prompt.push_str("\n\n");
        }

        // 8. Completion criteria + signal
        prompt.push_str(&Self::completion_section(config, format));

        prompt
    }

    /// Agent identity section — establishes specialist persona and track record.
    fn agent_identity(
        config: &WorkerPromptConfig,
        spec: &AgentSpec,
        format: ContextFormat,
    ) -> String {
        let mut content = format!(
            "You are {name}, a specialist agent.\n\n{preamble}",
            name = spec.name,
            preamble = spec.system_preamble,
        );

        if let (Some(rate), Some(total)) = (config.agent_success_rate, config.agent_total_tasks) {
            content.push_str(&format!(
                "\n\nYour accuracy record: {:.0}% across {} tasks. Maintain or improve this rate.",
                rate * 100.0,
                total,
            ));
        }

        format_section("Agent Identity", "agent_identity", &content, format)
    }

    /// System role preamble — instructs the AI to act as a focused worker agent.
    fn system_role() -> String {
        "You are an AI coding agent executing a specific task in a multi-worker \
         orchestration system called Mahalaxmi. Focus ONLY on your assigned task. \
         Other workers are handling other parts of the system simultaneously."
            .to_string()
    }

    /// Hard constraints for worker behavior — constraint/violation format.
    fn worker_constraints(config: &WorkerPromptConfig) -> String {
        let files_list = config.affected_files.join(", ");

        let git_constraint = match config.git_strategy {
            GitMergeStrategy::Disabled => String::new(),
            _ => format!(
                "\n\n\
C6: You are working in an isolated git worktree branch. You MUST stage and\n    \
    commit ALL code changes BEFORE emitting the TASK COMPLETE signal.\n    \
    Run `git add -A && git commit -m \"<descriptive message>\"` when your\n    \
    implementation is finished, then emit the completion signal.\n    \
    VIOLATION: Uncommitted changes are LOST when the worktree is cleaned\n    \
    up — your entire task output will be discarded. The git strategy for\n    \
    this cycle is \"{strategy}\"; the system will handle merging/PR creation\n    \
    after your branch is committed.",
                strategy = config.git_strategy,
            ),
        };

        format!(
            "\
HARD CONSTRAINTS — YOUR OUTPUT IS REJECTED IF ANY CONSTRAINT IS VIOLATED:

C1: You MUST NOT modify files outside your assigned scope: [{files}].
    VIOLATION: Any edit to a file not in this list will be reverted and your
    task will be marked as failed.

C2: You MUST emit \"TASK COMPLETE: {task_id}\" exactly once, only after ALL
    work described in this task is finished.
    VIOLATION: Emitting the signal before finishing causes premature
    termination. Not emitting it causes timeout failure.

C3: You MUST NOT leave TODO, FIXME, HACK, or placeholder comments in code.
    VIOLATION: Any such marker in your output means the task is incomplete
    and will be sent back for rework.

C4: Every function you create MUST handle all error paths explicitly.
    VIOLATION: Unhandled errors (unwrap on fallible operations, bare
    try/catch with empty handlers, untyped exceptions) cause task rejection.

C5: You MUST NOT introduce debug output (println!, console.log, print(),
    Debug.Log) in production code paths.
    VIOLATION: Debug statements cause the verification check to fail.{git_constraint}",
            files = files_list,
            task_id = config.task_id,
            git_constraint = git_constraint,
        )
    }

    /// Task assignment section — title, description, type, complexity.
    fn task_assignment(config: &WorkerPromptConfig, format: ContextFormat) -> String {
        let content = format!(
            "Task: {}\n\
             Task Type: {}\n\
             Complexity: {}/10\n\
             Task ID: {}\n\n\
             {}",
            config.task_title,
            config.task_type,
            config.complexity,
            config.task_id,
            config.task_description,
        );
        format_section("Task Assignment", "task", &content, format)
    }

    /// Files in scope section with boundary warning.
    fn files_in_scope(config: &WorkerPromptConfig, format: ContextFormat) -> String {
        let file_list = config
            .affected_files
            .iter()
            .map(|f| format!("- {f}"))
            .collect::<Vec<_>>()
            .join("\n");

        let content = format!(
            "{file_list}\n\n\
             These are the ONLY files you may modify (see constraint C1)."
        );
        format_section("Files In Scope", "files_in_scope", &content, format)
    }

    /// Completion criteria + signal.
    fn completion_section(config: &WorkerPromptConfig, format: ContextFormat) -> String {
        let mut content = String::new();

        // BUG-004: Make acceptance criteria a mandatory self-verification gate.
        // Workers must explicitly test each criterion and report results before
        // signaling completion — vague pass-through is not accepted.
        if !config.completion_criteria.is_empty() {
            content.push_str(
                "## Self-Verification Gate (REQUIRED)\n\
                 Before outputting TASK COMPLETE you MUST verify every criterion below.\n\
                 Run the commands or checks indicated, then output a brief result summary.\n\
                 Only proceed to TASK COMPLETE if ALL criteria pass.\n\n",
            );
            for (i, criterion) in config.completion_criteria.iter().enumerate() {
                content.push_str(&format!("  [{i}] {criterion}\n"));
            }
            content.push_str(
                "\nOutput format for self-verification:\n\
                 VERIFICATION:\n\
                 [0] <criterion text> → PASS / FAIL: <one-line evidence>\n\
                 ...\n\
                 If any criterion FAILs, fix the issue and re-verify before completing.\n\n",
            );
        }

        if config.git_strategy != GitMergeStrategy::Disabled {
            content.push_str(
                "IMPORTANT: Run `git add -A && git commit -m \"<descriptive message>\"` \
                 to commit all changes BEFORE signaling completion. Uncommitted work is lost.\n\n",
            );
        }

        content.push_str(&format!(
            "When your task is complete and ALL verification criteria pass, output exactly:\n\
             TASK COMPLETE: {}",
            config.task_id
        ));

        format_section("Completion", "completion", &content, format)
    }

    /// Build a minimal `WorkerTask` from prompt config for passing to the context router.
    ///
    /// The router uses `title`, `description`, and `affected_files` for scoring; the
    /// other fields are stubbed with sensible zero-values since they are not used for
    /// relevance ranking.
    fn worker_task_for_routing(config: &WorkerPromptConfig) -> crate::models::plan::WorkerTask {
        use mahalaxmi_core::types::{TaskId, WorkerId};
        crate::models::plan::WorkerTask {
            task_id: TaskId::new(&config.task_id),
            worker_id: WorkerId::new(0),
            title: config.task_title.clone(),
            description: config.task_description.clone(),
            dependencies: Vec::new(),
            complexity: config.complexity,
            affected_files: config.affected_files.clone(),
            context_preamble: None,
            task_type: None,
            acceptance_criteria: Vec::new(),
            source: crate::models::plan::TaskSource::default(),
            priority: crate::models::plan::BatchPriority::default(),
        }
    }

    /// Truncate requirements if they exceed MAX_CHARS.
    ///
    /// If the file is within the limit it is injected verbatim.
    /// If it exceeds the limit the builder injects:
    ///   - the first 1 000 chars (preamble / header)
    ///   - a ±2 000-char window around the task description text
    ///
    /// This ensures the worker always sees the context directly relevant to
    /// its assigned task even when the requirements file is large (REQ-001).
    fn truncate_requirements(requirements: &str, task_description: &str) -> String {
        const MAX_CHARS: usize = 32_000;
        const PREAMBLE_CHARS: usize = 1_000;
        const WINDOW_CHARS: usize = 2_000;

        let total_chars = requirements.len();

        if total_chars <= MAX_CHARS {
            tracing::debug!(
                injected_chars = total_chars,
                total_chars = total_chars,
                truncated = false,
                "Worker requirements: full file injected"
            );
            return requirements.to_string();
        }

        // Try to find the task description in the requirements file so we can
        // build a context window centred on the worker's specific task.
        let preamble = &requirements[..requirements.len().min(PREAMBLE_CHARS)];
        let search_key = task_description.get(..50.min(task_description.len())).unwrap_or("");

        let windowed = if !search_key.is_empty() {
            if let Some(task_pos) = requirements.find(search_key) {
                let start = task_pos.saturating_sub(WINDOW_CHARS);
                // Clamp to a char boundary
                let start = requirements
                    .char_indices()
                    .map(|(i, _)| i)
                    .rfind(|&i| i <= start)
                    .unwrap_or(0);
                let end = (task_pos + WINDOW_CHARS).min(requirements.len());
                let end = requirements
                    .char_indices()
                    .map(|(i, _)| i)
                    .rfind(|&i| i <= end)
                    .unwrap_or(requirements.len());
                let window = &requirements[start..end];
                format!(
                    "{preamble}\n[... requirements truncated — showing context around your task ...]\n{window}"
                )
            } else {
                // Task description not found — fall back to first MAX_CHARS
                let cut = requirements
                    .char_indices()
                    .map(|(i, _)| i)
                    .rfind(|&i| i <= MAX_CHARS)
                    .unwrap_or(MAX_CHARS.min(requirements.len()));
                format!(
                    "{}\n[... truncated at {} chars — full requirements available to manager ...]",
                    &requirements[..cut],
                    total_chars
                )
            }
        } else {
            let cut = MAX_CHARS.min(requirements.len());
            format!(
                "{}\n[... truncated at {} chars — full requirements available to manager ...]",
                &requirements[..cut],
                total_chars
            )
        };

        let injected_chars = windowed.len();
        tracing::info!(
            injected_chars,
            total_chars,
            truncated = true,
            "Worker requirements: TRUNCATED — {} of {} chars injected",
            injected_chars,
            total_chars
        );

        windowed
    }
}

//! Manager prompt builder — assembles the prompt sent to AI managers.
//!
//! The prompt instructs the AI manager to analyze a codebase and produce
//! a structured JSON task proposal. The builder is provider-aware and uses
//! XML formatting for Claude-like providers and Markdown for others.
//!
//! Three system-level sections (quality mandate, progress tracking, analysis
//! rules) are injected between the system role and context sections. These
//! replace boilerplate that was previously duplicated in every Ganesha template.

use std::sync::Arc;

use mahalaxmi_core::config::ContextFormat;
use mahalaxmi_core::domain::{DecompositionStrategy, DomainConfig};

/// Configuration for building a manager prompt.
pub struct ManagerPromptConfig {
    /// Template requirements text (from the activated template).
    pub requirements: String,
    /// Repository map (file tree / symbol index).
    pub repo_map: String,
    /// Shared memory context (cross-agent discoveries from prior cycles).
    pub shared_memory: String,
    /// Provider identifier (used to select formatting style).
    pub provider_id: String,
    /// Number of workers available for task assignment.
    pub worker_count: u32,
    /// Context format override (Auto detects from provider_id).
    pub format: ContextFormat,
    /// Whether to include the quality mandate, progress tracking, and
    /// analysis rules sections. Default `true`. Set to `false` for edge
    /// cases where templates supply their own quality directives.
    pub include_quality_mandate: bool,
    /// Summary from the previous cycle's report, if this is a follow-up cycle.
    /// Injected between context sections and output format so the manager can
    /// build on prior work and avoid repeating failures.
    pub previous_cycle_report: Option<String>,
    /// Summary from the previous cycle's validation verdict, if validation was
    /// enabled. Tells the manager which requirements were unmet so it can
    /// generate targeted fix tasks.
    pub previous_validation_verdict: Option<String>,
    /// Optional domain configuration. When `Some`, the domain's prompt strings
    /// override the built-in hardcoded strings for system role and constraints.
    /// When `None`, the builder falls back to the hardcoded defaults.
    pub domain: Option<Arc<dyn DomainConfig>>,
}

impl Default for ManagerPromptConfig {
    fn default() -> Self {
        Self {
            requirements: String::new(),
            repo_map: String::new(),
            shared_memory: String::new(),
            provider_id: String::new(),
            worker_count: 3,
            format: ContextFormat::Auto,
            include_quality_mandate: true,
            previous_cycle_report: None,
            previous_validation_verdict: None,
            domain: None,
        }
    }
}

/// Builds the prompt sent to AI manager agents.
///
/// The prompt has five logical sections:
/// 1. **System role** — instructs the AI to act as a project manager
/// 2. **Quality mandate** — production-ready code standards (injected)
/// 3. **Progress tracking** — completeness requirements (injected)
/// 4. **Analysis rules** — prevent guessing/assuming (injected)
/// 5. **Context** — codebase info (repo map, shared memory, requirements)
/// 6. **Output format** — exact JSON schema the AI must produce
pub struct ManagerPromptBuilder;

impl ManagerPromptBuilder {
    /// Build the full manager prompt from the given configuration.
    ///
    /// The returned string is ready to be passed to `AiProvider::build_command()`
    /// as the prompt argument.
    pub fn build(config: &ManagerPromptConfig) -> String {
        let effective_format = resolve_format(config.format, &config.provider_id);
        let mut prompt = String::with_capacity(8192);

        // System role — prefer domain override, fall back to hardcoded
        let system_role_text = config
            .domain
            .as_ref()
            .map(|d| d.manager_system_role(config.worker_count))
            .unwrap_or_else(|| Self::system_role(config.worker_count));
        prompt.push_str(&system_role_text);
        prompt.push_str("\n\n");

        // Decomposition strategy hint — only injected when non-empty (SoftwareDevelopment and ManagerDefined produce empty hints)
        if let Some(ref domain) = config.domain {
            let hint = domain.decomposition_hint();
            if !hint.is_empty() {
                let section_label =
                    if matches!(domain.decomposition_strategy(), DecompositionStrategy::RoleBased { .. }) {
                        "Decomposition Strategy (RoleBased)"
                    } else {
                        "Decomposition Strategy"
                    };
                prompt.push_str(&format_section(
                    section_label,
                    "decomposition_strategy",
                    &hint,
                    effective_format,
                ));
                prompt.push_str("\n\n");
            }
        }

        // Injected system-level sections (replace per-template boilerplate)
        if config.include_quality_mandate {
            if let Some(ref domain) = config.domain {
                // Domain provides an ordered list of named constraint sections.
                // Each section name is converted to a snake_case XML/MD tag.
                for (name, content) in domain.constraint_sections(config.worker_count) {
                    let tag = name.to_lowercase().replace(' ', "_");
                    prompt.push_str(&format_section(&name, &tag, &content, effective_format));
                    prompt.push_str("\n\n");
                }
            } else {
                prompt.push_str(&format_section(
                    "Quality Mandate",
                    "quality_mandate",
                    &Self::quality_mandate(config.worker_count),
                    effective_format,
                ));
                prompt.push_str("\n\n");

                prompt.push_str(&format_section(
                    "Progress Tracking",
                    "progress_tracking",
                    &Self::progress_tracking(),
                    effective_format,
                ));
                prompt.push_str("\n\n");

                prompt.push_str(&format_section(
                    "Analysis Rules",
                    "analysis_rules",
                    &Self::analysis_rules(),
                    effective_format,
                ));
                prompt.push_str("\n\n");
            }
        }

        // Context sections
        if !config.repo_map.is_empty() {
            prompt.push_str(&format_section(
                "Repository Structure",
                "repo_map",
                &config.repo_map,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        if !config.shared_memory.is_empty() {
            prompt.push_str(&format_section(
                "Shared Memory",
                "shared_memory",
                &config.shared_memory,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        if !config.requirements.is_empty() {
            prompt.push_str(&format_section(
                "Project Requirements",
                "requirements",
                &config.requirements,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Previous cycle report (for follow-up cycles)
        if let Some(ref report) = config.previous_cycle_report {
            prompt.push_str(&format_section(
                "Previous Cycle Report",
                "previous_cycle",
                report,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Previous validation verdict (for continuation cycles)
        if let Some(ref verdict) = config.previous_validation_verdict {
            prompt.push_str(&format_section(
                "Previous Validation Verdict",
                "previous_validation",
                verdict,
                effective_format,
            ));
            prompt.push_str("\n\n");
        }

        // Output format specification
        prompt.push_str(&Self::output_format_spec(effective_format));

        prompt
    }

    /// The system role preamble — instructs the AI to act as a project manager.
    fn system_role(worker_count: u32) -> String {
        if worker_count == 0 {
            "You are a senior software engineering manager. \
             Your job is to analyze the project context provided below and \
             decompose the work into as many concrete, independent tasks as the \
             requirements warrant. There is no upper cap on task count — produce \
             exactly as many tasks as the work requires."
                .to_owned()
        } else {
            format!(
                "You are a senior software engineering manager. \
                 Your job is to analyze the project context provided below and \
                 decompose the work into concrete, actionable tasks that can be \
                 assigned to {worker_count} AI coding agent workers."
            )
        }
    }

    /// Quality mandate — hard constraints for production-ready code.
    ///
    /// Uses constraint-style prompting (CONSTRAINT / VIOLATION pairs) which
    /// research shows produces higher-quality LLM output than soft guidelines.
    ///
    /// When `worker_count == 0` (auto-scale mode), C3 has no upper cap — the
    /// manager produces as many tasks as the requirements warrant. When
    /// `worker_count > 0` (explicit count), C3 enforces the hard ceiling.
    fn quality_mandate(worker_count: u32) -> String {
        let c3 = if worker_count == 0 {
            "Produce one task per independent unit of work. There is NO upper cap \
             on task count in auto-scale mode — create as many tasks as the \
             requirements genuinely require. Do not artificially merge unrelated work.\n    \
             VIOLATION: Merging two independent concerns into one task to reduce count."
                .to_owned()
        } else {
            format!(
                "Task count MUST NOT exceed {worker_count}.\n    \
                 VIOLATION: Producing more tasks than available workers means some tasks\n    \
                 cannot execute. Merge the lowest-complexity tasks until count <= {worker_count}."
            )
        };

        format!(
            "\
HARD CONSTRAINTS — VIOLATION OF ANY CONSTRAINT INVALIDATES THE PLAN:

C0: The tasks array MUST contain at least one task. Never return {{\"tasks\": []}}.
    VIOLATION: An empty tasks array means you failed to analyze the requirements.
    If every listed requirement appears already complete, create a verification
    task to confirm the implementation is correct end-to-end. There is always
    something meaningful to verify, test, document, or harden.

C1: Every task MUST be independently executable by a single AI agent.
    VIOLATION: If task B requires task A's uncommitted code to compile, and
    task A is not listed as a dependency of B, the plan is invalid.

C2: No two tasks may list the same file in affected_files.
    VIOLATION: If a file path appears in both task 1 and task 3, re-partition
    so exactly one task owns each file.

C3: {c3}

C4: Every task description MUST contain concrete acceptance criteria — a
    specific, testable condition that defines \"done\".
    VIOLATION: \"Implement the feature\" without specifying observable outcomes
    is rejected. Rewrite with measurable criteria.

C5: Dependencies MUST form a DAG (directed acyclic graph). No cycles.
    VIOLATION: If task A depends on B and B depends on A (directly or
    transitively), the plan is invalid. Break the cycle.

C6: No TODO, FIXME, HACK, or placeholder code in any worker output.
    VIOLATION: Any such marker means the task is incomplete.

C7: No hardcoded secrets, credentials, or API keys in any code.
    VIOLATION: Secrets in source code are a security failure.

C8: All functions must have explicit error handling — no unwrap() on
    fallible operations, no bare try/catch with empty handlers.
    VIOLATION: Unhandled errors cause task rejection."
        )
    }

    /// Progress tracking — completeness constraints.
    fn progress_tracking() -> String {
        "\
PROGRESS TRACKING — Completeness constraints:

C9: You MUST analyze ALL requirements, not just the obvious ones.
    Implicit requirements (error handling, edge cases, configuration,
    tests) MUST be covered by at least one task.
    VIOLATION: A requirement from the template that maps to zero tasks.

C10: Every task description MUST be detailed enough for an AI agent to
     implement without follow-up questions. Include specific file paths,
     function signatures, and expected behavior.
     VIOLATION: A task that says \"implement the API\" without specifying
     endpoints, request/response shapes, or error handling."
            .to_owned()
    }

    /// Analysis rules — constraints to prevent guessing and assumptions.
    fn analysis_rules() -> String {
        "\
ANALYSIS CONSTRAINTS:

C11: You MUST NOT guess file paths. Use only paths visible in the
     repository map provided below. If a required file does not exist
     yet, state that it will be created in the task description.
     VIOLATION: Referencing \"src/utils/helper.rs\" when no such file
     exists in the repo map and the task does not say \"create this file\".

C12: You MUST NOT assume external service availability. Every task that
     interacts with an external service MUST include error handling for
     unavailability as an acceptance criterion.
     VIOLATION: A task description that says \"call the API\" without
     mentioning timeout, retry, or error handling.

C13: When requirements are ambiguous, you MUST state your interpretation
     explicitly in the task description before proceeding.
     VIOLATION: Silently choosing an interpretation without documenting it.

C14: Every affected_files entry MUST be a real file path relative to the
     project root. No wildcards, no directories, no placeholders.
     VIOLATION: \"src/*.rs\" or \"TBD\" in affected_files.

C15: You MUST NOT propose tasks that modify files outside the current
     worktree scope. Paths under services/, db/migrations/,
     database/migrations/, packages/, scripts/stripe/, or scripts/seed/
     belong to a separate product platform repository and require
     credentials that AI coding agents do not have access to.
     VIOLATION: Any task whose affected_files contains a path starting
     with services/, db/migrations/, database/migrations/, packages/,
     scripts/stripe, or scripts/seed. Such tasks will be silently
     dropped before the worker queue is built — wasting a worker slot."
            .to_owned()
    }

    /// The output format specification — tells the AI exactly what JSON to produce.
    fn output_format_spec(format: ContextFormat) -> String {
        let json_example = r#"{
  "cycle_label": "security-hardening",
  "tasks": [
    {
      "title": "Short descriptive title",
      "description": "Detailed description including acceptance criteria",
      "complexity": 5,
      "priority": 1,
      "dependencies": [],
      "affected_files": ["src/example.rs"],
      "task_type": "code_generation",
      "acceptance_criteria": ["cargo build succeeds with no warnings", "all new tests pass"],
      "agent_spec": {
        "action": "create",
        "id": "rust-backend-specialist",
        "name": "Rust Backend Specialist",
        "system_preamble": "You are an expert in Rust backend development...",
        "domain_tags": ["rust", "backend"]
      }
    },
    {
      "title": "Second task that depends on first",
      "description": "This task builds on the first task's changes. Acceptance: endpoint returns 200 with valid JSON.",
      "complexity": 3,
      "priority": 2,
      "dependencies": ["short-descriptive-title"],
      "affected_files": ["src/other.rs"],
      "task_type": "code_review",
      "acceptance_criteria": ["endpoint returns 200 with valid JSON", "no regressions in existing tests"],
      "agent_spec": {
        "action": "reuse",
        "id": "rust-backend-specialist"
      }
    }
  ]
}"#;

        let instructions = "\
OUTPUT CONSTRAINT: Respond with EXACTLY this JSON structure. No prose before or after.\n\
VIOLATION: Any text outside the JSON block causes parse failure and the entire manager cycle is wasted.\n\
\n\
Top-level fields:\n\
- \"cycle_label\": string, REQUIRED. A 1-3 word, lowercase, hyphen-separated slug that describes the\n\
  overall work theme for this cycle (e.g. \"security-hardening\", \"cost-model\", \"oauth2-auth\").\n\
  Rules: only lowercase ASCII letters, digits, and hyphens. No spaces, no uppercase, no special chars.\n\
  Used as the git branch prefix so PRs are grouped under mahalaxmi/{cycle_label}/worker-N.\n\
  Choose a slug that a developer would recognize from the PR list without reading task details.\n\
- \"tasks\": array, REQUIRED. One or more task objects (see C3 for count limit).\n\
\n\
Task field constraints:\n\
- \"title\": string, REQUIRED, max 80 chars. Used as dependency key when normalized to lowercase-hyphenated.\n\
- \"description\": string, REQUIRED. Must contain concrete acceptance criteria.\n\
- \"complexity\": integer 1-10, REQUIRED.\n\
- \"priority\": integer, REQUIRED. Lower = higher priority.\n\
- \"dependencies\": array of title strings (normalized: lowercase, hyphens), REQUIRED. Empty [] for root tasks.\n\
- \"affected_files\": array of file paths, REQUIRED. At least one file per task.\n\
- \"acceptance_criteria\": array of strings, REQUIRED. 2-5 specific, testable conditions that define task completion.\n\
- \"task_type\": string, OPTIONAL. One of: code_generation, code_review, debugging, refactoring, testing, documentation, planning, general.\n\
- \"agent_spec\": object, OPTIONAL. Assigns a specialist agent persona to the worker.\n\
  - \"action\": \"create\" (define new agent) or \"reuse\" (reference existing agent by id).\n\
  - \"id\": unique agent identifier. For \"reuse\", must match a previously created agent.\n\
  - \"name\": display name (required for \"create\").\n\
  - \"system_preamble\": persona/expertise prompt (required for \"create\").\n\
  - \"domain_tags\": array of expertise tags (optional).\n\
\n\
Output ONLY valid JSON. No markdown code fences, no explanatory text.";

        match format {
            ContextFormat::Xml | ContextFormat::Auto => {
                format!(
                    "<output_format>\n{instructions}\n\nExample:\n{json_example}\n</output_format>"
                )
            }
            ContextFormat::Markdown => {
                format!(
                    "## Output Format\n\n{instructions}\n\nExample:\n```json\n{json_example}\n```"
                )
            }
            ContextFormat::PlainText => {
                format!("=== Output Format ===\n{instructions}\n\nExample:\n{json_example}\n---")
            }
        }
    }
}

/// Resolve `ContextFormat::Auto` to a concrete format based on provider ID.
pub(crate) fn resolve_format(format: ContextFormat, provider_id: &str) -> ContextFormat {
    match format {
        ContextFormat::Auto => {
            if provider_id.to_lowercase().contains("claude") {
                ContextFormat::Xml
            } else {
                ContextFormat::Markdown
            }
        }
        other => other,
    }
}

/// Format a single context section in the given format style.
pub(crate) fn format_section(
    header: &str,
    xml_tag: &str,
    content: &str,
    format: ContextFormat,
) -> String {
    match format {
        ContextFormat::Xml | ContextFormat::Auto => {
            format!("<{xml_tag}>\n{content}\n</{xml_tag}>")
        }
        ContextFormat::Markdown => {
            format!("## {header}\n\n{content}")
        }
        ContextFormat::PlainText => {
            format!("=== {header} ===\n{content}\n---")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use mahalaxmi_core::domain::LoadedDomain;
    use tempfile::TempDir;

    fn default_config() -> ManagerPromptConfig {
        ManagerPromptConfig {
            requirements: "Build a web app".to_owned(),
            repo_map: "src/\n  main.rs".to_owned(),
            shared_memory: String::new(),
            provider_id: "claude-code".to_owned(),
            worker_count: 3,
            format: ContextFormat::Auto,
            include_quality_mandate: true,
            previous_cycle_report: None,
            previous_validation_verdict: None,
            domain: None,
        }
    }

    #[test]
    fn build_includes_system_role() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(prompt.contains("senior software engineering manager"));
        assert!(prompt.contains("3 AI coding agent workers"));
    }

    #[test]
    fn build_includes_quality_mandate() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(prompt.contains("HARD CONSTRAINTS"));
        assert!(prompt.contains("VIOLATION"));
        assert!(prompt.contains("No TODO, FIXME, HACK"));
        assert!(prompt.contains("No hardcoded secrets"));
    }

    /// Regression test for the recurring empty-proposal anomaly.
    ///
    /// Manager-0 consistently returned `{"tasks": []}` in production cycles
    /// because the model saw a well-documented CLAUDE.md "Completed" section
    /// and concluded there was nothing to do.  C0 explicitly forbids this.
    #[test]
    fn build_quality_mandate_forbids_empty_tasks_array() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        // C0 must be present and must name the violation explicitly.
        assert!(
            prompt.contains("tasks array MUST contain at least one task"),
            "C0 constraint missing from quality mandate"
        );
        assert!(
            prompt.contains(r#"{"tasks": []}"#),
            "C0 must name the exact empty-proposal output so the model recognises it"
        );
        assert!(
            prompt.contains("verification\n    task"),
            "C0 must offer the fallback of a verification task"
        );
    }

    /// C0 must appear BEFORE C1 so it is the first hard constraint the model
    /// reads.  Ordering matters for LLM attention.
    #[test]
    fn c0_appears_before_c1_in_quality_mandate() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        let c0_pos = prompt.find("C0:").expect("C0 not found");
        let c1_pos = prompt.find("C1:").expect("C1 not found");
        assert!(c0_pos < c1_pos, "C0 must precede C1 in the prompt");
    }

    /// Verify the output format spec still contains the output constraint text
    /// (separate from C0 — belt-and-suspenders).
    #[test]
    fn build_output_format_contains_violation_warning() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(
            prompt.contains("Any text outside the JSON block causes parse failure"),
            "Output format spec must still warn about parse failure"
        );
    }

    #[test]
    fn build_includes_progress_tracking() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(prompt.contains("PROGRESS TRACKING"));
        assert!(prompt.contains("analyze ALL requirements"));
    }

    #[test]
    fn build_includes_analysis_rules() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(prompt.contains("ANALYSIS CONSTRAINTS"));
        assert!(prompt.contains("MUST NOT guess file paths"));
    }

    #[test]
    fn build_without_quality_mandate() {
        let config = ManagerPromptConfig {
            include_quality_mandate: false,
            ..default_config()
        };
        let prompt = ManagerPromptBuilder::build(&config);
        assert!(!prompt.contains("HARD CONSTRAINTS"));
        assert!(!prompt.contains("PROGRESS TRACKING"));
        assert!(!prompt.contains("ANALYSIS CONSTRAINTS"));
        // But still has system role and output format
        assert!(prompt.contains("senior software engineering manager"));
        assert!(prompt.contains("output_format"));
    }

    #[test]
    fn build_includes_repo_map() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(prompt.contains("src/\n  main.rs"));
    }

    #[test]
    fn build_includes_requirements() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(prompt.contains("Build a web app"));
    }

    #[test]
    fn build_skips_empty_shared_memory() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(!prompt.contains("shared_memory"));
    }

    #[test]
    fn build_includes_shared_memory_when_present() {
        let config = ManagerPromptConfig {
            shared_memory: "Previous discovery: use async/await".to_owned(),
            ..default_config()
        };
        let prompt = ManagerPromptBuilder::build(&config);
        assert!(prompt.contains("Previous discovery: use async/await"));
    }

    #[test]
    fn build_uses_xml_for_claude() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(prompt.contains("<quality_mandate>"));
        assert!(prompt.contains("</quality_mandate>"));
        assert!(prompt.contains("<output_format>"));
    }

    #[test]
    fn build_uses_markdown_for_non_claude() {
        let config = ManagerPromptConfig {
            provider_id: "openai-foundry".to_owned(),
            ..default_config()
        };
        let prompt = ManagerPromptBuilder::build(&config);
        assert!(prompt.contains("## Quality Mandate"));
        assert!(prompt.contains("## Output Format"));
    }

    #[test]
    fn build_includes_output_format() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        assert!(prompt.contains("\"tasks\""));
        assert!(prompt.contains("\"title\""));
        assert!(prompt.contains("\"dependencies\""));
    }

    #[test]
    fn quality_mandate_section_order() {
        let prompt = ManagerPromptBuilder::build(&default_config());
        let role_pos = prompt.find("senior software engineering manager").unwrap();
        let qm_pos = prompt.find("HARD CONSTRAINTS").unwrap();
        let pt_pos = prompt.find("PROGRESS TRACKING").unwrap();
        let ar_pos = prompt.find("ANALYSIS CONSTRAINTS").unwrap();
        let req_pos = prompt.find("Build a web app").unwrap();
        let out_pos = prompt.find("output_format").unwrap();

        assert!(role_pos < qm_pos, "system role before quality mandate");
        assert!(qm_pos < pt_pos, "quality mandate before progress tracking");
        assert!(pt_pos < ar_pos, "progress tracking before analysis rules");
        assert!(ar_pos < req_pos, "analysis rules before requirements");
        assert!(req_pos < out_pos, "requirements before output format");
    }

    // Helper: write a minimal config.yaml with the given decomposition_strategy YAML block
    // and load it as a LoadedDomain.
    fn make_domain_with_strategy(strategy_yaml: &str) -> Arc<dyn DomainConfig> {
        let tmp = TempDir::new().expect("tempdir");
        let config_yaml = format!(
            r#"
id: test_decomp
description: decomposition strategy test
manager_uncapped: "uncapped"
manager_capped: "capped"
worker_system_role: "worker"
constraint_sections: []
{strategy_yaml}
"#
        );
        std::fs::write(tmp.path().join("config.yaml"), &config_yaml)
            .expect("write config.yaml");
        let domain = LoadedDomain::load(tmp.path()).expect("load domain");
        // Keep tmp alive by leaking — acceptable in tests.
        let _ = std::mem::ManuallyDrop::new(tmp);
        Arc::new(domain)
    }

    // Helper: write a config.yaml with a constraint section that contains "HARD CONSTRAINTS"
    // so we can assert ordering relative to the constraints block.
    fn make_domain_with_strategy_and_constraints(strategy_yaml: &str) -> Arc<dyn DomainConfig> {
        let tmp = TempDir::new().expect("tempdir");
        let config_yaml = format!(
            r#"
id: test_decomp_constrained
description: decomposition strategy test with constraints
manager_uncapped: "uncapped"
manager_capped: "capped"
worker_system_role: "worker"
constraint_sections:
  - name: Quality Mandate
    source: "HARD CONSTRAINTS — test suite sentinel"
{strategy_yaml}
"#
        );
        std::fs::write(tmp.path().join("config.yaml"), &config_yaml)
            .expect("write config.yaml");
        let domain = LoadedDomain::load(tmp.path()).expect("load domain");
        let _ = std::mem::ManuallyDrop::new(tmp);
        Arc::new(domain)
    }

    #[test]
    fn decomposition_hint_not_injected_for_software_development() {
        let domain = make_domain_with_strategy("decomposition_strategy: software_development");
        let config = ManagerPromptConfig {
            domain: Some(domain),
            ..default_config()
        };
        let prompt = ManagerPromptBuilder::build(&config);
        assert!(
            !prompt.contains("decomposition_strategy"),
            "SoftwareDevelopment must not inject a decomposition_strategy section"
        );
    }

    #[test]
    fn decomposition_hint_injected_for_section_based() {
        let domain = make_domain_with_strategy(
            "decomposition_strategy:\n  section_based:\n    max_section_tokens: 4000\n    overlap_tokens: 200",
        );
        let config = ManagerPromptConfig {
            domain: Some(domain),
            ..default_config()
        };
        let prompt = ManagerPromptBuilder::build(&config);
        assert!(
            prompt.contains("4000"),
            "SectionBased hint must contain max_section_tokens value"
        );
        assert!(
            prompt.contains("200"),
            "SectionBased hint must contain overlap_tokens value"
        );
    }

    #[test]
    fn decomposition_hint_injected_for_role_based() {
        let domain = make_domain_with_strategy(
            "decomposition_strategy:\n  role_based:\n    roles:\n      - security_auditor",
        );
        let config = ManagerPromptConfig {
            domain: Some(domain),
            ..default_config()
        };
        let prompt = ManagerPromptBuilder::build(&config);
        assert!(
            prompt.contains("security_auditor"),
            "RoleBased hint must contain the role name"
        );
    }

    #[test]
    fn role_based_hint_appears_before_hard_constraints() {
        let domain = make_domain_with_strategy_and_constraints(
            "decomposition_strategy:\n  role_based:\n    roles:\n      - vulnerability_analyst\n      - dependency_auditor",
        );
        let config = ManagerPromptConfig {
            domain: Some(domain),
            include_quality_mandate: true,
            ..default_config()
        };
        let prompt = ManagerPromptBuilder::build(&config);
        assert!(
            prompt.contains("vulnerability_analyst"),
            "prompt must contain the hint text"
        );
        let hint_pos = prompt
            .find("vulnerability_analyst")
            .expect("hint text not found in prompt");
        let constraints_pos = prompt
            .find("HARD CONSTRAINTS")
            .expect("HARD CONSTRAINTS not found in prompt");
        assert!(
            hint_pos < constraints_pos,
            "decomposition hint must appear before HARD CONSTRAINTS in the prompt"
        );
    }

    #[test]
    fn role_based_section_label_includes_role_based_suffix() {
        let domain = make_domain_with_strategy(
            "decomposition_strategy:\n  role_based:\n    roles:\n      - secrets_scanner",
        );
        // Use Markdown format so the section header appears verbatim in the output.
        let config = ManagerPromptConfig {
            domain: Some(domain),
            format: ContextFormat::Markdown,
            provider_id: "openai-foundry".to_owned(),
            ..default_config()
        };
        let prompt = ManagerPromptBuilder::build(&config);
        assert!(
            prompt.contains("Decomposition Strategy (RoleBased)"),
            "RoleBased section header must include '(RoleBased)' suffix; got prompt: {prompt}"
        );
    }

    #[test]
    fn section_based_section_label_does_not_include_role_based_suffix() {
        let domain = make_domain_with_strategy(
            "decomposition_strategy:\n  section_based:\n    max_section_tokens: 4000\n    overlap_tokens: 200",
        );
        let config = ManagerPromptConfig {
            domain: Some(domain),
            format: ContextFormat::Markdown,
            provider_id: "openai-foundry".to_owned(),
            ..default_config()
        };
        let prompt = ManagerPromptBuilder::build(&config);
        assert!(
            !prompt.contains("Decomposition Strategy (RoleBased)"),
            "SectionBased must not include the '(RoleBased)' suffix"
        );
    }
}

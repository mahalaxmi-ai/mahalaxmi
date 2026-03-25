//! Tests for WorkerPromptBuilder.

use mahalaxmi_orchestration::prompt::worker_builder::{WorkerPromptBuilder, WorkerPromptConfig};

fn base_config() -> WorkerPromptConfig {
    WorkerPromptConfig {
        task_id: "task-0".into(),
        task_title: "Implement user authentication".into(),
        task_description: "Add JWT-based authentication to the REST API endpoints".into(),
        task_type: "code_generation".into(),
        complexity: 7,
        affected_files: vec!["src/auth.rs".into(), "src/middleware.rs".into()],
        requirements:
            "Build a secure REST API with user authentication and role-based access control".into(),
        context_preamble: "src/\n  main.rs\n  auth.rs\n  middleware.rs".into(),
        provider_id: "claude-code".into(),
        retry_context: None,
        parallel_work_notes: Vec::new(),
        completion_criteria: Vec::new(),
        agent_spec: None,
        agent_success_rate: None,
        agent_total_tasks: None,
        git_strategy: mahalaxmi_core::types::GitMergeStrategy::DirectMerge,
        context_router: None,
        context_router_config: None,
        codebase_index: None,
        last_cycle_report: None,
        domain: None,
    }
}

#[test]
fn prompt_includes_task_title() {
    let config = base_config();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("Implement user authentication"),
        "Prompt should contain the task title"
    );
}

#[test]
fn prompt_includes_task_type() {
    let config = base_config();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("code_generation"),
        "Prompt should contain the task type"
    );
}

#[test]
fn prompt_includes_requirements_when_present() {
    let config = base_config();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("Build a secure REST API"),
        "Prompt should contain the template requirements"
    );
}

#[test]
fn prompt_omits_requirements_when_empty() {
    let mut config = base_config();
    config.requirements = String::new();
    let prompt = WorkerPromptBuilder::build(&config);
    // Should not have a requirements section header
    assert!(
        !prompt.contains("Project Requirements") && !prompt.contains("<requirements>"),
        "Prompt should not contain requirements section when empty"
    );
}

#[test]
fn prompt_includes_completion_signal() {
    let config = base_config();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("TASK COMPLETE: task-0"),
        "Prompt should contain the completion signal with task ID"
    );
}

#[test]
fn prompt_xml_format_for_claude() {
    let config = base_config();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("<task>") && prompt.contains("</task>"),
        "Claude provider should use XML formatting"
    );
    assert!(
        prompt.contains("<requirements>") && prompt.contains("</requirements>"),
        "Requirements section should use XML tags for Claude"
    );
}

#[test]
fn prompt_markdown_for_non_claude() {
    let mut config = base_config();
    config.provider_id = "grok".into();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("## Task Assignment"),
        "Non-Claude provider should use Markdown headers"
    );
    assert!(
        prompt.contains("## Project Requirements"),
        "Requirements should use Markdown header for non-Claude"
    );
}

#[test]
fn prompt_includes_affected_files() {
    let config = base_config();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("src/auth.rs"),
        "Prompt should list affected files"
    );
    assert!(
        prompt.contains("src/middleware.rs"),
        "Prompt should list all affected files"
    );
}

#[test]
fn prompt_includes_scope_boundary() {
    let config = base_config();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("ONLY files you may modify"),
        "Prompt should contain the scope boundary warning"
    );
}

#[test]
fn prompt_includes_retry_context_when_present() {
    let mut config = base_config();
    config.retry_context =
        Some("Previous attempt failed: compilation error in auth.rs line 42".into());
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("compilation error in auth.rs line 42"),
        "Prompt should contain the retry context from previous failure"
    );
    assert!(
        prompt.contains("Previous Attempt") || prompt.contains("retry_context"),
        "Prompt should have a retry context section"
    );
}

#[test]
fn prompt_omits_retry_context_when_absent() {
    let config = base_config();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        !prompt.contains("Previous Attempt") && !prompt.contains("<retry_context>"),
        "Prompt should not contain retry section when no retry context"
    );
}

#[test]
fn prompt_includes_parallel_work_notes() {
    let mut config = base_config();
    config.parallel_work_notes =
        vec!["Another worker is simultaneously handling: Setup database schema".into()];
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("Setup database schema"),
        "Prompt should contain parallel work notes"
    );
}

#[test]
fn prompt_includes_completion_criteria() {
    let mut config = base_config();
    config.completion_criteria = vec!["All tests pass".into(), "No clippy warnings".into()];
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("All tests pass"),
        "Should contain first criterion"
    );
    assert!(
        prompt.contains("No clippy warnings"),
        "Should contain second criterion"
    );
}

#[test]
fn prompt_includes_context_preamble() {
    let config = base_config();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("main.rs"),
        "Prompt should include the context preamble content"
    );
}

#[test]
fn prompt_truncates_long_requirements() {
    let mut config = base_config();
    // Create requirements longer than MAX_CHARS (32_000) so truncation fires.
    // Using 35_000 chars ensures we exceed the threshold regardless of
    // the preamble/window sizes used inside truncate_requirements.
    config.requirements = "A".repeat(35_000);
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("truncated"),
        "Long requirements should be truncated with a note"
    );
    // The full 35_000-char string should NOT appear verbatim
    assert!(
        !prompt.contains(&"A".repeat(35_000)),
        "Full long requirements should not appear in prompt"
    );
}

// ── REQ-001: task-local windowing ─────────────────────────────────────────────

#[test]
fn req001_window_contains_task_description() {
    let mut config = base_config();
    // Requirements are 40_000 chars — well above MAX_CHARS (32_000).
    // The task description appears at position ~20_000 (inside the file).
    // After windowing, the task description text must be present in the prompt.
    let task_description = "Implement the FrobnicateWidget feature";
    let padding = "B".repeat(20_000);
    let suffix = "C".repeat(20_000);
    config.requirements = format!("{padding}{task_description}{suffix}");
    config.task_description = task_description.to_string();

    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains(task_description),
        "Windowed prompt must contain the task description text"
    );
    assert!(
        prompt.contains("truncated") || prompt.contains("context around"),
        "Windowed prompt should mention truncation"
    );
}

#[test]
fn req001_short_requirements_not_truncated() {
    let mut config = base_config();
    config.requirements = "Short requirements that fit easily.".to_string();
    let prompt = WorkerPromptBuilder::build(&config);
    assert!(
        prompt.contains("Short requirements that fit easily."),
        "Short requirements must appear verbatim"
    );
    assert!(
        !prompt.contains("truncated"),
        "Short requirements should not trigger truncation notice"
    );
}

#[test]
fn resolve_format_claude_returns_xml() {
    let format = WorkerPromptBuilder::resolve_format("claude-code");
    assert_eq!(format, mahalaxmi_core::config::ContextFormat::Xml);
}

#[test]
fn resolve_format_other_returns_markdown() {
    let format = WorkerPromptBuilder::resolve_format("grok");
    assert_eq!(format, mahalaxmi_core::config::ContextFormat::Markdown);
}

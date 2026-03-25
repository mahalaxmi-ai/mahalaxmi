use mahalaxmi_core::config::{BudgetAllocations, ContextConfig, ContextFormat};
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_orchestration::context::{
    builder::format_preamble, ContextBuilder, ContextSection, ContextSectionKind, TokenBudget,
};
use mahalaxmi_orchestration::models::plan::WorkerTask;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

fn make_task(description: &str, affected: &[&str]) -> WorkerTask {
    let mut task = WorkerTask::new(
        TaskId::new("task-1"),
        WorkerId::new(1),
        "Test Task",
        description,
    );
    for file in affected {
        task = task.with_affected_file(*file);
    }
    task
}

fn default_budget() -> TokenBudget {
    TokenBudget::from_total(10_000, BudgetAllocations::default())
}

#[test]
fn build_context_with_affected_files() {
    let config = ContextConfig::default();
    let builder = ContextBuilder::new(config);
    let task = make_task("Fix the bug in src/lib.rs", &["src/lib.rs"]);
    let budget = default_budget();
    let i18n = i18n();

    let result = builder.build(&task, &budget, None, None, &i18n).unwrap();
    assert!(!result.is_empty());
    assert!(!result.files_included.is_empty());
    assert!(result.token_usage.total > 0);
}

#[test]
fn build_context_disabled() {
    let config = ContextConfig {
        enabled: false,
        ..Default::default()
    };
    let builder = ContextBuilder::new(config);
    let task = make_task("Fix src/lib.rs", &["src/lib.rs"]);
    let budget = default_budget();
    let i18n = i18n();

    let result = builder.build(&task, &budget, None, None, &i18n).unwrap();
    assert!(result.is_empty());
    assert_eq!(result.file_count(), 0);
}

#[test]
fn format_preamble_xml() {
    let sections = vec![
        ContextSection::new(ContextSectionKind::RepoMap, "repo map content"),
        ContextSection::new(ContextSectionKind::TaskDescription, "task content"),
    ];
    let preamble = format_preamble(&sections, ContextFormat::Xml, None);
    assert!(preamble.contains("<repo_map>"));
    assert!(preamble.contains("</repo_map>"));
    assert!(preamble.contains("<task_description>"));
    assert!(preamble.contains("</task_description>"));
}

#[test]
fn format_preamble_markdown() {
    let sections = vec![
        ContextSection::new(ContextSectionKind::RelevantFiles, "file content"),
        ContextSection::new(ContextSectionKind::SharedMemory, "memory content"),
    ];
    let preamble = format_preamble(&sections, ContextFormat::Markdown, None);
    assert!(preamble.contains("## Relevant Files"));
    assert!(preamble.contains("## Shared Memory"));
}

#[test]
fn format_preamble_plaintext() {
    let sections = vec![ContextSection::new(
        ContextSectionKind::TaskDescription,
        "description",
    )];
    let preamble = format_preamble(&sections, ContextFormat::PlainText, None);
    assert!(preamble.contains("=== Task Description ==="));
    assert!(preamble.contains("---"));
}

#[test]
fn format_preamble_auto_claude_uses_xml() {
    let sections = vec![ContextSection::new(ContextSectionKind::RepoMap, "content")];
    let preamble = format_preamble(&sections, ContextFormat::Auto, Some("claude-code"));
    assert!(preamble.contains("<repo_map>"));
}

#[test]
fn format_preamble_auto_other_uses_markdown() {
    let sections = vec![ContextSection::new(ContextSectionKind::RepoMap, "content")];
    let preamble = format_preamble(&sections, ContextFormat::Auto, Some("openai-foundry"));
    assert!(preamble.contains("## Repo Map"));
}

#[test]
fn format_preamble_empty_sections() {
    let sections: Vec<ContextSection> = Vec::new();
    let preamble = format_preamble(&sections, ContextFormat::Xml, None);
    assert!(preamble.is_empty());
}

// --- Cross-provider context handoff tests ---

use mahalaxmi_orchestration::context::builder::build_dependency_sections;
use mahalaxmi_orchestration::context::DependencyHandoff;

#[test]
fn dependency_handoff_xml_provenance_annotation() {
    let handoffs = vec![DependencyHandoff {
        task_title: "Implement auth module".to_string(),
        source_provider_id: "gemini-cli".to_string(),
        output: "Created auth.rs with JWT validation".to_string(),
    }];
    let sections = build_dependency_sections(&handoffs, ContextFormat::Auto, Some("claude-code"));
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].kind, ContextSectionKind::DependencyOutput);
    assert!(sections[0]
        .content
        .contains("<provenance source_provider=\"gemini-cli\""));
    assert!(sections[0].content.contains("Implement auth module"));
    assert!(sections[0]
        .content
        .contains("Created auth.rs with JWT validation"));
}

#[test]
fn dependency_handoff_markdown_provenance_annotation() {
    let handoffs = vec![DependencyHandoff {
        task_title: "Write unit tests".to_string(),
        source_provider_id: "ollama".to_string(),
        output: "Added 5 tests for auth module".to_string(),
    }];
    let sections =
        build_dependency_sections(&handoffs, ContextFormat::Auto, Some("openai-foundry"));
    assert_eq!(sections.len(), 1);
    assert!(sections[0]
        .content
        .contains("> **Provenance**: Output generated by `ollama`"));
    assert!(sections[0].content.contains("Write unit tests"));
    assert!(sections[0]
        .content
        .contains("Added 5 tests for auth module"));
}

#[test]
fn dependency_handoff_empty_returns_none() {
    let sections = build_dependency_sections(&[], ContextFormat::Xml, Some("claude-code"));
    assert!(sections.is_empty());
}

#[test]
fn dependency_handoff_formats_in_target_provider_format() {
    let handoffs = vec![DependencyHandoff {
        task_title: "Refactor config".to_string(),
        source_provider_id: "claude-code".to_string(),
        output: "Refactored config.rs".to_string(),
    }];
    // Target is Gemini (non-Claude) so should get Markdown format
    let sections = build_dependency_sections(&handoffs, ContextFormat::Auto, Some("gemini-cli"));
    assert!(sections[0].content.contains("> **Provenance**"));
    assert!(!sections[0].content.contains("<provenance"));

    // Target is Claude so should get XML format
    let sections = build_dependency_sections(&handoffs, ContextFormat::Auto, Some("claude-code"));
    assert!(sections[0].content.contains("<provenance"));
    assert!(!sections[0].content.contains("> **Provenance**"));
}

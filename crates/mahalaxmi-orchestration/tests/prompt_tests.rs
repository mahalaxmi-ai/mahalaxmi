//! Tests for Step 03: Manager Prompt Builder and Output Parser.

use mahalaxmi_core::config::ContextFormat;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::ManagerId;
use mahalaxmi_orchestration::prompt::builder::{ManagerPromptBuilder, ManagerPromptConfig};
use mahalaxmi_orchestration::prompt::parser::ManagerOutputParser;
use mahalaxmi_orchestration::prompt::worker_builder::{WorkerPromptBuilder, WorkerPromptConfig};

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ===========================================================================
// ManagerPromptBuilder tests
// ===========================================================================

#[test]
fn prompt_builder_includes_system_role() {
    let config = ManagerPromptConfig {
        worker_count: 5,
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);

    assert!(
        prompt.contains("senior software engineering manager"),
        "Prompt should include system role"
    );
    assert!(prompt.contains("5"), "Prompt should mention worker count");
}

#[test]
fn prompt_builder_includes_output_format() {
    let config = ManagerPromptConfig::default();
    let prompt = ManagerPromptBuilder::build(&config);

    assert!(
        prompt.contains("\"tasks\""),
        "Prompt should include tasks JSON key"
    );
    assert!(
        prompt.contains("\"title\""),
        "Prompt should include title field"
    );
    assert!(
        prompt.contains("\"complexity\""),
        "Prompt should include complexity field"
    );
    assert!(
        prompt.contains("\"dependencies\""),
        "Prompt should include dependencies field"
    );
}

#[test]
fn prompt_builder_xml_format_for_claude() {
    let config = ManagerPromptConfig {
        provider_id: "claude-code".into(),
        requirements: "Build a REST API".into(),
        repo_map: "src/\n  main.rs\n  lib.rs".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);

    assert!(
        prompt.contains("<repo_map>"),
        "Claude should use XML format: {prompt}"
    );
    assert!(
        prompt.contains("</repo_map>"),
        "Claude should close XML tags"
    );
    assert!(
        prompt.contains("<requirements>"),
        "Requirements should be in XML"
    );
    assert!(
        prompt.contains("<output_format>"),
        "Output format should be in XML"
    );
}

#[test]
fn prompt_builder_markdown_format_for_other_providers() {
    let config = ManagerPromptConfig {
        provider_id: "openai-foundry".into(),
        requirements: "Build a REST API".into(),
        repo_map: "src/\n  main.rs".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);

    assert!(
        prompt.contains("## Repository Structure"),
        "Non-Claude should use Markdown: {prompt}"
    );
    assert!(
        prompt.contains("## Project Requirements"),
        "Requirements should use Markdown header"
    );
    assert!(
        prompt.contains("## Output Format"),
        "Output format should use Markdown"
    );
}

#[test]
fn prompt_builder_plaintext_format() {
    let config = ManagerPromptConfig {
        requirements: "Build something".into(),
        repo_map: "file tree".into(),
        format: ContextFormat::PlainText,
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);

    assert!(
        prompt.contains("=== Repository Structure ==="),
        "PlainText should use === headers"
    );
    assert!(
        prompt.contains("---"),
        "PlainText should use --- separators"
    );
}

#[test]
fn prompt_builder_omits_empty_sections() {
    let config = ManagerPromptConfig {
        requirements: String::new(),
        repo_map: String::new(),
        shared_memory: String::new(),
        format: ContextFormat::Markdown,
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);

    assert!(
        !prompt.contains("## Repository Structure"),
        "Empty repo_map should be omitted"
    );
    assert!(
        !prompt.contains("## Shared Memory"),
        "Empty shared_memory should be omitted"
    );
    assert!(
        !prompt.contains("## Project Requirements"),
        "Empty requirements should be omitted"
    );
    // Output format should always be present
    assert!(
        prompt.contains("## Output Format"),
        "Output format should always be present"
    );
}

#[test]
fn prompt_builder_includes_shared_memory() {
    let config = ManagerPromptConfig {
        shared_memory: "Convention: use snake_case for all functions".into(),
        provider_id: "claude-code".into(),
        format: ContextFormat::Auto,
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);

    assert!(prompt.contains("<shared_memory>"));
    assert!(prompt.contains("Convention: use snake_case"));
    assert!(prompt.contains("</shared_memory>"));
}

#[test]
fn prompt_builder_explicit_xml_overrides_auto() {
    let config = ManagerPromptConfig {
        provider_id: "openai-foundry".into(),
        requirements: "Build something".into(),
        format: ContextFormat::Xml,
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);

    // Even though provider is not Claude, explicit Xml format should win
    assert!(
        prompt.contains("<requirements>"),
        "Explicit Xml should override Auto detection"
    );
}

// ===========================================================================
// ManagerOutputParser tests
// ===========================================================================

#[test]
fn parser_extracts_json_from_code_fence() {
    let i18n = i18n();
    let output = r#"
Here is my analysis of the codebase:

```json
{
  "tasks": [
    {
      "title": "Add authentication",
      "description": "Implement JWT auth middleware",
      "complexity": 6,
      "priority": 1,
      "dependencies": [],
      "affected_files": ["src/auth.rs"]
    }
  ]
}
```

I hope this helps!
"#;

    let proposal =
        ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 2000, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 1);
    assert_eq!(proposal.tasks[0].title, "Add authentication");
    assert_eq!(proposal.tasks[0].complexity, 6);
}

#[test]
fn parser_extracts_json_from_plain_code_fence() {
    let i18n = i18n();
    let output = "Some analysis text\n\n```\n{\n  \"tasks\": [\n    { \"title\": \"Task one\", \"description\": \"Do something\" }\n  ]\n}\n```\n\nDone.";

    let proposal =
        ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 1000, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 1);
    assert_eq!(proposal.tasks[0].title, "Task one");
}

#[test]
fn parser_extracts_bare_json() {
    let i18n = i18n();
    let output = r#"
After careful analysis, I propose the following tasks:

{
  "tasks": [
    {
      "title": "Setup database",
      "description": "Create schema",
      "complexity": 4,
      "priority": 1,
      "dependencies": [],
      "affected_files": ["migrations/init.sql"]
    },
    {
      "title": "Build API",
      "description": "REST endpoints",
      "complexity": 7,
      "priority": 2,
      "dependencies": ["setup-database"],
      "affected_files": ["src/api.rs"]
    }
  ]
}

Let me know if you need changes.
"#;

    let proposal =
        ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 3000, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 2);
    assert_eq!(proposal.tasks[0].title, "Setup database");
    assert_eq!(proposal.tasks[1].title, "Build API");
    assert_eq!(proposal.tasks[1].dependencies, vec!["setup-database"]);
}

#[test]
fn parser_no_json_found_returns_error() {
    let i18n = i18n();
    let output = "I analyzed the codebase and found several issues:\n1. Poor test coverage\n2. Missing docs\n\nI recommend fixing these.";

    let result = ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 1000, &i18n);
    assert!(result.is_err(), "No JSON in output should produce error");
}

#[test]
fn parser_empty_output_returns_error() {
    let i18n = i18n();
    let result = ManagerOutputParser::parse("", ManagerId::new("mgr-1"), 0, &i18n);
    assert!(result.is_err());
}

#[test]
fn parser_json_without_tasks_key_rejected() {
    let i18n = i18n();
    let output = r#"{ "result": "no tasks here", "count": 0 }"#;

    let result = ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 500, &i18n);
    assert!(
        result.is_err(),
        "JSON without 'tasks' key should be rejected"
    );
}

#[test]
fn parser_handles_nested_json_in_description() {
    let i18n = i18n();
    // The description field itself contains JSON-like text
    let output = r#"{
  "tasks": [
    {
      "title": "Update config",
      "description": "Change the config from {\"key\": \"old\"} to {\"key\": \"new\"}",
      "complexity": 2,
      "priority": 1,
      "dependencies": [],
      "affected_files": ["config.json"]
    }
  ]
}"#;

    let proposal = ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 500, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 1);
    assert_eq!(proposal.tasks[0].title, "Update config");
    assert!(proposal.tasks[0].description.contains("old"));
}

#[test]
fn parser_prefers_code_fence_over_bare_json() {
    let i18n = i18n();
    // Output contains bare JSON (wrong) then fenced JSON (correct)
    let output = r#"
{ "tasks": [{ "title": "Wrong one", "description": "nope" }] }

Actually, here's the correct analysis:

```json
{
  "tasks": [
    { "title": "Correct task", "description": "yes" }
  ]
}
```
"#;

    let proposal =
        ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 1000, &i18n).unwrap();

    // Should prefer the code-fenced version
    assert_eq!(proposal.tasks[0].title, "Correct task");
}

#[test]
fn parser_extract_json_returns_none_for_non_json() {
    assert!(ManagerOutputParser::extract_json("just plain text").is_none());
    assert!(ManagerOutputParser::extract_json("{ incomplete json").is_none());
    assert!(ManagerOutputParser::extract_json("").is_none());
}

#[test]
fn parser_handles_crlf_output_from_pty() {
    let i18n = i18n();
    // PTY output uses \r\n line endings — the \r must not corrupt JSON parsing.
    let output = "Now I have the build output.\r\n\r\n```json\r\n{\r\n  \"tasks\": [\r\n    {\r\n      \"title\": \"Fix warnings\",\r\n      \"description\": \"Add the new keyword\",\r\n      \"complexity\": 3,\r\n      \"priority\": 1,\r\n      \"dependencies\": [],\r\n      \"affected_files\": [\"src/lib.rs\"]\r\n    }\r\n  ]\r\n}\r\n```\r\n";

    let proposal =
        ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 1000, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 1);
    assert_eq!(proposal.tasks[0].title, "Fix warnings");
}

#[test]
fn parser_falls_back_to_bare_json_when_code_fence_has_inner_backticks() {
    let i18n = i18n();
    // A task description contains ``` which confuses the closing-fence search.
    // The code fence strategy returns truncated/invalid JSON; the parser must
    // fall back to bare JSON extraction to find the full valid JSON.
    let output = "Analysis complete.\n\n```json\n{\n  \"tasks\": [\n    {\n      \"title\": \"Replace deprecated API\",\n      \"description\": \"Replace ```Bitmap.GetThumbnailImage``` with the new API\",\n      \"complexity\": 4,\n      \"priority\": 1,\n      \"dependencies\": [],\n      \"affected_files\": [\"src/imaging.rs\"]\n    }\n  ]\n}\n```\n";

    let proposal =
        ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 2000, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 1);
    assert_eq!(proposal.tasks[0].title, "Replace deprecated API");
}

#[test]
fn parser_handles_multiline_output_with_ansi() {
    let i18n = i18n();
    // Simulate terminal output with leading noise
    let output = "$ claude analyze\n\
                  Analyzing codebase...\n\
                  Found 42 files, 3 modules.\n\n\
                  {\n\
                    \"tasks\": [\n\
                      {\n\
                        \"title\": \"Fix tests\",\n\
                        \"description\": \"Update failing tests\",\n\
                        \"complexity\": 3,\n\
                        \"priority\": 1,\n\
                        \"dependencies\": [],\n\
                        \"affected_files\": [\"tests/unit.rs\"]\n\
                      }\n\
                    ]\n\
                  }\n\
                  $";

    let proposal =
        ManagerOutputParser::parse(output, ManagerId::new("mgr-1"), 5000, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 1);
    assert_eq!(proposal.tasks[0].title, "Fix tests");
}

#[test]
fn parser_rejects_echoed_prompt() {
    let i18n = i18n();
    // Simulate what `echo` (or any non-AI pass-through binary) does: it
    // reproduces the entire manager prompt verbatim. The prompt contains an
    // example JSON block at the end that would otherwise be mistakenly parsed
    // as the real manager response, yielding phantom placeholder tasks.
    let echoed = concat!(
        "You are a senior software engineering manager. Your job is to analyze the project context",
        " provided below and decompose the work into concrete, actionable tasks.\n\n",
        "## Output Format\n\nOUTPUT CONSTRAINT: Respond with EXACTLY this JSON structure.\n\n",
        "Example:\n```json\n{\n  \"tasks\": [\n    {\n",
        "      \"title\": \"Short descriptive title\",\n",
        "      \"description\": \"Detailed description including acceptance criteria\",\n",
        "      \"complexity\": 5,\n",
        "      \"priority\": 1,\n",
        "      \"dependencies\": [],\n",
        "      \"affected_files\": [\"src/example.rs\"],\n",
        "      \"task_type\": \"code_generation\"\n    }\n  ]\n}\n```\n"
    );

    let result = ManagerOutputParser::parse(echoed, ManagerId::new("mgr-1"), 10, &i18n);
    assert!(
        result.is_err(),
        "Output that is an echoed prompt must be rejected, not parsed as real tasks"
    );
}

// ===========================================================================
// Constraint-style prompt tests (Phase 2A — P2)
// ===========================================================================

#[test]
fn manager_prompt_contains_hard_constraints() {
    let config = ManagerPromptConfig {
        requirements: "Build an API".into(),
        provider_id: "claude-code".into(),
        ..Default::default()
    };
    let prompt = ManagerPromptBuilder::build(&config);

    assert!(
        prompt.contains("HARD CONSTRAINTS"),
        "Manager prompt must use constraint-style format"
    );
    assert!(
        prompt.contains("VIOLATION"),
        "Manager prompt must include VIOLATION clauses"
    );
}

#[test]
fn manager_prompt_contains_numbered_constraints() {
    let config = ManagerPromptConfig::default();
    let prompt = ManagerPromptBuilder::build(&config);

    // Quality mandate constraints
    assert!(prompt.contains("C1:"), "Missing constraint C1");
    assert!(prompt.contains("C2:"), "Missing constraint C2");
    assert!(prompt.contains("C3:"), "Missing constraint C3");
    assert!(prompt.contains("C4:"), "Missing constraint C4");
    assert!(prompt.contains("C5:"), "Missing constraint C5");
    // Progress tracking constraints
    assert!(prompt.contains("C9:"), "Missing constraint C9");
    assert!(prompt.contains("C10:"), "Missing constraint C10");
    // Analysis constraints
    assert!(prompt.contains("C11:"), "Missing constraint C11");
    assert!(prompt.contains("C14:"), "Missing constraint C14");
}

#[test]
fn manager_output_spec_contains_output_constraint() {
    let config = ManagerPromptConfig::default();
    let prompt = ManagerPromptBuilder::build(&config);

    assert!(
        prompt.contains("OUTPUT CONSTRAINT"),
        "Output format must use constraint language"
    );
    assert!(
        prompt.contains("\"tasks\""),
        "Output spec must contain JSON schema"
    );
    assert!(
        prompt.contains("affected_files"),
        "Output spec must document affected_files field"
    );
}

fn default_worker_config() -> WorkerPromptConfig {
    WorkerPromptConfig {
        task_id: "task-1".into(),
        task_title: "Add login endpoint".into(),
        task_description: "Create POST /login with JWT".into(),
        task_type: "code_generation".into(),
        complexity: 5,
        affected_files: vec!["src/auth.rs".into(), "src/routes.rs".into()],
        requirements: String::new(),
        context_preamble: String::new(),
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
fn worker_prompt_contains_hard_constraints() {
    let prompt = WorkerPromptBuilder::build(&default_worker_config());

    assert!(
        prompt.contains("HARD CONSTRAINTS"),
        "Worker prompt must use constraint-style format"
    );
    assert!(
        prompt.contains("VIOLATION"),
        "Worker prompt must include VIOLATION clauses"
    );
}

#[test]
fn worker_prompt_constraints_reference_task_files() {
    let prompt = WorkerPromptBuilder::build(&default_worker_config());

    assert!(
        prompt.contains("src/auth.rs, src/routes.rs"),
        "Worker constraints must list the specific affected files"
    );
}

#[test]
fn worker_prompt_constraints_reference_task_id() {
    let prompt = WorkerPromptBuilder::build(&default_worker_config());

    assert!(
        prompt.contains("TASK COMPLETE: task-1"),
        "Worker constraints must reference the specific task ID in completion signal"
    );
}

#[test]
fn worker_prompt_constraint_count() {
    let prompt = WorkerPromptBuilder::build(&default_worker_config());

    assert!(prompt.contains("C1:"), "Missing worker constraint C1");
    assert!(prompt.contains("C2:"), "Missing worker constraint C2");
    assert!(prompt.contains("C3:"), "Missing worker constraint C3");
    assert!(prompt.contains("C4:"), "Missing worker constraint C4");
    assert!(prompt.contains("C5:"), "Missing worker constraint C5");
    assert!(
        prompt.contains("C6:"),
        "Missing worker constraint C6 (git commit)"
    );
}

#[test]
fn worker_prompt_c6_absent_when_git_disabled() {
    let mut config = default_worker_config();
    config.git_strategy = mahalaxmi_core::types::GitMergeStrategy::Disabled;
    let prompt = WorkerPromptBuilder::build(&config);

    assert!(
        !prompt.contains("C6:"),
        "C6 should not appear when git is disabled"
    );
}

#[test]
fn worker_prompt_c6_includes_strategy_name() {
    let mut config = default_worker_config();
    config.git_strategy = mahalaxmi_core::types::GitMergeStrategy::BranchOnly;
    let prompt = WorkerPromptBuilder::build(&config);

    assert!(
        prompt.contains("C6:"),
        "C6 should appear for BranchOnly strategy"
    );
    assert!(
        prompt.contains("branch_only"),
        "C6 should include the configured git strategy name"
    );
}

#[test]
fn worker_prompt_completion_section_includes_git_reminder() {
    let config = default_worker_config();
    let prompt = WorkerPromptBuilder::build(&config);

    assert!(
        prompt.contains("git add -A && git commit"),
        "Completion section should remind worker to commit"
    );
}

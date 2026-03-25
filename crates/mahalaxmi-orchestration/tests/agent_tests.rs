//! Tests for AgentSpec, AgentRegistry, and agent-spec prompt integration.

use mahalaxmi_orchestration::agent::{AgentRecord, AgentRegistry, AgentSpec};
use mahalaxmi_orchestration::models::proposal::ProposedAgentSpec;
use mahalaxmi_orchestration::prompt::worker_builder::{WorkerPromptBuilder, WorkerPromptConfig};

fn test_spec(id: &str) -> AgentSpec {
    AgentSpec {
        id: id.to_string(),
        name: format!("{} Agent", id),
        system_preamble: format!("You are an expert in {}", id),
        preferred_provider: None,
        domain_tags: vec![id.to_string()],
    }
}

fn base_worker_config() -> WorkerPromptConfig {
    WorkerPromptConfig {
        task_id: "task-0".into(),
        task_title: "Implement feature".into(),
        task_description: "Build the feature".into(),
        task_type: "code_generation".into(),
        complexity: 5,
        affected_files: vec!["src/feature.rs".into()],
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
fn agent_registry_create_and_get() {
    let mut registry = AgentRegistry::new();
    let spec = test_spec("rust");

    registry.register(spec);
    let retrieved = registry.get("rust");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "rust Agent");
}

#[test]
fn agent_registry_get_returns_none_for_missing() {
    let registry = AgentRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn agent_registry_get_or_create_idempotent() {
    let mut registry = AgentRegistry::new();
    let spec1 = test_spec("rust");
    let spec2 = AgentSpec {
        id: "rust".to_string(),
        name: "Different Name".to_string(),
        system_preamble: "Different preamble".to_string(),
        preferred_provider: None,
        domain_tags: vec!["different".to_string()],
    };

    // First call creates
    registry.get_or_create(spec1);
    // Second call with same ID returns existing (doesn't overwrite)
    let result = registry.get_or_create(spec2);

    assert_eq!(result.name, "rust Agent"); // Original name preserved
    assert_eq!(registry.list_specs().len(), 1);
}

#[test]
fn agent_registry_record_result_updates_metrics() {
    let mut registry = AgentRegistry::new();
    registry.register(test_spec("rust"));

    registry.record_result("rust", true, 0.9);
    registry.record_result("rust", true, 0.8);
    registry.record_result("rust", true, 1.0);
    registry.record_result("rust", false, 0.0);

    let record = registry.get_record("rust").unwrap();
    assert_eq!(record.total_tasks, 4);
    assert_eq!(record.successful_tasks, 3);
    assert_eq!(record.failed_tasks, 1);
    assert!((record.success_rate() - 0.75).abs() < f64::EPSILON);
}

#[test]
fn agent_record_success_rate_zero_when_no_tasks() {
    let record = AgentRecord::new("test");
    assert!((record.success_rate() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn agent_record_avg_verification_score() {
    let mut record = AgentRecord::new("test");
    record.record_success(0.9);
    record.record_success(0.7);
    // Average should be (0.9 + 0.7) / 2 = 0.8
    assert!((record.avg_verification_score() - 0.8).abs() < 0.01);
}

#[test]
fn agent_registry_best_agent_for_tags() {
    let mut registry = AgentRegistry::new();

    registry.register(AgentSpec {
        id: "rust-expert".to_string(),
        name: "Rust Expert".to_string(),
        system_preamble: "Rust specialist".to_string(),
        preferred_provider: None,
        domain_tags: vec!["rust".to_string(), "backend".to_string()],
    });

    registry.register(AgentSpec {
        id: "ts-expert".to_string(),
        name: "TypeScript Expert".to_string(),
        system_preamble: "TypeScript specialist".to_string(),
        preferred_provider: None,
        domain_tags: vec!["typescript".to_string(), "frontend".to_string()],
    });

    // Search for rust+backend → should return rust expert
    let tags = vec!["rust".to_string(), "backend".to_string()];
    let best = registry.best_agent_for_tags(&tags);
    assert!(best.is_some());
    assert_eq!(best.unwrap().id, "rust-expert");

    // Search for frontend → should return ts expert
    let tags = vec!["frontend".to_string()];
    let best = registry.best_agent_for_tags(&tags);
    assert!(best.is_some());
    assert_eq!(best.unwrap().id, "ts-expert");

    // Search for unmatched tag → should return None
    let tags = vec!["python".to_string()];
    let best = registry.best_agent_for_tags(&tags);
    assert!(best.is_none());
}

#[test]
fn proposed_agent_spec_deserializes_from_json() {
    let json = r#"{
        "action": "create",
        "id": "rust-specialist",
        "name": "Rust Specialist",
        "system_preamble": "You are an expert Rust developer.",
        "domain_tags": ["rust", "backend"]
    }"#;

    let spec: ProposedAgentSpec = serde_json::from_str(json).unwrap();
    assert_eq!(spec.action, "create");
    assert_eq!(spec.id, "rust-specialist");
    assert_eq!(spec.name.as_deref(), Some("Rust Specialist"));
    assert_eq!(spec.domain_tags, vec!["rust", "backend"]);
}

#[test]
fn proposed_agent_spec_reuse_minimal_json() {
    let json = r#"{
        "action": "reuse",
        "id": "rust-specialist"
    }"#;

    let spec: ProposedAgentSpec = serde_json::from_str(json).unwrap();
    assert_eq!(spec.action, "reuse");
    assert_eq!(spec.id, "rust-specialist");
    assert!(spec.name.is_none());
    assert!(spec.system_preamble.is_none());
    assert!(spec.domain_tags.is_empty());
}

#[test]
fn proposed_task_without_agent_spec_defaults_to_none() {
    let json = r#"{
        "tasks": [{
            "title": "Basic task",
            "description": "No agent spec",
            "complexity": 5,
            "priority": 1,
            "dependencies": [],
            "affected_files": ["src/main.rs"]
        }]
    }"#;

    use mahalaxmi_core::i18n::locale::SupportedLocale;
    use mahalaxmi_core::i18n::I18nService;
    use mahalaxmi_core::types::ManagerId;
    use mahalaxmi_orchestration::models::proposal::ManagerProposal;

    let i18n = I18nService::new(SupportedLocale::EnUs);
    let proposal = ManagerProposal::from_json(json, ManagerId::new("m1"), 100, &i18n).unwrap();

    assert!(proposal.tasks[0].agent_spec.is_none());
}

#[test]
fn proposed_task_with_agent_spec_parses() {
    let json = r#"{
        "tasks": [{
            "title": "Rust task",
            "description": "With agent spec",
            "complexity": 7,
            "priority": 1,
            "dependencies": [],
            "affected_files": ["src/lib.rs"],
            "agent_spec": {
                "action": "create",
                "id": "rust-dev",
                "name": "Rust Developer",
                "system_preamble": "Expert in Rust",
                "domain_tags": ["rust"]
            }
        }]
    }"#;

    use mahalaxmi_core::i18n::locale::SupportedLocale;
    use mahalaxmi_core::i18n::I18nService;
    use mahalaxmi_core::types::ManagerId;
    use mahalaxmi_orchestration::models::proposal::ManagerProposal;

    let i18n = I18nService::new(SupportedLocale::EnUs);
    let proposal = ManagerProposal::from_json(json, ManagerId::new("m1"), 100, &i18n).unwrap();

    let spec = proposal.tasks[0].agent_spec.as_ref().unwrap();
    assert_eq!(spec.action, "create");
    assert_eq!(spec.id, "rust-dev");
    assert_eq!(spec.name.as_deref(), Some("Rust Developer"));
}

#[test]
fn worker_prompt_includes_agent_preamble() {
    let mut config = base_worker_config();
    config.agent_spec = Some(AgentSpec {
        id: "rust-specialist".to_string(),
        name: "Rust Specialist".to_string(),
        system_preamble: "You are an expert in Rust systems programming.".to_string(),
        preferred_provider: None,
        domain_tags: vec!["rust".to_string()],
    });
    config.agent_success_rate = Some(0.85);
    config.agent_total_tasks = Some(20);

    let prompt = WorkerPromptBuilder::build(&config);

    assert!(
        prompt.contains("Rust Specialist"),
        "Prompt should contain agent name"
    );
    assert!(
        prompt.contains("expert in Rust systems programming"),
        "Prompt should contain agent preamble"
    );
    assert!(
        prompt.contains("85%"),
        "Prompt should contain accuracy rate"
    );
    assert!(
        prompt.contains("20 tasks"),
        "Prompt should contain task count"
    );
}

#[test]
fn worker_prompt_omits_agent_section_when_none() {
    let config = base_worker_config();
    let prompt = WorkerPromptBuilder::build(&config);

    assert!(
        !prompt.contains("Agent Identity") && !prompt.contains("agent_identity"),
        "Prompt should not contain agent section when agent_spec is None"
    );
}

#[test]
fn worker_prompt_omits_accuracy_when_no_record() {
    let mut config = base_worker_config();
    config.agent_spec = Some(AgentSpec {
        id: "new-agent".to_string(),
        name: "New Agent".to_string(),
        system_preamble: "Fresh agent".to_string(),
        preferred_provider: None,
        domain_tags: Vec::new(),
    });
    // No success_rate or total_tasks set

    let prompt = WorkerPromptBuilder::build(&config);

    assert!(
        prompt.contains("New Agent"),
        "Prompt should contain agent name"
    );
    assert!(
        !prompt.contains("accuracy record"),
        "Prompt should not contain accuracy when no record exists"
    );
}

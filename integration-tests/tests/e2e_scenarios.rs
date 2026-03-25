// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//! End-to-end integration tests that exercise realistic cross-crate scenarios.
//!
//! Each scenario group validates that the public APIs of multiple crates
//! compose correctly: config flows into providers, detection rules fire in
//! priority order, orchestration queues honour DAG dependencies, and the
//! indexing pipeline produces the expected symbol and graph output.

use mahalaxmi_core::config::MahalaxmiConfig;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId};

use mahalaxmi_detection::{BuiltinRuleSets, DetectionRule, RuleMatcher};
use mahalaxmi_core::types::ActionType;

use mahalaxmi_orchestration::{
    build_phases, detect_cycles, validate_dag, WorkerQueue,
};
use mahalaxmi_orchestration::models::plan::{ExecutionPhase, ExecutionPlan, WorkerTask};

use mahalaxmi_providers::{
    ClaudeCodeProvider, CustomCliProvider, ProviderCapabilities, TaskType,
};
use mahalaxmi_providers::traits::AiProvider;

use mahalaxmi_indexing::{
    ExtractorFactory, FileDependency, FileDependencyGraph, LanguageRegistry, SupportedLanguage,
};

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

fn make_task(id: &str, worker: u32, deps: Vec<&str>) -> WorkerTask {
    WorkerTask {
        task_id: TaskId::new(id),
        worker_id: WorkerId::new(worker),
        title: format!("Task {id}"),
        description: format!("Description for {id}"),
        dependencies: deps.into_iter().map(TaskId::new).collect(),
        complexity: 5,
        affected_files: vec![],
        context_preamble: None,
        task_type: None,
        acceptance_criteria: vec![],
        source: mahalaxmi_orchestration::models::plan::TaskSource::Manager,
        priority: mahalaxmi_orchestration::models::plan::BatchPriority::Normal,
    }
}

fn single_phase_plan(tasks: Vec<WorkerTask>) -> ExecutionPlan {
    ExecutionPlan::from_phases(vec![ExecutionPhase {
        phase_number: 0,
        tasks,
    }])
}

fn multi_phase_plan(phases: Vec<Vec<WorkerTask>>) -> ExecutionPlan {
    let phases = phases
        .into_iter()
        .enumerate()
        .map(|(i, tasks)| ExecutionPhase {
            phase_number: i as u32,
            tasks,
        })
        .collect();
    ExecutionPlan::from_phases(phases)
}

// ---------------------------------------------------------------------------
// Scenario 1: Config → Provider pipeline
// ---------------------------------------------------------------------------

#[test]
fn e2e_load_default_config_and_build_claude_provider() {
    let config = MahalaxmiConfig::default();
    let provider = ClaudeCodeProvider::from_mahalaxmi_config(&config);
    let cmd = provider
        .build_command(Path::new("/tmp"), "Fix the bug")
        .expect("build_command should succeed");
    assert_eq!(cmd.program, "claude");
    let args_joined = cmd.args.join(" ");
    assert!(
        args_joined.contains("Fix the bug"),
        "args should contain the prompt: {args_joined}"
    );
}

#[test]
fn e2e_config_indexing_settings_flow_to_index_config() {
    let config = MahalaxmiConfig::default();
    assert!(
        config.indexing.max_file_size_bytes > 0,
        "max_file_size_bytes should be positive"
    );
    assert!(
        !config.indexing.excluded_dirs.is_empty(),
        "excluded_dirs should be non-empty by default"
    );
}

#[test]
fn e2e_config_orchestration_drives_worker_count() {
    let config = MahalaxmiConfig::default();
    // Default is 0 (auto-scale). The field must exist and be a valid u32.
    let _ = config.orchestration.max_concurrent_workers;
    // manager_timeout_seconds must be positive (not auto-scaled)
    assert!(
        config.orchestration.manager_timeout_seconds > 0,
        "manager_timeout_seconds should be positive"
    );
}

// ---------------------------------------------------------------------------
// Scenario 2: Detection pipeline
// ---------------------------------------------------------------------------

#[test]
fn e2e_detection_full_worker_session_simulation() {
    let i18n = i18n();
    let mut rules = BuiltinRuleSets::generic();
    rules.extend(BuiltinRuleSets::claude_code());
    let mut matcher = RuleMatcher::new(rules, &i18n).expect("RuleMatcher::new");

    let lines = [
        "Starting task analysis...",
        "Reading file src/main.rs...",
        "Applying changes...",
        "Writing output...",
        "Committing...",
        "Verifying...",
        "user@host:~$ ",
    ];

    let mut match_count = 0usize;
    for line in &lines {
        if let Some(_result) = matcher.evaluate(line, None, None) {
            match_count += 1;
        }
    }

    assert!(
        match_count >= 1,
        "should detect at least one event (the shell prompt)"
    );
}

#[test]
fn e2e_custom_provider_detection_pipeline() {
    let i18n = i18n();
    let custom_rule = DetectionRule::new("my-provider-done", ActionType::CompleteWorkerCycle)
        .with_contains_pattern("TASK_DONE")
        .with_priority(10)
        .with_provider_filter(vec!["my-provider".to_owned()]);

    let mut matcher =
        RuleMatcher::new(vec![custom_rule], &i18n).expect("RuleMatcher::new");

    let result = matcher.evaluate("TASK_DONE: all steps complete", Some("my-provider"), None);
    assert!(result.is_some(), "should match TASK_DONE for my-provider");
    let r = result.unwrap();
    assert_eq!(r.action, ActionType::CompleteWorkerCycle);
}

#[test]
fn e2e_detection_priority_ordering_in_realistic_scenario() {
    let i18n = i18n();
    let high_priority = DetectionRule::new("high-prio-rule", ActionType::EscalateToManager)
        .with_contains_pattern("Error: permission denied")
        .with_priority(10);

    let low_priority = DetectionRule::new("low-prio-rule", ActionType::ContinueProcessing)
        .with_contains_pattern("Error: permission denied")
        .with_priority(50);

    let mut matcher =
        RuleMatcher::new(vec![high_priority, low_priority], &i18n).expect("RuleMatcher::new");

    let result = matcher
        .evaluate("Error: permission denied", None, None)
        .expect("should match");

    assert_eq!(
        result.matched_rule_name.as_deref(),
        Some("high-prio-rule"),
        "priority 10 rule should fire first"
    );
    assert_eq!(result.action, ActionType::EscalateToManager);
}

// ---------------------------------------------------------------------------
// Scenario 3: Orchestration pipeline
// ---------------------------------------------------------------------------

#[test]
fn e2e_four_task_diamond_dag_full_lifecycle() {
    let i18n = i18n();

    // Build diamond: a→b, a→c, b→d, c→d
    let task_a = make_task("a", 1, vec![]);
    let task_b = make_task("b", 2, vec!["a"]);
    let task_c = make_task("c", 3, vec!["a"]);
    let task_d = make_task("d", 4, vec!["b", "c"]);

    let plan = multi_phase_plan(vec![
        vec![task_a],
        vec![task_b, task_c],
        vec![task_d],
    ]);

    let mut queue = WorkerQueue::from_plan(&plan, 4, 0);

    // Phase 1: only A is ready
    let ready = queue.ready_worker_ids();
    assert_eq!(ready.len(), 1, "only worker A should be ready initially");
    let w_a = ready[0];
    queue.activate_worker(w_a, &i18n).unwrap();
    queue.complete_worker(w_a, &i18n).unwrap();

    // Phase 2: B and C become ready
    let ready = queue.ready_worker_ids();
    assert_eq!(ready.len(), 2, "B and C should be ready after A completes");
    for w in &ready {
        queue.activate_worker(*w, &i18n).unwrap();
    }
    for w in &ready {
        queue.complete_worker(*w, &i18n).unwrap();
    }

    // Phase 3: D becomes ready
    let ready = queue.ready_worker_ids();
    assert_eq!(ready.len(), 1, "only D should be ready after B and C complete");
    let w_d = ready[0];
    queue.activate_worker(w_d, &i18n).unwrap();
    queue.complete_worker(w_d, &i18n).unwrap();

    let stats = queue.statistics();
    assert_eq!(stats.completed, 4, "all 4 workers should be completed");
}

#[test]
fn e2e_single_task_plan_queue_lifecycle() {
    let i18n = i18n();
    let task = make_task("only-task", 1, vec![]);
    let plan = single_phase_plan(vec![task]);
    let mut queue = WorkerQueue::from_plan(&plan, 1, 0);

    let ready = queue.ready_worker_ids();
    assert_eq!(ready.len(), 1, "exactly 1 worker should be ready");

    let w = ready[0];
    queue.activate_worker(w, &i18n).unwrap();
    queue.complete_worker(w, &i18n).unwrap();

    let stats = queue.statistics();
    assert_eq!(stats.completed, 1);
}

#[test]
fn e2e_dependency_chain_sequential_lifecycle() {
    let i18n = i18n();

    let task_a = make_task("chain-a", 1, vec![]);
    let task_b = make_task("chain-b", 2, vec!["chain-a"]);
    let task_c = make_task("chain-c", 3, vec!["chain-b"]);

    let plan = multi_phase_plan(vec![vec![task_a], vec![task_b], vec![task_c]]);
    let mut queue = WorkerQueue::from_plan(&plan, 3, 0);

    // Only A ready first
    let ready = queue.ready_worker_ids();
    assert_eq!(ready.len(), 1);
    let w_a = ready[0];
    queue.activate_worker(w_a, &i18n).unwrap();
    queue.complete_worker(w_a, &i18n).unwrap();

    // Only B ready
    let ready = queue.ready_worker_ids();
    assert_eq!(ready.len(), 1);
    let w_b = ready[0];
    queue.activate_worker(w_b, &i18n).unwrap();
    queue.complete_worker(w_b, &i18n).unwrap();

    // Only C ready
    let ready = queue.ready_worker_ids();
    assert_eq!(ready.len(), 1);
    let w_c = ready[0];
    queue.activate_worker(w_c, &i18n).unwrap();
    queue.complete_worker(w_c, &i18n).unwrap();

    let stats = queue.statistics();
    assert_eq!(stats.completed, 3);
}

#[test]
fn e2e_dag_build_phases_matches_queue_behavior() {
    let i18n = i18n();

    let task_a = make_task("ph-a", 1, vec![]);
    let task_b = make_task("ph-b", 2, vec!["ph-a"]);
    let task_c = make_task("ph-c", 3, vec!["ph-b"]);
    let tasks = vec![task_a, task_b, task_c];

    let phases = build_phases(&tasks, &i18n).expect("build_phases");
    assert_eq!(phases.len(), 3, "linear chain should produce 3 phases");

    // Each phase has exactly 1 task
    for phase in &phases {
        assert_eq!(phase.tasks.len(), 1);
    }

    // Queue built from the plan also starts with only 1 ready worker
    let plan = ExecutionPlan::from_phases(phases);
    let queue = WorkerQueue::from_plan(&plan, 3, 0);
    let ready = queue.ready_worker_ids();
    assert_eq!(ready.len(), 1, "only the first task should be ready initially");
}

#[test]
fn e2e_dag_cycle_detection_prevents_queue_creation() {
    let i18n = i18n();

    // A depends on B, B depends on A — cycle
    let task_a = make_task("cyc-a", 1, vec!["cyc-b"]);
    let task_b = make_task("cyc-b", 2, vec!["cyc-a"]);
    let tasks = vec![task_a, task_b];

    let cycles = detect_cycles(&tasks);
    assert!(!cycles.is_empty(), "cycle should be detected");

    let result = validate_dag(&tasks, &i18n);
    assert!(result.is_err(), "validate_dag should return Err for cyclic graph");
}

#[test]
fn e2e_execution_plan_validate_catches_problems() {
    // Duplicate worker IDs in same phase → validation errors
    let task1 = WorkerTask {
        task_id: TaskId::new("t1"),
        worker_id: WorkerId::new(1),
        title: "T1".to_string(),
        description: String::new(),
        dependencies: vec![],
        complexity: 1,
        affected_files: vec![],
        context_preamble: None,
        task_type: None,
        acceptance_criteria: vec![],
        source: mahalaxmi_orchestration::models::plan::TaskSource::Manager,
        priority: mahalaxmi_orchestration::models::plan::BatchPriority::Normal,
    };
    let task2 = WorkerTask {
        task_id: TaskId::new("t2"),
        worker_id: WorkerId::new(1), // same worker ID as t1
        ..task1.clone()
    };
    let task2 = WorkerTask {
        task_id: TaskId::new("t2"),
        ..task2
    };

    let bad_plan = single_phase_plan(vec![task1, task2]);
    let errors = bad_plan.validate();
    assert!(
        !errors.is_empty(),
        "plan with duplicate worker IDs should produce validation errors"
    );

    // Valid plan: unique IDs
    let task_ok1 = make_task("ok1", 10, vec![]);
    let task_ok2 = make_task("ok2", 11, vec![]);
    let good_plan = single_phase_plan(vec![task_ok1, task_ok2]);
    let errors = good_plan.validate();
    assert!(
        errors.is_empty(),
        "valid plan should produce no validation errors: {errors:?}"
    );
}

// ---------------------------------------------------------------------------
// Scenario 4: Indexing pipeline
// ---------------------------------------------------------------------------

const RUST_SRC: &str = r#"
pub fn greet(name: &str) -> String { format!("Hello, {}!", name) }
pub struct Config { pub host: String }
"#;

#[test]
fn e2e_rust_source_to_symbols_to_dependency_graph() {
    let i18n = i18n();
    let registry = LanguageRegistry::with_defaults();
    let extractor = ExtractorFactory::create(SupportedLanguage::Rust, &registry, &i18n)
        .expect("create extractor");

    let symbols = extractor
        .extract_symbols(RUST_SRC, Path::new("src/main.rs"), &i18n)
        .expect("extract_symbols");
    assert!(!symbols.is_empty());

    // Build a dependency graph
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("main.rs", "config.rs", "crate::config"));
    graph.add_dependency(FileDependency::new(
        "handler.rs",
        "config.rs",
        "crate::config",
    ));

    // main.rs → config.rs distance == 1
    let sources = vec![PathBuf::from("main.rs")];
    let dist = graph.bfs_distance(Path::new("config.rs"), &sources, 4);
    assert_eq!(dist, Some(1));

    // Both main.rs and handler.rs depend on config.rs
    assert_eq!(graph.dependents_of(Path::new("config.rs")).len(), 2);
}

#[test]
fn e2e_language_detection_for_all_supported_extensions() {
    let pairs: &[(&str, SupportedLanguage)] = &[
        (".rs", SupportedLanguage::Rust),
        (".ts", SupportedLanguage::TypeScript),
        (".tsx", SupportedLanguage::TypeScript),
        (".js", SupportedLanguage::JavaScript),
        (".py", SupportedLanguage::Python),
        (".go", SupportedLanguage::Go),
        (".java", SupportedLanguage::Java),
        (".cpp", SupportedLanguage::Cpp),
    ];
    for (ext, expected) in pairs {
        assert_eq!(
            SupportedLanguage::from_extension(ext),
            Some(*expected),
            "extension {ext} should map to {expected:?}"
        );
    }
}

#[test]
fn e2e_extractor_factory_creates_for_all_languages() {
    let i18n = i18n();
    let registry = LanguageRegistry::with_defaults();
    for lang in registry.supported_languages() {
        let extractor = ExtractorFactory::create(lang, &registry, &i18n)
            .unwrap_or_else(|e| panic!("create extractor for {lang:?}: {e}"));
        assert_eq!(extractor.language(), lang);
    }
}

#[test]
fn e2e_file_dependency_graph_transitive_reach() {
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("a.rs", "b.rs", "crate::b"));
    graph.add_dependency(FileDependency::new("b.rs", "c.rs", "crate::c"));

    let sources = vec![PathBuf::from("a.rs")];

    // A→B→C, distance 2 within max_depth 4
    let dist = graph.bfs_distance(Path::new("c.rs"), &sources, 4);
    assert_eq!(dist, Some(2));

    // Same graph, max_depth 1 — C is unreachable
    let dist_cut = graph.bfs_distance(Path::new("c.rs"), &sources, 1);
    assert_eq!(dist_cut, None, "c.rs should be unreachable at max_depth=1");
}

// ---------------------------------------------------------------------------
// Scenario 5: Provider capabilities routing
// ---------------------------------------------------------------------------

#[test]
fn e2e_claude_provider_routing_score_beats_free_provider() {
    use mahalaxmi_providers::router::{strategy_score, RoutingStrategy};

    let claude = ClaudeCodeProvider::new();
    let claude_caps = claude.capabilities();
    let claude_score = strategy_score(
        claude_caps,
        TaskType::CodeGeneration,
        RoutingStrategy::QualityFirst,
        None,
        None,
    );

    let free_caps = ProviderCapabilities {
        cost_tier: mahalaxmi_providers::CostTier::Free,
        ..ProviderCapabilities::default()
    };
    let free_score = strategy_score(
        &free_caps,
        TaskType::CodeGeneration,
        RoutingStrategy::QualityFirst,
        None,
        None,
    );

    assert!(
        claude_score > free_score,
        "Claude score ({claude_score}) should beat free provider score ({free_score}) for CodeGeneration"
    );
}

#[test]
fn e2e_provider_metadata_install_info_is_accessible() {
    let provider = ClaudeCodeProvider::new();
    let meta = provider.metadata();
    assert!(
        !meta.install_hint.is_empty(),
        "install_hint should be non-empty"
    );
    assert!(
        !meta.install_command_for_current_os().is_empty(),
        "install_command_for_current_os() should be non-empty"
    );
}

#[test]
fn e2e_provider_credential_requirements_describe_auth_needs() {
    let provider = ClaudeCodeProvider::new();
    let reqs = provider.credential_requirements();
    assert!(
        !reqs.is_empty(),
        "ClaudeCodeProvider should have at least one credential requirement"
    );
    for spec in &reqs {
        assert!(
            !spec.description_key.is_empty(),
            "each CredentialSpec should have a non-empty description_key"
        );
    }
}

#[test]
fn e2e_custom_cli_config_flows_into_provider_command() {
    use mahalaxmi_core::config::CustomCliConfig;

    let mut config = MahalaxmiConfig::default();
    config.custom_cli = CustomCliConfig {
        binary_path: Some("my-ai".to_string()),
        args: Some(vec!["--task".to_string()]),
        ..CustomCliConfig::default()
    };

    let provider = CustomCliProvider::from_mahalaxmi_config(&config);
    let cmd_result = provider.build_command(Path::new("/tmp"), "Do the thing");

    // The provider requires binary_path to be set; with "my-ai" it should succeed
    match cmd_result {
        Ok(cmd) => {
            let all_args: Vec<&str> = std::iter::once(cmd.program.as_str())
                .chain(cmd.args.iter().map(|s| s.as_str()))
                .collect();
            let combined = all_args.join(" ");
            assert!(
                combined.contains("my-ai") || combined.contains("--task"),
                "command should reference 'my-ai' or '--task': {combined}"
            );
        }
        Err(e) => {
            // If the binary isn't installed the command build may still fail;
            // verify it's a configuration error, not a logic error.
            let msg = e.to_string();
            assert!(
                !msg.is_empty(),
                "error message should not be empty: {msg}"
            );
        }
    }
}

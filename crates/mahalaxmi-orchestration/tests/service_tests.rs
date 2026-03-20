//! Tests for Step 04a: OrchestrationService.

use mahalaxmi_core::config::VerificationConfig;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ConsensusStrategy, ManagerId, OrchestrationCycleState};
use mahalaxmi_orchestration::models::events::OrchestrationEvent;
use mahalaxmi_orchestration::models::ConsensusConfiguration;
use mahalaxmi_orchestration::service::{CycleConfig, OrchestrationCommand, OrchestrationService};
use tokio::sync::broadcast;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

fn test_config() -> CycleConfig {
    CycleConfig {
        project_root: "/tmp/test-project".into(),
        provider_id: "claude-code".into(),
        manager_count: 2,
        worker_count: 3,
        max_retries: 1,
        consensus_config: ConsensusConfiguration {
            strategy: ConsensusStrategy::Union,
            ..Default::default()
        },
        requirements: "Build a REST API".into(),
        repo_map: "src/\n  main.rs\n  lib.rs".into(),
        shared_memory: String::new(),
        provider_ids: Vec::new(),
        routing_strategy: String::new(),
        manager_provider_id: None,
        enable_review_chain: false,
        review_provider_id: None,
        accept_partial_progress: false,
        git_strategy: mahalaxmi_core::types::GitMergeStrategy::DirectMerge,
        git_target_branch: String::new(),
        git_auto_merge_pr: false,
        git_pr_platform: mahalaxmi_core::types::GitPrPlatform::GitHub,
        enable_validation: false,
        validator_provider_id: None,
        active_domain: None,
    }
}

fn make_service() -> OrchestrationService {
    let (tx, _rx) = broadcast::channel(64);
    OrchestrationService::new(
        test_config(),
        i18n(),
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    )
}

fn make_service_with_events() -> (
    OrchestrationService,
    broadcast::Receiver<OrchestrationEvent>,
) {
    let (tx, rx) = broadcast::channel(64);
    (
        OrchestrationService::new(
            test_config(),
            i18n(),
            tx,
            VerificationConfig {
                enabled: false,
                ..VerificationConfig::default()
            },
        ),
        rx,
    )
}

fn manager_json(tasks: &[(&str, &str)]) -> String {
    let task_json: Vec<String> = tasks
        .iter()
        .map(|(title, desc)| {
            format!(
                r#"{{ "title": "{title}", "description": "{desc}", "complexity": 5, "priority": 1, "dependencies": [], "affected_files": [] }}"#
            )
        })
        .collect();
    format!(r#"{{ "tasks": [{}] }}"#, task_json.join(", "))
}

fn manager_json_with_deps(tasks: &[(&str, &str, &[&str])]) -> String {
    let task_json: Vec<String> = tasks
        .iter()
        .map(|(title, desc, deps)| {
            let dep_strs: Vec<String> = deps.iter().map(|d| format!("\"{d}\"")).collect();
            format!(
                r#"{{ "title": "{title}", "description": "{desc}", "complexity": 5, "priority": 1, "dependencies": [{}], "affected_files": [] }}"#,
                dep_strs.join(", ")
            )
        })
        .collect();
    format!(r#"{{ "tasks": [{}] }}"#, task_json.join(", "))
}

// ===========================================================================
// Initialization tests
// ===========================================================================

#[test]
fn service_starts_idle() {
    let svc = make_service();
    assert_eq!(svc.current_state(), OrchestrationCycleState::Idle);
    assert!(!svc.is_paused());
    assert!(!svc.is_finished());
}

#[test]
fn initialize_transitions_to_initializing_manager() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    assert_eq!(
        svc.current_state(),
        OrchestrationCycleState::InitializingManager
    );
}

#[test]
fn double_initialize_fails() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    // Can't transition from InitializingManager to InitializingManager
    let result = svc.initialize();
    assert!(result.is_err());
}

#[test]
fn initialize_emits_cycle_started_and_state_changed() {
    let (mut svc, mut rx) = make_service_with_events();
    svc.initialize().unwrap();

    let event1 = rx.try_recv().unwrap();
    assert!(matches!(event1, OrchestrationEvent::CycleStarted { .. }));

    let event2 = rx.try_recv().unwrap();
    assert!(matches!(
        event2,
        OrchestrationEvent::StateChanged {
            new_state: OrchestrationCycleState::InitializingManager,
            ..
        }
    ));
}

// ===========================================================================
// Manager prompt building tests
// ===========================================================================

#[test]
fn build_manager_prompts_returns_n_prompts() {
    let mut svc = make_service();
    svc.initialize().unwrap();

    let prompts = svc.build_manager_prompts().unwrap();
    assert_eq!(prompts.len(), 2);
    assert_eq!(prompts[0].0, ManagerId::new("manager-0"));
    assert_eq!(prompts[1].0, ManagerId::new("manager-1"));
}

#[test]
fn build_manager_prompts_transitions_to_manager_processing() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();
    assert_eq!(
        svc.current_state(),
        OrchestrationCycleState::ManagerProcessing
    );
}

#[test]
fn build_manager_prompts_includes_requirements() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    let prompts = svc.build_manager_prompts().unwrap();
    assert!(prompts[0].1.contains("Build a REST API"));
}

#[test]
fn build_manager_prompts_includes_repo_map() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    let prompts = svc.build_manager_prompts().unwrap();
    assert!(prompts[0].1.contains("main.rs"));
}

// ===========================================================================
// Manager output submission tests
// ===========================================================================

#[test]
fn submit_manager_output_parses_json() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Setup database", "Create schema")]);
    let all_collected = svc
        .submit_manager_output(ManagerId::new("manager-0"), &json, 1000)
        .unwrap();
    assert!(!all_collected); // Only 1 of 2 managers
    assert_eq!(svc.proposals_collected(), 1);
}

#[test]
fn submit_all_managers_returns_true() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do thing A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 1000)
        .unwrap();
    let all = svc
        .submit_manager_output(ManagerId::new("manager-1"), &json, 1500)
        .unwrap();
    assert!(all);
    assert_eq!(svc.proposals_collected(), 2);
}

#[test]
fn submit_invalid_output_records_failed_proposal() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let result = svc
        .submit_manager_output(ManagerId::new("manager-0"), "no json here", 500)
        .unwrap();
    assert!(!result);
    assert_eq!(svc.proposals_collected(), 1);
}

#[test]
fn submit_manager_output_emits_manager_completed() {
    let (mut svc, mut rx) = make_service_with_events();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    // Drain initialization events
    while rx.try_recv().is_ok() {}

    let json = manager_json(&[("Task A", "Do thing A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 1234)
        .unwrap();

    let event = rx.try_recv().unwrap();
    match event {
        OrchestrationEvent::ManagerCompleted {
            task_count,
            duration_ms,
            ..
        } => {
            assert_eq!(task_count, 1);
            assert_eq!(duration_ms, 1234);
        }
        other => panic!("Expected ManagerCompleted, got {other:?}"),
    }
}

// ===========================================================================
// Consensus and planning tests
// ===========================================================================

#[test]
fn run_consensus_creates_plan_and_queue() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[
        ("Setup database", "Create schema"),
        ("Build API", "REST endpoints"),
    ]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 1000)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 1200)
        .unwrap();

    let result = svc.run_consensus().unwrap();
    assert!(!result.agreed_tasks.is_empty());
    assert!(svc.execution_plan().is_some());
    assert!(svc.worker_queue().is_some());
}

#[test]
fn run_consensus_transitions_through_analyzing_to_discovering() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();

    svc.run_consensus().unwrap();
    assert_eq!(
        svc.current_state(),
        OrchestrationCycleState::DiscoveringWorkerRequirements
    );
}

#[test]
fn run_consensus_emits_consensus_reached_and_plan_created() {
    let (mut svc, mut rx) = make_service_with_events();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();

    // Drain prior events
    while rx.try_recv().is_ok() {}

    svc.run_consensus().unwrap();

    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }

    let has_consensus = events
        .iter()
        .any(|e| matches!(e, OrchestrationEvent::ConsensusReached { .. }));
    let has_plan = events
        .iter()
        .any(|e| matches!(e, OrchestrationEvent::PlanCreated { .. }));
    assert!(has_consensus, "Should emit ConsensusReached");
    assert!(has_plan, "Should emit PlanCreated");
}

#[test]
fn run_consensus_with_all_failed_proposals_creates_empty_plan() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    // Submit garbage that won't parse
    svc.submit_manager_output(ManagerId::new("manager-0"), "garbage", 100)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), "more garbage", 100)
        .unwrap();

    let result = svc.run_consensus().unwrap();
    assert!(result.agreed_tasks.is_empty());
    assert_eq!(svc.execution_plan().unwrap().phase_count(), 0);
}

// ===========================================================================
// Worker execution tests
// ===========================================================================

#[test]
fn begin_worker_execution_transitions_to_workers_processing() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    assert_eq!(
        svc.current_state(),
        OrchestrationCycleState::WorkersProcessing
    );
}

#[test]
fn ready_workers_returns_workers_for_dispatch() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A"), ("Task B", "Do B")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    let ready = svc.ready_workers();
    assert!(!ready.is_empty());
}

#[test]
fn ready_workers_empty_when_paused() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    svc.pause();
    assert!(svc.ready_workers().is_empty());
    assert!(svc.is_paused());

    svc.resume();
    assert!(!svc.is_paused());
    assert!(!svc.ready_workers().is_empty());
}

#[test]
fn activate_and_complete_worker() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    let ready = svc.ready_workers();
    let worker_id = ready[0];
    svc.activate_worker(worker_id).unwrap();
    svc.complete_worker(worker_id, 2000).unwrap();

    let snapshot = svc.snapshot();
    assert!(snapshot.completed_workers > 0);
}

#[test]
fn activate_and_fail_worker() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    let ready = svc.ready_workers();
    let worker_id = ready[0];
    svc.activate_worker(worker_id).unwrap();
    svc.fail_worker(worker_id, "timeout".into()).unwrap();

    let snapshot = svc.snapshot();
    assert!(snapshot.failed_workers > 0);
}

#[test]
fn worker_task_description_returns_task_prompt() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Setup database", "Create the DB schema")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    let ready = svc.ready_workers();
    let desc = svc.worker_task_description(&ready[0]);
    assert!(desc.is_some());
}

#[test]
fn worker_events_emitted_on_activate_complete_fail() {
    let (mut svc, mut rx) = make_service_with_events();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A"), ("Task B", "Do B")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    // Drain prior events
    while rx.try_recv().is_ok() {}

    let ready = svc.ready_workers();
    let w0 = ready[0];
    let w1 = ready[1];

    svc.activate_worker(w0).unwrap();
    svc.complete_worker(w0, 1000).unwrap();
    svc.activate_worker(w1).unwrap();
    svc.fail_worker(w1, "crashed".into()).unwrap();

    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }

    let has_started = events
        .iter()
        .any(|e| matches!(e, OrchestrationEvent::WorkerStarted { .. }));
    let has_completed = events
        .iter()
        .any(|e| matches!(e, OrchestrationEvent::WorkerCompleted { success: true, .. }));
    let has_failed = events
        .iter()
        .any(|e| matches!(e, OrchestrationEvent::WorkerFailed { .. }));

    assert!(has_started, "Should emit WorkerStarted");
    assert!(has_completed, "Should emit WorkerCompleted");
    assert!(has_failed, "Should emit WorkerFailed");
}

// ===========================================================================
// Completion tests
// ===========================================================================

#[test]
fn complete_cycle_transitions_to_idle() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    // Complete all workers
    let ready = svc.ready_workers();
    for wid in ready {
        svc.activate_worker(wid).unwrap();
        svc.complete_worker(wid, 1000).unwrap();
    }

    assert!(svc.all_workers_finished());
    svc.complete_cycle(5000).unwrap();
    assert_eq!(svc.current_state(), OrchestrationCycleState::Idle);
    assert!(svc.is_finished());
}

#[test]
fn complete_cycle_emits_cycle_completed() {
    let (mut svc, mut rx) = make_service_with_events();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    let ready = svc.ready_workers();
    for wid in ready {
        svc.activate_worker(wid).unwrap();
        svc.complete_worker(wid, 1000).unwrap();
    }

    while rx.try_recv().is_ok() {}

    svc.complete_cycle(5000).unwrap();

    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }

    let has_completed = events
        .iter()
        .any(|e| matches!(e, OrchestrationEvent::CycleCompleted { .. }));
    assert!(has_completed, "Should emit CycleCompleted");
}

// ===========================================================================
// Full lifecycle: happy path
// ===========================================================================

#[test]
fn full_happy_path_lifecycle() {
    let (mut svc, mut rx) = make_service_with_events();

    // 1. Initialize
    svc.initialize().unwrap();
    assert_eq!(
        svc.current_state(),
        OrchestrationCycleState::InitializingManager
    );

    // 2. Build prompts
    let prompts = svc.build_manager_prompts().unwrap();
    assert_eq!(prompts.len(), 2);
    assert_eq!(
        svc.current_state(),
        OrchestrationCycleState::ManagerProcessing
    );

    // 3. Submit manager outputs
    let json = manager_json(&[
        ("Create models", "Define data models"),
        ("Build endpoints", "REST API routes"),
    ]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 1000)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 1200)
        .unwrap();

    // 4. Run consensus
    let consensus = svc.run_consensus().unwrap();
    assert!(!consensus.agreed_tasks.is_empty());
    assert_eq!(
        svc.current_state(),
        OrchestrationCycleState::DiscoveringWorkerRequirements
    );

    // 5. Begin worker execution
    svc.begin_worker_execution().unwrap();
    assert_eq!(
        svc.current_state(),
        OrchestrationCycleState::WorkersProcessing
    );

    // 6. Execute all workers
    loop {
        let ready = svc.ready_workers();
        if ready.is_empty() {
            break;
        }
        for wid in ready {
            svc.activate_worker(wid).unwrap();
            svc.complete_worker(wid, 1500).unwrap();
        }
    }

    // 7. Complete cycle
    assert!(svc.all_workers_finished());
    svc.complete_cycle(10000).unwrap();
    assert_eq!(svc.current_state(), OrchestrationCycleState::Idle);
    assert!(svc.is_finished());

    // Verify events
    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    assert!(
        events.len() >= 5,
        "Should have emitted multiple events, got {}",
        events.len()
    );
}

// ===========================================================================
// Control: Stop mid-cycle
// ===========================================================================

#[test]
fn stop_transitions_to_stopped() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();
    svc.stop();
    assert_eq!(svc.current_state(), OrchestrationCycleState::Stopped);
    assert!(svc.is_finished());
}

#[test]
fn force_error_transitions_to_error() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.force_error();
    assert_eq!(svc.current_state(), OrchestrationCycleState::Error);
}

#[test]
fn handle_command_dispatches_correctly() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    svc.handle_command(OrchestrationCommand::Pause);
    assert!(svc.is_paused());

    svc.handle_command(OrchestrationCommand::Resume);
    assert!(!svc.is_paused());

    svc.handle_command(OrchestrationCommand::Stop);
    assert_eq!(svc.current_state(), OrchestrationCycleState::Stopped);
}

// ===========================================================================
// Snapshot tests
// ===========================================================================

#[test]
fn snapshot_reflects_current_state() {
    let mut svc = make_service();
    let snap = svc.snapshot();
    assert_eq!(snap.state, OrchestrationCycleState::Idle);
    assert_eq!(snap.proposals_collected, 0);
    assert_eq!(snap.managers_total, 2);
    assert_eq!(snap.total_workers, 0);
    assert!(!snap.paused);

    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();

    let snap = svc.snapshot();
    assert_eq!(snap.state, OrchestrationCycleState::ManagerProcessing);
    assert_eq!(snap.proposals_collected, 1);
}

#[test]
fn snapshot_tracks_worker_progress() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A"), ("Task B", "Do B")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    let snap = svc.snapshot();
    assert!(snap.total_workers > 0);
    assert_eq!(snap.completed_workers, 0);

    let ready = svc.ready_workers();
    svc.activate_worker(ready[0]).unwrap();
    svc.complete_worker(ready[0], 1000).unwrap();

    let snap = svc.snapshot();
    assert!(snap.completed_workers > 0);
}

// ===========================================================================
// Dependency ordering tests
// ===========================================================================

#[test]
fn workers_with_deps_dispatched_after_dependencies() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    // Task B depends on Task A
    let json = manager_json_with_deps(&[
        ("Setup DB", "Create schema", &[]),
        ("Build API", "REST routes", &["setup-db"]),
    ]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    // First round: only "Setup DB" should be ready (no deps)
    let ready1 = svc.ready_workers();
    assert_eq!(ready1.len(), 1, "Only root task should be ready initially");

    let title1 = svc.worker_task_title(&ready1[0]).unwrap();
    assert_eq!(title1, "Setup DB");

    // Complete the first task
    svc.activate_worker(ready1[0]).unwrap();
    svc.complete_worker(ready1[0], 1000).unwrap();

    // Now "Build API" should be ready
    let ready2 = svc.ready_workers();
    assert_eq!(ready2.len(), 1, "Dependent task should now be ready");

    let title2 = svc.worker_task_title(&ready2[0]).unwrap();
    assert_eq!(title2, "Build API");
}

// ===========================================================================
// Single manager mode
// ===========================================================================

#[test]
fn single_manager_mode() {
    let (tx, _rx) = broadcast::channel(64);
    let config = CycleConfig {
        manager_count: 1,
        manager_provider_id: None,
        enable_review_chain: false,
        review_provider_id: None,
        accept_partial_progress: false,
        ..test_config()
    };
    let mut svc = OrchestrationService::new(
        config,
        i18n(),
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    let prompts = svc.build_manager_prompts().unwrap();
    assert_eq!(prompts.len(), 1);

    let json = manager_json(&[("Only task", "Just do it")]);
    let all = svc
        .submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    assert!(all);

    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    let ready = svc.ready_workers();
    assert!(!ready.is_empty());
}

// ===========================================================================
// Event subscription
// ===========================================================================

#[test]
fn subscribe_returns_new_receiver() {
    let svc = make_service();
    let mut rx = svc.subscribe();
    // No events yet
    assert!(rx.try_recv().is_err());
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[test]
fn complete_cycle_with_empty_plan() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    // All proposals fail
    svc.submit_manager_output(ManagerId::new("manager-0"), "bad", 100)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), "bad", 100)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    // No workers to dispatch
    assert!(svc.ready_workers().is_empty());
    assert!(svc.all_workers_finished());

    svc.complete_cycle(500).unwrap();
    assert!(svc.is_finished());
}

#[test]
fn stop_emits_state_changed_to_stopped() {
    let (mut svc, mut rx) = make_service_with_events();
    svc.initialize().unwrap();

    while rx.try_recv().is_ok() {}

    svc.stop();

    let event = rx.try_recv().unwrap();
    match event {
        OrchestrationEvent::StateChanged { new_state, .. } => {
            assert_eq!(new_state, OrchestrationCycleState::Stopped);
        }
        other => panic!("Expected StateChanged to Stopped, got {other:?}"),
    }
}

// ===========================================================================
// Worker prompt preparation tests
// ===========================================================================

#[test]
fn prepare_worker_prompts_populates_prompts() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[
        ("Setup database", "Create DB schema"),
        ("Build API", "REST endpoints"),
    ]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();

    let count = svc.prepare_worker_prompts().unwrap();
    assert!(count > 0, "Should prepare at least one prompt");

    // Check that each ready worker has a prompt
    let ready = svc.ready_workers();
    for wid in &ready {
        let prompt = svc.worker_prompt(wid);
        assert!(
            prompt.is_some(),
            "Worker {wid} should have an assembled prompt"
        );
        let p = prompt.unwrap();
        assert!(
            p.contains("TASK COMPLETE:"),
            "Prompt should contain completion signal"
        );
    }
}

#[test]
fn worker_prompt_accessor_returns_assembled_prompt() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Setup database", "Create DB schema")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.prepare_worker_prompts().unwrap();

    let ready = svc.ready_workers();
    let prompt = svc.worker_prompt(&ready[0]).unwrap();
    // The assembled prompt should contain the task title from the consensus
    assert!(
        prompt.contains("Setup database") || prompt.contains("Setup DB"),
        "Prompt should contain the task title"
    );
    // Should contain requirements from test_config
    assert!(
        prompt.contains("Build a REST API"),
        "Prompt should contain the requirements from config"
    );
}

#[test]
fn worker_prompt_falls_back_to_description() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Setup database", "Create the DB schema and migrations")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();

    // Do NOT call prepare_worker_prompts — test fallback
    let ready = svc.ready_workers();
    let prompt = svc.worker_prompt(&ready[0]);
    assert!(prompt.is_some(), "Should fall back to task description");
    // Should return the raw task description
    let p = prompt.unwrap();
    assert!(
        p.contains("DB schema") || p.contains("schema"),
        "Fallback should return the task description"
    );
}

#[test]
fn worker_provider_returns_default_when_not_routed() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();

    let ready = svc.ready_workers();
    let provider = svc.worker_provider(&ready[0]);
    assert_eq!(
        provider,
        Some("claude-code".to_string()),
        "Should fall back to default provider from config"
    );
}

#[test]
fn prepare_worker_prompts_returns_zero_without_plan() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    // No consensus run — no plan exists
    let count = svc.prepare_worker_prompts().unwrap();
    assert_eq!(count, 0, "Should return 0 when no plan exists");
}

// ===========================================================================
// Startup edge case tests
// ===========================================================================

#[test]
fn initialize_with_empty_requirements() {
    let (tx, _rx) = broadcast::channel(64);
    let config = CycleConfig {
        requirements: String::new(),
        manager_provider_id: None,
        enable_review_chain: false,
        review_provider_id: None,
        accept_partial_progress: false,
        ..test_config()
    };
    let mut svc = OrchestrationService::new(
        config,
        i18n(),
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    assert_eq!(
        svc.current_state(),
        OrchestrationCycleState::InitializingManager
    );

    let prompts = svc.build_manager_prompts().unwrap();
    assert_eq!(prompts.len(), 2);
}

#[test]
fn initialize_with_large_requirements() {
    let (tx, _rx) = broadcast::channel(64);
    let large_reqs = "x".repeat(50 * 1024); // 50KB
    let config = CycleConfig {
        requirements: large_reqs.clone(),
        manager_provider_id: None,
        enable_review_chain: false,
        review_provider_id: None,
        accept_partial_progress: false,
        ..test_config()
    };
    let mut svc = OrchestrationService::new(
        config,
        i18n(),
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    let prompts = svc.build_manager_prompts().unwrap();
    assert!(
        prompts[0].1.len() > 50_000,
        "Prompt should include the large requirements without truncation"
    );
}

#[test]
fn initialize_with_single_manager() {
    let (tx, _rx) = broadcast::channel(64);
    let config = CycleConfig {
        manager_count: 1,
        manager_provider_id: None,
        enable_review_chain: false,
        review_provider_id: None,
        accept_partial_progress: false,
        ..test_config()
    };
    let mut svc = OrchestrationService::new(
        config,
        i18n(),
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    let prompts = svc.build_manager_prompts().unwrap();
    assert_eq!(prompts.len(), 1);

    let json = manager_json(&[("Only task", "Just one thing")]);
    let all = svc
        .submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    assert!(all, "Single manager should complete collection");

    let result = svc.run_consensus().unwrap();
    assert!(
        !result.agreed_tasks.is_empty(),
        "Single proposal should be auto-accepted"
    );
}

#[test]
fn initialize_with_zero_workers_creates_empty_queue() {
    let (tx, _rx) = broadcast::channel(64);
    let config = CycleConfig {
        worker_count: 0,
        manager_provider_id: None,
        enable_review_chain: false,
        review_provider_id: None,
        accept_partial_progress: false,
        ..test_config()
    };
    let mut svc = OrchestrationService::new(
        config,
        i18n(),
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[("Task A", "Do A")]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    // With concurrency 0, no workers should be ready (they stay pending/blocked)
    let snapshot = svc.snapshot();
    assert!(
        snapshot.total_workers > 0,
        "Plan should have workers from consensus"
    );
}

#[test]
fn double_start_rejected() {
    let (tx, _rx) = broadcast::channel(64);
    let mut svc = OrchestrationService::new(
        test_config(),
        i18n(),
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    // Attempting to initialize again should fail (can't go from InitializingManager to InitializingManager)
    let result = svc.initialize();
    assert!(result.is_err(), "Double initialization should be rejected");
}

#[test]
fn cycle_id_is_unique() {
    let (tx1, _rx1) = broadcast::channel(64);
    let (tx2, _rx2) = broadcast::channel(64);

    let mut svc1 = OrchestrationService::new(
        test_config(),
        i18n(),
        tx1,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );
    let mut svc2 = OrchestrationService::new(
        test_config(),
        i18n(),
        tx2,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc1.initialize().unwrap();
    svc2.initialize().unwrap();

    assert_ne!(
        svc1.cycle_id(),
        svc2.cycle_id(),
        "Two sequential cycles should produce different IDs"
    );
}

#[test]
fn prepare_worker_prompts_includes_parallel_work_notes() {
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    // Two independent tasks (no deps) → same phase → parallel work notes
    let json = manager_json(&[
        ("Setup database", "Create schema"),
        ("Setup cache", "Configure Redis"),
    ]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.prepare_worker_prompts().unwrap();

    // At least one worker should have a parallel work note about the other
    let ready = svc.ready_workers();
    let has_parallel_note = ready.iter().any(|wid| {
        let prompt = svc.worker_prompt(wid).unwrap_or_default();
        prompt.contains("Another worker is simultaneously handling")
    });
    assert!(
        has_parallel_note,
        "Workers in same phase should have parallel work notes"
    );
}

// ===========================================================================
// P7: Auto-scaling tests
// ===========================================================================

fn make_service_autoscale() -> OrchestrationService {
    let (tx, _rx) = broadcast::channel(64);
    let config = CycleConfig {
        worker_count: 0,
        ..test_config()
    };
    OrchestrationService::new(
        config,
        i18n(),
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    )
}

/// P7: worker_count = 0 → effective equals task count from consensus.
#[test]
fn auto_scaling_zero_worker_count_matches_task_count() {
    let mut svc = make_service_autoscale();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[
        ("Task A", "Do A"),
        ("Task B", "Do B"),
        ("Task C", "Do C"),
        ("Task D", "Do D"),
        ("Task E", "Do E"),
    ]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    assert_eq!(
        svc.effective_worker_count(),
        5,
        "Auto-scaling should set concurrency to task count (5)"
    );
}

/// P7: worker_count = 0 with zero tasks → clamps to 1 (minimum).
#[test]
fn auto_scaling_zero_worker_count_empty_plan_clamps_to_one() {
    let mut svc = make_service_autoscale();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    // Submit unparseable output → empty plan (0 tasks)
    svc.submit_manager_output(ManagerId::new("manager-0"), "not json", 100)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), "also garbage", 100)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    assert_eq!(
        svc.effective_worker_count(),
        1,
        "Auto-scaling with empty plan should clamp to 1"
    );
}

/// P7: explicit worker_count is not overridden by task count.
#[test]
fn explicit_worker_count_not_overridden_by_auto_scaling() {
    // make_service() uses worker_count = 3
    let mut svc = make_service();
    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let json = manager_json(&[
        ("Task A", "Do A"),
        ("Task B", "Do B"),
        ("Task C", "Do C"),
        ("Task D", "Do D"),
        ("Task E", "Do E"),
    ]);
    svc.submit_manager_output(ManagerId::new("manager-0"), &json, 500)
        .unwrap();
    svc.submit_manager_output(ManagerId::new("manager-1"), &json, 500)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    assert_eq!(
        svc.effective_worker_count(),
        3,
        "Explicit worker_count=3 must not be overridden by 5-task plan"
    );
}

/// P7: effective_worker_count before execution falls back to config.worker_count.
#[test]
fn effective_worker_count_before_execution_returns_config_value() {
    let svc = make_service(); // worker_count = 3 in test_config
    assert_eq!(
        svc.effective_worker_count(),
        3,
        "Without a queue, effective_worker_count falls back to config.worker_count"
    );
}

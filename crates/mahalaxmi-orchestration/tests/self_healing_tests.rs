//! Tests for relaxed correctness / self-healing (P7).

use mahalaxmi_core::config::VerificationConfig;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ConsensusStrategy, ManagerId};
use mahalaxmi_orchestration::models::ConsensusConfiguration;
use mahalaxmi_orchestration::service::{CycleConfig, OrchestrationService};
use tokio::sync::broadcast;

fn test_config() -> CycleConfig {
    CycleConfig {
        project_root: "/tmp/test".into(),
        provider_id: "claude-code".into(),
        manager_count: 1,
        worker_count: 3,
        max_retries: 1,
        consensus_config: ConsensusConfiguration {
            strategy: ConsensusStrategy::Union,
            ..Default::default()
        },
        requirements: "Build something".into(),
        repo_map: String::new(),
        shared_memory: String::new(),
        provider_ids: vec![],
        routing_strategy: "quality_first".into(),
        manager_provider_id: None,
        enable_review_chain: false,
        review_provider_id: None,
        accept_partial_progress: true,
        git_strategy: mahalaxmi_core::types::GitMergeStrategy::DirectMerge,
        git_target_branch: String::new(),
        git_auto_merge_pr: false,
        git_pr_platform: mahalaxmi_core::types::GitPrPlatform::GitHub,
        enable_validation: false,
        validator_provider_id: None,
        active_domain: None,
    }
}

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

fn setup_service_with_tasks() -> OrchestrationService {
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let manager_output = r#"{"tasks":[
        {"title":"Task Alpha","description":"Implement auth module with JWT tokens","complexity":5,"priority":1,"dependencies":[],"affected_files":["src/auth.rs","src/jwt.rs"]},
        {"title":"Task Beta","description":"Add API endpoints for user management","complexity":3,"priority":2,"dependencies":[],"affected_files":["src/api.rs"]},
        {"title":"Task Gamma","description":"Write integration tests","complexity":4,"priority":3,"dependencies":["task-alpha"],"affected_files":["tests/auth_test.rs"]}
    ]}"#;

    svc.submit_manager_output(ManagerId::new("manager-0"), manager_output, 1000)
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();
    svc
}

// =========================================================================
// Partial progress recording
// =========================================================================

#[test]
fn record_partial_progress_on_failed_worker() {
    let mut svc = setup_service_with_tasks();

    let ready = svc.ready_workers();
    assert!(!ready.is_empty());

    let worker_id = ready[0];
    svc.activate_worker(worker_id).unwrap();
    svc.fail_worker(worker_id, "Compilation error".into())
        .unwrap();

    // Record partial progress
    svc.record_partial_progress(
        worker_id,
        vec!["src/auth.rs".into()],
        Some("Partially implemented auth module".into()),
    );

    // Verify it's stored
    let queue = svc.worker_queue().unwrap();
    let worker = queue.get_worker(&worker_id).unwrap();
    assert_eq!(worker.partial_files_modified, vec!["src/auth.rs"]);
    assert_eq!(
        worker.partial_output.as_deref(),
        Some("Partially implemented auth module")
    );
}

#[test]
fn record_partial_progress_on_nonexistent_worker_is_noop() {
    let mut svc = setup_service_with_tasks();

    // Recording on a nonexistent worker should silently do nothing
    svc.record_partial_progress(
        mahalaxmi_core::types::WorkerId::new(999),
        vec!["foo.rs".into()],
        None,
    );
    // No panic, no error
}

// =========================================================================
// Fix task generation
// =========================================================================

#[test]
fn generate_fix_tasks_from_failures() {
    let mut svc = setup_service_with_tasks();

    let ready = svc.ready_workers();
    // Activate and fail the first two, complete the third dependency-blocked one won't be ready
    for &wid in &ready {
        svc.activate_worker(wid).unwrap();
    }

    svc.fail_worker(ready[0], "Auth token validation bug".into())
        .unwrap();
    svc.complete_worker(ready[1], 5000).unwrap();

    // Record partial progress on the failed worker
    svc.record_partial_progress(ready[0], vec!["src/auth.rs".into()], None);

    let fix_tasks = svc.generate_fix_tasks();

    assert_eq!(
        fix_tasks.len(),
        1,
        "Should generate 1 fix task for 1 failure"
    );
    let fix = &fix_tasks[0];
    assert!(fix.title.starts_with("Fix:"));
    assert!(fix.description.contains("FIX TASK"));
    assert!(fix.description.contains("Previous attempt failed"));
    assert!(fix.description.contains("Auth token validation bug"));
    assert!(fix.description.contains("src/auth.rs"));
    assert!(!fix.affected_files.is_empty());
}

#[test]
fn generate_fix_tasks_empty_when_all_succeeded() {
    let mut svc = setup_service_with_tasks();

    let ready = svc.ready_workers();
    for &wid in &ready {
        svc.activate_worker(wid).unwrap();
    }
    for &wid in &ready {
        svc.complete_worker(wid, 5000).unwrap();
    }

    let fix_tasks = svc.generate_fix_tasks();
    assert!(fix_tasks.is_empty(), "No fix tasks when all succeeded");
}

#[test]
fn generate_fix_tasks_multiple_failures() {
    let mut svc = setup_service_with_tasks();

    let ready = svc.ready_workers();
    for &wid in &ready {
        svc.activate_worker(wid).unwrap();
    }

    // Fail all ready workers
    for &wid in &ready {
        svc.fail_worker(wid, format!("Error in worker {}", wid))
            .unwrap();
    }

    let fix_tasks = svc.generate_fix_tasks();
    assert_eq!(
        fix_tasks.len(),
        ready.len(),
        "Should generate one fix task per failure"
    );

    // Each fix task should have a unique task ID
    let ids: std::collections::HashSet<_> = fix_tasks.iter().map(|t| t.task_id.clone()).collect();
    assert_eq!(ids.len(), fix_tasks.len(), "Fix task IDs should be unique");
}

#[test]
fn generate_fix_tasks_includes_partial_progress_in_description() {
    let mut svc = setup_service_with_tasks();

    let ready = svc.ready_workers();
    svc.activate_worker(ready[0]).unwrap();
    svc.fail_worker(ready[0], "Timeout".into()).unwrap();
    svc.record_partial_progress(
        ready[0],
        vec!["src/auth.rs".into(), "src/jwt.rs".into()],
        Some("Half done".into()),
    );

    let fix_tasks = svc.generate_fix_tasks();
    assert_eq!(fix_tasks.len(), 1);

    let desc = &fix_tasks[0].description;
    assert!(
        desc.contains("Partial progress"),
        "Should mention partial progress"
    );
    assert!(
        desc.contains("src/auth.rs"),
        "Should list partially modified files"
    );
    assert!(
        desc.contains("src/jwt.rs"),
        "Should list all partially modified files"
    );
}

// =========================================================================
// Fix cycle setup and execution
// =========================================================================

#[test]
fn setup_fix_cycle_returns_true_with_tasks() {
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    // Generate some fix tasks manually
    let fix_tasks = vec![mahalaxmi_orchestration::models::plan::WorkerTask::new(
        mahalaxmi_core::types::TaskId::new("fix-task-0"),
        mahalaxmi_core::types::WorkerId::new(0),
        "Fix: Task Alpha",
        "Fix the auth module",
    )];

    let result = svc.setup_fix_cycle(fix_tasks);
    assert!(result, "Should return true when fix tasks provided");
    assert!(svc.is_fix_cycle(), "Should be in fix cycle mode");
}

#[test]
fn setup_fix_cycle_returns_false_with_empty_tasks() {
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    let result = svc.setup_fix_cycle(vec![]);
    assert!(!result, "Should return false for empty fix tasks");
    assert!(!svc.is_fix_cycle(), "Should not be in fix cycle mode");
}

#[test]
fn fix_cycle_builds_plan_directly() {
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    let fix_tasks = vec![
        mahalaxmi_orchestration::models::plan::WorkerTask::new(
            mahalaxmi_core::types::TaskId::new("fix-0"),
            mahalaxmi_core::types::WorkerId::new(0),
            "Fix: Auth",
            "Fix auth module",
        ),
        mahalaxmi_orchestration::models::plan::WorkerTask::new(
            mahalaxmi_core::types::TaskId::new("fix-1"),
            mahalaxmi_core::types::WorkerId::new(1),
            "Fix: Tests",
            "Fix test failures",
        ),
    ];

    svc.setup_fix_cycle(fix_tasks);
    svc.build_fix_plan().unwrap();

    let plan = svc.execution_plan().unwrap();
    assert_eq!(plan.phase_count(), 1, "Fix cycle should have 1 phase");
    assert_eq!(
        plan.all_workers().len(),
        2,
        "Fix plan should have 2 workers"
    );

    let queue = svc.worker_queue().unwrap();
    let stats = queue.statistics();
    assert_eq!(stats.total, 2);
    assert_eq!(stats.pending, 2, "Fix workers should start as pending");
}

#[test]
fn fix_cycle_skips_manager_phase() {
    let mut svc = setup_service_with_tasks();

    // Fail a worker and generate fix tasks
    let ready = svc.ready_workers();
    svc.activate_worker(ready[0]).unwrap();
    svc.fail_worker(ready[0], "Error".into()).unwrap();

    let fix_tasks = svc.generate_fix_tasks();
    assert!(!fix_tasks.is_empty());

    // Create a new service for the fix cycle
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut fix_svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    fix_svc.setup_fix_cycle(fix_tasks);
    assert!(fix_svc.is_fix_cycle());

    // Build plan directly (no manager prompts, no consensus)
    fix_svc.build_fix_plan().unwrap();

    let plan = fix_svc.execution_plan().unwrap();
    assert_eq!(plan.all_workers().len(), 1);

    // Workers should be ready for dispatch
    let queue = fix_svc.worker_queue().unwrap();
    assert_eq!(queue.statistics().pending, 1);
}

#[test]
fn fix_cycle_end_to_end() {
    let mut svc = setup_service_with_tasks();

    // Fail two workers
    let ready = svc.ready_workers();
    for &wid in &ready {
        svc.activate_worker(wid).unwrap();
    }
    svc.fail_worker(ready[0], "Compilation error".into())
        .unwrap();
    svc.complete_worker(ready[1], 5000).unwrap();

    // Generate fix tasks from the original cycle
    let fix_tasks = svc.generate_fix_tasks();
    assert_eq!(fix_tasks.len(), 1);

    // Set up a fix cycle
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut fix_svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    fix_svc.setup_fix_cycle(fix_tasks);
    fix_svc.build_fix_plan().unwrap();

    // Execute the fix cycle
    let fix_ready = fix_svc.worker_queue().unwrap().statistics();
    assert_eq!(fix_ready.pending, 1);
}

// =========================================================================
// QueuedWorker partial progress fields
// =========================================================================

#[test]
fn queued_worker_new_has_empty_partial_fields() {
    let worker = mahalaxmi_orchestration::models::queue::QueuedWorker::new(
        mahalaxmi_core::types::WorkerId::new(0),
        mahalaxmi_core::types::TaskId::new("t-0"),
        vec![],
        2,
    );

    assert!(worker.partial_files_modified.is_empty());
    assert!(worker.partial_output.is_none());
}

#[test]
fn queued_worker_serialization_skips_empty_partial_fields() {
    let worker = mahalaxmi_orchestration::models::queue::QueuedWorker::new(
        mahalaxmi_core::types::WorkerId::new(0),
        mahalaxmi_core::types::TaskId::new("t-0"),
        vec![],
        2,
    );

    let json = serde_json::to_string(&worker).unwrap();
    // Empty partial fields should be skipped due to skip_serializing_if
    assert!(!json.contains("partial_files_modified"));
    assert!(!json.contains("partial_output"));
}

#[test]
fn queued_worker_serialization_includes_nonempty_partial_fields() {
    let mut worker = mahalaxmi_orchestration::models::queue::QueuedWorker::new(
        mahalaxmi_core::types::WorkerId::new(0),
        mahalaxmi_core::types::TaskId::new("t-0"),
        vec![],
        2,
    );
    worker.partial_files_modified = vec!["src/main.rs".into()];
    worker.partial_output = Some("half done".into());

    let json = serde_json::to_string(&worker).unwrap();
    assert!(json.contains("partial_files_modified"));
    assert!(json.contains("partial_output"));
    assert!(json.contains("src/main.rs"));
    assert!(json.contains("half done"));
}

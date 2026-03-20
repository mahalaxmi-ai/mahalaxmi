use std::sync::Arc;

use mahalaxmi_core::config::VerificationConfig;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, VerificationCheck, WorkerId, WorkerStatus};
use mahalaxmi_orchestration::models::plan::{ExecutionPhase, ExecutionPlan, WorkerTask};
use mahalaxmi_orchestration::{VerificationPipeline, WorkerQueue};

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

/// Build an execution plan with the given phases.
/// Each phase is a slice of (task_id_suffix, worker_id_number, dependencies).
fn build_plan(phases: &[&[(u32, u32, &[u32])]]) -> ExecutionPlan {
    let mut exec_phases = Vec::new();
    for (phase_idx, phase) in phases.iter().enumerate() {
        let tasks: Vec<WorkerTask> = phase
            .iter()
            .map(|(task_num, worker_num, deps)| {
                let mut wt = WorkerTask::new(
                    TaskId::new(format!("task-{}", task_num)),
                    WorkerId::new(*worker_num),
                    format!("Task {}", task_num),
                    format!("Description for task {}", task_num),
                );
                for dep in *deps {
                    wt = wt.with_dependency(TaskId::new(format!("task-{}", dep)));
                }
                wt
            })
            .collect();
        exec_phases.push(ExecutionPhase {
            phase_number: phase_idx as u32,
            tasks,
        });
    }
    ExecutionPlan::from_phases(exec_phases)
}

fn verification_config_enabled() -> VerificationConfig {
    VerificationConfig {
        enabled: true,
        checks: vec![VerificationCheck::Tests],
        max_retries: 2,
        fail_fast: true,
        auto_fix: true,
        timeout_seconds: 300,
        parse_verification_failures: true,
        require_verification_output: false,
        run_build_check_per_worker: false,
    }
}

fn test_pipeline(config: &VerificationConfig) -> VerificationPipeline {
    VerificationPipeline::new(config.clone(), Arc::new(i18n()))
}

const CLEAN_OUTPUT: &str = "Build succeeded\nNo issues found";

const FAILING_CARGO_TEST_OUTPUT: &str = "\
running 2 tests
test tests::test_one ... ok
test tests::test_two ... FAILED

failures:

---- tests::test_two stdout ----
thread 'tests::test_two' panicked at 'assertion failed', src/lib.rs:42:5

failures:
    tests::test_two

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out";

// ============================================================================
// 1. from_plan_with_verification
// ============================================================================

#[test]
fn queue_with_verification_stores_config() {
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config);

    assert!(queue.verification_config().enabled);
    assert_eq!(queue.verification_config().max_retries, 2);
    assert!(queue.verification_config().fail_fast);
    assert_eq!(
        queue.verification_config().checks,
        vec![VerificationCheck::Tests]
    );
}

#[test]
fn queue_default_verification_disabled() {
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let queue = WorkerQueue::from_plan(&plan, 5, 3);

    assert!(!queue.verification_config().enabled);
}

// ============================================================================
// 2. complete_worker with verification enabled
// ============================================================================

#[test]
fn complete_worker_transitions_to_verifying() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w.status, WorkerStatus::Verifying);
    assert!(w.finished_at.is_none());
}

#[test]
fn complete_worker_without_verification_completes() {
    let i18n = i18n();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w.status, WorkerStatus::Completed);
    assert!(w.finished_at.is_some());
}

#[test]
fn verifying_workers_occupy_slots() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[]), (2, 2, &[]), (3, 3, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 3, 3, config);

    // Activate and complete workers 1 and 2 — they should be Verifying
    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.activate_worker(WorkerId::new(2), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(2), &i18n).unwrap();

    let w1 = queue.get_worker(&WorkerId::new(1)).unwrap();
    let w2 = queue.get_worker(&WorkerId::new(2)).unwrap();
    assert_eq!(w1.status, WorkerStatus::Verifying);
    assert_eq!(w2.status, WorkerStatus::Verifying);

    // Verifying workers occupy slots: 2 occupied out of 3
    assert_eq!(queue.available_slots(), 1);

    let stats = queue.statistics();
    assert_eq!(stats.verifying, 2);
}

// ============================================================================
// 3. verify_worker
// ============================================================================

#[test]
fn verify_worker_passes_on_clean_output() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])], &[(2, 2, &[1])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config.clone());
    let pipeline = test_pipeline(&config);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();
    assert_eq!(
        queue.get_worker(&WorkerId::new(1)).unwrap().status,
        WorkerStatus::Verifying
    );

    let result = queue
        .verify_worker(WorkerId::new(1), CLEAN_OUTPUT, &pipeline, &i18n)
        .unwrap();

    assert!(result.status.is_success());

    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w.status, WorkerStatus::Completed);
    assert!(w.finished_at.is_some());

    // Dependent worker 2 should be unblocked
    let w2 = queue.get_worker(&WorkerId::new(2)).unwrap();
    assert_eq!(w2.status, WorkerStatus::Pending);
}

#[test]
fn verify_worker_fails_on_failing_tests() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config.clone());
    let pipeline = test_pipeline(&config);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    let result = queue
        .verify_worker(
            WorkerId::new(1),
            FAILING_CARGO_TEST_OUTPUT,
            &pipeline,
            &i18n,
        )
        .unwrap();

    assert!(!result.status.is_success());

    // Retries remain (max_retries=2, verification_retries was 0, now 1) — should be Active for retry
    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w.status, WorkerStatus::Active);
}

#[test]
fn verify_worker_fails_no_retries_left() {
    let i18n = i18n();
    let config = VerificationConfig {
        enabled: true,
        checks: vec![VerificationCheck::Tests],
        max_retries: 0,
        fail_fast: true,
        auto_fix: true,
        timeout_seconds: 300,
        parse_verification_failures: true,
        require_verification_output: false,
        run_build_check_per_worker: false,
    };
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config.clone());
    let pipeline = test_pipeline(&config);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    let result = queue
        .verify_worker(
            WorkerId::new(1),
            FAILING_CARGO_TEST_OUTPUT,
            &pipeline,
            &i18n,
        )
        .unwrap();

    assert!(!result.status.is_success());

    // max_retries=0, verification_retries=0, so 0 >= 0 => Failed
    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w.status, WorkerStatus::Failed);
    assert!(w.finished_at.is_some());
}

#[test]
fn verify_worker_increments_retry_counter() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config.clone());
    let pipeline = test_pipeline(&config);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    let before = queue
        .get_worker(&WorkerId::new(1))
        .unwrap()
        .verification_retries;

    queue
        .verify_worker(
            WorkerId::new(1),
            FAILING_CARGO_TEST_OUTPUT,
            &pipeline,
            &i18n,
        )
        .unwrap();

    let after = queue
        .get_worker(&WorkerId::new(1))
        .unwrap()
        .verification_retries;
    assert_eq!(after, before + 1);
}

#[test]
fn verify_worker_stores_last_verification() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config.clone());
    let pipeline = test_pipeline(&config);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    assert!(queue
        .get_worker(&WorkerId::new(1))
        .unwrap()
        .last_verification
        .is_none());

    queue
        .verify_worker(WorkerId::new(1), CLEAN_OUTPUT, &pipeline, &i18n)
        .unwrap();

    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert!(w.last_verification.is_some());
    assert!(w.last_verification.as_ref().unwrap().status.is_success());
}

// ============================================================================
// 4. retry_worker_with_context
// ============================================================================

#[test]
fn retry_worker_with_context_reactivates() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    // Worker is now Verifying; retry with context
    let context = "Fix test_two: assertion at src/lib.rs:42".to_owned();
    queue
        .retry_worker_with_context(WorkerId::new(1), context.clone(), &i18n)
        .unwrap();

    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w.status, WorkerStatus::Active);
    assert_eq!(w.retry_context.as_deref(), Some(context.as_str()));
    assert_eq!(w.verification_retries, 1);
}

#[test]
fn retry_worker_with_context_exceeds_max() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    // Exhaust retries (max_retries=2)
    queue
        .retry_worker_with_context(WorkerId::new(1), "retry 1".to_owned(), &i18n)
        .unwrap();
    queue
        .retry_worker_with_context(WorkerId::new(1), "retry 2".to_owned(), &i18n)
        .unwrap();

    // Now verification_retries == 2 == max_retries, next retry should error
    let result = queue.retry_worker_with_context(WorkerId::new(1), "retry 3".to_owned(), &i18n);
    assert!(result.is_err());
}

#[test]
fn retry_worker_with_context_clears_finished_at() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();

    // Without verification the worker would get finished_at set. Manually set
    // finished_at to simulate it being non-None before retry.
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    // Retry should clear finished_at
    queue
        .retry_worker_with_context(WorkerId::new(1), "fix issue".to_owned(), &i18n)
        .unwrap();

    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert!(w.finished_at.is_none());
}

// ============================================================================
// 5. Full lifecycle
// ============================================================================

#[test]
fn verification_lifecycle_pass() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])], &[(2, 2, &[1])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config.clone());
    let pipeline = test_pipeline(&config);

    // Worker 2 starts blocked
    assert_eq!(
        queue.get_worker(&WorkerId::new(2)).unwrap().status,
        WorkerStatus::Blocked
    );

    // Activate worker 1
    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    assert_eq!(
        queue.get_worker(&WorkerId::new(1)).unwrap().status,
        WorkerStatus::Active
    );

    // Complete worker 1 -> transitions to Verifying (not Completed)
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();
    assert_eq!(
        queue.get_worker(&WorkerId::new(1)).unwrap().status,
        WorkerStatus::Verifying
    );

    // Worker 2 should still be blocked (worker 1 not yet verified)
    assert_eq!(
        queue.get_worker(&WorkerId::new(2)).unwrap().status,
        WorkerStatus::Blocked
    );

    // Verify worker 1 with clean output -> Completed, unblocks worker 2
    let result = queue
        .verify_worker(WorkerId::new(1), CLEAN_OUTPUT, &pipeline, &i18n)
        .unwrap();
    assert!(result.status.is_success());
    assert_eq!(
        queue.get_worker(&WorkerId::new(1)).unwrap().status,
        WorkerStatus::Completed
    );
    assert_eq!(
        queue.get_worker(&WorkerId::new(2)).unwrap().status,
        WorkerStatus::Pending
    );

    // Activate and complete worker 2 through verification
    queue.activate_worker(WorkerId::new(2), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(2), &i18n).unwrap();
    let result = queue
        .verify_worker(WorkerId::new(2), CLEAN_OUTPUT, &pipeline, &i18n)
        .unwrap();
    assert!(result.status.is_success());
    assert_eq!(
        queue.get_worker(&WorkerId::new(2)).unwrap().status,
        WorkerStatus::Completed
    );

    let stats = queue.statistics();
    assert!(stats.all_completed());
}

#[test]
fn verification_lifecycle_fail_then_pass() {
    let i18n = i18n();
    let config = verification_config_enabled();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan_with_verification(&plan, 5, 3, config.clone());
    let pipeline = test_pipeline(&config);

    // Activate worker 1
    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    assert_eq!(
        queue.get_worker(&WorkerId::new(1)).unwrap().status,
        WorkerStatus::Active
    );

    // Complete worker 1 -> Verifying
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();
    assert_eq!(
        queue.get_worker(&WorkerId::new(1)).unwrap().status,
        WorkerStatus::Verifying
    );

    // Verify with failing output -> Active (retry available, verification_retries incremented)
    let result = queue
        .verify_worker(
            WorkerId::new(1),
            FAILING_CARGO_TEST_OUTPUT,
            &pipeline,
            &i18n,
        )
        .unwrap();
    assert!(!result.status.is_success());
    assert_eq!(
        queue.get_worker(&WorkerId::new(1)).unwrap().status,
        WorkerStatus::Active
    );
    assert_eq!(
        queue
            .get_worker(&WorkerId::new(1))
            .unwrap()
            .verification_retries,
        1
    );

    // Worker re-executes and completes again -> Verifying
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();
    assert_eq!(
        queue.get_worker(&WorkerId::new(1)).unwrap().status,
        WorkerStatus::Verifying
    );

    // This time verification passes -> Completed
    let result = queue
        .verify_worker(WorkerId::new(1), CLEAN_OUTPUT, &pipeline, &i18n)
        .unwrap();
    assert!(result.status.is_success());
    assert_eq!(
        queue.get_worker(&WorkerId::new(1)).unwrap().status,
        WorkerStatus::Completed
    );
    assert!(queue
        .get_worker(&WorkerId::new(1))
        .unwrap()
        .finished_at
        .is_some());

    let stats = queue.statistics();
    assert!(stats.all_completed());
}

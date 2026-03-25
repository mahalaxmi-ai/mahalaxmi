use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId, WorkerStatus};
use mahalaxmi_orchestration::models::plan::{ExecutionPhase, ExecutionPlan, WorkerTask};
use mahalaxmi_orchestration::WorkerQueue;

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

#[test]
fn queue_from_plan_creates_workers() {
    let plan = build_plan(&[
        &[(1, 1, &[]), (2, 2, &[]), (3, 3, &[])],
        &[(4, 4, &[1, 2]), (5, 5, &[3])],
    ]);
    let queue = WorkerQueue::from_plan(&plan, 5, 3);
    let stats = queue.statistics();
    assert_eq!(stats.total, 5);
}

#[test]
fn queue_from_plan_initial_status() {
    let plan = build_plan(&[&[(1, 1, &[]), (2, 2, &[])], &[(3, 3, &[1])]]);
    let queue = WorkerQueue::from_plan(&plan, 5, 3);

    // Workers with no deps should be Pending
    let w1 = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w1.status, WorkerStatus::Pending);
    let w2 = queue.get_worker(&WorkerId::new(2)).unwrap();
    assert_eq!(w2.status, WorkerStatus::Pending);

    // Worker with deps should be Blocked
    let w3 = queue.get_worker(&WorkerId::new(3)).unwrap();
    assert_eq!(w3.status, WorkerStatus::Blocked);
}

#[test]
fn queue_ready_worker_ids_returns_pending() {
    let plan = build_plan(&[&[(1, 1, &[]), (2, 2, &[])], &[(3, 3, &[1])]]);
    let queue = WorkerQueue::from_plan(&plan, 5, 3);
    let ready = queue.ready_worker_ids();

    // Workers 1 and 2 have no deps -> ready
    assert!(ready.contains(&WorkerId::new(1)));
    assert!(ready.contains(&WorkerId::new(2)));
    // Worker 3 is blocked -> not ready
    assert!(!ready.contains(&WorkerId::new(3)));
}

#[test]
fn queue_ready_worker_ids_respects_max_concurrent() {
    let plan = build_plan(&[&[(1, 1, &[]), (2, 2, &[]), (3, 3, &[]), (4, 4, &[])]]);
    let queue = WorkerQueue::from_plan(&plan, 2, 3);
    let ready = queue.ready_worker_ids();

    // All 4 workers are pending, but max_concurrent=2
    assert_eq!(ready.len(), 2);
}

#[test]
fn queue_activate_worker() {
    let i18n = i18n();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w.status, WorkerStatus::Active);
    assert!(w.started_at.is_some());
}

#[test]
fn queue_activate_nonexistent_worker_errors() {
    let i18n = i18n();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    let result = queue.activate_worker(WorkerId::new(999), &i18n);
    assert!(result.is_err());
}

#[test]
fn queue_complete_worker() {
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
fn queue_complete_unblocks_dependents() {
    let i18n = i18n();
    let plan = build_plan(&[&[(1, 1, &[])], &[(2, 2, &[1])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    // Worker 2 starts as Blocked
    assert_eq!(
        queue.get_worker(&WorkerId::new(2)).unwrap().status,
        WorkerStatus::Blocked
    );

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    // After completing worker 1, worker 2 should be unblocked -> Pending
    assert_eq!(
        queue.get_worker(&WorkerId::new(2)).unwrap().status,
        WorkerStatus::Pending
    );
}

#[test]
fn queue_fail_worker() {
    let i18n = i18n();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.fail_worker(WorkerId::new(1), &i18n).unwrap();

    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w.status, WorkerStatus::Failed);
    assert_eq!(w.failure_count, 1);
}

#[test]
fn queue_retry_worker_resets_status() {
    let i18n = i18n();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.fail_worker(WorkerId::new(1), &i18n).unwrap();
    queue.retry_worker(WorkerId::new(1), &i18n).unwrap();

    let w = queue.get_worker(&WorkerId::new(1)).unwrap();
    assert_eq!(w.status, WorkerStatus::Pending);
    assert!(w.started_at.is_none());
    assert!(w.finished_at.is_none());
}

#[test]
fn queue_retry_worker_exceeds_max() {
    let i18n = i18n();
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    // Fail 3 times (max_retries=3)
    for _ in 0..3 {
        queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
        queue.fail_worker(WorkerId::new(1), &i18n).unwrap();
    }

    // Now failure_count == 3 == max_retries, retry should error
    let result = queue.retry_worker(WorkerId::new(1), &i18n);
    assert!(result.is_err());
}

#[test]
fn queue_statistics_counts() {
    let i18n = i18n();
    let plan = build_plan(&[
        &[(1, 1, &[]), (2, 2, &[]), (3, 3, &[])],
        &[(4, 4, &[1]), (5, 5, &[2])],
    ]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    // Activate worker 1 and complete it
    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    // Activate worker 2
    queue.activate_worker(WorkerId::new(2), &i18n).unwrap();

    // Activate worker 3 and fail it
    queue.activate_worker(WorkerId::new(3), &i18n).unwrap();
    queue.fail_worker(WorkerId::new(3), &i18n).unwrap();

    let stats = queue.statistics();
    assert_eq!(stats.total, 5);
    assert_eq!(stats.completed, 1); // worker 1
    assert_eq!(stats.active, 1); // worker 2
    assert_eq!(stats.failed, 1); // worker 3
                                 // worker 4 should be unblocked (dep on task-1 completed), worker 5 still blocked (dep on task-2 active)
    assert_eq!(stats.pending, 1); // worker 4
    assert_eq!(stats.blocked, 1); // worker 5
}

#[test]
fn queue_available_slots() {
    let i18n = i18n();
    let plan = build_plan(&[&[
        (1, 1, &[]),
        (2, 2, &[]),
        (3, 3, &[]),
        (4, 4, &[]),
        (5, 5, &[]),
    ]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    assert_eq!(queue.available_slots(), 5);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.activate_worker(WorkerId::new(2), &i18n).unwrap();
    queue.activate_worker(WorkerId::new(3), &i18n).unwrap();

    assert_eq!(queue.available_slots(), 2);
}

#[test]
fn queue_get_worker() {
    let plan = build_plan(&[&[(1, 1, &[])]]);
    let queue = WorkerQueue::from_plan(&plan, 5, 3);

    assert!(queue.get_worker(&WorkerId::new(1)).is_some());
    assert!(queue.get_worker(&WorkerId::new(999)).is_none());
}

#[test]
fn queue_full_lifecycle() {
    let i18n = i18n();
    let plan = build_plan(&[&[(1, 1, &[])], &[(2, 2, &[1])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    // Phase 1: activate and complete worker 1
    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    // Phase 2: worker 2 should now be ready
    let ready = queue.ready_worker_ids();
    assert!(ready.contains(&WorkerId::new(2)));

    queue.activate_worker(WorkerId::new(2), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(2), &i18n).unwrap();

    let stats = queue.statistics();
    assert!(stats.all_completed());
    assert!((stats.completion_percentage() - 100.0).abs() < 0.001);
}

#[test]
fn queue_blocked_workers_become_pending_after_deps() {
    let i18n = i18n();
    // Chain: A -> B -> C
    let plan = build_plan(&[&[(1, 1, &[])], &[(2, 2, &[1])], &[(3, 3, &[2])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);

    // Initially: 1=Pending, 2=Blocked, 3=Blocked
    assert_eq!(
        queue.get_worker(&WorkerId::new(2)).unwrap().status,
        WorkerStatus::Blocked
    );
    assert_eq!(
        queue.get_worker(&WorkerId::new(3)).unwrap().status,
        WorkerStatus::Blocked
    );

    // Complete A -> B should become Pending, C still Blocked
    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    assert_eq!(
        queue.get_worker(&WorkerId::new(2)).unwrap().status,
        WorkerStatus::Pending
    );
    assert_eq!(
        queue.get_worker(&WorkerId::new(3)).unwrap().status,
        WorkerStatus::Blocked
    );
}

#[test]
fn queue_ready_worker_ids_sorted_by_id() {
    let plan = build_plan(&[&[(5, 5, &[]), (3, 3, &[]), (1, 1, &[]), (4, 4, &[])]]);
    let queue = WorkerQueue::from_plan(&plan, 10, 3);
    let ready = queue.ready_worker_ids();

    // Should be sorted by worker ID (as_u32)
    let ids: Vec<u32> = ready.iter().map(|w| w.as_u32()).collect();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    assert_eq!(ids, sorted_ids);
}

#[test]
fn queue_parallel_workers_activate_correctly() {
    let i18n = i18n();
    let plan = build_plan(&[&[(1, 1, &[]), (2, 2, &[]), (3, 3, &[]), (4, 4, &[])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 3, 3);

    // Activate 3 workers (max_concurrent = 3)
    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.activate_worker(WorkerId::new(2), &i18n).unwrap();
    queue.activate_worker(WorkerId::new(3), &i18n).unwrap();

    assert_eq!(queue.available_slots(), 0);

    // No more ready workers (max concurrent reached)
    let ready = queue.ready_worker_ids();
    assert_eq!(ready.len(), 0);
}

// ===========================================================================
// P7: max_concurrent getter / set_max_concurrent setter
// ===========================================================================

/// P7: max_concurrent() returns the initial value set at construction.
#[test]
fn queue_max_concurrent_getter_returns_initial_value() {
    let plan = build_plan(&[&[(1, 1, &[]), (2, 2, &[])]]);
    let queue = WorkerQueue::from_plan(&plan, 5, 3);
    assert_eq!(queue.max_concurrent(), 5);
}

/// P7: set_max_concurrent() updates the concurrency limit in-place.
#[test]
fn queue_set_max_concurrent_updates_value() {
    let plan = build_plan(&[&[(1, 1, &[]), (2, 2, &[]), (3, 3, &[])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 2, 3);
    assert_eq!(queue.max_concurrent(), 2);

    queue.set_max_concurrent(10);

    assert_eq!(queue.max_concurrent(), 10);
    assert_eq!(queue.available_slots(), 10);
}

/// P7: set_max_concurrent() to match task count makes all tasks ready.
#[test]
fn queue_set_max_concurrent_to_task_count_unlocks_all() {
    let plan = build_plan(&[&[(1, 1, &[]), (2, 2, &[]), (3, 3, &[])]]);
    let mut queue = WorkerQueue::from_plan(&plan, 1, 3);

    // With max_concurrent = 1 only 1 ready at a time
    assert_eq!(queue.ready_worker_ids().len(), 1);

    // Auto-scale to 3 (= task count)
    queue.set_max_concurrent(3);
    assert_eq!(queue.ready_worker_ids().len(), 3);
}

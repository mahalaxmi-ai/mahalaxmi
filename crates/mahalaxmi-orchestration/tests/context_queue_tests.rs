use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_orchestration::models::plan::{ExecutionPhase, ExecutionPlan, WorkerTask};
use mahalaxmi_orchestration::WorkerQueue;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

fn single_worker_plan() -> ExecutionPlan {
    let task = WorkerTask::new(
        TaskId::new("task-1"),
        WorkerId::new(1),
        "Task 1",
        "Description for task 1",
    );
    ExecutionPlan::from_phases(vec![ExecutionPhase {
        phase_number: 0,
        tasks: vec![task],
    }])
}

#[test]
fn set_and_get_worker_context() {
    let plan = single_worker_plan();
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);
    let i18n = i18n();

    let preamble = "## Context\nHere is the context for this worker.".to_owned();
    queue
        .set_worker_context(WorkerId::new(1), preamble.clone(), &i18n)
        .unwrap();

    let retrieved = queue.worker_context(&WorkerId::new(1));
    assert_eq!(retrieved, Some(preamble.as_str()));
}

#[test]
fn worker_context_returns_none_when_not_set() {
    let plan = single_worker_plan();
    let queue = WorkerQueue::from_plan(&plan, 5, 3);

    assert_eq!(queue.worker_context(&WorkerId::new(1)), None);
}

#[test]
fn set_worker_context_nonexistent_worker_fails() {
    let plan = single_worker_plan();
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);
    let i18n = i18n();

    let result = queue.set_worker_context(WorkerId::new(999), "context".to_owned(), &i18n);
    assert!(result.is_err());
}

#[test]
fn context_survives_activation() {
    let plan = single_worker_plan();
    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);
    let i18n = i18n();

    queue
        .set_worker_context(WorkerId::new(1), "my context".to_owned(), &i18n)
        .unwrap();
    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();

    // Context should still be there after activation
    assert_eq!(queue.worker_context(&WorkerId::new(1)), Some("my context"));
}

#[test]
fn backward_compat_existing_tests_pass() {
    // Verify that existing queue operations work without context
    let task1 = WorkerTask::new(
        TaskId::new("task-1"),
        WorkerId::new(1),
        "Task 1",
        "Description 1",
    );
    let task2 = WorkerTask::new(
        TaskId::new("task-2"),
        WorkerId::new(2),
        "Task 2",
        "Description 2",
    )
    .with_dependency(TaskId::new("task-1"));

    let plan = ExecutionPlan::from_phases(vec![
        ExecutionPhase {
            phase_number: 0,
            tasks: vec![task1],
        },
        ExecutionPhase {
            phase_number: 1,
            tasks: vec![task2],
        },
    ]);

    let mut queue = WorkerQueue::from_plan(&plan, 5, 3);
    let i18n = i18n();

    // Activate and complete without setting any context
    let ready = queue.ready_worker_ids();
    assert_eq!(ready, vec![WorkerId::new(1)]);

    queue.activate_worker(WorkerId::new(1), &i18n).unwrap();
    queue.complete_worker(WorkerId::new(1), &i18n).unwrap();

    let ready = queue.ready_worker_ids();
    assert_eq!(ready, vec![WorkerId::new(2)]);

    let stats = queue.statistics();
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.pending, 1);
}

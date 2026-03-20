use mahalaxmi_core::types::{ConsensusStrategy, ManagerId, TaskId, WorkerId};
use mahalaxmi_orchestration::models::{
    ConsensusResult, ConsensusTask, ExecutionPhase, ExecutionPlan, ManagerProposal, ProposedTask,
    QueueStatistics, WorkerTask,
};

// ---------------------------------------------------------------------------
// ProposedTask tests
// ---------------------------------------------------------------------------

#[test]
fn proposed_task_new_defaults() {
    let task = ProposedTask::new("Fix bug", "Fix the null pointer bug");
    assert_eq!(task.title, "Fix bug");
    assert_eq!(task.description, "Fix the null pointer bug");
    assert_eq!(task.complexity, 5);
    assert_eq!(task.priority, 100);
    assert!(task.dependencies.is_empty());
    assert!(task.affected_files.is_empty());
}

#[test]
fn proposed_task_builder_pattern() {
    let task = ProposedTask::new("Refactor", "Refactor the module")
        .with_complexity(7)
        .with_priority(3)
        .with_dependency("dep-1")
        .with_affected_file("main.rs");

    assert_eq!(task.complexity, 7);
    assert_eq!(task.priority, 3);
    assert_eq!(task.dependencies, vec!["dep-1".to_string()]);
    assert_eq!(task.affected_files, vec!["main.rs".to_string()]);
}

#[test]
fn proposed_task_normalized_key() {
    let task1 = ProposedTask::new("Fix the  Bug!", "desc");
    assert_eq!(task1.normalized_key(), "fix-the-bug");

    let task2 = ProposedTask::new("  spaces  and  CAPS  ", "desc");
    assert_eq!(task2.normalized_key(), "spaces-and-caps");
}

// ---------------------------------------------------------------------------
// ManagerProposal tests
// ---------------------------------------------------------------------------

#[test]
fn manager_proposal_successful() {
    let tasks = vec![
        ProposedTask::new("Task 1", "Description 1"),
        ProposedTask::new("Task 2", "Description 2"),
    ];
    let proposal = ManagerProposal::new(ManagerId::new("mgr-1"), tasks, 5000);

    assert!(proposal.completed);
    assert!(proposal.error_message.is_none());
    assert_eq!(proposal.tasks.len(), 2);
    assert_eq!(proposal.duration_ms, 5000);
    assert_eq!(proposal.manager_id.as_str(), "mgr-1");
}

#[test]
fn manager_proposal_failed() {
    let proposal =
        ManagerProposal::failed(ManagerId::new("mgr-2"), "Timeout during analysis", 12000);

    assert!(!proposal.completed);
    assert!(proposal.tasks.is_empty());
    assert_eq!(
        proposal.error_message,
        Some("Timeout during analysis".to_string())
    );
    assert_eq!(proposal.duration_ms, 12000);
}

// ---------------------------------------------------------------------------
// ConsensusResult tests
// ---------------------------------------------------------------------------

#[test]
fn consensus_result_no_consensus() {
    let result = ConsensusResult::no_consensus(ConsensusStrategy::WeightedVoting);
    assert!(result.agreed_tasks.is_empty());
    assert!(result.dissenting_tasks.is_empty());
    assert_eq!(result.strategy, ConsensusStrategy::WeightedVoting);
    assert_eq!(result.metrics.total_proposals, 0);
}

#[test]
fn consensus_result_from_single_proposal() {
    let tasks = vec![
        ConsensusTask {
            normalized_key: "fix-bug".to_string(),
            title: "Fix Bug".to_string(),
            description: "Fix it".to_string(),
            average_complexity: 5.0,
            vote_count: 1,
            total_managers: 1,
            proposed_by: vec![ManagerId::new("mgr-1")],
            dependencies: Vec::new(),
            affected_files: Vec::new(),
            acceptance_criteria: vec![],
        },
        ConsensusTask {
            normalized_key: "add-feature".to_string(),
            title: "Add Feature".to_string(),
            description: "Add it".to_string(),
            average_complexity: 7.0,
            vote_count: 1,
            total_managers: 1,
            proposed_by: vec![ManagerId::new("mgr-1")],
            dependencies: Vec::new(),
            affected_files: Vec::new(),
            acceptance_criteria: vec![],
        },
    ];

    let result = ConsensusResult::from_single_proposal(ConsensusStrategy::Union, tasks);

    assert_eq!(result.metrics.total_proposals, 1);
    assert_eq!(result.metrics.successful_proposals, 1);
    assert_eq!(result.metrics.agreed_task_count, 2);
    assert_eq!(result.metrics.dissenting_task_count, 0);
    assert_eq!(result.agreed_tasks.len(), 2);
    assert!(result.dissenting_tasks.is_empty());
}

// ---------------------------------------------------------------------------
// WorkerTask tests
// ---------------------------------------------------------------------------

#[test]
fn worker_task_new_defaults() {
    let task = WorkerTask::new(
        TaskId::new("t-1"),
        WorkerId::new(1),
        "Task title",
        "Task description",
    );
    assert!(task.has_no_dependencies());
    assert_eq!(task.complexity, 5);
    assert!(task.affected_files.is_empty());
}

#[test]
fn worker_task_with_dependencies() {
    let task = WorkerTask::new(
        TaskId::new("t-2"),
        WorkerId::new(2),
        "Dependent task",
        "Depends on t-1",
    )
    .with_dependency(TaskId::new("t-1"));

    assert!(!task.has_no_dependencies());
    assert_eq!(task.dependencies.len(), 1);
    assert_eq!(task.dependencies[0].as_str(), "t-1");
}

// ---------------------------------------------------------------------------
// ExecutionPlan tests
// ---------------------------------------------------------------------------

#[test]
fn execution_plan_empty() {
    let plan = ExecutionPlan::new();
    assert_eq!(plan.phase_count(), 0);
    assert!(plan.all_workers().is_empty());
}

#[test]
fn execution_plan_validate_catches_duplicate_task_ids() {
    let plan = ExecutionPlan::from_phases(vec![ExecutionPhase {
        phase_number: 0,
        tasks: vec![
            WorkerTask::new(TaskId::new("dup"), WorkerId::new(1), "A", "desc A"),
            WorkerTask::new(TaskId::new("dup"), WorkerId::new(2), "B", "desc B"),
        ],
    }]);

    let errors = plan.validate();
    assert!(
        !errors.is_empty(),
        "Expected validation errors for duplicate task IDs"
    );
    assert!(
        errors.iter().any(|e| e.contains("Duplicate task ID")),
        "Expected a 'Duplicate task ID' error, got: {:?}",
        errors
    );
}

#[test]
fn execution_plan_validate_catches_unresolvable_deps() {
    let task_with_missing_dep =
        WorkerTask::new(TaskId::new("t-1"), WorkerId::new(1), "Needs ghost", "desc")
            .with_dependency(TaskId::new("ghost-task"));

    let plan = ExecutionPlan::from_phases(vec![ExecutionPhase {
        phase_number: 0,
        tasks: vec![task_with_missing_dep],
    }]);

    let errors = plan.validate();
    assert!(
        !errors.is_empty(),
        "Expected validation errors for unresolvable dependencies"
    );
    assert!(
        errors.iter().any(|e| e.contains("ghost-task")),
        "Expected error mentioning 'ghost-task', got: {:?}",
        errors
    );
}

#[test]
fn execution_plan_validate_clean() {
    let plan = ExecutionPlan::from_phases(vec![
        ExecutionPhase {
            phase_number: 0,
            tasks: vec![WorkerTask::new(
                TaskId::new("t-1"),
                WorkerId::new(1),
                "Phase 0 task",
                "desc",
            )],
        },
        ExecutionPhase {
            phase_number: 1,
            tasks: vec![WorkerTask::new(
                TaskId::new("t-2"),
                WorkerId::new(2),
                "Phase 1 task",
                "desc",
            )
            .with_dependency(TaskId::new("t-1"))],
        },
    ]);

    let errors = plan.validate();
    assert!(
        errors.is_empty(),
        "Expected no validation errors, got: {:?}",
        errors
    );
}

// ---------------------------------------------------------------------------
// QueueStatistics tests
// ---------------------------------------------------------------------------

#[test]
fn queue_statistics_all_completed() {
    let stats_done = QueueStatistics {
        total: 5,
        completed: 5,
        active: 0,
        pending: 0,
        blocked: 0,
        verifying: 0,
        failed: 0,
        skipped: 0,
    };
    assert!(stats_done.all_completed());

    let stats_not_done = QueueStatistics {
        total: 5,
        completed: 4,
        active: 1,
        pending: 0,
        blocked: 0,
        verifying: 0,
        failed: 0,
        skipped: 0,
    };
    assert!(!stats_not_done.all_completed());

    // Edge case: empty queue should not be "all completed"
    let stats_empty = QueueStatistics::default();
    assert!(!stats_empty.all_completed());
}

#[test]
fn queue_statistics_completion_percentage() {
    let stats = QueueStatistics {
        total: 10,
        completed: 7,
        active: 1,
        pending: 1,
        blocked: 0,
        verifying: 0,
        failed: 1,
        skipped: 0,
    };
    let pct = stats.completion_percentage();
    assert!((pct - 70.0).abs() < f64::EPSILON);

    // Edge case: zero total should return 0.0
    let empty = QueueStatistics::default();
    assert!((empty.completion_percentage() - 0.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Multi-provider routing fields
// ---------------------------------------------------------------------------

#[test]
fn worker_task_type_defaults_to_none() {
    let task = WorkerTask::new(TaskId::new("t-1"), WorkerId::new(0), "Test", "Desc");
    assert!(task.task_type.is_none());
}

#[test]
fn worker_task_with_task_type_builder() {
    let task = WorkerTask::new(TaskId::new("t-1"), WorkerId::new(0), "Test", "Desc")
        .with_task_type(mahalaxmi_providers::TaskType::CodeReview);
    assert_eq!(
        task.task_type,
        Some(mahalaxmi_providers::TaskType::CodeReview)
    );
}

#[test]
fn queued_worker_provider_fields_default() {
    use mahalaxmi_orchestration::models::QueuedWorker;
    let qw = QueuedWorker::new(WorkerId::new(0), TaskId::new("t-1"), vec![], 3);
    assert!(qw.provider_id.is_none());
    assert!(qw.fallback_provider_ids.is_empty());
}

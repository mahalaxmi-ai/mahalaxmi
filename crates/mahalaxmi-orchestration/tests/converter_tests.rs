//! Tests for Step 02 converters:
//! - `ExecutionPlan::from_consensus_result()` — ConsensusResult → ExecutionPlan
//! - `ManagerProposal::from_json()` — JSON → ManagerProposal

use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ConsensusStrategy, ManagerId};
use mahalaxmi_orchestration::models::{
    ConsensusMetrics, ConsensusResult, ConsensusTask, ExecutionPlan, ManagerProposal,
};

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ===========================================================================
// ExecutionPlan::from_consensus_result tests
// ===========================================================================

#[test]
fn from_consensus_single_task_no_dependencies() {
    let i18n = i18n();
    let result = ConsensusResult {
        strategy: ConsensusStrategy::Union,
        agreed_tasks: vec![ConsensusTask {
            normalized_key: "fix-auth-bug".into(),
            title: "Fix auth bug".into(),
            description: "Fix the authentication bypass vulnerability".into(),
            average_complexity: 6.0,
            vote_count: 2,
            total_managers: 2,
            proposed_by: vec![ManagerId::new("m1"), ManagerId::new("m2")],
            dependencies: vec![],
            affected_files: vec!["src/auth.rs".into()],
            acceptance_criteria: vec![],
        }],
        dissenting_tasks: vec![],
        metrics: ConsensusMetrics::default(),
        cycle_label: None,
    };

    let plan = ExecutionPlan::from_consensus_result(&result, &i18n).unwrap();

    assert_eq!(plan.phase_count(), 1, "Single task should produce 1 phase");
    assert_eq!(plan.phases[0].tasks.len(), 1);
    assert_eq!(plan.phases[0].tasks[0].title, "Fix auth bug");
    assert_eq!(plan.phases[0].tasks[0].complexity, 6);
    assert_eq!(plan.phases[0].tasks[0].affected_files, vec!["src/auth.rs"]);
    assert!(plan.phases[0].tasks[0].dependencies.is_empty());

    let errors = plan.validate();
    assert!(errors.is_empty(), "Plan should be valid: {errors:?}");
}

#[test]
fn from_consensus_linear_dependencies() {
    let i18n = i18n();
    // A → B → C (linear chain)
    let result = ConsensusResult {
        strategy: ConsensusStrategy::WeightedVoting,
        agreed_tasks: vec![
            ConsensusTask {
                normalized_key: "task-a".into(),
                title: "Task A".into(),
                description: "First task".into(),
                average_complexity: 3.0,
                vote_count: 2,
                total_managers: 2,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec![],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
            ConsensusTask {
                normalized_key: "task-b".into(),
                title: "Task B".into(),
                description: "Depends on A".into(),
                average_complexity: 5.0,
                vote_count: 2,
                total_managers: 2,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec!["task-a".into()],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
            ConsensusTask {
                normalized_key: "task-c".into(),
                title: "Task C".into(),
                description: "Depends on B".into(),
                average_complexity: 7.0,
                vote_count: 2,
                total_managers: 2,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec!["task-b".into()],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
        ],
        dissenting_tasks: vec![],
        metrics: ConsensusMetrics::default(),
        cycle_label: None,
    };

    let plan = ExecutionPlan::from_consensus_result(&result, &i18n).unwrap();

    assert_eq!(
        plan.phase_count(),
        3,
        "Linear chain A→B→C should produce 3 phases"
    );
    assert_eq!(plan.phases[0].tasks.len(), 1);
    assert_eq!(plan.phases[1].tasks.len(), 1);
    assert_eq!(plan.phases[2].tasks.len(), 1);

    // Verify ordering: A first, then B, then C
    assert_eq!(plan.phases[0].tasks[0].title, "Task A");
    assert_eq!(plan.phases[1].tasks[0].title, "Task B");
    assert_eq!(plan.phases[2].tasks[0].title, "Task C");

    let errors = plan.validate();
    assert!(errors.is_empty(), "Plan should be valid: {errors:?}");
}

#[test]
fn from_consensus_parallel_tasks() {
    let i18n = i18n();
    // A and B have no dependencies — should be in same phase
    let result = ConsensusResult {
        strategy: ConsensusStrategy::Union,
        agreed_tasks: vec![
            ConsensusTask {
                normalized_key: "task-a".into(),
                title: "Task A".into(),
                description: "Independent task A".into(),
                average_complexity: 4.0,
                vote_count: 2,
                total_managers: 2,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec![],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
            ConsensusTask {
                normalized_key: "task-b".into(),
                title: "Task B".into(),
                description: "Independent task B".into(),
                average_complexity: 4.0,
                vote_count: 2,
                total_managers: 2,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec![],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
        ],
        dissenting_tasks: vec![],
        metrics: ConsensusMetrics::default(),
        cycle_label: None,
    };

    let plan = ExecutionPlan::from_consensus_result(&result, &i18n).unwrap();

    assert_eq!(
        plan.phase_count(),
        1,
        "Two independent tasks should produce 1 phase"
    );
    assert_eq!(plan.phases[0].tasks.len(), 2);

    let errors = plan.validate();
    assert!(errors.is_empty(), "Plan should be valid: {errors:?}");
}

#[test]
fn from_consensus_diamond_dependency() {
    let i18n = i18n();
    // Diamond: A→B, A→C, B→D, C→D
    // Phase 0: A | Phase 1: B, C | Phase 2: D
    let result = ConsensusResult {
        strategy: ConsensusStrategy::ComplexityWeighted,
        agreed_tasks: vec![
            ConsensusTask {
                normalized_key: "task-a".into(),
                title: "Task A".into(),
                description: "Root task".into(),
                average_complexity: 3.0,
                vote_count: 2,
                total_managers: 2,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec![],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
            ConsensusTask {
                normalized_key: "task-b".into(),
                title: "Task B".into(),
                description: "Depends on A".into(),
                average_complexity: 5.0,
                vote_count: 2,
                total_managers: 2,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec!["task-a".into()],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
            ConsensusTask {
                normalized_key: "task-c".into(),
                title: "Task C".into(),
                description: "Depends on A".into(),
                average_complexity: 5.0,
                vote_count: 2,
                total_managers: 2,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec!["task-a".into()],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
            ConsensusTask {
                normalized_key: "task-d".into(),
                title: "Task D".into(),
                description: "Depends on B and C".into(),
                average_complexity: 8.0,
                vote_count: 2,
                total_managers: 2,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec!["task-b".into(), "task-c".into()],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
        ],
        dissenting_tasks: vec![],
        metrics: ConsensusMetrics::default(),
        cycle_label: None,
    };

    let plan = ExecutionPlan::from_consensus_result(&result, &i18n).unwrap();

    assert_eq!(
        plan.phase_count(),
        3,
        "Diamond A→(B,C)→D should produce 3 phases"
    );
    assert_eq!(plan.phases[0].tasks.len(), 1, "Phase 0: just A");
    assert_eq!(
        plan.phases[1].tasks.len(),
        2,
        "Phase 1: B and C in parallel"
    );
    assert_eq!(plan.phases[2].tasks.len(), 1, "Phase 2: just D");

    assert_eq!(plan.phases[0].tasks[0].title, "Task A");
    assert_eq!(plan.phases[2].tasks[0].title, "Task D");

    let errors = plan.validate();
    assert!(errors.is_empty(), "Plan should be valid: {errors:?}");
}

#[test]
fn from_consensus_empty_produces_empty_plan() {
    let i18n = i18n();
    let result = ConsensusResult {
        strategy: ConsensusStrategy::Union,
        agreed_tasks: vec![],
        dissenting_tasks: vec![],
        metrics: ConsensusMetrics::default(),
        cycle_label: None,
    };

    let plan = ExecutionPlan::from_consensus_result(&result, &i18n).unwrap();

    assert_eq!(
        plan.phase_count(),
        0,
        "Empty consensus should produce empty plan"
    );
    assert!(plan.all_workers().is_empty());
}

#[test]
fn from_consensus_preserves_task_metadata() {
    let i18n = i18n();
    let result = ConsensusResult {
        strategy: ConsensusStrategy::Intersection,
        agreed_tasks: vec![ConsensusTask {
            normalized_key: "refactor-db".into(),
            title: "Refactor Database Layer".into(),
            description: "Extract database operations into repository pattern".into(),
            average_complexity: 8.5,
            vote_count: 3,
            total_managers: 3,
            proposed_by: vec![
                ManagerId::new("m1"),
                ManagerId::new("m2"),
                ManagerId::new("m3"),
            ],
            dependencies: vec![],
            affected_files: vec![
                "src/db/mod.rs".into(),
                "src/db/repository.rs".into(),
                "src/models/user.rs".into(),
            ],
            acceptance_criteria: vec![],
        }],
        dissenting_tasks: vec![],
        metrics: ConsensusMetrics::default(),
        cycle_label: None,
    };

    let plan = ExecutionPlan::from_consensus_result(&result, &i18n).unwrap();

    let task = &plan.phases[0].tasks[0];
    assert_eq!(task.title, "Refactor Database Layer");
    assert_eq!(
        task.description,
        "Extract database operations into repository pattern"
    );
    // 8.5 rounds to 9
    assert_eq!(task.complexity, 9);
    assert_eq!(task.affected_files.len(), 3);
    assert!(task.affected_files.contains(&"src/db/mod.rs".to_string()));
    assert!(task
        .affected_files
        .contains(&"src/db/repository.rs".to_string()));
    assert!(task
        .affected_files
        .contains(&"src/models/user.rs".to_string()));
}

#[test]
fn from_consensus_complexity_clamped() {
    let i18n = i18n();
    let result = ConsensusResult {
        strategy: ConsensusStrategy::Union,
        agreed_tasks: vec![
            ConsensusTask {
                normalized_key: "high".into(),
                title: "High Complexity".into(),
                description: "Very complex".into(),
                average_complexity: 15.0,
                vote_count: 1,
                total_managers: 1,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec![],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
            ConsensusTask {
                normalized_key: "low".into(),
                title: "Low Complexity".into(),
                description: "Trivial".into(),
                average_complexity: 0.2,
                vote_count: 1,
                total_managers: 1,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec![],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
        ],
        dissenting_tasks: vec![],
        metrics: ConsensusMetrics::default(),
        cycle_label: None,
    };

    let plan = ExecutionPlan::from_consensus_result(&result, &i18n).unwrap();

    let tasks = &plan.phases[0].tasks;
    let high = tasks.iter().find(|t| t.title == "High Complexity").unwrap();
    let low = tasks.iter().find(|t| t.title == "Low Complexity").unwrap();
    assert_eq!(high.complexity, 10, "Complexity should be clamped to 10");
    assert_eq!(low.complexity, 1, "Complexity should be clamped to 1");
}

#[test]
fn from_consensus_unresolved_deps_ignored() {
    let i18n = i18n();
    // Task B depends on "nonexistent" which is not in agreed_tasks
    let result = ConsensusResult {
        strategy: ConsensusStrategy::Union,
        agreed_tasks: vec![
            ConsensusTask {
                normalized_key: "task-a".into(),
                title: "Task A".into(),
                description: "Root".into(),
                average_complexity: 3.0,
                vote_count: 1,
                total_managers: 1,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec![],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
            ConsensusTask {
                normalized_key: "task-b".into(),
                title: "Task B".into(),
                description: "Depends on nonexistent".into(),
                average_complexity: 3.0,
                vote_count: 1,
                total_managers: 1,
                proposed_by: vec![ManagerId::new("m1")],
                dependencies: vec!["nonexistent-task".into()],
                affected_files: vec![],
                acceptance_criteria: vec![],
            },
        ],
        dissenting_tasks: vec![],
        metrics: ConsensusMetrics::default(),
        cycle_label: None,
    };

    let plan = ExecutionPlan::from_consensus_result(&result, &i18n).unwrap();

    // Both tasks should be in phase 0 since the nonexistent dep is ignored
    assert_eq!(plan.phase_count(), 1);
    assert_eq!(plan.phases[0].tasks.len(), 2);
}

// ===========================================================================
// ManagerProposal::from_json tests
// ===========================================================================

#[test]
fn from_json_valid_parses() {
    let i18n = i18n();
    let json = r#"{
        "tasks": [
            {
                "title": "Implement auth module",
                "description": "Add JWT-based authentication",
                "complexity": 7,
                "priority": 1,
                "dependencies": [],
                "affected_files": ["src/auth.rs", "src/middleware.rs"]
            },
            {
                "title": "Write tests",
                "description": "Unit tests for auth module",
                "complexity": 4,
                "priority": 2,
                "dependencies": ["implement-auth-module"],
                "affected_files": ["tests/auth_test.rs"]
            }
        ]
    }"#;

    let proposal = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 1500, &i18n).unwrap();

    assert!(proposal.completed);
    assert_eq!(proposal.tasks.len(), 2);
    assert_eq!(proposal.tasks[0].title, "Implement auth module");
    assert_eq!(proposal.tasks[0].complexity, 7);
    assert_eq!(proposal.tasks[0].priority, 1);
    assert_eq!(
        proposal.tasks[0].affected_files,
        vec!["src/auth.rs", "src/middleware.rs"]
    );
    assert_eq!(proposal.tasks[1].title, "Write tests");
    assert_eq!(
        proposal.tasks[1].dependencies,
        vec!["implement-auth-module"]
    );
    assert_eq!(proposal.duration_ms, 1500);
}

#[test]
fn from_json_missing_fields_defaults() {
    let i18n = i18n();
    let json = r#"{
        "tasks": [
            {
                "title": "Simple task"
            }
        ]
    }"#;

    let proposal = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 500, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 1);
    let task = &proposal.tasks[0];
    assert_eq!(task.title, "Simple task");
    assert_eq!(task.description, "");
    assert_eq!(task.complexity, 5, "Default complexity should be 5");
    assert_eq!(task.priority, 100, "Default priority should be 100");
    assert!(task.dependencies.is_empty());
    assert!(task.affected_files.is_empty());
}

#[test]
fn from_json_clamps_complexity() {
    let i18n = i18n();
    let json = r#"{
        "tasks": [
            { "title": "Over-complex", "complexity": 15 },
            { "title": "Zero-complex", "complexity": 0 }
        ]
    }"#;

    let proposal = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 100, &i18n).unwrap();

    assert_eq!(proposal.tasks[0].complexity, 10, "Should clamp to 10");
    assert_eq!(proposal.tasks[1].complexity, 1, "Should clamp to 1");
}

#[test]
fn from_json_empty_title_rejected() {
    let i18n = i18n();
    let json = r#"{
        "tasks": [
            { "title": "" },
            { "title": "Valid task" }
        ]
    }"#;

    let result = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 100, &i18n);
    assert!(result.is_err(), "Empty title should produce an error");
}

#[test]
fn from_json_whitespace_only_title_rejected() {
    let i18n = i18n();
    let json = r#"{
        "tasks": [
            { "title": "   " }
        ]
    }"#;

    let result = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 100, &i18n);
    assert!(
        result.is_err(),
        "Whitespace-only title should produce an error"
    );
}

#[test]
fn from_json_malformed_json_error() {
    let i18n = i18n();
    let json = "this is not json at all {{{";

    let result = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 100, &i18n);
    assert!(result.is_err(), "Malformed JSON should produce an error");
}

#[test]
fn from_json_extra_fields_ignored() {
    let i18n = i18n();
    let json = r#"{
        "tasks": [
            {
                "title": "Valid task",
                "description": "Do the thing",
                "unknown_field": true,
                "another_extra": [1, 2, 3],
                "metadata": { "key": "value" }
            }
        ],
        "model_version": "gpt-4",
        "confidence": 0.95
    }"#;

    let proposal = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 200, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 1);
    assert_eq!(proposal.tasks[0].title, "Valid task");
}

#[test]
fn from_json_empty_tasks_array_rejected() {
    let i18n = i18n();
    let json = r#"{ "tasks": [] }"#;

    let result = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 100, &i18n);
    assert!(result.is_err(), "Empty tasks array should produce an error");
}

#[test]
fn from_json_multiple_tasks_with_deps() {
    let i18n = i18n();
    let json = r#"{
        "tasks": [
            {
                "title": "Setup database schema",
                "description": "Create migration files",
                "complexity": 3,
                "priority": 1,
                "affected_files": ["migrations/001_init.sql"]
            },
            {
                "title": "Implement repository",
                "description": "Data access layer",
                "complexity": 6,
                "priority": 2,
                "dependencies": ["setup-database-schema"],
                "affected_files": ["src/repo.rs"]
            },
            {
                "title": "Add API endpoints",
                "description": "REST endpoints",
                "complexity": 5,
                "priority": 3,
                "dependencies": ["implement-repository"],
                "affected_files": ["src/api.rs"]
            }
        ]
    }"#;

    let proposal = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 3000, &i18n).unwrap();

    assert_eq!(proposal.tasks.len(), 3);
    assert!(proposal.tasks[0].dependencies.is_empty());
    assert_eq!(
        proposal.tasks[1].dependencies,
        vec!["setup-database-schema"]
    );
    assert_eq!(proposal.tasks[2].dependencies, vec!["implement-repository"]);
}

#[test]
fn from_json_title_trimmed() {
    let i18n = i18n();
    let json = r#"{
        "tasks": [
            { "title": "  Padded title  ", "description": "desc" }
        ]
    }"#;

    let proposal = ManagerProposal::from_json(json, ManagerId::new("mgr-1"), 100, &i18n).unwrap();

    assert_eq!(proposal.tasks[0].title, "Padded title");
}

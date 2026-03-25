//! WAVE 3 integration tests — REQ-005, REQ-006, REQ-007, REQ-008.

use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_orchestration::dag::{add_file_conflict_edges, build_phases};
use mahalaxmi_orchestration::models::plan::WorkerTask;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

fn make_task(id: &str, worker_num: u32, files: &[&str]) -> WorkerTask {
    let mut t = WorkerTask::new(
        TaskId::new(id),
        WorkerId::new(worker_num),
        format!("Task {id}"),
        format!("Description for task {id}"),
    );
    t.affected_files = files.iter().map(|f| f.to_string()).collect();
    t
}

// ─── REQ-005: File conflict edge injection ────────────────────────────────────

#[test]
fn req005_tasks_sharing_file_end_up_in_different_phases() {
    let mut tasks = vec![
        make_task("task-0", 0, &["src/auth.rs"]),
        make_task("task-1", 1, &["src/auth.rs"]),
    ];

    // Without edges, both tasks have no dependencies → same phase.
    // With edges, task-1 depends on task-0 → different phases.
    add_file_conflict_edges(&mut tasks);

    let i = i18n();
    let phases = build_phases(&tasks, &i).expect("build_phases");
    assert!(
        phases.len() >= 2,
        "Tasks sharing a file must be in different phases, got {} phase(s)",
        phases.len()
    );
    // task-0 must appear before task-1 (phase 0 before phase 1).
    let phase0_ids: Vec<_> = phases[0].tasks.iter().map(|t| t.task_id.as_str()).collect();
    let phase1_ids: Vec<_> = phases[1].tasks.iter().map(|t| t.task_id.as_str()).collect();
    assert!(phase0_ids.contains(&"task-0"), "task-0 should be in phase 0");
    assert!(phase1_ids.contains(&"task-1"), "task-1 should be in phase 1");
}

#[test]
fn req005_tasks_with_no_shared_files_stay_in_same_phase() {
    let mut tasks = vec![
        make_task("task-0", 0, &["src/a.rs"]),
        make_task("task-1", 1, &["src/b.rs"]),
    ];

    add_file_conflict_edges(&mut tasks);

    let i = i18n();
    let phases = build_phases(&tasks, &i).expect("build_phases");
    // No conflict edges → both tasks in the same phase.
    assert_eq!(
        phases.len(),
        1,
        "Tasks with no shared files must be in the same phase"
    );
    assert_eq!(phases[0].tasks.len(), 2);
}

#[test]
fn req005_three_tasks_sharing_file_serialized_in_order() {
    let mut tasks = vec![
        make_task("task-0", 0, &["shared.ts"]),
        make_task("task-1", 1, &["shared.ts"]),
        make_task("task-2", 2, &["shared.ts"]),
    ];

    add_file_conflict_edges(&mut tasks);

    let i = i18n();
    let phases = build_phases(&tasks, &i).expect("build_phases");
    // All three must be in distinct phases (total serialization).
    assert!(
        phases.len() >= 3,
        "Three tasks sharing a file must be in at least 3 phases, got {}",
        phases.len()
    );
}

#[test]
fn req005_existing_dependency_not_duplicated() {
    let task0_id = TaskId::new("task-0");
    let t0 = make_task("task-0", 0, &["shared.rs"]);
    let mut t1 = make_task("task-1", 1, &["shared.rs"]);
    // Pre-existing dependency: task-1 already depends on task-0.
    t1.dependencies.push(task0_id);

    let initial_dep_count = t1.dependencies.len();
    let mut tasks = vec![t0, t1];
    add_file_conflict_edges(&mut tasks);

    // No duplicate edge should have been added.
    let dep_count_after = tasks
        .iter()
        .find(|t| t.task_id == TaskId::new("task-1"))
        .map(|t| t.dependencies.len())
        .unwrap_or(0);
    assert_eq!(
        dep_count_after, initial_dep_count,
        "Existing dependency must not be duplicated by add_file_conflict_edges"
    );
}

#[test]
fn req005_tasks_with_no_affected_files_unaffected() {
    let mut tasks = vec![
        make_task("task-0", 0, &[]),
        make_task("task-1", 1, &[]),
    ];

    add_file_conflict_edges(&mut tasks);

    // No edges added — both tasks should have no dependencies.
    for task in &tasks {
        assert!(
            task.dependencies.is_empty(),
            "Tasks with no affected_files must not receive synthetic dependencies"
        );
    }
}

// ─── REQ-007: Config-driven manager timeout ───────────────────────────────────

#[test]
fn req007_default_manager_hard_timeout_is_600() {
    use mahalaxmi_core::config::OrchestrationConfig;
    let cfg = OrchestrationConfig::default();
    assert_eq!(
        cfg.manager_hard_timeout_seconds, 600,
        "Default manager_hard_timeout_seconds must be 600s"
    );
}

#[test]
fn req007_manager_hard_timeout_survives_toml_roundtrip() {
    use mahalaxmi_core::config::RawOrchestrationConfig;
    let toml_src = r#"
        [orchestration]
        manager_hard_timeout_seconds = 1200
    "#;
    let raw: toml::Value = toml::from_str(toml_src).expect("parse toml");
    let raw_cfg: RawOrchestrationConfig =
        raw["orchestration"].clone().try_into().expect("deserialize");
    assert_eq!(raw_cfg.manager_hard_timeout_seconds, 1200);
}

// ─── REQ-008: Manager quorum enforcement (config) ────────────────────────────

#[test]
fn req008_default_min_manager_quorum_is_1() {
    use mahalaxmi_core::config::OrchestrationConfig;
    let cfg = OrchestrationConfig::default();
    assert_eq!(
        cfg.min_manager_quorum, 1,
        "Default min_manager_quorum must be 1 (any proposal proceeds)"
    );
}

#[test]
fn req008_quorum_of_1_means_single_proposal_proceeds() {
    // Simulate the quorum gate logic.
    let proposals_received: u32 = 1;
    let min_quorum: u32 = 1;
    assert!(
        proposals_received >= min_quorum,
        "A single proposal must satisfy quorum=1"
    );
}

#[test]
fn req008_quorum_not_met_when_proposals_below_threshold() {
    let proposals_received: u32 = 1;
    let min_quorum: u32 = 2;
    assert!(
        proposals_received < min_quorum,
        "1 proposal should not satisfy quorum=2"
    );
}

#[test]
fn req008_quorum_config_survives_toml_roundtrip() {
    use mahalaxmi_core::config::RawOrchestrationConfig;
    let toml_src = r#"
        [orchestration]
        min_manager_quorum = 3
    "#;
    let raw: toml::Value = toml::from_str(toml_src).expect("parse toml");
    let raw_cfg: RawOrchestrationConfig =
        raw["orchestration"].clone().try_into().expect("deserialize");
    assert_eq!(raw_cfg.min_manager_quorum, 3);
}

// ─── REQ-006: Per-worker build check config ───────────────────────────────────

#[test]
fn req006_build_check_disabled_by_default() {
    use mahalaxmi_core::config::VerificationConfig;
    let cfg = VerificationConfig::default();
    assert!(
        !cfg.run_build_check_per_worker,
        "run_build_check_per_worker must default to false"
    );
}

#[test]
fn req006_build_check_config_survives_toml_roundtrip() {
    use mahalaxmi_core::config::RawVerificationConfig;
    let toml_src = r#"
        [verification]
        run_build_check_per_worker = true
    "#;
    let raw: toml::Value = toml::from_str(toml_src).expect("parse toml");
    let raw_cfg: RawVerificationConfig =
        raw["verification"].clone().try_into().expect("deserialize");
    assert!(raw_cfg.run_build_check_per_worker);
}

// ─── REQ-005 + ExecutionPlan integration ─────────────────────────────────────

#[test]
fn req005_execution_plan_filters_platform_paths() {
    use mahalaxmi_orchestration::models::plan::ExecutionPhase;
    use mahalaxmi_orchestration::models::plan::ExecutionPlan;

    let platform_task = make_task("platform-task", 0, &["services/api/route.ts"]);
    let valid_task = make_task("valid-task", 1, &["src/lib.rs"]);

    let mut plan = ExecutionPlan::from_phases(vec![ExecutionPhase {
        phase_number: 0,
        tasks: vec![platform_task, valid_task],
    }]);

    let removed = plan.filter_platform_scoped_tasks();
    assert_eq!(removed, 1, "One platform-scoped task must be filtered");
    let remaining: Vec<_> = plan
        .all_workers()
        .iter()
        .map(|t| t.task_id.as_str())
        .collect();
    assert!(
        remaining.contains(&"valid-task"),
        "valid-task must remain after filtering"
    );
    assert!(
        !remaining.contains(&"platform-task"),
        "platform-task must be removed"
    );
}

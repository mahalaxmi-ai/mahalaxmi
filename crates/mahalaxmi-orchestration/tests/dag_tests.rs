use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_orchestration::models::plan::WorkerTask;
use mahalaxmi_orchestration::{build_phases, detect_cycles, topological_sort, validate_dag};

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

fn make_task(id: &str, deps: &[&str]) -> WorkerTask {
    let mut wt = WorkerTask::new(
        TaskId::new(id),
        WorkerId::new(
            id.chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<u32>()
                .unwrap_or(0),
        ),
        format!("Task {}", id),
        format!("Description for {}", id),
    );
    for dep in deps {
        wt = wt.with_dependency(TaskId::new(*dep));
    }
    wt
}

#[test]
fn dag_validate_acyclic() {
    let i18n = i18n();
    let tasks = vec![
        make_task("1", &[]),
        make_task("2", &["1"]),
        make_task("3", &["2"]),
    ];
    let result = validate_dag(&tasks, &i18n);
    assert!(result.is_ok());
}

#[test]
fn dag_validate_catches_cycle() {
    let i18n = i18n();
    let tasks = vec![
        make_task("1", &["3"]),
        make_task("2", &["1"]),
        make_task("3", &["2"]),
    ];
    let result = validate_dag(&tasks, &i18n);
    assert!(result.is_err());
}

#[test]
fn dag_validate_self_referencing() {
    let i18n = i18n();
    let tasks = vec![make_task("1", &["1"])];
    let result = validate_dag(&tasks, &i18n);
    assert!(result.is_err());
}

#[test]
fn dag_detect_cycles_none() {
    let tasks = vec![
        make_task("1", &[]),
        make_task("2", &["1"]),
        make_task("3", &["2"]),
    ];
    let cycles = detect_cycles(&tasks);
    assert!(cycles.is_empty());
}

#[test]
fn dag_detect_cycles_simple() {
    let tasks = vec![make_task("1", &["2"]), make_task("2", &["1"])];
    let cycles = detect_cycles(&tasks);
    assert!(!cycles.is_empty());
}

#[test]
fn dag_topological_sort_linear() {
    let i18n = i18n();
    let tasks = vec![
        make_task("1", &[]),
        make_task("2", &["1"]),
        make_task("3", &["2"]),
    ];
    let sorted = topological_sort(&tasks, &i18n).unwrap();

    // Task 1 must come before task 2, task 2 before task 3
    let pos_1 = sorted.iter().position(|id| id.as_str() == "1").unwrap();
    let pos_2 = sorted.iter().position(|id| id.as_str() == "2").unwrap();
    let pos_3 = sorted.iter().position(|id| id.as_str() == "3").unwrap();

    assert!(pos_1 < pos_2);
    assert!(pos_2 < pos_3);
}

#[test]
fn dag_topological_sort_diamond() {
    let i18n = i18n();
    // Diamond: A -> B, A -> C, B -> D, C -> D
    let tasks = vec![
        make_task("10", &[]),           // A
        make_task("20", &["10"]),       // B depends on A
        make_task("30", &["10"]),       // C depends on A
        make_task("40", &["20", "30"]), // D depends on B and C
    ];
    let sorted = topological_sort(&tasks, &i18n).unwrap();

    let pos_a = sorted.iter().position(|id| id.as_str() == "10").unwrap();
    let pos_b = sorted.iter().position(|id| id.as_str() == "20").unwrap();
    let pos_c = sorted.iter().position(|id| id.as_str() == "30").unwrap();
    let pos_d = sorted.iter().position(|id| id.as_str() == "40").unwrap();

    // A must come first
    assert!(pos_a < pos_b);
    assert!(pos_a < pos_c);
    // D must come last
    assert!(pos_b < pos_d);
    assert!(pos_c < pos_d);
}

#[test]
fn dag_topological_sort_rejects_cycle() {
    let i18n = i18n();
    let tasks = vec![make_task("1", &["2"]), make_task("2", &["1"])];
    let result = topological_sort(&tasks, &i18n);
    assert!(result.is_err());
}

#[test]
fn dag_build_phases_linear() {
    let i18n = i18n();
    let tasks = vec![
        make_task("1", &[]),
        make_task("2", &["1"]),
        make_task("3", &["2"]),
    ];
    let phases = build_phases(&tasks, &i18n).unwrap();

    // Linear chain -> 3 phases, 1 task each
    assert_eq!(phases.len(), 3);
    assert_eq!(phases[0].tasks.len(), 1);
    assert_eq!(phases[1].tasks.len(), 1);
    assert_eq!(phases[2].tasks.len(), 1);

    // Verify order
    assert_eq!(phases[0].tasks[0].task_id.as_str(), "1");
    assert_eq!(phases[1].tasks[0].task_id.as_str(), "2");
    assert_eq!(phases[2].tasks[0].task_id.as_str(), "3");
}

#[test]
fn dag_build_phases_parallel() {
    let i18n = i18n();
    // A, B, C have no deps; D depends on all three
    let tasks = vec![
        make_task("1", &[]),
        make_task("2", &[]),
        make_task("3", &[]),
        make_task("4", &["1", "2", "3"]),
    ];
    let phases = build_phases(&tasks, &i18n).unwrap();

    // Phase 0: [A, B, C] (no deps), Phase 1: [D]
    assert_eq!(phases.len(), 2);
    assert_eq!(phases[0].tasks.len(), 3);
    assert_eq!(phases[1].tasks.len(), 1);

    // D should be in phase 1
    assert_eq!(phases[1].tasks[0].task_id.as_str(), "4");

    // A, B, C should be in phase 0
    let phase0_ids: Vec<&str> = phases[0].tasks.iter().map(|t| t.task_id.as_str()).collect();
    assert!(phase0_ids.contains(&"1"));
    assert!(phase0_ids.contains(&"2"));
    assert!(phase0_ids.contains(&"3"));
}

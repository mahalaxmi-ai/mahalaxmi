use mahalaxmi_core::config::{BudgetAllocations, ContextConfig};
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_orchestration::context::{ContextBuilder, TokenBudget};
use mahalaxmi_orchestration::models::plan::WorkerTask;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

fn make_task(description: &str, affected: &[&str]) -> WorkerTask {
    let mut task = WorkerTask::new(
        TaskId::new("task-1"),
        WorkerId::new(1),
        "Test Task",
        description,
    );
    for file in affected {
        task = task.with_affected_file(*file);
    }
    task
}

fn default_budget() -> TokenBudget {
    TokenBudget::from_total(10_000, BudgetAllocations::default())
}

#[test]
fn graceful_without_memory() {
    let config = ContextConfig::default();
    let builder = ContextBuilder::new(config);
    let task = make_task("Fix src/lib.rs", &["src/lib.rs"]);
    let budget = default_budget();
    let i18n = i18n();

    // No memory store provided
    let result = builder.build(&task, &budget, None, None, &i18n).unwrap();
    assert!(!result.is_empty());
    assert_eq!(result.memory_entries_included, 0);
}

#[test]
fn graceful_without_index() {
    let config = ContextConfig::default();
    let builder = ContextBuilder::new(config);
    let task = make_task("Fix src/lib.rs", &["src/lib.rs"]);
    let budget = default_budget();
    let i18n = i18n();

    // No index provided
    let result = builder.build(&task, &budget, None, None, &i18n).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn graceful_without_memory_or_index() {
    let config = ContextConfig::default();
    let builder = ContextBuilder::new(config);
    let task = make_task("Fix src/lib.rs", &["src/lib.rs"]);
    let budget = default_budget();
    let i18n = i18n();

    // Neither provided
    let result = builder.build(&task, &budget, None, None, &i18n).unwrap();
    assert!(!result.is_empty());
    // Should still produce task description at minimum
    assert!(result.preamble.contains("Test Task"));
}

#[test]
fn disabled_config_produces_empty_context() {
    let config = ContextConfig {
        enabled: false,
        ..Default::default()
    };
    let builder = ContextBuilder::new(config);
    let task = make_task("Fix src/lib.rs", &["src/lib.rs"]);
    let budget = default_budget();
    let i18n = i18n();

    let result = builder.build(&task, &budget, None, None, &i18n).unwrap();
    assert!(result.is_empty());
    assert_eq!(result.token_usage.total, 0);
}

#[test]
fn partial_availability_still_builds() {
    let config = ContextConfig {
        include_repo_map: false,
        include_memory: false,
        ..Default::default()
    };
    let builder = ContextBuilder::new(config);
    let task = make_task("Fix src/lib.rs", &["src/lib.rs"]);
    let budget = default_budget();
    let i18n = i18n();

    let result = builder.build(&task, &budget, None, None, &i18n).unwrap();
    // Should still have task description and file references
    assert!(!result.is_empty());
    assert!(result.preamble.contains("Test Task"));
}

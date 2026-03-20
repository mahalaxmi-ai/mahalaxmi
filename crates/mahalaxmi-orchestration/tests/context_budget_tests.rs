use mahalaxmi_core::config::{BudgetAllocations, ContextConfig};
use mahalaxmi_orchestration::context::{estimate_tokens, TokenBudget, TokenUsage};
use mahalaxmi_providers::types::ProviderCapabilities;

#[test]
fn estimate_tokens_empty_string() {
    assert_eq!(estimate_tokens(""), 0);
}

#[test]
fn estimate_tokens_short_string() {
    // "hello" = 5 chars / 4 = 1 token
    assert_eq!(estimate_tokens("hello"), 1);
}

#[test]
fn estimate_tokens_longer_string() {
    // 100 chars / 4 = 25 tokens
    let text = "a".repeat(100);
    assert_eq!(estimate_tokens(&text), 25);
}

#[test]
fn token_budget_from_provider_default_context() {
    let caps = ProviderCapabilities {
        max_context_tokens: 100_000,
        ..Default::default()
    };
    let config = ContextConfig::default();
    let budget = TokenBudget::from_provider(&caps, &config);

    // 80% of 100_000 = 80_000
    assert_eq!(budget.total(), 80_000);
}

#[test]
fn token_budget_from_provider_zero_context_uses_default() {
    let caps = ProviderCapabilities::default(); // max_context_tokens = 0
    let config = ContextConfig::default();
    let budget = TokenBudget::from_provider(&caps, &config);

    // Should default to 8192
    assert_eq!(budget.total(), 8192);
}

#[test]
fn token_budget_section_allocations() {
    let alloc = BudgetAllocations {
        repo_map_pct: 0.25,
        relevant_files_pct: 0.50,
        memory_pct: 0.15,
        task_description_pct: 0.10,
    };
    let budget = TokenBudget::from_total(10_000, alloc);

    assert_eq!(budget.tokens_for_repo_map(), 2500);
    assert_eq!(budget.tokens_for_files(), 5000);
    assert_eq!(budget.tokens_for_memory(), 1500);
    assert_eq!(budget.tokens_for_task(), 1000);
}

#[test]
fn token_budget_remaining() {
    let budget = TokenBudget::from_total(10_000, BudgetAllocations::default());
    assert_eq!(budget.remaining(3000), 7000);
    assert_eq!(budget.remaining(10_000), 0);
    assert_eq!(budget.remaining(15_000), 0); // saturating
}

#[test]
fn token_usage_tracking() {
    let mut usage = TokenUsage::new(1000);
    usage.add_repo_map(200);
    usage.add_files(300);
    usage.add_memory(100);
    usage.add_task(50);

    assert_eq!(usage.repo_map, 200);
    assert_eq!(usage.files, 300);
    assert_eq!(usage.memory, 100);
    assert_eq!(usage.task, 50);
    assert_eq!(usage.total, 650);
    assert!(!usage.is_over_budget());
}

#[test]
fn token_usage_over_budget() {
    let mut usage = TokenUsage::new(100);
    usage.add_files(150);
    assert!(usage.is_over_budget());
}

#[test]
fn token_usage_utilization() {
    let mut usage = TokenUsage::new(1000);
    usage.add_files(500);
    let util = usage.utilization();
    assert!((util - 0.5).abs() < f64::EPSILON);

    let empty = TokenUsage::new(0);
    assert!((empty.utilization() - 0.0).abs() < f64::EPSILON);
}

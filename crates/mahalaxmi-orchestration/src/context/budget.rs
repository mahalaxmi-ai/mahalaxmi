//! Token budget allocation and tracking for context preparation.
//!
//! Divides the available context window into sections (repo map, files,
//! memory, task description) according to configurable percentages.

use mahalaxmi_core::config::{BudgetAllocations, ContextConfig};
use mahalaxmi_providers::types::ProviderCapabilities;

/// Estimates the number of tokens in a text string.
///
/// Uses the common heuristic of ~4 characters per token.
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Token budget for context preparation.
///
/// Tracks how many tokens are available overall and per-section,
/// based on provider capabilities and budget allocation percentages.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    total_tokens: usize,
    allocations: BudgetAllocations,
}

impl TokenBudget {
    /// Create a token budget from provider capabilities and context config.
    ///
    /// Uses 80% of the provider's context window as the total budget,
    /// leaving room for the worker's own output. If the provider reports
    /// 0 max tokens, defaults to 8192.
    pub fn from_provider(capabilities: &ProviderCapabilities, config: &ContextConfig) -> Self {
        let max_tokens = if capabilities.max_context_tokens == 0 {
            8192
        } else {
            (capabilities.max_context_tokens as f64 * 0.8) as usize
        };
        Self {
            total_tokens: max_tokens,
            allocations: config.budget_allocations.clone(),
        }
    }

    /// Create a token budget from a fixed total and allocation percentages.
    pub fn from_total(total_tokens: usize, allocations: BudgetAllocations) -> Self {
        Self {
            total_tokens,
            allocations,
        }
    }

    /// Returns the total token budget.
    pub fn total(&self) -> usize {
        self.total_tokens
    }

    /// Returns the token budget for the repo map section.
    pub fn tokens_for_repo_map(&self) -> usize {
        (self.total_tokens as f64 * self.allocations.repo_map_pct) as usize
    }

    /// Returns the token budget for the relevant files section.
    pub fn tokens_for_files(&self) -> usize {
        (self.total_tokens as f64 * self.allocations.relevant_files_pct) as usize
    }

    /// Returns the token budget for the shared memory section.
    pub fn tokens_for_memory(&self) -> usize {
        (self.total_tokens as f64 * self.allocations.memory_pct) as usize
    }

    /// Returns the token budget for the task description section.
    pub fn tokens_for_task(&self) -> usize {
        (self.total_tokens as f64 * self.allocations.task_description_pct) as usize
    }

    /// Returns how many tokens remain after `used` have been consumed.
    pub fn remaining(&self, used: usize) -> usize {
        self.total_tokens.saturating_sub(used)
    }

    /// Returns a reference to the budget allocations.
    pub fn allocations(&self) -> &BudgetAllocations {
        &self.allocations
    }
}

/// Tracks actual token usage across context sections.
#[derive(Debug, Clone)]
pub struct TokenUsage {
    /// Tokens used by the repo map section.
    pub repo_map: usize,
    /// Tokens used by file content sections.
    pub files: usize,
    /// Tokens used by shared memory entries.
    pub memory: usize,
    /// Tokens used by the task description.
    pub task: usize,
    /// Total tokens used across all sections.
    pub total: usize,
    /// Total token budget.
    pub budget: usize,
}

impl TokenUsage {
    /// Create a new usage tracker with the given budget.
    pub fn new(budget: usize) -> Self {
        Self {
            repo_map: 0,
            files: 0,
            memory: 0,
            task: 0,
            total: 0,
            budget,
        }
    }

    /// Record tokens used by the repo map section.
    pub fn add_repo_map(&mut self, tokens: usize) {
        self.repo_map += tokens;
        self.total += tokens;
    }

    /// Record tokens used by file content.
    pub fn add_files(&mut self, tokens: usize) {
        self.files += tokens;
        self.total += tokens;
    }

    /// Record tokens used by shared memory.
    pub fn add_memory(&mut self, tokens: usize) {
        self.memory += tokens;
        self.total += tokens;
    }

    /// Record tokens used by the task description.
    pub fn add_task(&mut self, tokens: usize) {
        self.task += tokens;
        self.total += tokens;
    }

    /// Returns true if total usage exceeds the budget.
    pub fn is_over_budget(&self) -> bool {
        self.total > self.budget
    }

    /// Returns the fraction of budget used (0.0 to 1.0+).
    pub fn utilization(&self) -> f64 {
        if self.budget == 0 {
            return 0.0;
        }
        self.total as f64 / self.budget as f64
    }
}

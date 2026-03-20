use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Categories of tasks that AI providers can handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// Generating new code from scratch.
    CodeGeneration,
    /// Reviewing existing code for issues and improvements.
    CodeReview,
    /// Finding and fixing bugs.
    Debugging,
    /// Restructuring code without changing behavior.
    Refactoring,
    /// Writing unit, integration, or end-to-end tests.
    Testing,
    /// Writing documentation, comments, or READMEs.
    Documentation,
    /// Planning and designing architecture or features.
    Planning,
    /// General-purpose tasks that don't fit a specific category.
    General,
}

impl TaskType {
    /// All task types as a slice.
    pub fn all() -> &'static [TaskType] {
        &[
            Self::CodeGeneration,
            Self::CodeReview,
            Self::Debugging,
            Self::Refactoring,
            Self::Testing,
            Self::Documentation,
            Self::Planning,
            Self::General,
        ]
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CodeGeneration => write!(f, "code_generation"),
            Self::CodeReview => write!(f, "code_review"),
            Self::Debugging => write!(f, "debugging"),
            Self::Refactoring => write!(f, "refactoring"),
            Self::Testing => write!(f, "testing"),
            Self::Documentation => write!(f, "documentation"),
            Self::Planning => write!(f, "planning"),
            Self::General => write!(f, "general"),
        }
    }
}

/// Cost tier for a provider — indicates relative expense of usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostTier {
    /// Free or open-source (e.g., local Ollama models).
    Free,
    /// Low cost per token/request.
    Low,
    /// Medium cost per token/request.
    Medium,
    /// High cost per token/request.
    High,
    /// Premium tier (e.g., large reasoning models).
    Premium,
}

impl CostTier {
    /// Numeric weight for cost comparison (0 = free, 4 = premium).
    pub fn weight(&self) -> u32 {
        match self {
            Self::Free => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Premium => 4,
        }
    }
}

impl std::fmt::Display for CostTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free => write!(f, "free"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Premium => write!(f, "premium"),
        }
    }
}

/// Proficiency rating for a provider on a specific task type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Proficiency {
    /// Not capable of this task type.
    None,
    /// Basic capability, may need supervision.
    Basic,
    /// Good capability, suitable for most tasks.
    Good,
    /// Excellent capability, provider strength.
    Excellent,
}

impl Proficiency {
    /// Numeric score for routing comparisons (0 = none, 3 = excellent).
    pub fn score(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::Basic => 1,
            Self::Good => 2,
            Self::Excellent => 3,
        }
    }
}

/// Describes the capabilities and limits of an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    /// Whether this provider supports streaming output.
    pub supports_streaming: bool,
    /// Whether this provider supports managing multiple agents as a team.
    pub supports_agent_teams: bool,
    /// Whether this provider supports tool use / function calling.
    pub supports_tool_use: bool,
    /// Maximum context window size in tokens (0 if unknown).
    pub max_context_tokens: u64,
    /// Cost tier for this provider.
    pub cost_tier: CostTier,
    /// Expected average latency in milliseconds for a typical request (0 if unknown).
    pub avg_latency_ms: u64,
    /// Whether this provider supports concurrent sessions.
    pub supports_concurrent_sessions: bool,
    /// Proficiency ratings per task type.
    pub task_proficiency: HashMap<TaskType, Proficiency>,
    /// Whether this provider runs entirely locally (no cloud API calls).
    /// True for Ollama and other on-device models.
    pub supports_local_only: bool,
    /// Whether this provider supports web search capabilities.
    /// True for Gemini CLI, Grok CLI.
    pub supports_web_search: bool,
    /// Whether this provider supports native structured JSON output without markdown fences.
    /// When true, the validator prompt instructs the model to return raw JSON; the driver
    /// parses via serde_json::from_str directly instead of the markdown-extraction heuristic.
    #[serde(default)]
    pub supports_structured_output: bool,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            supports_streaming: true,
            supports_agent_teams: false,
            supports_tool_use: false,
            max_context_tokens: 0,
            cost_tier: CostTier::Medium,
            avg_latency_ms: 0,
            supports_concurrent_sessions: true,
            task_proficiency: HashMap::new(),
            supports_local_only: false,
            supports_web_search: false,
            supports_structured_output: false,
        }
    }
}

impl ProviderCapabilities {
    /// Get the proficiency for a specific task type.
    ///
    /// Returns `Proficiency::Good` as default if no explicit rating exists.
    pub fn proficiency_for(&self, task_type: TaskType) -> Proficiency {
        self.task_proficiency
            .get(&task_type)
            .copied()
            .unwrap_or(Proficiency::Good)
    }

    /// Check if the provider can handle a specific task type.
    ///
    /// Returns false only if proficiency is explicitly `None`.
    pub fn supports_task(&self, task_type: TaskType) -> bool {
        self.proficiency_for(task_type) != Proficiency::None
    }

    /// Calculate a routing score for a given task type.
    ///
    /// Higher scores indicate better suitability. Considers proficiency,
    /// cost, and context window size.
    pub fn routing_score(&self, task_type: TaskType) -> u32 {
        let proficiency = self.proficiency_for(task_type).score();
        let cost_bonus = 4u32.saturating_sub(self.cost_tier.weight());
        let context_bonus = if self.max_context_tokens >= 100_000 {
            2
        } else if self.max_context_tokens >= 32_000 {
            1
        } else {
            0
        };
        proficiency * 10 + cost_bonus + context_bonus
    }
}

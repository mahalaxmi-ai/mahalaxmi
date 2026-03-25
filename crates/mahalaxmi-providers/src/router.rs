//! Task router — assigns tasks to the optimal AI provider based on capabilities.
//!
//! The `TaskRouter` evaluates all registered providers against a task's requirements
//! (task type, complexity, context size) and selects the best match using a scoring
//! algorithm that considers proficiency, cost, context capacity, and availability.

use mahalaxmi_core::types::ProviderId;
use serde::{Deserialize, Serialize};

use crate::credentials::{AuthMode, ProviderStatus};
use crate::metadata::find_binary;
use crate::metrics::{PerformanceTracker, ProviderMetrics};
use crate::registry::ProviderRegistry;
use crate::traits::AiProvider;
use crate::types::{CostTier, ProviderCapabilities, TaskType};

/// Routing decision for a single task.
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// The selected provider ID.
    pub provider_id: ProviderId,
    /// Routing score (higher = better fit).
    pub score: u32,
    /// Human-readable reason for the selection.
    pub reason: String,
}

/// User-facing routing strategy — controls how provider selection is optimized.
///
/// Users choose a strategy; Mahalaxmi handles the scoring details automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    /// Maximize task quality — proficiency dominates scoring (default).
    #[default]
    QualityFirst,
    /// Minimize cost — prefer cheaper providers that meet minimum proficiency.
    CostOptimized,
    /// Minimize latency — prefer fastest providers for interactive workflows.
    SpeedFirst,
}

/// Constraints that influence provider selection.
#[derive(Debug, Clone, Default)]
pub struct RoutingConstraints {
    /// Maximum cost tier allowed.
    pub max_cost_tier: Option<CostTier>,
    /// Minimum context window required (in tokens).
    pub min_context_tokens: Option<u64>,
    /// Preferred provider ID (gets a bonus).
    pub preferred_provider: Option<ProviderId>,
    /// Providers to exclude from selection.
    pub excluded_providers: Vec<ProviderId>,
    /// Routing strategy (default: QualityFirst).
    pub strategy: RoutingStrategy,
    /// Task complexity (1-10). Higher complexity steers toward better providers.
    /// When set, simple tasks (1-3) can use cheaper providers even in QualityFirst mode.
    pub complexity: Option<u8>,
}

/// Calculate a strategy-aware routing score for a provider.
///
/// The scoring formula changes based on strategy:
/// - **QualityFirst**: `proficiency * 10 + cost_bonus + context_bonus`
/// - **CostOptimized**: `cost_bonus * 10 + proficiency + context_bonus`
/// - **SpeedFirst**: `speed_bonus * 10 + proficiency + context_bonus`
///
/// When `complexity` is set, simple tasks (1-3) get extra cost bonus and
/// complex tasks (8-10) get extra proficiency bonus, regardless of strategy.
pub fn strategy_score(
    caps: &ProviderCapabilities,
    task_type: TaskType,
    strategy: RoutingStrategy,
    complexity: Option<u8>,
    performance: Option<&ProviderMetrics>,
) -> u32 {
    let proficiency = caps.proficiency_for(task_type).score();
    let cost_bonus = 4u32.saturating_sub(caps.cost_tier.weight());
    let context_bonus = if caps.max_context_tokens >= 100_000 {
        2
    } else if caps.max_context_tokens >= 32_000 {
        1
    } else {
        0
    };

    // Latency bonus: lower latency = higher bonus (0-4 scale)
    let speed_bonus = if caps.avg_latency_ms == 0 {
        2 // unknown latency gets neutral score
    } else if caps.avg_latency_ms <= 2000 {
        4
    } else if caps.avg_latency_ms <= 3500 {
        3
    } else if caps.avg_latency_ms <= 5000 {
        2
    } else if caps.avg_latency_ms <= 8000 {
        1
    } else {
        0
    };

    let base_score = match strategy {
        RoutingStrategy::QualityFirst => proficiency * 10 + cost_bonus + context_bonus,
        RoutingStrategy::CostOptimized => cost_bonus * 10 + proficiency + context_bonus,
        RoutingStrategy::SpeedFirst => speed_bonus * 10 + proficiency + context_bonus,
    };

    // Complexity adjustment: simple tasks favor cost, complex tasks favor quality
    let complexity_adj = match complexity {
        Some(c) if c <= 3 => cost_bonus * 2, // simple → extra cost bonus
        Some(c) if c >= 8 => proficiency * 3, // complex → extra proficiency bonus
        _ => 0,                              // moderate → no adjustment
    };

    // Performance affinity: providers with proven track records get a bonus
    let affinity_bonus = match performance {
        Some(metrics) if metrics.total_requests > 5 => {
            let success_rate = metrics.success_rate();
            (success_rate * 4.0) as u32 // 0-4 based on success rate
        }
        _ => 0,
    };

    base_score + complexity_adj + affinity_bonus
}

/// Routes tasks to the optimal AI provider from a registry.
///
/// The router evaluates all available providers and picks the one with the
/// highest routing score for the given task type and constraints.
pub struct TaskRouter;

impl TaskRouter {
    /// Route a task to the best available provider.
    ///
    /// Returns `None` if no provider can handle the task.
    pub fn route(
        registry: &ProviderRegistry,
        task_type: TaskType,
        constraints: &RoutingConstraints,
    ) -> Option<RoutingDecision> {
        Self::route_with_tracker(registry, task_type, constraints, None)
    }

    /// Route a task using historical performance data for affinity scoring.
    ///
    /// Providers with higher success rates get a routing bonus.
    pub fn route_with_tracker(
        registry: &ProviderRegistry,
        task_type: TaskType,
        constraints: &RoutingConstraints,
        tracker: Option<&PerformanceTracker>,
    ) -> Option<RoutingDecision> {
        let candidates = Self::score_candidates(registry, task_type, constraints, tracker);

        candidates.first().map(|(provider, score)| RoutingDecision {
            provider_id: provider.id().clone(),
            score: *score,
            reason: format!(
                "{} selected for {} (score: {}, strategy: {:?})",
                provider.name(),
                task_type,
                score,
                constraints.strategy
            ),
        })
    }

    /// Route a task with fallback — returns a ranked list of providers.
    ///
    /// Useful for resilience: if the first provider fails, try the next one.
    pub fn route_with_fallbacks(
        registry: &ProviderRegistry,
        task_type: TaskType,
        constraints: &RoutingConstraints,
        max_fallbacks: usize,
    ) -> Vec<RoutingDecision> {
        Self::route_with_fallbacks_and_tracker(
            registry,
            task_type,
            constraints,
            max_fallbacks,
            None,
        )
    }

    /// Route with fallbacks using historical performance data.
    pub fn route_with_fallbacks_and_tracker(
        registry: &ProviderRegistry,
        task_type: TaskType,
        constraints: &RoutingConstraints,
        max_fallbacks: usize,
        tracker: Option<&PerformanceTracker>,
    ) -> Vec<RoutingDecision> {
        let candidates = Self::score_candidates(registry, task_type, constraints, tracker);

        candidates
            .iter()
            .take(max_fallbacks + 1)
            .map(|(provider, score)| RoutingDecision {
                provider_id: provider.id().clone(),
                score: *score,
                reason: format!(
                    "{} for {} (score: {}, strategy: {:?})",
                    provider.name(),
                    task_type,
                    score,
                    constraints.strategy
                ),
            })
            .collect()
    }

    /// Score and rank all eligible providers for a task.
    fn score_candidates<'a>(
        registry: &'a ProviderRegistry,
        task_type: TaskType,
        constraints: &RoutingConstraints,
        tracker: Option<&PerformanceTracker>,
    ) -> Vec<(&'a dyn AiProvider, u32)> {
        let mut candidates: Vec<(&dyn AiProvider, u32)> = Vec::new();

        for provider_id in registry.list() {
            let provider = match registry.get(provider_id) {
                Some(p) => p,
                None => continue,
            };

            if constraints.excluded_providers.contains(provider_id) {
                continue;
            }

            // Skip providers whose binary is not installed or whose credentials
            // are not configured — they would fail immediately at spawn time.
            if !is_provider_usable(provider) {
                continue;
            }

            let caps = provider.capabilities();

            if !caps.supports_task(task_type) {
                continue;
            }

            if let Some(max_cost) = &constraints.max_cost_tier {
                if caps.cost_tier > *max_cost {
                    continue;
                }
            }

            if let Some(min_ctx) = constraints.min_context_tokens {
                if caps.max_context_tokens > 0 && caps.max_context_tokens < min_ctx {
                    continue;
                }
            }

            let perf_metrics = tracker.and_then(|t| t.get_metrics(provider_id));

            let mut score = strategy_score(
                caps,
                task_type,
                constraints.strategy,
                constraints.complexity,
                perf_metrics,
            );

            if let Some(pref) = &constraints.preferred_provider {
                if provider_id == pref {
                    score += 5;
                }
            }

            candidates.push((provider, score));
        }

        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        candidates
    }

    /// Check the readiness status of a provider (binary availability).
    pub fn check_provider_status(provider: &dyn AiProvider) -> ProviderStatus {
        if find_binary(provider.cli_binary()).is_some() {
            ProviderStatus::Ready
        } else {
            ProviderStatus::NotInstalled
        }
    }
}

/// Return `true` if a provider is usable right now — binary installed and
/// credentials available.
///
/// A provider is considered usable when:
/// 1. Its CLI binary exists on PATH.
/// 2. It has at least one credential-free auth mode (`None` or `CliLogin`),
///    OR at least one `ApiKey` auth mode whose env var is currently set.
///
/// This prevents uninstalled or unconfigured providers from ever appearing in
/// the routing candidates or fallback chain, saving the orchestration engine
/// from wasted spawn attempts that always fail.
fn is_provider_usable(provider: &dyn AiProvider) -> bool {
    if find_binary(provider.cli_binary()).is_none() {
        return false;
    }

    let auth_modes = &provider.metadata().auth_modes;

    // Credential-free auth modes don't require an env var to be set.
    let has_credential_free = auth_modes
        .iter()
        .any(|m| matches!(m, AuthMode::None | AuthMode::CliLogin { .. }));
    if has_credential_free {
        return true;
    }

    // All remaining modes require API keys — at least one must be configured.
    auth_modes.iter().any(|m| {
        if let AuthMode::ApiKey { env_var } = m {
            !std::env::var(env_var).unwrap_or_default().is_empty()
        } else {
            false
        }
    })
}

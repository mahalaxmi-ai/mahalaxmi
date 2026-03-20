//! Orchestration-specific error types for plan approval and context routing.
//!
//! `OrchestrationError` is a lightweight, thiserror-derived error enum used by
//! the orchestration driver for plan-approval flow control.  Context router
//! types are defined here as the canonical interface that the Phase 5 context
//! routing subsystem implements.

use thiserror::Error;

// =============================================================================
// Orchestration driver errors
// =============================================================================

/// Errors that arise during orchestration cycle execution.
///
/// Distinct from [`mahalaxmi_core::error::MahalaxmiError`] — this enum
/// carries only the driver-specific plan-approval variants and is mapped to
/// `String` at the `Result<(), String>` boundary of the async driver.
#[derive(Debug, Error)]
pub enum OrchestrationError {
    /// Developer did not approve the plan within the configured timeout.
    #[error("Developer did not approve the plan within the configured timeout.")]
    PlanApprovalTimeout,
    /// Plan approval channel was closed before a response was received.
    #[error("Plan approval channel was closed before a response was received.")]
    PlanApprovalDropped,
}

// =============================================================================
// Context routing interface (Phase 5 integration surface)
// =============================================================================

/// Scored file context produced by the context router.
#[derive(Debug, Clone, Default)]
pub struct ScoredContext {
    /// Files ranked by relevance to the worker task.
    pub files: Vec<FileScore>,
    /// Estimated token count for all included file contents.
    pub estimated_tokens: usize,
}

/// A single file with its relevance score.
#[derive(Debug, Clone)]
pub struct FileScore {
    /// Relative file path within the project.
    pub path: String,
    /// Normalised relevance score in the range \[0.0, 1.0\].
    pub score: f64,
}

/// Weights and token budget for the three-signal context scoring algorithm.
///
/// All weight fields (`alpha`, `beta`, `gamma`) are in `[0.0, 1.0]` and
/// should sum to 1.0.  The router does not enforce the sum — callers are
/// responsible for providing sensible values.
#[derive(Debug, Clone)]
pub struct ContextRouterConfig {
    /// Weight for the lexical-overlap signal. Default 0.5.
    pub alpha: f64,
    /// Weight for the graph-proximity signal. Default 0.3.
    pub beta: f64,
    /// Weight for the historical co-occurrence signal. Default 0.2.
    pub gamma: f64,
    /// Maximum token budget for injected file contents. Default 8192.
    pub token_budget: usize,
}

impl Default for ContextRouterConfig {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            beta: 0.3,
            gamma: 0.2,
            token_budget: 8192,
        }
    }
}

/// Trait implemented by context routers that rank project files by relevance to
/// an individual worker task.
///
/// The Phase 5 worker provides the full implementation (`DefaultContextRouter`).
/// All integrations use this trait so swapping implementations requires no
/// changes to the call sites.
pub trait ContextRouter: Send + Sync {
    /// Rank project files by relevance to `task`.
    ///
    /// `index` provides the codebase symbol index.  `last_report` is the
    /// previous cycle's report, used for historical co-occurrence scoring.
    /// `config` controls signal weights and the token budget.
    fn route(
        &self,
        task: &crate::models::plan::WorkerTask,
        index: &mahalaxmi_indexing::CodebaseIndex,
        last_report: Option<&crate::models::report::CycleReport>,
        config: &ContextRouterConfig,
    ) -> ScoredContext;
}

/// Default no-op context router used when the full Phase 5 implementation is
/// not yet available.  Returns an empty `ScoredContext`, causing the prompt
/// builder to fall back to the existing `repo_map`-based context path.
pub struct DefaultContextRouter;

impl ContextRouter for DefaultContextRouter {
    fn route(
        &self,
        _task: &crate::models::plan::WorkerTask,
        _index: &mahalaxmi_indexing::CodebaseIndex,
        _last_report: Option<&crate::models::report::CycleReport>,
        _config: &ContextRouterConfig,
    ) -> ScoredContext {
        ScoredContext::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_approval_timeout_display() {
        let err = OrchestrationError::PlanApprovalTimeout;
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn plan_approval_dropped_display() {
        let err = OrchestrationError::PlanApprovalDropped;
        assert!(err.to_string().contains("closed"));
    }

    #[test]
    fn context_router_config_defaults() {
        let config = ContextRouterConfig::default();
        assert!((config.alpha - 0.5).abs() < f64::EPSILON);
        assert!((config.beta - 0.3).abs() < f64::EPSILON);
        assert!((config.gamma - 0.2).abs() < f64::EPSILON);
        assert_eq!(config.token_budget, 8192);
    }

    #[test]
    fn scored_context_default_is_empty() {
        let ctx = ScoredContext::default();
        assert!(ctx.files.is_empty());
        assert_eq!(ctx.estimated_tokens, 0);
    }
}

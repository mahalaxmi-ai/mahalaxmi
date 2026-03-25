//! Intelligent context routing for worker activation.
//!
//! Combines three orthogonal signals — lexical overlap, import-graph proximity,
//! and historical co-occurrence — into a single ranked file list that fits
//! within the worker's token budget.

use mahalaxmi_indexing::CodebaseIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::models::plan::WorkerTask;
use crate::models::report::CycleReport;

use super::{graph_proximity, historical, lexical};

// ── Default helpers ───────────────────────────────────────────────────────────

fn default_alpha() -> f64 {
    0.5
}

fn default_beta() -> f64 {
    0.3
}

fn default_gamma() -> f64 {
    0.2
}

fn default_token_budget() -> usize {
    8192
}

// ── Config ────────────────────────────────────────────────────────────────────

/// Weights and token budget for the three-signal context router.
///
/// All weights must be non-negative. Scores from each signal are independently
/// normalised to `[0, 1]` before weighting, so the weights control relative
/// influence rather than raw magnitude.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRouterConfig {
    /// Weight applied to the lexical similarity signal.
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    /// Weight applied to the import-graph proximity signal.
    #[serde(default = "default_beta")]
    pub beta: f64,
    /// Weight applied to the historical co-occurrence signal.
    #[serde(default = "default_gamma")]
    pub gamma: f64,
    /// Maximum number of tokens (estimated at 4 bytes per token) to include
    /// in the scored context returned to the worker.
    #[serde(default = "default_token_budget")]
    pub token_budget: usize,
}

impl Default for ContextRouterConfig {
    fn default() -> Self {
        Self {
            alpha: default_alpha(),
            beta: default_beta(),
            gamma: default_gamma(),
            token_budget: default_token_budget(),
        }
    }
}

// ── Output types ──────────────────────────────────────────────────────────────

/// Score breakdown for a single file included in routed context.
#[derive(Debug, Clone)]
pub struct FileScore {
    /// Absolute or project-relative path to the file.
    pub path: PathBuf,
    /// Combined weighted score (alpha·lexical + beta·graph + gamma·historical).
    pub score: f64,
    /// Raw lexical similarity score in `[0, 1]`.
    pub lexical: f64,
    /// Raw graph-proximity score in `[0, 1]`.
    pub graph: f64,
    /// Raw historical co-occurrence score in `[0, 1]`.
    pub historical: f64,
}

/// The result of a context routing pass — files sorted by score within budget.
#[derive(Debug, Default)]
pub struct ScoredContext {
    /// Files to include in the worker's context, sorted by combined score descending.
    pub files: Vec<FileScore>,
    /// Estimated total token count for all included files (4 bytes per token).
    pub estimated_tokens: usize,
}

// ── Trait ─────────────────────────────────────────────────────────────────────

/// Routes codebase files for a worker based on task content and index signals.
///
/// Implementations must be `Send + Sync` so they can be called from async
/// orchestration tasks.
pub trait ContextRouter: Send + Sync {
    /// Scores all indexed files and returns the subset that fits within the
    /// configured `token_budget`, sorted by combined score descending.
    fn route(
        &self,
        task: &WorkerTask,
        index: &CodebaseIndex,
        last_report: Option<&CycleReport>,
        config: &ContextRouterConfig,
    ) -> ScoredContext;
}

// ── Default implementation ────────────────────────────────────────────────────

/// Production implementation of [`ContextRouter`] combining lexical,
/// graph-proximity, and historical co-occurrence signals.
pub struct DefaultContextRouter;

impl ContextRouter for DefaultContextRouter {
    fn route(
        &self,
        task: &WorkerTask,
        index: &CodebaseIndex,
        last_report: Option<&CycleReport>,
        config: &ContextRouterConfig,
    ) -> ScoredContext {
        // Compute per-signal score maps.
        let lexical_scores: HashMap<PathBuf, f64> = lexical::score_all(task, index);
        let graph_scores: HashMap<PathBuf, f64> = graph_proximity::score_all(task, index);
        let hist_scores: HashMap<PathBuf, f64> = historical::score_all(task, last_report);

        // Union all path keys from all three maps.
        let mut all_paths: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
        all_paths.extend(lexical_scores.keys().cloned());
        all_paths.extend(graph_scores.keys().cloned());
        all_paths.extend(hist_scores.keys().cloned());

        // Compute combined score for each path, filtering out zero scores.
        let mut file_scores: Vec<FileScore> = all_paths
            .into_iter()
            .filter_map(|path| {
                let l = lexical_scores.get(&path).copied().unwrap_or(0.0);
                let g = graph_scores.get(&path).copied().unwrap_or(0.0);
                let h = hist_scores.get(&path).copied().unwrap_or(0.0);
                let combined = config.alpha * l + config.beta * g + config.gamma * h;
                if combined > 0.0 {
                    Some(FileScore {
                        path,
                        score: combined,
                        lexical: l,
                        graph: g,
                        historical: h,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort descending by combined score; NaN falls back to Equal.
        file_scores.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply token budget: include files until adding the next would exceed the limit.
        let mut estimated_tokens: usize = 0;
        let mut included: Vec<FileScore> = Vec::new();

        for file_score in file_scores {
            let file_tokens = index
                .fingerprint(&file_score.path)
                .map(|fp| (fp.size_bytes as usize).saturating_div(4))
                .unwrap_or(0);
            if estimated_tokens + file_tokens > config.token_budget {
                break;
            }
            estimated_tokens += file_tokens;
            included.push(file_score);
        }

        ScoredContext {
            files: included,
            estimated_tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_router_config_default_weights() {
        let config = ContextRouterConfig::default();
        assert!((config.alpha - 0.5).abs() < f64::EPSILON);
        assert!((config.beta - 0.3).abs() < f64::EPSILON);
        assert!((config.gamma - 0.2).abs() < f64::EPSILON);
        assert_eq!(config.token_budget, 8192);
    }

    #[test]
    fn context_router_config_serde_roundtrip() {
        let config = ContextRouterConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: ContextRouterConfig = serde_json::from_str(&json).expect("deserialize");
        assert!((deserialized.alpha - config.alpha).abs() < f64::EPSILON);
        assert!((deserialized.beta - config.beta).abs() < f64::EPSILON);
        assert!((deserialized.gamma - config.gamma).abs() < f64::EPSILON);
        assert_eq!(deserialized.token_budget, config.token_budget);
    }

    #[test]
    fn scored_context_default_is_empty() {
        let ctx = ScoredContext::default();
        assert!(ctx.files.is_empty());
        assert_eq!(ctx.estimated_tokens, 0);
    }

    #[test]
    fn router_with_empty_signals_returns_empty() {
        use mahalaxmi_core::config::IndexingConfig;
        use mahalaxmi_core::i18n::locale::SupportedLocale;
        use mahalaxmi_core::i18n::I18nService;
        use mahalaxmi_core::types::{TaskId, WorkerId};
        use mahalaxmi_indexing::CodebaseIndex;

        let i18n = I18nService::new(SupportedLocale::EnUs);
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        // Write a file with content unrelated to any task we'll create.
        std::fs::write(root.join("zzzunrelated.rs"), "fn zzzunrelated() {}").expect("write");

        let config = IndexingConfig::default();
        let index = CodebaseIndex::build(&root, &config, &i18n).expect("build index");

        let task =
            crate::models::plan::WorkerTask::new(TaskId::new("task-0"), WorkerId::new(0), "", "");

        let router = DefaultContextRouter;
        let router_config = ContextRouterConfig::default();
        let ctx = router.route(&task, &index, None, &router_config);
        // Empty task title/description → lexical scores are all zero.
        // No affected files → graph scores are empty.
        // No report → historical is empty.
        assert!(ctx.files.is_empty());
        assert_eq!(ctx.estimated_tokens, 0);
    }

    #[test]
    fn router_respects_token_budget() {
        use mahalaxmi_core::config::IndexingConfig;
        use mahalaxmi_core::i18n::locale::SupportedLocale;
        use mahalaxmi_core::i18n::I18nService;
        use mahalaxmi_core::types::{TaskId, WorkerId};
        use mahalaxmi_indexing::CodebaseIndex;

        let i18n = I18nService::new(SupportedLocale::EnUs);
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();

        // Write a file that matches the task keywords.
        std::fs::write(root.join("auth_handler.rs"), "pub fn auth_handler() {}")
            .expect("write auth");

        let config = IndexingConfig::default();
        let index = CodebaseIndex::build(&root, &config, &i18n).expect("build index");

        let task = crate::models::plan::WorkerTask::new(
            TaskId::new("task-0"),
            WorkerId::new(0),
            "auth handler implementation",
            "Implement the auth handler module",
        );

        let router = DefaultContextRouter;
        // Token budget of 0 means nothing fits.
        let router_config = ContextRouterConfig {
            token_budget: 0,
            ..ContextRouterConfig::default()
        };
        let ctx = router.route(&task, &index, None, &router_config);
        assert_eq!(ctx.estimated_tokens, 0);
        assert!(ctx.files.is_empty());
    }

    #[test]
    fn router_sorted_descending_by_score() {
        use mahalaxmi_core::config::IndexingConfig;
        use mahalaxmi_core::i18n::locale::SupportedLocale;
        use mahalaxmi_core::i18n::I18nService;
        use mahalaxmi_core::types::{TaskId, WorkerId};
        use mahalaxmi_indexing::CodebaseIndex;

        let i18n = I18nService::new(SupportedLocale::EnUs);
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();

        // auth_handler.rs should score higher for an "auth" task than database_pool.rs.
        std::fs::write(root.join("auth_handler.rs"), "pub fn auth_handler() {}")
            .expect("write auth");
        std::fs::write(root.join("database_pool.rs"), "pub fn database_pool() {}")
            .expect("write db");

        let config = IndexingConfig::default();
        let index = CodebaseIndex::build(&root, &config, &i18n).expect("build index");

        let task = crate::models::plan::WorkerTask::new(
            TaskId::new("task-0"),
            WorkerId::new(0),
            "auth handler",
            "Implement the auth handler",
        );

        let router = DefaultContextRouter;
        let router_config = ContextRouterConfig::default();
        let ctx = router.route(&task, &index, None, &router_config);

        // Verify descending order.
        let scores: Vec<f64> = ctx.files.iter().map(|f| f.score).collect();
        for window in scores.windows(2) {
            assert!(
                window[0] >= window[1],
                "scores must be non-increasing: {} < {}",
                window[0],
                window[1]
            );
        }
    }
}

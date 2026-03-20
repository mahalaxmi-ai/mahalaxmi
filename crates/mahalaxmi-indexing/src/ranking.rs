//! Symbol importance ranking using a PageRank-style algorithm.
//!
//! Scores files and symbols based on the file dependency graph structure.
//! Files that are imported by many other files receive higher scores, and
//! symbols inherit their file's score multiplied by a kind-specific bonus.
//! Used to prioritize which symbols appear in token-budgeted repo maps.

use crate::graph::FileDependencyGraph;
use crate::types::{ExtractedSymbol, SymbolKind};
use mahalaxmi_core::config::IndexingConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration for the PageRank-style ranking algorithm.
///
/// Controls the damping factor, iteration limit, and convergence threshold
/// for the iterative ranking computation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankingConfig {
    /// Damping factor for PageRank (probability of following a link vs. random jump).
    /// Standard value is 0.85.
    pub damping_factor: f64,
    /// Maximum number of iterations before stopping, even without convergence.
    pub max_iterations: usize,
    /// Stop iterating when the maximum score change is below this threshold.
    pub convergence_threshold: f64,
}

impl RankingConfig {
    /// Creates a new ranking configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a ranking configuration from the global indexing config.
    pub fn from_indexing_config(config: &IndexingConfig) -> Self {
        Self {
            damping_factor: config.ranking_damping_factor,
            max_iterations: config.ranking_max_iterations,
            convergence_threshold: 1e-6,
        }
    }
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            damping_factor: 0.85,
            max_iterations: 20,
            convergence_threshold: 1e-6,
        }
    }
}

/// Ranked importance scores for symbols and files.
///
/// Computed using a PageRank-style algorithm on the file dependency graph.
/// Files imported by many other files receive higher scores, and symbols
/// inherit their file's score with a kind-specific multiplier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolRanking {
    /// Symbol qualified name -> importance score.
    scores: HashMap<String, f64>,
    /// File path -> PageRank score.
    file_scores: HashMap<PathBuf, f64>,
}

impl SymbolRanking {
    /// Computes symbol and file rankings from the dependency graph.
    ///
    /// Algorithm:
    /// 1. Collects all files from the graph.
    /// 2. Initializes all file scores to `1.0 / file_count`.
    /// 3. Iterates PageRank until convergence or max iterations:
    ///    - New score = `(1 - damping) / N + damping * sum(score[dep] / out_degree[dep])`
    ///      for all files that point TO this file (its dependents in the reverse graph).
    /// 4. Assigns symbol scores as `file_pagerank * kind_bonus`.
    pub fn compute(
        symbols: &[ExtractedSymbol],
        graph: &FileDependencyGraph,
        config: &RankingConfig,
    ) -> Self {
        let files = graph.files();
        let file_count = files.len();

        if file_count == 0 {
            return Self::default();
        }

        let n = file_count as f64;
        let initial_score = 1.0 / n;

        // Initialize file scores
        let mut file_scores: HashMap<PathBuf, f64> = HashMap::new();
        for file in &files {
            file_scores.insert((*file).clone(), initial_score);
        }

        // Iterative PageRank computation
        for _ in 0..config.max_iterations {
            let mut new_scores: HashMap<PathBuf, f64> = HashMap::new();
            let mut max_delta: f64 = 0.0;

            for file in &files {
                // Sum contributions from all files that depend on this file
                // (files that have this file as a dependency, i.e., dependents_of returns
                // files that import this file - those are in reverse_adjacency)
                // Wait: dependents_of returns files that import this file.
                // But for PageRank, we want: for each file that LINKS TO this file.
                // In our graph, adjacency[A] = [B] means A imports B (A -> B).
                // So B is "linked to" by A. dependents_of(B) = [A].
                // PageRank for B: sum over all A that link to B: score(A) / out_degree(A)
                // out_degree(A) = how many files A links to = adjacency[A].len() = A's imports count.
                let dependents = graph.dependents_of(file);

                let mut rank_sum = 0.0;
                for dependent in &dependents {
                    let dep_score = file_scores.get(dependent.as_path()).copied().unwrap_or(0.0);
                    let dep_out_degree = graph.out_degree(dependent);
                    if dep_out_degree > 0 {
                        rank_sum += dep_score / dep_out_degree as f64;
                    }
                }

                let new_score =
                    (1.0 - config.damping_factor) / n + config.damping_factor * rank_sum;

                let old_score = file_scores.get(file.as_path()).copied().unwrap_or(0.0);
                let delta = (new_score - old_score).abs();
                if delta > max_delta {
                    max_delta = delta;
                }

                new_scores.insert((*file).clone(), new_score);
            }

            file_scores = new_scores;

            // Check convergence
            if max_delta < config.convergence_threshold {
                break;
            }
        }

        // Assign symbol scores based on file PageRank and kind bonus
        let mut symbol_scores: HashMap<String, f64> = HashMap::new();
        for symbol in symbols {
            let file_score = file_scores.get(&symbol.file_path).copied().unwrap_or(1.0);
            let kind_bonus = Self::kind_bonus(symbol.kind);
            let score = file_score * kind_bonus;
            symbol_scores.insert(symbol.qualified_name(), score);
        }

        Self {
            scores: symbol_scores,
            file_scores,
        }
    }

    /// Returns the kind-specific bonus multiplier for a symbol kind.
    ///
    /// Higher-level abstractions like traits and interfaces get higher bonuses
    /// because they tend to define important contracts in a codebase.
    fn kind_bonus(kind: SymbolKind) -> f64 {
        match kind {
            SymbolKind::Trait | SymbolKind::Interface => 1.5,
            SymbolKind::Struct | SymbolKind::Class => 1.3,
            SymbolKind::Function => 1.0,
            SymbolKind::Method => 1.0,
            SymbolKind::Enum => 1.2,
            SymbolKind::TypeAlias => 0.9,
            SymbolKind::Constant => 0.8,
            SymbolKind::Module => 1.1,
            SymbolKind::Field => 0.5,
            SymbolKind::EnumVariant => 0.6,
        }
    }

    /// Returns the top N symbols by score in descending order.
    pub fn top_n(&self, n: usize) -> Vec<(&str, f64)> {
        let mut entries: Vec<(&str, f64)> = self
            .scores
            .iter()
            .map(|(name, &score)| (name.as_str(), score))
            .collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        entries.truncate(n);
        entries
    }

    /// Returns the ranking score for a specific symbol.
    ///
    /// Returns 0.0 if the symbol is not found in the ranking.
    pub fn score_for(&self, symbol_name: &str) -> f64 {
        self.scores.get(symbol_name).copied().unwrap_or(0.0)
    }

    /// Returns the PageRank score for a specific file.
    ///
    /// Returns 0.0 if the file is not found in the ranking.
    pub fn file_score(&self, file: &Path) -> f64 {
        self.file_scores.get(file).copied().unwrap_or(0.0)
    }

    /// Returns the top N files by PageRank score in descending order.
    pub fn top_files(&self, n: usize) -> Vec<(&Path, f64)> {
        let mut entries: Vec<(&Path, f64)> = self
            .file_scores
            .iter()
            .map(|(path, &score)| (path.as_path(), score))
            .collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        entries.truncate(n);
        entries
    }

    /// Returns the total number of ranked symbols.
    pub fn symbol_count(&self) -> usize {
        self.scores.len()
    }

    /// Returns true if no symbols have been ranked.
    pub fn is_empty(&self) -> bool {
        self.scores.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{FileDependency, FileDependencyGraph};
    use crate::types::{ExtractedSymbol, SymbolKind, Visibility};

    fn make_symbol(name: &str, kind: SymbolKind, file: &str) -> ExtractedSymbol {
        ExtractedSymbol::new(
            name.to_string(),
            kind,
            Visibility::Public,
            PathBuf::from(file),
            1,
            10,
        )
    }

    #[test]
    fn empty_graph_ranking() {
        let graph = FileDependencyGraph::new();
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&[], &graph, &config);
        assert!(ranking.is_empty());
        assert_eq!(ranking.symbol_count(), 0);
    }

    #[test]
    fn single_file_ranking() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/a.rs");

        let symbols = vec![make_symbol("foo", SymbolKind::Function, "/a.rs")];
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        assert_eq!(ranking.symbol_count(), 1);
        assert!(ranking.score_for("foo") > 0.0);
        assert!(ranking.file_score(Path::new("/a.rs")) > 0.0);
    }

    #[test]
    fn highly_imported_file_ranks_higher() {
        let mut graph = FileDependencyGraph::new();
        // Many files import /core.rs
        graph.add_dependency(FileDependency::new("/a.rs", "/core.rs", "core"));
        graph.add_dependency(FileDependency::new("/b.rs", "/core.rs", "core"));
        graph.add_dependency(FileDependency::new("/c.rs", "/core.rs", "core"));
        // /leaf.rs is imported by nobody
        graph.add_file("/leaf.rs");

        let symbols = vec![
            make_symbol("core_fn", SymbolKind::Function, "/core.rs"),
            make_symbol("leaf_fn", SymbolKind::Function, "/leaf.rs"),
        ];
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        assert!(
            ranking.file_score(Path::new("/core.rs")) > ranking.file_score(Path::new("/leaf.rs"))
        );
        assert!(ranking.score_for("core_fn") > ranking.score_for("leaf_fn"));
    }

    #[test]
    fn kind_bonus_affects_score() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/a.rs");

        let symbols = vec![
            make_symbol("MyTrait", SymbolKind::Trait, "/a.rs"),
            make_symbol("my_fn", SymbolKind::Function, "/a.rs"),
            make_symbol("my_field", SymbolKind::Field, "/a.rs"),
        ];
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        // Trait (1.5x) > Function (1.0x) > Field (0.5x)
        assert!(ranking.score_for("MyTrait") > ranking.score_for("my_fn"));
        assert!(ranking.score_for("my_fn") > ranking.score_for("my_field"));
    }

    #[test]
    fn top_n_returns_sorted() {
        let mut graph = FileDependencyGraph::new();
        graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "b"));
        graph.add_file("/c.rs");

        let symbols = vec![
            make_symbol("TraitX", SymbolKind::Trait, "/b.rs"),
            make_symbol("func_a", SymbolKind::Function, "/a.rs"),
            make_symbol("func_c", SymbolKind::Function, "/c.rs"),
        ];
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        let top = ranking.top_n(2);
        assert_eq!(top.len(), 2);
        // Scores should be in descending order
        assert!(top[0].1 >= top[1].1);
    }

    #[test]
    fn top_files_returns_sorted() {
        let mut graph = FileDependencyGraph::new();
        graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "b"));
        graph.add_dependency(FileDependency::new("/c.rs", "/b.rs", "b"));

        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&[], &graph, &config);

        let top = ranking.top_files(3);
        assert!(!top.is_empty());
        for i in 1..top.len() {
            assert!(top[i - 1].1 >= top[i].1);
        }
    }

    #[test]
    fn score_for_missing_symbol_returns_zero() {
        let ranking = SymbolRanking::default();
        assert_eq!(ranking.score_for("nonexistent"), 0.0);
    }

    #[test]
    fn file_score_missing_returns_zero() {
        let ranking = SymbolRanking::default();
        assert_eq!(ranking.file_score(Path::new("/nonexistent.rs")), 0.0);
    }

    #[test]
    fn ranking_config_from_indexing_config() {
        let indexing_config = IndexingConfig {
            ranking_damping_factor: 0.9,
            ranking_max_iterations: 50,
            ..Default::default()
        };
        let config = RankingConfig::from_indexing_config(&indexing_config);
        assert_eq!(config.damping_factor, 0.9);
        assert_eq!(config.max_iterations, 50);
    }

    #[test]
    fn default_ranking_config() {
        let config = RankingConfig::new();
        assert_eq!(config.damping_factor, 0.85);
        assert_eq!(config.max_iterations, 20);
        assert_eq!(config.convergence_threshold, 1e-6);
    }
}

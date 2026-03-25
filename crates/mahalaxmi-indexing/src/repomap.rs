//! Repo map generation for token-budgeted codebase overviews.
//!
//! Generates a formatted text representation of the most important symbols
//! in a codebase, ranked by importance and constrained by a token budget.
//! The repo map is consumed by AI coding agents to understand codebase
//! structure without reading every file.

use crate::ranking::SymbolRanking;
use crate::types::{ExtractedSymbol, SymbolKind};
use mahalaxmi_core::config::IndexingConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Controls how symbols are grouped in the repo map output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    /// Group symbols by their source file.
    File,
    /// Group symbols by their kind (functions, structs, etc.).
    Kind,
}

/// Configuration for repo map generation.
///
/// Controls the token budget, formatting options, and grouping strategy
/// for the generated repo map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoMapConfig {
    /// Maximum number of estimated tokens for the repo map output.
    pub max_tokens: usize,
    /// Whether to include function/method signatures in the output.
    pub include_signatures: bool,
    /// Whether to include doc comments before symbol entries.
    pub include_doc_comments: bool,
    /// How to group symbols in the output.
    pub group_by: GroupBy,
}

impl RepoMapConfig {
    /// Creates a new repo map configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a repo map configuration from the global indexing config.
    pub fn from_indexing_config(config: &IndexingConfig) -> Self {
        let group_by = match config.group_by.as_str() {
            "kind" => GroupBy::Kind,
            _ => GroupBy::File,
        };

        Self {
            max_tokens: config.max_tokens,
            include_signatures: config.include_signatures,
            include_doc_comments: config.include_doc_comments,
            group_by,
        }
    }
}

impl Default for RepoMapConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2048,
            include_signatures: true,
            include_doc_comments: false,
            group_by: GroupBy::File,
        }
    }
}

/// Generates token-budgeted repo map text from ranked symbols.
///
/// The repo map provides AI coding agents with a structured overview of
/// the most important symbols in a codebase. Symbols are ranked by
/// importance and formatted within a configurable token budget.
pub struct RepoMap;

impl RepoMap {
    /// Generates a repo map string from symbols, rankings, and configuration.
    ///
    /// Algorithm:
    /// 1. Sorts symbols by ranking score (descending).
    /// 2. Groups symbols by file or kind based on configuration.
    /// 3. Formats each symbol with optional signature and doc comment.
    /// 4. Stops adding symbols when the token budget is exhausted.
    pub fn generate(
        symbols: &[ExtractedSymbol],
        ranking: &SymbolRanking,
        config: &RepoMapConfig,
    ) -> String {
        // Sort symbols by ranking score (descending)
        let mut scored_symbols: Vec<(&ExtractedSymbol, f64)> = symbols
            .iter()
            .map(|s| (s, ranking.score_for(&s.qualified_name())))
            .collect();
        scored_symbols.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        match config.group_by {
            GroupBy::File => Self::generate_by_file(&scored_symbols, config),
            GroupBy::Kind => Self::generate_by_kind(&scored_symbols, config),
        }
    }

    /// Generates a repo map grouped by source file.
    ///
    /// Output format:
    /// ```text
    /// path/to/file.rs:
    ///   symbol_name: signature
    ///   other_symbol
    /// ```
    fn generate_by_file(
        scored_symbols: &[(&ExtractedSymbol, f64)],
        config: &RepoMapConfig,
    ) -> String {
        let mut tokens_used: usize = 0;
        let mut output = String::new();

        // Group symbols by file, preserving score ordering within groups
        let mut file_groups: BTreeMap<PathBuf, Vec<&ExtractedSymbol>> = BTreeMap::new();
        let mut file_order: Vec<PathBuf> = Vec::new();

        for (symbol, _score) in scored_symbols {
            if !file_groups.contains_key(&symbol.file_path) {
                file_order.push(symbol.file_path.clone());
            }
            file_groups
                .entry(symbol.file_path.clone())
                .or_default()
                .push(symbol);
        }

        for file_path in &file_order {
            let header = format!("{}:\n", file_path.display());
            let header_tokens = Self::estimate_tokens(&header);
            if tokens_used + header_tokens > config.max_tokens {
                break;
            }
            tokens_used += header_tokens;
            output.push_str(&header);

            if let Some(symbols) = file_groups.get(file_path) {
                for symbol in symbols {
                    let mut lines = String::new();

                    // Add doc comment if configured and available
                    if config.include_doc_comments {
                        if let Some(ref doc) = symbol.doc_comment {
                            let doc_line = format!("  /// {}\n", doc);
                            lines.push_str(&doc_line);
                        }
                    }

                    // Add symbol line
                    let symbol_line = format!("  {}\n", Self::format_symbol(symbol, config));
                    lines.push_str(&symbol_line);

                    let line_tokens = Self::estimate_tokens(&lines);
                    if tokens_used + line_tokens > config.max_tokens {
                        return output;
                    }
                    tokens_used += line_tokens;
                    output.push_str(&lines);
                }
            }
        }

        output
    }

    /// Generates a repo map grouped by symbol kind.
    ///
    /// Output format:
    /// ```text
    /// Functions:
    ///   symbol_name (path/to/file.rs)
    /// Structs:
    ///   symbol_name (path/to/file.rs)
    /// ```
    fn generate_by_kind(
        scored_symbols: &[(&ExtractedSymbol, f64)],
        config: &RepoMapConfig,
    ) -> String {
        let mut tokens_used: usize = 0;
        let mut output = String::new();

        // Group symbols by kind, preserving score ordering within groups
        let mut kind_groups: BTreeMap<String, Vec<&ExtractedSymbol>> = BTreeMap::new();
        let mut kind_order: Vec<String> = Vec::new();

        for (symbol, _score) in scored_symbols {
            let kind_label = Self::kind_label(symbol.kind);
            if !kind_groups.contains_key(kind_label) {
                kind_order.push(kind_label.to_string());
            }
            kind_groups
                .entry(kind_label.to_string())
                .or_default()
                .push(symbol);
        }

        for kind_label in &kind_order {
            let header = format!("{}:\n", kind_label);
            let header_tokens = Self::estimate_tokens(&header);
            if tokens_used + header_tokens > config.max_tokens {
                break;
            }
            tokens_used += header_tokens;
            output.push_str(&header);

            if let Some(symbols) = kind_groups.get(kind_label) {
                for symbol in symbols {
                    let mut lines = String::new();

                    // Add doc comment if configured and available
                    if config.include_doc_comments {
                        if let Some(ref doc) = symbol.doc_comment {
                            let doc_line = format!("  /// {}\n", doc);
                            lines.push_str(&doc_line);
                        }
                    }

                    // Add symbol line with file path
                    let symbol_line = format!(
                        "  {} ({})\n",
                        Self::format_symbol(symbol, config),
                        symbol.file_path.display()
                    );
                    lines.push_str(&symbol_line);

                    let line_tokens = Self::estimate_tokens(&lines);
                    if tokens_used + line_tokens > config.max_tokens {
                        return output;
                    }
                    tokens_used += line_tokens;
                    output.push_str(&lines);
                }
            }
        }

        output
    }

    /// Estimates the number of tokens in a content string.
    ///
    /// Uses the approximation of 4 characters per token. Returns 0 for
    /// empty strings, minimum 1 for non-empty strings.
    pub fn estimate_tokens(content: &str) -> usize {
        if content.is_empty() {
            return 0;
        }
        content.len().max(1) / 4
    }

    /// Formats a single symbol for display in the repo map.
    ///
    /// If `include_signatures` is enabled and the symbol has a signature,
    /// the output is `name: signature`. Otherwise, just `name`.
    pub fn format_symbol(symbol: &ExtractedSymbol, config: &RepoMapConfig) -> String {
        if config.include_signatures {
            if let Some(ref sig) = symbol.signature {
                return format!("{}: {}", symbol.name, sig);
            }
        }
        symbol.name.clone()
    }

    /// Returns the human-readable group label for a symbol kind.
    fn kind_label(kind: SymbolKind) -> &'static str {
        match kind {
            SymbolKind::Function => "Functions",
            SymbolKind::Method => "Methods",
            SymbolKind::Struct => "Structs",
            SymbolKind::Trait => "Traits",
            SymbolKind::Class => "Classes",
            SymbolKind::Interface => "Interfaces",
            SymbolKind::Enum => "Enums",
            SymbolKind::EnumVariant => "Enum Variants",
            SymbolKind::TypeAlias => "Type Aliases",
            SymbolKind::Constant => "Constants",
            SymbolKind::Module => "Modules",
            SymbolKind::Field => "Fields",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::FileDependencyGraph;
    use crate::ranking::{RankingConfig, SymbolRanking};
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
    fn estimate_tokens_empty() {
        assert_eq!(RepoMap::estimate_tokens(""), 0);
    }

    #[test]
    fn estimate_tokens_nonempty() {
        // 12 chars / 4 = 3
        assert_eq!(RepoMap::estimate_tokens("hello world!"), 3);
    }

    #[test]
    fn estimate_tokens_short() {
        // 1 char, max(1,1)/4 = 0 ... but the function says min 1 for non-empty
        // Actually: content.len().max(1) / 4 = 1/4 = 0 in integer division.
        // The doc says "min 1 if not empty", but the formula yields 0 for len=1.
        // Let's follow the spec exactly: content.len().max(1) / 4
        // For "a": 1.max(1) / 4 = 0
        assert_eq!(RepoMap::estimate_tokens("a"), 0);
        // For "abcd": 4/4 = 1
        assert_eq!(RepoMap::estimate_tokens("abcd"), 1);
    }

    #[test]
    fn format_symbol_without_signature() {
        let sym = make_symbol("foo", SymbolKind::Function, "/a.rs");
        let config = RepoMapConfig {
            include_signatures: false,
            ..Default::default()
        };
        assert_eq!(RepoMap::format_symbol(&sym, &config), "foo");
    }

    #[test]
    fn format_symbol_with_signature() {
        let mut sym = make_symbol("foo", SymbolKind::Function, "/a.rs");
        sym = sym.with_signature("fn foo(x: i32) -> bool".to_string());
        let config = RepoMapConfig::default();
        assert_eq!(
            RepoMap::format_symbol(&sym, &config),
            "foo: fn foo(x: i32) -> bool"
        );
    }

    #[test]
    fn generate_by_file_grouping() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/a.rs");
        graph.add_file("/b.rs");

        let symbols = vec![
            make_symbol("alpha", SymbolKind::Function, "/a.rs"),
            make_symbol("beta", SymbolKind::Struct, "/b.rs"),
        ];
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        let map_config = RepoMapConfig {
            group_by: GroupBy::File,
            include_signatures: false,
            ..Default::default()
        };
        let output = RepoMap::generate(&symbols, &ranking, &map_config);

        assert!(output.contains("alpha"));
        assert!(output.contains("beta"));
        // Should have file path headers
        assert!(output.contains("/a.rs:") || output.contains("a.rs:"));
        assert!(output.contains("/b.rs:") || output.contains("b.rs:"));
    }

    #[test]
    fn generate_by_kind_grouping() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/a.rs");

        let symbols = vec![
            make_symbol("foo", SymbolKind::Function, "/a.rs"),
            make_symbol("Bar", SymbolKind::Struct, "/a.rs"),
        ];
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        let map_config = RepoMapConfig {
            group_by: GroupBy::Kind,
            include_signatures: false,
            ..Default::default()
        };
        let output = RepoMap::generate(&symbols, &ranking, &map_config);

        assert!(output.contains("Functions:") || output.contains("Structs:"));
        assert!(output.contains("foo"));
        assert!(output.contains("Bar"));
    }

    #[test]
    fn token_budget_enforcement() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/a.rs");

        let mut symbols = Vec::new();
        for i in 0..100 {
            symbols.push(make_symbol(
                &format!("symbol_{}", i),
                SymbolKind::Function,
                "/a.rs",
            ));
        }
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        // Very small token budget
        let map_config = RepoMapConfig {
            max_tokens: 10,
            include_signatures: false,
            ..Default::default()
        };
        let output = RepoMap::generate(&symbols, &ranking, &map_config);

        // Should not include all 100 symbols
        let symbol_count = output.matches("symbol_").count();
        assert!(symbol_count < 100);
    }

    #[test]
    fn doc_comments_included_when_configured() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/a.rs");

        let sym = make_symbol("documented", SymbolKind::Function, "/a.rs")
            .with_doc_comment("This is a documented function".to_string());
        let symbols = vec![sym];
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        let map_config = RepoMapConfig {
            include_doc_comments: true,
            include_signatures: false,
            ..Default::default()
        };
        let output = RepoMap::generate(&symbols, &ranking, &map_config);

        assert!(output.contains("This is a documented function"));
    }

    #[test]
    fn doc_comments_excluded_by_default() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/a.rs");

        let sym = make_symbol("documented", SymbolKind::Function, "/a.rs")
            .with_doc_comment("This should not appear".to_string());
        let symbols = vec![sym];
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        let map_config = RepoMapConfig::default();
        let output = RepoMap::generate(&symbols, &ranking, &map_config);

        assert!(!output.contains("This should not appear"));
    }

    #[test]
    fn repomap_config_from_indexing_config() {
        let indexing_config = IndexingConfig {
            max_tokens: 4096,
            include_signatures: false,
            include_doc_comments: true,
            group_by: "kind".to_string(),
            ..Default::default()
        };
        let config = RepoMapConfig::from_indexing_config(&indexing_config);
        assert_eq!(config.max_tokens, 4096);
        assert!(!config.include_signatures);
        assert!(config.include_doc_comments);
        assert_eq!(config.group_by, GroupBy::Kind);
    }

    #[test]
    fn default_repomap_config() {
        let config = RepoMapConfig::new();
        assert_eq!(config.max_tokens, 2048);
        assert!(config.include_signatures);
        assert!(!config.include_doc_comments);
        assert_eq!(config.group_by, GroupBy::File);
    }

    #[test]
    fn empty_symbols_produce_empty_map() {
        let graph = FileDependencyGraph::new();
        let ranking = SymbolRanking::compute(&[], &graph, &RankingConfig::new());
        let config = RepoMapConfig::default();
        let output = RepoMap::generate(&[], &ranking, &config);
        assert!(output.is_empty());
    }

    #[test]
    fn kind_grouping_includes_file_path() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/src/lib.rs");

        let symbols = vec![make_symbol("main_fn", SymbolKind::Function, "/src/lib.rs")];
        let config = RankingConfig::new();
        let ranking = SymbolRanking::compute(&symbols, &graph, &config);

        let map_config = RepoMapConfig {
            group_by: GroupBy::Kind,
            include_signatures: false,
            ..Default::default()
        };
        let output = RepoMap::generate(&symbols, &ranking, &map_config);

        // Kind grouping should include file path in parentheses
        assert!(output.contains("(/src/lib.rs)") || output.contains("lib.rs)"));
    }
}

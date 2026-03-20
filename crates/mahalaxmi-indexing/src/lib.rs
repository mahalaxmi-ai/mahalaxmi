//! Codebase indexing, symbol extraction, and repo map generation for Mahalaxmi.
//!
//! This crate provides the foundation for intelligent context preparation.
//! AI coding agents operate under token limits — the repo map gives them a
//! structured overview of the most important symbols in a codebase, enabling
//! better task decomposition and file selection.
//!
//! # Architecture
//!
//! - **Types**: Core domain types (SupportedLanguage, SymbolKind, ExtractedSymbol, FileFingerprint)
//! - **Languages**: Tree-sitter grammar registry and generic symbol extraction
//! - **Extractors**: Language-specific extractors for Rust, TypeScript, Python, Go, Java, C, C++
//! - **Graph**: File-level dependency graph from import analysis
//! - **Ranking**: PageRank-style importance scoring for symbols
//! - **RepoMap**: Token-budgeted repo map generation
//! - **Index**: Top-level CodebaseIndex with incremental update support

pub mod extractors;
pub mod graph;
pub mod index;
pub mod languages;
pub mod ranking;
pub mod repomap;
pub mod types;

// Re-export key types for convenience.
pub use extractors::ExtractorFactory;
pub use graph::{FileDependency, FileDependencyGraph, ImportResolver};
pub use index::{CodebaseIndex, UpdateStats};
pub use languages::{LanguageConfig, LanguageRegistry, SymbolExtractor, TreeSitterExtractor};
pub use ranking::{RankingConfig, SymbolRanking};
pub use repomap::{GroupBy, RepoMap, RepoMapConfig};
pub use types::{
    ConfidenceLevel, ExtractedSymbol, FileFingerprint, SupportedLanguage, SymbolKind, Visibility,
};

// Core re-exports for convenience.
pub use mahalaxmi_core::config::IndexingConfig;
pub use mahalaxmi_core::error::MahalaxmiError;
pub use mahalaxmi_core::i18n::locale::SupportedLocale;
pub use mahalaxmi_core::i18n::I18nService;
pub use mahalaxmi_core::MahalaxmiResult;

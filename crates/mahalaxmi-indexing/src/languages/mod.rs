//! Language support for codebase indexing.
//!
//! Provides grammar configuration, a language registry, and the
//! [`SymbolExtractor`] trait backed by Tree-sitter queries.

pub mod config;
pub mod extractor;
pub mod registry;

pub use config::*;
pub use extractor::*;
pub use registry::*;

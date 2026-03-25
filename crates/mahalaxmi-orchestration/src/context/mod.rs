//! Intelligent context preparation for worker activation.
//!
//! Assembles optimal context for each worker by combining repo map data,
//! file relevance scoring, shared memory, and token budgeting into a
//! preamble string injected into the worker's prompt.

pub mod budget;
pub mod builder;
pub mod chunker;
pub mod relevance;

pub use budget::{estimate_tokens, TokenBudget, TokenUsage};
pub use builder::{
    build_dependency_sections, ContextBuilder, ContextSection, ContextSectionKind,
    DependencyHandoff, WorkerContext,
};
pub use chunker::{ChunkerConfig, CodeChunk, CodeChunker};
pub use relevance::{extract_file_paths, score_files, FileRelevance, RelevanceReason};

pub mod graph_proximity;
pub mod historical;
pub mod lexical;
pub mod router;

pub use router::{
    ContextRouter, ContextRouterConfig, DefaultContextRouter, FileScore, ScoredContext,
};

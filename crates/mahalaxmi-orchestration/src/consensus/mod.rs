//! Consensus engine for merging multiple manager proposals.
//!
//! Implements four strategies: Union, Intersection, WeightedVoting,
//! and ComplexityWeighted.
//!
//! ## Deduplication Pipeline
//!
//! Manager proposals pass through a two-stage deduplication pipeline before
//! strategy evaluation:
//!
//! **Stage 1 (Lexical):** CamelCase-aware tokenization + stop-word removal
//! ensures that "GitHubIssuesAdapter" and "GitHub Issues Work Intake Adapter"
//! produce the same normalized tokens and are grouped together.  Multi-field
//! similarity scoring (title + files + description + criteria) with graceful
//! fallback when `affected_files` is absent fixes the Cycle 2 class of
//! over-dispatch bugs.
//!
//! **Stage 2 (LLM Arbitration, opt-in):** Pairs whose similarity falls in a
//! configurable "ambiguous band" are resolved by a one-shot AI call.
//! Requires `ANTHROPIC_API_KEY` and the `arbitration` Cargo feature.
//! Gracefully degrades to Stage 1 only when unavailable.

pub mod arbitrator;
pub mod complexity;
pub mod engine;
pub mod intersection;
pub mod normalizer;
pub mod similarity;
pub mod strategy;
pub mod union;
pub mod weighted;

pub use arbitrator::ArbitrationConfig;
pub use engine::ConsensusEngine;
pub use mahalaxmi_core::domain::ConsensusAlgorithm;
pub use normalizer::{group_matching_tasks, TaskGroup};
pub use similarity::SimilarityWeights;
pub use strategy::ConsensusStrategyImpl;

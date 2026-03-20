//! Error pattern analysis for detecting recurring failures.
//!
//! Tracks error occurrences, clusters related errors, and generates
//! root cause hypotheses to support orchestration decision-making.

pub mod analysis;
pub mod cluster;
pub mod hypothesis;
pub mod recurring;

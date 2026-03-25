//! Manager prompt building and output parsing.
//!
//! This module provides two key components for the manager session lifecycle:
//!
//! - [`ManagerPromptBuilder`] — assembles the full prompt sent to an AI manager,
//!   including codebase context, template requirements, and JSON output format spec.
//!
//! - [`ManagerOutputParser`] — extracts structured JSON from raw AI terminal output,
//!   handling code fences, noisy text, and multiple JSON blocks.

pub mod builder;
pub mod normalizer;
pub mod parser;
pub mod review_builder;
pub mod validator_builder;
pub mod worker_builder;

pub use builder::ManagerPromptBuilder;
pub use normalizer::{is_normalized, NormalizationProvider, RequirementsNormalizer};
pub use parser::ManagerOutputParser;
pub use review_builder::{ReviewPromptBuilder, ReviewPromptConfig};
pub use validator_builder::{CommandResult, ValidatorPromptBuilder, ValidatorPromptConfig};
pub use worker_builder::{WorkerPromptBuilder, WorkerPromptConfig};

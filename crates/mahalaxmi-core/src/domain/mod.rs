//! Domain configuration — externalised prompt text for manager and worker agents.
//!
//! A *domain* bundles the prompt strings used by the orchestration engine for a
//! specific category of work (e.g. "coding"). Prompts live in `.prompt` files
//! alongside a `config.yaml` descriptor, and are loaded at startup by
//! [`DomainRegistry::load_from_dir`].
//!
//! # Quick start
//!
//! ```no_run
//! use mahalaxmi_core::domain::DomainRegistry;
//! use std::path::Path;
//!
//! let mut registry = DomainRegistry::new();
//! registry.load_from_dir(Path::new("domains")).unwrap();
//! let coding = registry.get("coding").unwrap();
//! println!("{}", coding.manager_system_role(4));
//! ```

pub mod config;
pub mod loader;
pub mod registry;

pub use config::{
    ConsensusAlgorithm, ConstraintSection, DecompositionStrategy, DomainBehavior, DomainConfig,
    InputFormat, OutputFormat, PromptSource,
};
pub use loader::LoadedDomain;
pub use registry::DomainRegistry;

#[cfg(test)]
mod tests;

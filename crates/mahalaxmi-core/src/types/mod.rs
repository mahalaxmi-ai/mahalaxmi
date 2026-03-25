//! Shared domain types used across Mahalaxmi crates.

mod detection;
pub mod developer;
mod enums;
mod ids;
mod orchestration;
mod process;
mod templates;
mod terminal;

pub use detection::{ActionType, MatchType, RootCauseCategory};
pub use developer::{
    Developer, DeveloperId, DeveloperRegistry, DeveloperSession, DeveloperSessionStatus,
};
pub use enums::{ExecutionMode, TerminalPurpose, TerminalRole, TerminalState};
pub use ids::{ProviderId, SessionId, TerminalId};
pub use orchestration::{
    ConsensusStrategy, CycleId, GitMergeStrategy, GitPrPlatform, ManagerId,
    OrchestrationCycleState, TaskId, VerificationCheck, WorkerId, WorkerStatus,
};
pub use process::ProcessCommand;
pub use templates::{CategoryId, LicenseTier, TemplateDifficulty, TemplateId, ValidationSeverity};
pub use terminal::TerminalConfig;

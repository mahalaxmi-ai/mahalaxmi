//! Orchestration engine for Mahalaxmi.
//!
//! Implements the Manager-Worker DAG execution model, consensus engine,
//! execution plans, worker queue, and cycle state machine.

pub mod agent;
pub mod carry_forward;
pub mod classifier;
pub mod consensus;
pub mod context;
pub mod dag;
pub mod deliberation;
pub mod diff_scan;
pub mod error;
pub mod input_adapter;
pub mod modes;
pub mod models;
pub mod monitor;
pub mod output_format;
pub mod plan_hash;
pub mod prompt;
pub mod queue;
pub mod service;
pub mod state_machine;
pub mod verification;
pub mod worktree;

pub use consensus::arbitrator::{ArbitrationConfig, ConsensusArbitrator};
pub use consensus::engine::ConsensusEngine;
pub use consensus::normalizer::{group_matching_tasks, TaskGroup};
pub use consensus::similarity::SimilarityWeights;
pub use dag::{build_phases, detect_cycles, topological_sort, validate_dag};
pub use mahalaxmi_core::config::MahalaxmiConfig;
pub use mahalaxmi_core::error::MahalaxmiError;
pub use mahalaxmi_core::i18n::locale::SupportedLocale;
pub use mahalaxmi_core::i18n::I18nService;
pub use mahalaxmi_core::MahalaxmiResult;
pub use modes::{
    build_mode_driver, BatchAnalysisOutput, BatchResult, CompletionVerdict, CycleState,
    CycleStatus, ModeDriver, OrchestrationMode, PreciseDriverConfig, PreciseModeDriver,
    ReviewDriverConfig, ReviewModeDriver, StandardModeDriver,
};
pub use monitor::{MonitorAction, StreamMonitor, TerminalRole};
pub use prompt::{ManagerOutputParser, ManagerPromptBuilder, WorkerPromptBuilder};
pub use queue::WorkerQueue;
pub use service::{
    CycleConfig, CycleHandle, CycleSnapshot, OrchestrationCommand, OrchestrationService,
};
pub use state_machine::CycleStateMachine;
pub use verification::{
    format_retry_context, CheckResult, VerificationPipeline, VerificationResult, VerificationStatus,
};

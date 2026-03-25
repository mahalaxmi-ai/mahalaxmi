//! Detection domain types shared across crates.
//!
//! These types are defined in `mahalaxmi-core` so that both `mahalaxmi-detection`
//! and `mahalaxmi-orchestration` can reference them without cross-dependency.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Category for root cause analysis of recurring errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RootCauseCategory {
    /// Root cause is unknown.
    Unknown,
    /// Authentication or credential issue.
    Authentication,
    /// Network connectivity issue.
    Network,
    /// File system permission or access issue.
    FileSystem,
    /// Dependency resolution or version issue.
    Dependency,
    /// Syntax error in code or configuration.
    Syntax,
    /// Runtime error during execution.
    Runtime,
    /// Resource exhaustion (memory, disk, CPU).
    Resource,
    /// Configuration error or misconfiguration.
    Configuration,
    /// API rate limiting or quota exceeded.
    RateLimit,
    /// Permission denied (non-filesystem).
    Permission,
    /// Build or compilation failure.
    Build,
    /// Test failure.
    Test,
    /// Merge conflict or version control issue.
    VersionControl,
    /// External service unavailable.
    ExternalService,
    /// Operation timed out.
    Timeout,
}

impl fmt::Display for RootCauseCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Unknown => "Unknown",
            Self::Authentication => "Authentication",
            Self::Network => "Network",
            Self::FileSystem => "FileSystem",
            Self::Dependency => "Dependency",
            Self::Syntax => "Syntax",
            Self::Runtime => "Runtime",
            Self::Resource => "Resource",
            Self::Configuration => "Configuration",
            Self::RateLimit => "RateLimit",
            Self::Permission => "Permission",
            Self::Build => "Build",
            Self::Test => "Test",
            Self::VersionControl => "VersionControl",
            Self::ExternalService => "ExternalService",
            Self::Timeout => "Timeout",
        };
        write!(f, "{}", label)
    }
}

/// Action to take when a detection rule matches.
///
/// Ported from Ganesha's action types. Each action represents a response
/// that the orchestrator can execute when terminal output matches a rule.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    /// Send Enter key (`\r`) to the terminal.
    SendEnter,
    /// Send specific text to the terminal.
    SendText,
    /// Send text followed by Enter.
    SendTextWithEnter,
    /// No input action — just log and continue monitoring.
    ContinueProcessing,
    /// Signal that the manager cycle is complete.
    CompleteManagerCycle,
    /// Signal that the worker cycle is complete.
    CompleteWorkerCycle,
    /// Trigger manager start.
    LaunchManager,
    /// Trigger worker launch.
    LaunchWorkers,
    /// Close the terminal session.
    CloseTerminal,
    /// Restart the AI session in the terminal.
    RestartSession,
    /// Restart the entire orchestration cycle.
    RestartOrchestration,
    /// Halt all orchestration.
    StopOrchestration,
    /// Worker needs manager help — escalate.
    EscalateToManager,
    /// Wait for a specified duration before re-evaluating.
    WaitAndRetry,
}

impl ActionType {
    /// Returns true if this action sends input to the terminal.
    pub fn sends_input(&self) -> bool {
        matches!(
            self,
            Self::SendEnter | Self::SendText | Self::SendTextWithEnter
        )
    }

    /// Returns true if this action signals an orchestration cycle event.
    pub fn is_cycle_signal(&self) -> bool {
        matches!(
            self,
            Self::CompleteManagerCycle
                | Self::CompleteWorkerCycle
                | Self::LaunchManager
                | Self::LaunchWorkers
                | Self::RestartOrchestration
                | Self::StopOrchestration
        )
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::SendEnter => "SendEnter",
            Self::SendText => "SendText",
            Self::SendTextWithEnter => "SendTextWithEnter",
            Self::ContinueProcessing => "ContinueProcessing",
            Self::CompleteManagerCycle => "CompleteManagerCycle",
            Self::CompleteWorkerCycle => "CompleteWorkerCycle",
            Self::LaunchManager => "LaunchManager",
            Self::LaunchWorkers => "LaunchWorkers",
            Self::CloseTerminal => "CloseTerminal",
            Self::RestartSession => "RestartSession",
            Self::RestartOrchestration => "RestartOrchestration",
            Self::StopOrchestration => "StopOrchestration",
            Self::EscalateToManager => "EscalateToManager",
            Self::WaitAndRetry => "WaitAndRetry",
        };
        write!(f, "{}", label)
    }
}

/// How a detection pattern should match against terminal output text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchType {
    /// Pattern must match the entire text exactly.
    Exact,
    /// Pattern must appear somewhere in the text.
    Contains,
    /// Pattern is a regular expression.
    Regex,
    /// Text must start with the pattern.
    StartsWith,
    /// Text must end with the pattern.
    EndsWith,
}

impl fmt::Display for MatchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Exact => "Exact",
            Self::Contains => "Contains",
            Self::Regex => "Regex",
            Self::StartsWith => "StartsWith",
            Self::EndsWith => "EndsWith",
        };
        write!(f, "{}", label)
    }
}

use serde::{Deserialize, Serialize};

/// Role of a terminal in the orchestration system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalRole {
    /// Manager terminal — analyzes codebase and produces execution plans.
    Manager,
    /// Worker terminal — executes tasks from the execution plan.
    Worker,
}

/// Lifecycle state of a managed terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalState {
    /// Terminal has been created but not yet started.
    Created,
    /// Terminal is in the process of starting.
    Starting,
    /// Terminal is running and processing.
    Running,
    /// Terminal is temporarily paused.
    Paused,
    /// Terminal is in the process of stopping.
    Stopping,
    /// Terminal has stopped normally.
    Stopped,
    /// Terminal has encountered a fatal error.
    Failed,
}

impl TerminalState {
    /// Returns true if this state represents an active (non-terminal) state.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Created | Self::Starting | Self::Running | Self::Paused
        )
    }

    /// Returns true if this state represents a terminal (final) state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed)
    }
}

/// Purpose of a terminal session, used to partition pool capacity.
///
/// Orchestration terminals (managers/workers) and utility terminals (install, login)
/// use separate pool limits so utility operations never block orchestration workers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminalPurpose {
    /// Orchestration terminal (manager or worker). Uses main pool.
    #[default]
    Orchestration,
    /// Utility terminal (install, login, test). Uses separate pool.
    Utility,
}

impl std::fmt::Display for TerminalPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Orchestration => write!(f, "orchestration"),
            Self::Utility => write!(f, "utility"),
        }
    }
}

/// Execution mode for terminal sessions.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Interactive mode — user can observe and interact with terminals.
    #[default]
    Interactive,
    /// Headless mode — terminals run without UI attachment.
    Headless,
}

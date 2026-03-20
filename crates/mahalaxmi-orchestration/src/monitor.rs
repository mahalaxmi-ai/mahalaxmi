//! Stream monitor — bridges PTY terminal output to detection rules and orchestration actions.
//!
//! The `StreamMonitor` tracks terminal-to-worker assignments and evaluates
//! incoming text output against detection rules. When a rule matches, the
//! monitor produces a `MonitorAction` describing what the orchestration
//! layer should do (send input, complete a worker, escalate, etc.).
//!
//! The monitor is **synchronous and stateless** with respect to the
//! orchestration service — it does not hold a reference to the service.
//! Instead, it returns actions that the caller (Tauri command layer)
//! dispatches to the service and terminal manager.

use std::collections::HashMap;

use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ActionType, TerminalId, WorkerId};
use mahalaxmi_core::MahalaxmiResult;
use mahalaxmi_detection::{BuiltinRuleSets, DetectionResult, DetectionRule, RuleMatcher};

/// An action the orchestration layer should take in response to a stream detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorAction {
    /// Send text input to a terminal (auto-response).
    SendInput {
        terminal_id: TerminalId,
        text: String,
        append_enter: bool,
    },
    /// Signal that a worker has completed its task.
    WorkerCompleted {
        worker_id: WorkerId,
        terminal_id: TerminalId,
    },
    /// Signal that a worker has failed and should be escalated.
    WorkerFailed {
        worker_id: WorkerId,
        terminal_id: TerminalId,
        error_summary: String,
    },
    /// Signal to close a terminal session.
    CloseTerminal { terminal_id: TerminalId },
    /// Signal to restart an AI session in the terminal.
    RestartSession { terminal_id: TerminalId },
    /// Signal to stop the entire orchestration cycle.
    StopOrchestration,
    /// No action needed — continue monitoring.
    Continue,
}

/// Role of a monitored terminal (determines which detection rules apply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalRole {
    /// A manager terminal analyzing the codebase.
    Manager,
    /// A worker terminal executing a task.
    Worker,
}

impl TerminalRole {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Manager => "manager",
            Self::Worker => "worker",
        }
    }
}

/// Registration of a terminal with the stream monitor.
#[derive(Debug, Clone)]
struct MonitoredTerminal {
    terminal_id: TerminalId,
    worker_id: Option<WorkerId>,
    role: TerminalRole,
    provider_id: String,
}

/// Bridges PTY terminal output to detection rules and orchestration actions.
///
/// Usage pattern:
/// 1. Create a `StreamMonitor` with detection rules for the provider.
/// 2. Register terminals as they are spawned (`register_terminal`).
/// 3. Feed text output from PTY events (`evaluate`).
/// 4. Dispatch the returned `MonitorAction` to the orchestration service / terminal manager.
/// 5. Unregister terminals when they close (`unregister_terminal`).
pub struct StreamMonitor {
    /// Detection rule matcher (compiled patterns with cooldown tracking).
    matcher: RuleMatcher,
    /// Tracked terminals keyed by terminal ID.
    terminals: HashMap<TerminalId, MonitoredTerminal>,
    /// Provider ID used for this monitor (for rule filtering).
    provider_id: String,
}

impl StreamMonitor {
    /// Create a new stream monitor with the given detection rules.
    ///
    /// Rules are compiled and sorted by priority. Invalid regex patterns
    /// in rules produce an error.
    pub fn new(
        rules: Vec<DetectionRule>,
        provider_id: impl Into<String>,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Self> {
        let matcher = RuleMatcher::new(rules, i18n)?;
        Ok(Self {
            matcher,
            terminals: HashMap::new(),
            provider_id: provider_id.into(),
        })
    }

    /// Create a stream monitor with the default built-in rules for the provider.
    pub fn with_defaults(
        provider_id: impl Into<String>,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Self> {
        let rules = BuiltinRuleSets::all_defaults();
        Self::new(rules, provider_id, i18n)
    }

    /// Register a terminal for monitoring.
    ///
    /// Worker terminals should pass `Some(worker_id)` so the monitor can
    /// map completion/failure signals to specific workers.
    pub fn register_terminal(
        &mut self,
        terminal_id: TerminalId,
        role: TerminalRole,
        worker_id: Option<WorkerId>,
    ) {
        self.terminals.insert(
            terminal_id,
            MonitoredTerminal {
                terminal_id,
                worker_id,
                role,
                provider_id: self.provider_id.clone(),
            },
        );
    }

    /// Unregister a terminal (no longer monitored).
    pub fn unregister_terminal(&mut self, terminal_id: &TerminalId) {
        self.terminals.remove(terminal_id);
    }

    /// Get the worker ID assigned to a terminal, if any.
    pub fn worker_for_terminal(&self, terminal_id: &TerminalId) -> Option<WorkerId> {
        self.terminals.get(terminal_id).and_then(|t| t.worker_id)
    }

    /// Get the role of a terminal, if registered.
    pub fn terminal_role(&self, terminal_id: &TerminalId) -> Option<TerminalRole> {
        self.terminals.get(terminal_id).map(|t| t.role)
    }

    /// Get the number of registered terminals.
    pub fn terminal_count(&self) -> usize {
        self.terminals.len()
    }

    /// Evaluate text output from a terminal against detection rules.
    ///
    /// Returns a `MonitorAction` describing what the orchestration layer
    /// should do in response. If no rule matches or the terminal is not
    /// registered, returns `MonitorAction::Continue`.
    pub fn evaluate(&mut self, terminal_id: &TerminalId, text: &str) -> MonitorAction {
        let monitored = match self.terminals.get(terminal_id) {
            Some(t) => t.clone(),
            None => return MonitorAction::Continue,
        };

        let role_str = monitored.role.as_str();
        let detection = self
            .matcher
            .evaluate(text, Some(&monitored.provider_id), Some(role_str));

        match detection {
            Some(result) => self.action_from_detection(result, &monitored),
            None => {
                // Try without role filter (generic rules)
                match self
                    .matcher
                    .evaluate(text, Some(&monitored.provider_id), None)
                {
                    Some(result) => self.action_from_detection(result, &monitored),
                    None => {
                        // Try without any filter (catch-all rules)
                        match self.matcher.evaluate(text, None, None) {
                            Some(result) => self.action_from_detection(result, &monitored),
                            None => MonitorAction::Continue,
                        }
                    }
                }
            }
        }
    }

    /// Get the number of active detection rules.
    pub fn rule_count(&self) -> usize {
        self.matcher.rule_count()
    }

    /// Reset all cooldown timers.
    pub fn reset_cooldowns(&mut self) {
        self.matcher.reset_cooldowns();
    }

    /// Map a detection result to a monitor action.
    fn action_from_detection(
        &self,
        result: DetectionResult,
        monitored: &MonitoredTerminal,
    ) -> MonitorAction {
        match result.action {
            ActionType::SendEnter => MonitorAction::SendInput {
                terminal_id: monitored.terminal_id,
                text: String::new(),
                append_enter: true,
            },
            ActionType::SendText => MonitorAction::SendInput {
                terminal_id: monitored.terminal_id,
                text: result.response_text.unwrap_or_default(),
                append_enter: false,
            },
            ActionType::SendTextWithEnter => MonitorAction::SendInput {
                terminal_id: monitored.terminal_id,
                text: result.response_text.unwrap_or_default(),
                append_enter: true,
            },
            ActionType::CompleteManagerCycle | ActionType::CompleteWorkerCycle => {
                if let Some(worker_id) = monitored.worker_id {
                    MonitorAction::WorkerCompleted {
                        worker_id,
                        terminal_id: monitored.terminal_id,
                    }
                } else {
                    MonitorAction::Continue
                }
            }
            ActionType::EscalateToManager => {
                if let Some(worker_id) = monitored.worker_id {
                    MonitorAction::WorkerFailed {
                        worker_id,
                        terminal_id: monitored.terminal_id,
                        error_summary: result
                            .matched_rule_name
                            .unwrap_or_else(|| "Unknown error".into()),
                    }
                } else {
                    MonitorAction::Continue
                }
            }
            ActionType::CloseTerminal => MonitorAction::CloseTerminal {
                terminal_id: monitored.terminal_id,
            },
            ActionType::RestartSession => MonitorAction::RestartSession {
                terminal_id: monitored.terminal_id,
            },
            ActionType::StopOrchestration => MonitorAction::StopOrchestration,
            ActionType::ContinueProcessing
            | ActionType::LaunchManager
            | ActionType::LaunchWorkers
            | ActionType::RestartOrchestration
            | ActionType::WaitAndRetry => MonitorAction::Continue,
        }
    }
}

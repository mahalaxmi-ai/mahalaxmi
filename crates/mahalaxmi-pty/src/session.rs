use mahalaxmi_core::config::OrchestrationConfig;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{
    ProcessCommand, TerminalConfig, TerminalId, TerminalPurpose, TerminalState,
};
use mahalaxmi_core::MahalaxmiResult;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::{debug, info};

use crate::events::TerminalEvent;
use crate::spawner::PtySpawner;
use crate::terminal::ManagedTerminal;

/// Default maximum number of concurrent utility terminals.
const DEFAULT_MAX_UTILITY: u32 = 10;

/// Manages the lifecycle of multiple concurrent PTY terminal sessions.
///
/// Enforces separate capacity limits for orchestration and utility terminals.
/// Orchestration terminals (managers/workers) use `max_concurrent` from config.
/// Utility terminals (install, login, test) use a separate `max_utility` limit.
/// Broadcasts terminal events to all subscribers.
pub struct TerminalSessionManager {
    /// Active terminal sessions keyed by ID, paired with their purpose.
    terminals: HashMap<TerminalId, (ManagedTerminal, TerminalPurpose)>,
    /// Maximum number of concurrent orchestration terminals allowed.
    max_concurrent: u32,
    /// Maximum number of concurrent utility terminals allowed.
    max_utility: u32,
    /// Broadcast channel for terminal events.
    event_tx: broadcast::Sender<TerminalEvent>,
    /// Internationalization service for localized error messages.
    i18n: I18nService,
}

/// Compute the broadcast channel capacity from the configured max concurrent workers.
///
/// Returns `1024` when `max_concurrent_workers` is `0` (auto-scale mode).
/// Otherwise returns `max(max_concurrent_workers * 4, 256)` so small deployments
/// keep a sensible floor while large deployments (100+ workers) get proportionally
/// larger buffers to avoid event drops under sustained load.
pub fn compute_channel_capacity(max_concurrent_workers: usize) -> usize {
    if max_concurrent_workers == 0 {
        1024
    } else {
        (max_concurrent_workers * 4).max(256)
    }
}

impl TerminalSessionManager {
    /// Create a new session manager with limits from the orchestration config.
    ///
    /// The broadcast channel capacity is derived automatically from
    /// `config.max_concurrent_workers` via [`compute_channel_capacity`], so the
    /// event buffer scales with the configured worker count.
    pub fn new(config: &OrchestrationConfig, i18n: I18nService) -> Self {
        let capacity = compute_channel_capacity(config.max_concurrent_workers as usize);
        let (event_tx, _) = broadcast::channel(capacity);
        Self {
            terminals: HashMap::new(),
            max_concurrent: config.max_concurrent_workers,
            max_utility: DEFAULT_MAX_UTILITY,
            event_tx,
            i18n,
        }
    }

    /// Create a session manager with a specific orchestration concurrency limit.
    /// Uses the default utility limit and a 256-slot broadcast channel suitable
    /// for testing and low-concurrency scenarios.
    pub fn with_max_concurrent(max_concurrent: u32, i18n: I18nService) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            terminals: HashMap::new(),
            max_concurrent,
            max_utility: DEFAULT_MAX_UTILITY,
            event_tx,
            i18n,
        }
    }

    /// Create a session manager with explicit orchestration and utility limits.
    /// Uses a 256-slot broadcast channel suitable for testing and low-concurrency
    /// scenarios.
    pub fn with_limits(max_concurrent: u32, max_utility: u32, i18n: I18nService) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            terminals: HashMap::new(),
            max_concurrent,
            max_utility,
            event_tx,
            i18n,
        }
    }

    /// Count active terminals by purpose.
    pub fn count_by_purpose(&self, purpose: TerminalPurpose) -> usize {
        self.terminals
            .values()
            .filter(|(_, p)| *p == purpose)
            .count()
    }

    /// Remove terminals whose child processes have exited.
    ///
    /// Polls each terminal with `try_wait` and removes any that have finished,
    /// freeing slots for new terminals.
    pub fn reap_completed(&mut self) -> usize {
        let ids: Vec<TerminalId> = self.terminals.keys().copied().collect();
        let mut reaped = 0;
        for id in ids {
            let exited = self
                .terminals
                .get_mut(&id)
                .and_then(|(t, _)| t.try_wait(&self.i18n).ok())
                .flatten()
                .is_some();
            if exited {
                let old_state = self
                    .terminals
                    .get(&id)
                    .map(|(t, _)| t.state())
                    .unwrap_or(TerminalState::Running);
                self.terminals.remove(&id);
                info!(terminal_id = %id, "Reaped completed terminal");
                let _ = self.event_tx.send(TerminalEvent::StateChanged {
                    terminal_id: id,
                    old_state,
                    new_state: TerminalState::Stopped,
                });
                reaped += 1;
            }
        }
        if reaped > 0 {
            debug!(
                reaped,
                total_remaining = self.terminals.len(),
                "Reap cycle complete"
            );
        }
        reaped
    }

    /// Spawn a new orchestration terminal session (backward-compatible wrapper).
    ///
    /// Equivalent to `spawn_terminal_with_purpose(command, config, TerminalPurpose::Orchestration)`.
    pub fn spawn_terminal(
        &mut self,
        command: &ProcessCommand,
        config: &TerminalConfig,
    ) -> MahalaxmiResult<TerminalId> {
        self.spawn_terminal_with_purpose(command, config, TerminalPurpose::Orchestration)
    }

    /// Spawn a new terminal session with the given purpose.
    ///
    /// Returns the `TerminalId` of the new session.
    /// Automatically reaps completed terminals before checking the purpose-specific limit.
    /// Fails if the pool for the given purpose is full.
    pub fn spawn_terminal_with_purpose(
        &mut self,
        command: &ProcessCommand,
        config: &TerminalConfig,
        purpose: TerminalPurpose,
    ) -> MahalaxmiResult<TerminalId> {
        let pool_limit = match purpose {
            TerminalPurpose::Orchestration => self.max_concurrent,
            TerminalPurpose::Utility => self.max_utility,
        };

        // 0 = unlimited: skip capacity enforcement entirely.
        // This matches the platform convention used throughout Mahalaxmi where
        // 0 means "no cap" (max_managers, max_workers, max_concurrent_workers).
        if pool_limit > 0 {
            let pool_count = self.count_by_purpose(purpose) as u32;

            // Reap completed terminals to free slots before checking capacity
            if pool_count >= pool_limit {
                let reaped = self.reap_completed();
                if reaped > 0 {
                    info!(reaped, %purpose, "Reaped completed terminals before spawn");
                }
            }

            let pool_count = self.count_by_purpose(purpose) as u32;
            if pool_count >= pool_limit {
                return Err(MahalaxmiError::pty(
                    &self.i18n,
                    keys::pty::MAX_CONCURRENT_REACHED,
                    &[("max", &pool_limit.to_string())],
                ));
            }
        }

        let terminal_id = TerminalId::new();
        let terminal = PtySpawner::spawn(command, config, terminal_id, &self.i18n)?;

        info!(
            terminal_id = %terminal_id,
            %purpose,
            active_orchestration = self.count_by_purpose(TerminalPurpose::Orchestration),
            active_utility = self.count_by_purpose(TerminalPurpose::Utility),
            max_orchestration = self.max_concurrent,
            max_utility = self.max_utility,
            "Terminal session spawned"
        );

        let _ = self.event_tx.send(TerminalEvent::StateChanged {
            terminal_id,
            old_state: TerminalState::Created,
            new_state: TerminalState::Running,
        });

        self.terminals.insert(terminal_id, (terminal, purpose));
        Ok(terminal_id)
    }

    /// Close a terminal session by killing its process and removing it from the registry.
    pub fn close_terminal(&mut self, id: &TerminalId) -> MahalaxmiResult<()> {
        let (mut terminal, _purpose) = self.terminals.remove(id).ok_or_else(|| {
            MahalaxmiError::pty(
                &self.i18n,
                keys::pty::TERMINAL_NOT_FOUND,
                &[("terminal_id", &id.to_string())],
            )
        })?;

        let old_state = terminal.state();
        terminal.kill(&self.i18n)?;

        info!(
            terminal_id = %id,
            remaining = self.terminals.len(),
            "Terminal session closed"
        );

        let _ = self.event_tx.send(TerminalEvent::StateChanged {
            terminal_id: *id,
            old_state,
            new_state: TerminalState::Stopped,
        });

        Ok(())
    }

    /// Resize a terminal.
    pub fn resize_terminal(&self, id: &TerminalId, rows: u16, cols: u16) -> MahalaxmiResult<()> {
        let (terminal, _) = self.terminals.get(id).ok_or_else(|| {
            MahalaxmiError::pty(
                &self.i18n,
                keys::pty::TERMINAL_NOT_FOUND,
                &[("terminal_id", &id.to_string())],
            )
        })?;
        terminal.resize(rows, cols, &self.i18n)
    }

    /// Get a reference to a terminal by ID.
    pub fn get_terminal(&self, id: &TerminalId) -> Option<&ManagedTerminal> {
        self.terminals.get(id).map(|(t, _)| t)
    }

    /// Get a mutable reference to a terminal by ID.
    pub fn get_terminal_mut(&mut self, id: &TerminalId) -> Option<&mut ManagedTerminal> {
        self.terminals.get_mut(id).map(|(t, _)| t)
    }

    /// Return the OS process ID of the PTY child for `id`, if known.
    ///
    /// Returns `None` when the terminal is not found or when the PTY backend
    /// does not expose the child PID (Windows stub). Used by the build
    /// infrastructure to check for active compiler children before triggering
    /// stall detection.
    pub fn terminal_child_pid(&mut self, id: &TerminalId) -> Option<u32> {
        self.terminals
            .get_mut(id)
            .and_then(|(t, _)| t.child_pid())
    }

    /// Get the purpose of a terminal by ID.
    pub fn get_terminal_purpose(&self, id: &TerminalId) -> Option<TerminalPurpose> {
        self.terminals.get(id).map(|(_, p)| *p)
    }

    /// List all active terminal IDs.
    pub fn list_terminals(&self) -> Vec<TerminalId> {
        self.terminals.keys().copied().collect()
    }

    /// Return the total number of active terminals (all purposes).
    pub fn active_count(&self) -> usize {
        self.terminals.len()
    }

    /// Subscribe to terminal events.
    pub fn subscribe(&self) -> broadcast::Receiver<TerminalEvent> {
        self.event_tx.subscribe()
    }

    /// Get a clone of the terminal event broadcast sender.
    ///
    /// Used by command handlers to pass to `spawn_reader_task` so PTY output
    /// flows through the same event channel as state-change events.
    pub fn event_sender(&self) -> broadcast::Sender<TerminalEvent> {
        self.event_tx.clone()
    }

    /// Close all terminal sessions.
    pub fn close_all(&mut self) {
        let ids: Vec<TerminalId> = self.terminals.keys().copied().collect();
        for id in ids {
            if let Err(e) = self.close_terminal(&id) {
                // Downgraded to DEBUG: processes routinely self-exit before the
                // shutdown sweep calls kill(), producing harmless "No such
                // process" errors that pollute WARN logs on every app exit.
                debug!(terminal_id = %id, error = %e, "Failed to close terminal (already exited)");
            }
        }
    }

    /// Return the maximum concurrent orchestration terminal limit.
    pub fn max_concurrent(&self) -> u32 {
        self.max_concurrent
    }

    /// Return the maximum concurrent utility terminal limit.
    pub fn max_utility(&self) -> u32 {
        self.max_utility
    }
}

impl Drop for TerminalSessionManager {
    fn drop(&mut self) {
        self.close_all();
    }
}

#[cfg(test)]
mod capacity_tests {
    use super::compute_channel_capacity;

    #[test]
    fn capacity_formula_min() {
        assert_eq!((64_usize * 4).max(256), 256);
        assert_eq!(compute_channel_capacity(64), 256);
    }

    #[test]
    fn capacity_formula_100() {
        assert_eq!((100_usize * 4).max(256), 400);
        assert_eq!(compute_channel_capacity(100), 400);
    }

    #[test]
    fn capacity_formula_500() {
        assert_eq!((500_usize * 4).max(256), 2000);
        assert_eq!(compute_channel_capacity(500), 2000);
    }

    #[test]
    fn capacity_formula_auto() {
        assert_eq!(compute_channel_capacity(0), 1024);
    }
}

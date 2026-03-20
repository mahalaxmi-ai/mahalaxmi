//! Background memory decay task.
//!
//! Provides a configurable sweep that archives stale memory entries based on
//! their last-access timestamp and memory scope. Project-scoped entries have
//! a shorter time-to-live than global-scoped entries.

#[cfg(feature = "sqlite")]
use crate::sqlite::SqliteMemoryPersistence;
use std::time::Duration;

#[cfg(feature = "sqlite")]
use std::sync::{Arc, Mutex};

/// Configuration for the background memory decay sweep.
pub struct DecayConfig {
    /// How often to run the archival sweep.
    pub sweep_interval: Duration,
    /// Project-scoped entries not accessed within this many days are archived.
    pub project_ttl_days: u64,
    /// Global-scoped entries not accessed within this many days are archived.
    pub global_ttl_days: u64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            sweep_interval: Duration::from_secs(3600),
            project_ttl_days: 90,
            global_ttl_days: 365,
        }
    }
}

/// Spawns a Tokio task that periodically archives stale memories.
///
/// The returned `JoinHandle` should be stored for the process lifetime;
/// dropping it cancels the task.
#[cfg(feature = "sqlite")]
pub fn spawn_decay_task(
    store: Arc<Mutex<SqliteMemoryPersistence>>,
    config: DecayConfig,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(config.sweep_interval);
        loop {
            interval.tick().await;
            let project_threshold = (config.project_ttl_days * 86_400) as i64;
            let global_threshold = (config.global_ttl_days * 86_400) as i64;
            if let Ok(guard) = store.lock() {
                let _ = guard.archive_stale("Project", project_threshold);
                let _ = guard.archive_stale("Global", global_threshold);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `DecayConfig::default()` produces the expected field values.
    #[test]
    fn decay_config_default_values() {
        let cfg = DecayConfig::default();
        assert_eq!(
            cfg.sweep_interval,
            Duration::from_secs(3600),
            "sweep_interval must default to 1 hour"
        );
        assert_eq!(
            cfg.project_ttl_days, 90,
            "project_ttl_days must default to 90"
        );
        assert_eq!(
            cfg.global_ttl_days, 365,
            "global_ttl_days must default to 365"
        );
    }

    /// `spawn_decay_task` can be immediately aborted without panic.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn spawn_decay_task_abort_does_not_panic() {
        let store = crate::sqlite::SqliteMemoryPersistence::open_in_memory()
            .expect("in-memory SQLite must open successfully");
        let store_arc = Arc::new(Mutex::new(store));
        let handle = spawn_decay_task(store_arc, DecayConfig::default());
        handle.abort();
        let result = handle.await;
        assert!(
            result.is_err(),
            "aborted task must return JoinError (is_cancelled)"
        );
        assert!(
            result.unwrap_err().is_cancelled(),
            "JoinError must be a cancellation error"
        );
    }
}

//! Memory entry source attribution.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifies who or what created a memory entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id")]
pub enum MemorySource {
    /// Created by a worker agent.
    Worker {
        /// The worker identifier.
        worker_id: String,
    },
    /// Created by a manager agent.
    Manager {
        /// The manager identifier.
        manager_id: String,
    },
    /// Created by the system automatically.
    System,
    /// Created by the user manually.
    User,
}

impl MemorySource {
    /// Returns the source kind as a static string.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Worker { .. } => "worker",
            Self::Manager { .. } => "manager",
            Self::System => "system",
            Self::User => "user",
        }
    }

    /// Returns the source identifier, if any.
    pub fn source_id(&self) -> Option<&str> {
        match self {
            Self::Worker { worker_id } => Some(worker_id),
            Self::Manager { manager_id } => Some(manager_id),
            Self::System | Self::User => None,
        }
    }
}

impl fmt::Display for MemorySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Worker { worker_id } => write!(f, "worker:{worker_id}"),
            Self::Manager { manager_id } => write!(f, "manager:{manager_id}"),
            Self::System => write!(f, "system"),
            Self::User => write!(f, "user"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_id_returns_id_for_worker_and_manager() {
        let worker = MemorySource::Worker {
            worker_id: "w1".to_owned(),
        };
        assert_eq!(worker.source_id(), Some("w1"));

        let manager = MemorySource::Manager {
            manager_id: "m1".to_owned(),
        };
        assert_eq!(manager.source_id(), Some("m1"));

        assert_eq!(MemorySource::System.source_id(), None);
        assert_eq!(MemorySource::User.source_id(), None);
    }

    #[test]
    fn as_str_returns_kind() {
        assert_eq!(
            MemorySource::Worker {
                worker_id: "w1".to_owned()
            }
            .as_str(),
            "worker"
        );
        assert_eq!(MemorySource::System.as_str(), "system");
    }

    #[test]
    fn display_format() {
        let worker = MemorySource::Worker {
            worker_id: "w1".to_owned(),
        };
        assert_eq!(format!("{worker}"), "worker:w1");
        assert_eq!(format!("{}", MemorySource::System), "system");
    }
}

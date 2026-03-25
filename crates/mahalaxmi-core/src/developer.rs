//! Developer session model for Phase 16 team collaboration.
//!
//! Provides the foundational types that represent individual developers
//! participating in Mahalaxmi orchestration cycles: identifiers, profiles,
//! a registry loaded from configuration, and ephemeral session state.

use crate::config::MahalaxmiConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;

/// Opaque string identifier for a developer.
///
/// Wraps a `String` with newtype semantics and transparent serialisation so
/// that JSON/TOML representations are indistinguishable from a plain string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeveloperId(String);

impl DeveloperId {
    /// Creates a new `DeveloperId` from the given string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Deref for DeveloperId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for DeveloperId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for DeveloperId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DeveloperId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// Returns the default consensus weight for a developer (1.0).
fn default_weight() -> f32 {
    1.0
}

/// Returns the default maximum number of concurrent managers per developer (2).
fn default_max_managers() -> u8 {
    2
}

/// A developer registered with the Mahalaxmi orchestration system.
///
/// Developers are the human (or automated) principals that own orchestration
/// cycles. Each developer may be assigned a different AI provider and carries
/// a weighted-voting weight used by the consensus engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Developer {
    /// Unique identifier for this developer.
    pub id: DeveloperId,
    /// Human-readable display name.
    pub name: String,
    /// AI provider identifier to use for this developer's sessions.
    pub provider: String,
    /// Weighted-voting weight for consensus; defaults to 1.0.
    #[serde(default = "default_weight")]
    pub weight: f32,
    /// Maximum concurrent manager sessions allowed; defaults to 2.
    #[serde(default = "default_max_managers")]
    pub max_managers: u8,
    /// Optional enterprise seat identifier.
    pub seat_id: Option<String>,
}

/// In-memory registry of all known developers.
///
/// Populated at startup from [`MahalaxmiConfig`] and used throughout the
/// orchestration pipeline to look up developer profiles by [`DeveloperId`].
#[derive(Debug, Clone, Default)]
pub struct DeveloperRegistry {
    developers: HashMap<DeveloperId, Developer>,
}

impl DeveloperRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            developers: HashMap::new(),
        }
    }

    /// Inserts or replaces a developer entry.
    pub fn add(&mut self, developer: Developer) {
        self.developers.insert(developer.id.clone(), developer);
    }

    /// Removes and returns the developer with the given id, if present.
    pub fn remove(&mut self, id: &DeveloperId) -> Option<Developer> {
        self.developers.remove(id)
    }

    /// Returns a reference to the developer with the given id, if present.
    pub fn get(&self, id: &DeveloperId) -> Option<&Developer> {
        self.developers.get(id)
    }

    /// Returns a list of references to all registered developers.
    ///
    /// The order is unspecified (determined by the underlying hash map).
    pub fn list(&self) -> Vec<&Developer> {
        self.developers.values().collect()
    }

    /// Constructs a registry from the `[team]` section of [`MahalaxmiConfig`].
    ///
    /// Returns an empty registry when the config has no `team` section or when
    /// the developers list is empty — no panic, no error.
    pub fn from_config(config: &MahalaxmiConfig) -> Self {
        let entries = config
            .team
            .as_ref()
            .map(|t| t.developers.as_slice())
            .unwrap_or(&[]);

        let mut registry = Self::new();
        for entry in entries {
            let developer = Developer {
                id: DeveloperId::from(entry.id.as_str()),
                name: entry.name.clone(),
                provider: entry.provider.clone(),
                weight: entry.weight,
                max_managers: entry.max_managers,
                seat_id: entry.seat_id.clone(),
            };
            registry.add(developer);
        }
        registry
    }
}

/// Status of a developer's orchestration session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeveloperSessionStatus {
    /// Scheduled but not yet dispatched.
    Pending,
    /// Currently running workers.
    Active,
    /// All workers merged successfully.
    Merged,
    /// Stopped before completion.
    Cancelled,
}

/// An active or historical orchestration session owned by a developer.
///
/// Tracks which developer initiated the cycle, the requirements text provided,
/// the UTC wall-clock time the session began, and its current lifecycle status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeveloperSession {
    /// The developer that owns this session.
    pub developer_id: DeveloperId,
    /// Requirements text supplied at session start.
    pub requirements: String,
    /// UTC timestamp when the session was created.
    pub started_at: DateTime<Utc>,
    /// Current lifecycle status.
    pub status: DeveloperSessionStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MahalaxmiConfig;
    use crate::config::{DeveloperConfig, TeamConfig};
    #[test]
    fn test_developer_registry_from_config() {
        let config = MahalaxmiConfig {
            team: Some(TeamConfig {
                developers: vec![
                    DeveloperConfig {
                        id: "alice".to_string(),
                        name: "Alice".to_string(),
                        provider: "claude".to_string(),
                        weight: 1.5,
                        max_managers: 3,
                        seat_id: None,
                    },
                    DeveloperConfig {
                        id: "bob".to_string(),
                        name: "Bob".to_string(),
                        provider: "openai".to_string(),
                        weight: 0.8,
                        max_managers: 2,
                        seat_id: Some("seat-42".to_string()),
                    },
                ],
            }),
            ..Default::default()
        };

        let registry = DeveloperRegistry::from_config(&config);
        assert_eq!(
            registry.list().len(),
            2,
            "registry must contain exactly 2 developers"
        );

        let alice_id = DeveloperId::from("alice");
        let alice = registry.get(&alice_id).expect("alice must be present");
        assert!(
            (alice.weight - 1.5).abs() < f32::EPSILON,
            "alice weight must be 1.5"
        );
        assert_eq!(alice.max_managers, 3);

        let bob_id = DeveloperId::from("bob");
        let bob = registry.get(&bob_id).expect("bob must be present");
        assert!(
            (bob.weight - 0.8).abs() < f32::EPSILON,
            "bob weight must be 0.8"
        );
        assert_eq!(bob.seat_id, Some("seat-42".to_string()));
    }

    #[test]
    fn test_developer_registry_empty_when_no_team_section() {
        let config = MahalaxmiConfig {
            team: None,
            ..Default::default()
        };
        let registry = DeveloperRegistry::from_config(&config);
        assert!(
            registry.list().is_empty(),
            "registry must be empty when config has no team section"
        );
    }
}

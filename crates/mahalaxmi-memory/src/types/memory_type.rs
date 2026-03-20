//! Memory type classification for shared memory entries.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Classification of memory entries.
///
/// Each type represents a different kind of knowledge that agents can share:
/// - `CodebaseFact`: Discovered facts about the codebase structure or behavior
/// - `Convention`: Coding conventions, naming patterns, or style rules
/// - `Decision`: Architectural or implementation decisions made during a cycle
/// - `Warning`: Pitfalls, gotchas, or issues to avoid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    /// Discovered facts about the codebase.
    CodebaseFact,
    /// Coding conventions or patterns.
    Convention,
    /// Architectural or implementation decisions.
    Decision,
    /// Pitfalls, gotchas, or issues to avoid.
    Warning,
}

impl MemoryType {
    /// Returns the default confidence score for this memory type.
    pub fn default_confidence(&self) -> f64 {
        match self {
            Self::CodebaseFact => 0.7,
            Self::Convention => 0.6,
            Self::Decision => 0.8,
            Self::Warning => 0.9,
        }
    }

    /// Returns the type as a static string identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CodebaseFact => "codebase_fact",
            Self::Convention => "convention",
            Self::Decision => "decision",
            Self::Warning => "warning",
        }
    }
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_confidences_are_valid() {
        for mt in [
            MemoryType::CodebaseFact,
            MemoryType::Convention,
            MemoryType::Decision,
            MemoryType::Warning,
        ] {
            let c = mt.default_confidence();
            assert!(
                (0.0..=1.0).contains(&c),
                "{mt} has invalid default confidence {c}"
            );
        }
    }

    #[test]
    fn as_str_roundtrips_via_serde() {
        for mt in [
            MemoryType::CodebaseFact,
            MemoryType::Convention,
            MemoryType::Decision,
            MemoryType::Warning,
        ] {
            let json = serde_json::to_string(&mt).unwrap();
            let deserialized: MemoryType = serde_json::from_str(&json).unwrap();
            assert_eq!(mt, deserialized);
        }
    }

    #[test]
    fn display_matches_as_str() {
        for mt in [
            MemoryType::CodebaseFact,
            MemoryType::Convention,
            MemoryType::Decision,
            MemoryType::Warning,
        ] {
            assert_eq!(format!("{mt}"), mt.as_str());
        }
    }
}

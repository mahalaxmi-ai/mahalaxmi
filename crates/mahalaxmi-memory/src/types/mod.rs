//! Core types for the memory system.

pub mod confidence;
pub mod memory_type;
pub mod source;

pub use confidence::ConfidenceLevel;
pub use memory_type::MemoryType;
pub use source::MemorySource;

pub use crate::entry::MemoryScope;

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a memory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(Uuid);

impl MemoryId {
    /// Create a new random memory ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID value.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for MemoryId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MemoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_id_uniqueness() {
        let id1 = MemoryId::new();
        let id2 = MemoryId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn memory_id_from_uuid_roundtrip() {
        let uuid = Uuid::new_v4();
        let id = MemoryId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn memory_id_display() {
        let uuid = Uuid::new_v4();
        let id = MemoryId::from_uuid(uuid);
        assert_eq!(format!("{id}"), format!("{uuid}"));
    }
}

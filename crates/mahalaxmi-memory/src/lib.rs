//! Cross-agent shared memory for Mahalaxmi.
//!
//! This crate provides an inter-worker knowledge-sharing system for orchestration cycles.
//! Agents can share discovered facts, conventions, decisions, and warnings through
//! a shared memory store that supports querying, persistence, and injection into
//! worker contexts.

pub mod entry;
pub mod injector;
pub mod persistence;
pub mod query;
pub mod store;
pub mod types;

#[cfg(feature = "sqlite")]
pub mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::SqliteMemoryPersistence;
#[cfg(feature = "sqlite")]
pub mod decay;
#[cfg(feature = "sqlite")]
pub mod sync;

// Re-export core types.
pub use entry::{MemoryEntry, MemoryEntryBuilder, MemoryMetadata, MemoryScope};
pub use injector::{InjectionFormat, InjectorConfig, MemoryInjector};
pub use persistence::{FileMemoryPersistence, MemoryPersistence};
pub use query::{MemoryQuery, QueryOrder};
pub use store::{MemoryStats, MemoryStore};
pub use types::{ConfidenceLevel, MemoryId, MemorySource, MemoryType};

// Re-export core types for convenience.
pub use mahalaxmi_core::config::MahalaxmiConfig;
pub use mahalaxmi_core::error::MahalaxmiError;
pub use mahalaxmi_core::i18n::locale::SupportedLocale;
pub use mahalaxmi_core::i18n::I18nService;
pub use mahalaxmi_core::MahalaxmiResult;

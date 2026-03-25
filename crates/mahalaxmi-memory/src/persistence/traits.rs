//! Trait definition for memory persistence backends.

use crate::store::MemoryStore;
use async_trait::async_trait;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

/// Trait for persisting and loading memory stores.
#[async_trait]
pub trait MemoryPersistence: Send + Sync {
    /// Save a memory store to the given path.
    async fn save(
        &self,
        store: &MemoryStore,
        path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<()>;

    /// Load a memory store from the given path.
    async fn load(&self, path: &Path, i18n: &I18nService) -> MahalaxmiResult<MemoryStore>;

    /// Check if a persisted store exists at the given path.
    async fn exists(&self, path: &Path) -> bool;

    /// Delete a persisted store at the given path.
    async fn delete(&self, path: &Path, i18n: &I18nService) -> MahalaxmiResult<()>;

    /// List session IDs from persisted files in a directory.
    async fn list_sessions(&self, dir: &Path) -> Vec<String>;
}

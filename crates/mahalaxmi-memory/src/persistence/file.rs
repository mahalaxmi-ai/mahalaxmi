//! File-based JSON persistence for memory stores.

use crate::persistence::traits::MemoryPersistence;
use crate::store::MemoryStore;
use async_trait::async_trait;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

/// File-based persistence using pretty-printed JSON.
///
/// Files are named `{session_id}.json`. Directories are created automatically.
pub struct FileMemoryPersistence;

impl FileMemoryPersistence {
    /// Create a new file-based persistence instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileMemoryPersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryPersistence for FileMemoryPersistence {
    async fn save(
        &self,
        store: &MemoryStore,
        path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                MahalaxmiError::memory(
                    i18n,
                    keys::memory::PERSISTENCE_FAILED,
                    &[("reason", &e.to_string())],
                )
            })?;
        }

        let json = serde_json::to_string_pretty(store).map_err(|e| {
            MahalaxmiError::memory(
                i18n,
                keys::memory::SERIALIZATION,
                &[("reason", &e.to_string())],
            )
        })?;

        tokio::fs::write(path, json).await.map_err(|e| {
            MahalaxmiError::memory(
                i18n,
                keys::memory::PERSISTENCE_FAILED,
                &[("reason", &e.to_string())],
            )
        })?;

        Ok(())
    }

    async fn load(&self, path: &Path, i18n: &I18nService) -> MahalaxmiResult<MemoryStore> {
        let data = tokio::fs::read_to_string(path).await.map_err(|e| {
            MahalaxmiError::memory(
                i18n,
                keys::memory::LOAD_FAILED,
                &[("reason", &e.to_string())],
            )
        })?;

        let store: MemoryStore = serde_json::from_str(&data).map_err(|e| {
            MahalaxmiError::memory(
                i18n,
                keys::memory::LOAD_FAILED,
                &[("reason", &e.to_string())],
            )
        })?;

        Ok(store)
    }

    async fn exists(&self, path: &Path) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }

    async fn delete(&self, path: &Path, i18n: &I18nService) -> MahalaxmiResult<()> {
        tokio::fs::remove_file(path).await.map_err(|e| {
            MahalaxmiError::memory(
                i18n,
                keys::memory::PERSISTENCE_FAILED,
                &[("reason", &e.to_string())],
            )
        })?;
        Ok(())
    }

    async fn list_sessions(&self, dir: &Path) -> Vec<String> {
        let mut sessions = Vec::new();
        let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
            return sessions;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    sessions.push(stem.to_owned());
                }
            }
        }
        sessions.sort();
        sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_persistence_default() {
        let _p = FileMemoryPersistence::default();
    }
}

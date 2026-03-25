//! Domain registry — stores and retrieves [`DomainConfig`] implementations.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

use super::config::DomainConfig;
use super::loader::LoadedDomain;

/// A registry of named domain configurations.
///
/// Domains are identified by their `id` string (e.g. `"coding"`). The registry
/// owns the loaded domain objects and hands out `Arc` references so callers can
/// hold onto a domain cheaply across threads.
#[derive(Default)]
pub struct DomainRegistry {
    domains: HashMap<String, Arc<dyn DomainConfig>>,
}

impl DomainRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
        }
    }

    /// Register a domain configuration.
    ///
    /// If a domain with the same `id` is already present it is replaced.
    pub fn register(&mut self, domain: Arc<dyn DomainConfig>) {
        self.domains.insert(domain.id().to_owned(), domain);
    }

    /// Look up a domain by its `id`.
    pub fn get(&self, id: &str) -> Option<Arc<dyn DomainConfig>> {
        self.domains.get(id).cloned()
    }

    /// Scan `domains_dir` for subdirectories that contain a `config.yaml` and
    /// load each one as a [`LoadedDomain`].
    ///
    /// Returns the number of domains successfully loaded.
    pub fn load_from_dir(&mut self, domains_dir: &Path) -> Result<usize> {
        let mut count = 0usize;
        for entry in std::fs::read_dir(domains_dir)
            .with_context(|| format!("reading domains directory {}", domains_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.join("config.yaml").exists() {
                let domain = LoadedDomain::load(&path)?;
                self.register(Arc::new(domain));
                count += 1;
            }
        }
        Ok(count)
    }
}

use anyhow::Context as _;

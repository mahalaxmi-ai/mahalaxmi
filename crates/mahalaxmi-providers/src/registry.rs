use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::ProviderId;
use mahalaxmi_core::MahalaxmiResult;
use std::collections::HashMap;

use crate::cost::{built_in_pricing, ProviderPricing};
use crate::traits::AiProvider;

/// Look up the built-in pricing for a model by its lowercase model ID.
///
/// Returns `None` if the model ID is not present in the built-in pricing table.
pub fn pricing_for(model_id: &str) -> Option<ProviderPricing> {
    built_in_pricing().get(model_id).cloned()
}

/// Registry of available AI providers.
///
/// Manages a collection of providers and provides lookup by ID.
/// Exactly one provider can be marked as the default.
pub struct ProviderRegistry {
    providers: HashMap<ProviderId, Box<dyn AiProvider>>,
    default_provider: Option<ProviderId>,
}

impl ProviderRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    /// Register a provider. If a provider with the same ID already exists, it is replaced.
    pub fn register(&mut self, provider: Box<dyn AiProvider>) {
        let id = provider.id().clone();
        self.providers.insert(id, provider);
    }

    /// Register a provider and set it as the default.
    pub fn register_default(&mut self, provider: Box<dyn AiProvider>) {
        let id = provider.id().clone();
        self.default_provider = Some(id.clone());
        self.providers.insert(id, provider);
    }

    /// Get a provider by ID.
    pub fn get(&self, id: &ProviderId) -> Option<&dyn AiProvider> {
        self.providers.get(id).map(|p| p.as_ref())
    }

    /// Get the default provider.
    pub fn default_provider(&self, i18n: &I18nService) -> MahalaxmiResult<&dyn AiProvider> {
        let id = self
            .default_provider
            .as_ref()
            .ok_or_else(|| MahalaxmiError::provider(i18n, keys::provider::NO_DEFAULT, &[]))?;
        self.get(id).ok_or_else(|| {
            MahalaxmiError::provider(
                i18n,
                keys::provider::NOT_FOUND,
                &[("provider_id", id.as_str())],
            )
        })
    }

    /// List all registered provider IDs.
    pub fn list(&self) -> Vec<&ProviderId> {
        self.providers.keys().collect()
    }

    /// Return the number of registered providers.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Return true if no providers are registered.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

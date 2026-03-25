//! Language-specific symbol extractors and factory.

pub mod c_extractor;
pub mod cpp_extractor;
pub mod go_extractor;
pub mod java_extractor;
pub mod python_extractor;
pub mod rust_extractor;
pub mod typescript_extractor;

pub use c_extractor::*;
pub use cpp_extractor::*;
pub use go_extractor::*;
pub use java_extractor::*;
pub use python_extractor::*;
pub use rust_extractor::*;
pub use typescript_extractor::*;

use crate::languages::{LanguageRegistry, SymbolExtractor, TreeSitterExtractor};
use crate::types::SupportedLanguage;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use std::collections::HashMap;

/// Factory for creating language-specific extractors.
pub struct ExtractorFactory;

impl ExtractorFactory {
    /// Create an extractor for the given language.
    pub fn create(
        lang: SupportedLanguage,
        registry: &LanguageRegistry,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Box<dyn SymbolExtractor>> {
        let config = registry.get(&lang).ok_or_else(|| {
            mahalaxmi_core::error::MahalaxmiError::indexing(
                i18n,
                mahalaxmi_core::i18n::messages::keys::indexing::UNSUPPORTED_LANGUAGE,
                &[("extension", lang.as_str())],
            )
        })?;
        let extractor = TreeSitterExtractor::new(lang, config, i18n)?;
        Ok(Box::new(extractor))
    }

    /// Create extractors for all registered languages.
    pub fn create_all(
        registry: &LanguageRegistry,
        i18n: &I18nService,
    ) -> MahalaxmiResult<HashMap<SupportedLanguage, Box<dyn SymbolExtractor>>> {
        let mut extractors = HashMap::new();
        for lang in registry.supported_languages() {
            let extractor = Self::create(lang, registry, i18n)?;
            extractors.insert(lang, extractor);
        }
        Ok(extractors)
    }
}

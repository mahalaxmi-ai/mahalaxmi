//! Python-specific symbol extractor.

use crate::languages::{LanguageRegistry, SymbolExtractor, TreeSitterExtractor};
use crate::types::{ExtractedSymbol, SupportedLanguage, Visibility};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

/// Python-specific extractor with underscore-based visibility.
pub struct PythonExtractor {
    inner: TreeSitterExtractor,
}

impl PythonExtractor {
    /// Create a new Python extractor.
    pub fn new(registry: &LanguageRegistry, i18n: &I18nService) -> MahalaxmiResult<Self> {
        let config = registry.get(&SupportedLanguage::Python).ok_or_else(|| {
            mahalaxmi_core::error::MahalaxmiError::indexing(
                i18n,
                mahalaxmi_core::i18n::messages::keys::indexing::UNSUPPORTED_LANGUAGE,
                &[("extension", "py")],
            )
        })?;
        let inner = TreeSitterExtractor::new(SupportedLanguage::Python, config, i18n)?;
        Ok(Self { inner })
    }
}

impl SymbolExtractor for PythonExtractor {
    fn extract_symbols(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<ExtractedSymbol>> {
        let mut symbols = self.inner.extract_symbols(source, file_path, i18n)?;
        // Python: underscore naming convention for visibility
        for symbol in &mut symbols {
            if symbol.name.starts_with("__") && !symbol.name.ends_with("__") {
                symbol.visibility = Visibility::Restricted;
            } else if symbol.name.starts_with('_') {
                symbol.visibility = Visibility::Private;
            } else {
                symbol.visibility = Visibility::Public;
            }
        }
        Ok(symbols)
    }

    fn extract_imports(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<String>> {
        self.inner.extract_imports(source, file_path, i18n)
    }

    fn language(&self) -> SupportedLanguage {
        SupportedLanguage::Python
    }
}

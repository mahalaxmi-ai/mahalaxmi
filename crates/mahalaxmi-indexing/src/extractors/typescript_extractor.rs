//! TypeScript-specific symbol extractor.

use crate::languages::{LanguageRegistry, SymbolExtractor, TreeSitterExtractor};
use crate::types::{ExtractedSymbol, SupportedLanguage, Visibility};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

/// TypeScript-specific extractor with export visibility detection.
pub struct TypeScriptExtractor {
    inner: TreeSitterExtractor,
}

impl TypeScriptExtractor {
    /// Create a new TypeScript extractor.
    pub fn new(registry: &LanguageRegistry, i18n: &I18nService) -> MahalaxmiResult<Self> {
        let config = registry
            .get(&SupportedLanguage::TypeScript)
            .ok_or_else(|| {
                mahalaxmi_core::error::MahalaxmiError::indexing(
                    i18n,
                    mahalaxmi_core::i18n::messages::keys::indexing::UNSUPPORTED_LANGUAGE,
                    &[("extension", "ts")],
                )
            })?;
        let inner = TreeSitterExtractor::new(SupportedLanguage::TypeScript, config, i18n)?;
        Ok(Self { inner })
    }
}

impl SymbolExtractor for TypeScriptExtractor {
    fn extract_symbols(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<ExtractedSymbol>> {
        let mut symbols = self.inner.extract_symbols(source, file_path, i18n)?;
        // TypeScript: detect export keyword for visibility
        for symbol in &mut symbols {
            let line_text = source
                .lines()
                .nth(symbol.line_start.saturating_sub(1))
                .unwrap_or("");
            if line_text.trim_start().starts_with("export ") {
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
        SupportedLanguage::TypeScript
    }
}

//! Java-specific symbol extractor.

use crate::languages::{LanguageRegistry, SymbolExtractor, TreeSitterExtractor};
use crate::types::{ExtractedSymbol, SupportedLanguage, Visibility};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

/// Java-specific extractor with modifier-based visibility.
pub struct JavaExtractor {
    inner: TreeSitterExtractor,
}

impl JavaExtractor {
    /// Create a new Java extractor.
    pub fn new(registry: &LanguageRegistry, i18n: &I18nService) -> MahalaxmiResult<Self> {
        let config = registry.get(&SupportedLanguage::Java).ok_or_else(|| {
            mahalaxmi_core::error::MahalaxmiError::indexing(
                i18n,
                mahalaxmi_core::i18n::messages::keys::indexing::UNSUPPORTED_LANGUAGE,
                &[("extension", "java")],
            )
        })?;
        let inner = TreeSitterExtractor::new(SupportedLanguage::Java, config, i18n)?;
        Ok(Self { inner })
    }
}

impl SymbolExtractor for JavaExtractor {
    fn extract_symbols(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<ExtractedSymbol>> {
        let mut symbols = self.inner.extract_symbols(source, file_path, i18n)?;
        // Java: detect access modifiers from source
        for symbol in &mut symbols {
            let line_text = source
                .lines()
                .nth(symbol.line_start.saturating_sub(1))
                .unwrap_or("");
            let trimmed = line_text.trim_start();
            if trimmed.starts_with("public ") || trimmed.contains(" public ") {
                symbol.visibility = Visibility::Public;
            } else if trimmed.starts_with("private ") || trimmed.contains(" private ") {
                symbol.visibility = Visibility::Private;
            } else if trimmed.starts_with("protected ") || trimmed.contains(" protected ") {
                symbol.visibility = Visibility::Restricted;
            }
            // Default (package-private) stays as the default Private
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
        SupportedLanguage::Java
    }
}

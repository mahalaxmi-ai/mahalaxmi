//! C++-specific symbol extractor.

use crate::languages::{LanguageRegistry, SymbolExtractor, TreeSitterExtractor};
use crate::types::{ExtractedSymbol, SupportedLanguage, Visibility};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

/// C++-specific extractor with access specifier visibility.
pub struct CppExtractor {
    inner: TreeSitterExtractor,
}

impl CppExtractor {
    /// Create a new C++ extractor.
    pub fn new(registry: &LanguageRegistry, i18n: &I18nService) -> MahalaxmiResult<Self> {
        let config = registry.get(&SupportedLanguage::Cpp).ok_or_else(|| {
            mahalaxmi_core::error::MahalaxmiError::indexing(
                i18n,
                mahalaxmi_core::i18n::messages::keys::indexing::UNSUPPORTED_LANGUAGE,
                &[("extension", "cpp")],
            )
        })?;
        let inner = TreeSitterExtractor::new(SupportedLanguage::Cpp, config, i18n)?;
        Ok(Self { inner })
    }
}

impl SymbolExtractor for CppExtractor {
    fn extract_symbols(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<ExtractedSymbol>> {
        let mut symbols = self.inner.extract_symbols(source, file_path, i18n)?;
        // C++: look for access specifiers and static keyword
        for symbol in &mut symbols {
            let line_text = source
                .lines()
                .nth(symbol.line_start.saturating_sub(1))
                .unwrap_or("");
            let trimmed = line_text.trim_start();
            if trimmed.starts_with("public:") || trimmed.starts_with("public ") {
                symbol.visibility = Visibility::Public;
            } else if trimmed.starts_with("private:") || trimmed.starts_with("private ") {
                symbol.visibility = Visibility::Private;
            } else if trimmed.starts_with("protected:") || trimmed.starts_with("protected ") {
                symbol.visibility = Visibility::Restricted;
            } else if trimmed.starts_with("static ") {
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
        SupportedLanguage::Cpp
    }
}

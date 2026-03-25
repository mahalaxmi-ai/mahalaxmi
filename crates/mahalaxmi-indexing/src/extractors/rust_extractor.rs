//! Rust-specific symbol extractor.

use crate::languages::{LanguageRegistry, SymbolExtractor, TreeSitterExtractor};
use crate::types::{ExtractedSymbol, SupportedLanguage, Visibility};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use std::path::Path;

/// Rust-specific extractor that wraps TreeSitterExtractor with Rust visibility rules.
pub struct RustExtractor {
    inner: TreeSitterExtractor,
}

impl RustExtractor {
    /// Create a new Rust extractor.
    pub fn new(registry: &LanguageRegistry, i18n: &I18nService) -> MahalaxmiResult<Self> {
        let config = registry.get(&SupportedLanguage::Rust).ok_or_else(|| {
            mahalaxmi_core::error::MahalaxmiError::indexing(
                i18n,
                mahalaxmi_core::i18n::messages::keys::indexing::UNSUPPORTED_LANGUAGE,
                &[("extension", "rs")],
            )
        })?;
        let inner = TreeSitterExtractor::new(SupportedLanguage::Rust, config, i18n)?;
        Ok(Self { inner })
    }
}

impl SymbolExtractor for RustExtractor {
    fn extract_symbols(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<ExtractedSymbol>> {
        let mut symbols = self.inner.extract_symbols(source, file_path, i18n)?;
        // Rust-specific: detect pub visibility from source text
        for symbol in &mut symbols {
            let line_text = source
                .lines()
                .nth(symbol.line_start.saturating_sub(1))
                .unwrap_or("");
            if line_text.trim_start().starts_with("pub ")
                || line_text.contains("pub fn ")
                || line_text.contains("pub struct ")
                || line_text.contains("pub enum ")
                || line_text.contains("pub trait ")
                || line_text.contains("pub type ")
                || line_text.contains("pub const ")
                || line_text.contains("pub mod ")
            {
                if line_text.contains("pub(crate)") {
                    symbol.visibility = Visibility::Crate;
                } else if line_text.contains("pub(super)") || line_text.contains("pub(in ") {
                    symbol.visibility = Visibility::Restricted;
                } else {
                    symbol.visibility = Visibility::Public;
                }
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
        SupportedLanguage::Rust
    }
}

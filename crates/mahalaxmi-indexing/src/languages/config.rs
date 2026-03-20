//! Language configuration for Tree-sitter grammars and query patterns.
//!
//! Each [`LanguageConfig`] bundles a Tree-sitter grammar with the S-expression
//! query patterns needed to extract symbol definitions and import statements.

/// Configuration for a single language's Tree-sitter grammar and queries.
///
/// Pairs a Tree-sitter [`Language`](tree_sitter::Language) grammar with the
/// S-expression query patterns used by the extraction engine.
pub struct LanguageConfig {
    /// The Tree-sitter grammar for this language.
    pub language: tree_sitter::Language,
    /// Tree-sitter S-expression query pattern for extracting symbol definitions.
    pub symbol_query: String,
    /// Tree-sitter query pattern for extracting import/use statements.
    pub import_query: String,
}

impl std::fmt::Debug for LanguageConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageConfig")
            .field("symbol_query_len", &self.symbol_query.len())
            .field("import_query_len", &self.import_query.len())
            .finish()
    }
}

impl LanguageConfig {
    /// Create a new language configuration.
    ///
    /// # Arguments
    /// * `language` - The Tree-sitter grammar for parsing
    /// * `symbol_query` - S-expression query for extracting symbol definitions
    /// * `import_query` - S-expression query for extracting import statements
    pub fn new(
        language: tree_sitter::Language,
        symbol_query: impl Into<String>,
        import_query: impl Into<String>,
    ) -> Self {
        Self {
            language,
            symbol_query: symbol_query.into(),
            import_query: import_query.into(),
        }
    }
}

//! Language registry for managing Tree-sitter grammars and query patterns.
//!
//! The [`LanguageRegistry`] stores [`LanguageConfig`] entries keyed by
//! [`SupportedLanguage`], enabling runtime lookup of grammars and queries
//! for any supported programming language.

use std::collections::HashMap;

use crate::languages::config::LanguageConfig;
use crate::types::SupportedLanguage;

/// Registry of supported languages with their Tree-sitter configurations.
///
/// Use [`LanguageRegistry::with_defaults`] to create a registry pre-populated
/// with all supported languages, or [`LanguageRegistry::new`] followed by
/// [`register`](LanguageRegistry::register) calls for custom setups.
pub struct LanguageRegistry {
    /// Map from language identifier to its Tree-sitter configuration.
    registered: HashMap<SupportedLanguage, LanguageConfig>,
}

impl std::fmt::Debug for LanguageRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageRegistry")
            .field("language_count", &self.registered.len())
            .field("languages", &self.registered.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl LanguageRegistry {
    /// Create an empty language registry.
    pub fn new() -> Self {
        Self {
            registered: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with all 8 supported languages.
    ///
    /// Registers Rust, TypeScript, JavaScript, Python, Go, Java, C, and C++
    /// with their Tree-sitter grammars and default query patterns.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        registry.register(
            SupportedLanguage::Rust,
            LanguageConfig::new(
                SupportedLanguage::Rust.tree_sitter_language(),
                RUST_SYMBOL_QUERY,
                RUST_IMPORT_QUERY,
            ),
        );

        registry.register(
            SupportedLanguage::TypeScript,
            LanguageConfig::new(
                SupportedLanguage::TypeScript.tree_sitter_language(),
                TYPESCRIPT_SYMBOL_QUERY,
                TYPESCRIPT_IMPORT_QUERY,
            ),
        );

        registry.register(
            SupportedLanguage::JavaScript,
            LanguageConfig::new(
                SupportedLanguage::JavaScript.tree_sitter_language(),
                JAVASCRIPT_SYMBOL_QUERY,
                JAVASCRIPT_IMPORT_QUERY,
            ),
        );

        registry.register(
            SupportedLanguage::Python,
            LanguageConfig::new(
                SupportedLanguage::Python.tree_sitter_language(),
                PYTHON_SYMBOL_QUERY,
                PYTHON_IMPORT_QUERY,
            ),
        );

        registry.register(
            SupportedLanguage::Go,
            LanguageConfig::new(
                SupportedLanguage::Go.tree_sitter_language(),
                GO_SYMBOL_QUERY,
                GO_IMPORT_QUERY,
            ),
        );

        registry.register(
            SupportedLanguage::Java,
            LanguageConfig::new(
                SupportedLanguage::Java.tree_sitter_language(),
                JAVA_SYMBOL_QUERY,
                JAVA_IMPORT_QUERY,
            ),
        );

        registry.register(
            SupportedLanguage::C,
            LanguageConfig::new(
                SupportedLanguage::C.tree_sitter_language(),
                C_SYMBOL_QUERY,
                C_IMPORT_QUERY,
            ),
        );

        registry.register(
            SupportedLanguage::Cpp,
            LanguageConfig::new(
                SupportedLanguage::Cpp.tree_sitter_language(),
                CPP_SYMBOL_QUERY,
                CPP_IMPORT_QUERY,
            ),
        );

        registry
    }

    /// Register a language with its Tree-sitter configuration.
    ///
    /// Returns `&mut Self` for method chaining.
    pub fn register(&mut self, lang: SupportedLanguage, config: LanguageConfig) -> &mut Self {
        self.registered.insert(lang, config);
        self
    }

    /// Look up the configuration for a language.
    pub fn get(&self, lang: &SupportedLanguage) -> Option<&LanguageConfig> {
        self.registered.get(lang)
    }

    /// Check whether a language is registered.
    pub fn is_supported(&self, lang: &SupportedLanguage) -> bool {
        self.registered.contains_key(lang)
    }

    /// Return all registered languages.
    pub fn supported_languages(&self) -> Vec<SupportedLanguage> {
        self.registered.keys().copied().collect()
    }

    /// Return the number of registered languages.
    pub fn language_count(&self) -> usize {
        self.registered.len()
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tree-sitter query patterns for each language
// ---------------------------------------------------------------------------

/// Rust symbol extraction query.
const RUST_SYMBOL_QUERY: &str = r#"
(function_item name: (identifier) @name) @definition
(struct_item name: (type_identifier) @name) @definition
(trait_item name: (type_identifier) @name) @definition
(enum_item name: (type_identifier) @name) @definition
(type_item name: (type_identifier) @name) @definition
(const_item name: (identifier) @name) @definition
(mod_item name: (identifier) @name) @definition
(impl_item type: (type_identifier) @impl_type body: (declaration_list (function_item name: (identifier) @name) @definition))
"#;

/// Rust import extraction query.
const RUST_IMPORT_QUERY: &str = r#"
(use_declaration argument: (_) @import)
"#;

/// TypeScript symbol extraction query.
const TYPESCRIPT_SYMBOL_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @definition
(class_declaration name: (type_identifier) @name) @definition
(interface_declaration name: (type_identifier) @name) @definition
(enum_declaration name: (identifier) @name) @definition
(type_alias_declaration name: (type_identifier) @name) @definition
(method_definition name: (property_identifier) @name) @definition
"#;

/// TypeScript import extraction query.
const TYPESCRIPT_IMPORT_QUERY: &str = r#"
(import_statement source: (string) @import)
"#;

/// JavaScript symbol extraction query (uses TSX grammar, same queries as TypeScript).
const JAVASCRIPT_SYMBOL_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @definition
(class_declaration name: (type_identifier) @name) @definition
(interface_declaration name: (type_identifier) @name) @definition
(enum_declaration name: (identifier) @name) @definition
(type_alias_declaration name: (type_identifier) @name) @definition
(method_definition name: (property_identifier) @name) @definition
"#;

/// JavaScript import extraction query.
const JAVASCRIPT_IMPORT_QUERY: &str = r#"
(import_statement source: (string) @import)
"#;

/// Python symbol extraction query.
const PYTHON_SYMBOL_QUERY: &str = r#"
(function_definition name: (identifier) @name) @definition
(class_definition name: (identifier) @name) @definition
"#;

/// Python import extraction query.
const PYTHON_IMPORT_QUERY: &str = r#"
(import_statement name: (dotted_name) @import)
(import_from_statement module_name: (dotted_name) @import)
"#;

/// Go symbol extraction query.
const GO_SYMBOL_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @definition
(method_declaration name: (field_identifier) @name) @definition
(type_declaration (type_spec name: (type_identifier) @name)) @definition
"#;

/// Go import extraction query.
const GO_IMPORT_QUERY: &str = r#"
(import_spec path: (interpreted_string_literal) @import)
"#;

/// Java symbol extraction query.
const JAVA_SYMBOL_QUERY: &str = r#"
(class_declaration name: (identifier) @name) @definition
(interface_declaration name: (identifier) @name) @definition
(enum_declaration name: (identifier) @name) @definition
(method_declaration name: (identifier) @name) @definition
"#;

/// Java import extraction query.
const JAVA_IMPORT_QUERY: &str = r#"
(import_declaration (scoped_identifier) @import)
"#;

/// C symbol extraction query.
const C_SYMBOL_QUERY: &str = r#"
(function_definition declarator: (function_declarator declarator: (identifier) @name)) @definition
(struct_specifier name: (type_identifier) @name) @definition
(enum_specifier name: (type_identifier) @name) @definition
(type_definition declarator: (type_identifier) @name) @definition
"#;

/// C import extraction query.
const C_IMPORT_QUERY: &str = r#"
(preproc_include path: (_) @import)
"#;

/// C++ symbol extraction query.
const CPP_SYMBOL_QUERY: &str = r#"
(function_definition declarator: (function_declarator declarator: (identifier) @name)) @definition
(class_specifier name: (type_identifier) @name) @definition
(struct_specifier name: (type_identifier) @name) @definition
(enum_specifier name: (type_identifier) @name) @definition
"#;

/// C++ import extraction query.
const CPP_IMPORT_QUERY: &str = r#"
(preproc_include path: (_) @import)
"#;

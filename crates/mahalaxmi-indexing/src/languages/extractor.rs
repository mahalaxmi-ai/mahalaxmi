//! Symbol extraction trait and Tree-sitter based implementation.
//!
//! The [`SymbolExtractor`] trait defines the interface for extracting symbol
//! definitions and import statements from source code. The
//! [`TreeSitterExtractor`] provides a concrete implementation backed by
//! Tree-sitter queries, supporting any language with a Tree-sitter grammar.

use std::path::Path;

use streaming_iterator::StreamingIterator;

use crate::languages::config::LanguageConfig;
use crate::types::{ExtractedSymbol, SupportedLanguage, SymbolKind, Visibility};
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;

/// Trait for extracting symbol definitions and imports from source code.
///
/// Implementations must be `Send + Sync` to allow concurrent extraction
/// across multiple files.
pub trait SymbolExtractor: Send + Sync {
    /// Extract symbol definitions from source code.
    ///
    /// Parses the source, runs Tree-sitter queries, and returns all
    /// recognized symbol definitions (functions, structs, classes, etc.).
    fn extract_symbols(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<ExtractedSymbol>>;

    /// Extract import/use statements from source code.
    ///
    /// Returns the raw import path strings (e.g. `crate::types::Symbol`,
    /// `"./components"`, `from foo.bar import baz`).
    fn extract_imports(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<String>>;

    /// The language this extractor handles.
    fn language(&self) -> SupportedLanguage;
}

/// Tree-sitter based symbol extractor.
///
/// Stores the language grammar and query strings. A fresh
/// [`tree_sitter::Parser`] is created on each extraction call because
/// `Parser` is not `Send + Sync`.
pub struct TreeSitterExtractor {
    lang: SupportedLanguage,
    ts_language: tree_sitter::Language,
    symbol_query: String,
    import_query: String,
}

// Safety: tree_sitter::Language is Send + Sync.
// String is Send + Sync. SupportedLanguage is Send + Sync.
// We do not store Parser (which is NOT Send).
unsafe impl Send for TreeSitterExtractor {}
unsafe impl Sync for TreeSitterExtractor {}

impl TreeSitterExtractor {
    /// Create a new Tree-sitter based extractor.
    ///
    /// Stores the grammar and query patterns for later use. Does not create
    /// a parser yet — parsers are created per-call in extraction methods.
    ///
    /// # Errors
    /// Returns an error if the language configuration is invalid.
    pub fn new(
        lang: SupportedLanguage,
        config: &LanguageConfig,
        _i18n: &I18nService,
    ) -> MahalaxmiResult<Self> {
        Ok(Self {
            lang,
            ts_language: config.language.clone(),
            symbol_query: config.symbol_query.clone(),
            import_query: config.import_query.clone(),
        })
    }

    /// Parse source code into a Tree-sitter syntax tree.
    ///
    /// Creates a fresh parser, sets the language, and parses. Returns an error
    /// only on total parse failure (parser returns `None`). Partial parse
    /// errors are tolerated — the tree is returned with a warning logged.
    pub fn parse_source(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&self.ts_language).map_err(|_| {
            MahalaxmiError::indexing(
                i18n,
                keys::indexing::PARSE_FAILED,
                &[
                    ("file", &file_path.display().to_string()),
                    ("reason", "failed to set parser language"),
                ],
            )
        })?;

        let tree = parser.parse(source, None).ok_or_else(|| {
            MahalaxmiError::indexing(
                i18n,
                keys::indexing::PARSE_FAILED,
                &[
                    ("file", &file_path.display().to_string()),
                    ("reason", "parser returned no tree"),
                ],
            )
        })?;

        if tree.root_node().has_error() {
            tracing::warn!(
                file = %file_path.display(),
                "Partial parse errors detected, extracting available symbols"
            );
        }

        Ok(tree)
    }

    /// Run the symbol query against a parsed tree and extract symbols.
    fn extract_symbols_from_tree(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<ExtractedSymbol>> {
        let query =
            tree_sitter::Query::new(&self.ts_language, &self.symbol_query).map_err(|e| {
                MahalaxmiError::indexing(
                    i18n,
                    keys::indexing::EXTRACTION_FAILED,
                    &[
                        ("file", &file_path.display().to_string()),
                        ("reason", &format!("symbol query compilation: {e}")),
                    ],
                )
            })?;

        let source_bytes = source.as_bytes();
        let root_node = tree.root_node();
        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&query, root_node, source_bytes);

        let capture_names = query.capture_names();
        let name_idx = capture_index_by_name(capture_names, "name");
        let definition_idx = capture_index_by_name(capture_names, "definition");
        let impl_type_idx = capture_index_by_name(capture_names, "impl_type");

        let mut symbols = Vec::new();

        while let Some(m) = matches.next() {
            let mut name_text: Option<&str> = None;
            let mut definition_node: Option<tree_sitter::Node<'_>> = None;
            let mut impl_type_text: Option<&str> = None;

            for capture in m.captures {
                if Some(capture.index) == name_idx {
                    name_text = capture.node.utf8_text(source_bytes).ok();
                } else if Some(capture.index) == definition_idx {
                    definition_node = Some(capture.node);
                } else if Some(capture.index) == impl_type_idx {
                    impl_type_text = capture.node.utf8_text(source_bytes).ok();
                }
            }

            let name = match name_text {
                Some(n) if !n.is_empty() => n,
                _ => continue,
            };

            let def_node = match definition_node {
                Some(n) => n,
                None => continue,
            };

            let kind = match node_type_to_symbol_kind(def_node.kind()) {
                Some(k) => k,
                None => continue,
            };

            let visibility = detect_visibility(&def_node, source_bytes, self.lang);

            let line_start = def_node.start_position().row + 1;
            let line_end = def_node.end_position().row + 1;

            let signature = extract_signature(&def_node, source_bytes);

            let mut symbol = ExtractedSymbol::new(
                name.to_owned(),
                kind,
                visibility,
                file_path.to_path_buf(),
                line_start,
                line_end,
            );

            if let Some(sig) = signature {
                symbol = symbol.with_signature(sig);
            }

            if let Some(parent) = impl_type_text {
                symbol = symbol.with_parent(parent.to_owned());
            }

            symbols.push(symbol);
        }

        Ok(symbols)
    }

    /// Run the import query against a parsed tree and extract import paths.
    fn extract_imports_from_tree(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<String>> {
        let query =
            tree_sitter::Query::new(&self.ts_language, &self.import_query).map_err(|e| {
                MahalaxmiError::indexing(
                    i18n,
                    keys::indexing::EXTRACTION_FAILED,
                    &[
                        ("file", &file_path.display().to_string()),
                        ("reason", &format!("import query compilation: {e}")),
                    ],
                )
            })?;

        let source_bytes = source.as_bytes();
        let root_node = tree.root_node();
        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&query, root_node, source_bytes);

        let capture_names = query.capture_names();
        let import_idx = capture_index_by_name(capture_names, "import");

        let mut imports = Vec::new();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                if Some(capture.index) == import_idx {
                    if let Ok(text) = capture.node.utf8_text(source_bytes) {
                        let text: &str = text;
                        let cleaned = text.trim().trim_matches('"').trim_matches('\'');
                        if !cleaned.is_empty() {
                            imports.push(cleaned.to_owned());
                        }
                    }
                }
            }
        }

        Ok(imports)
    }
}

impl SymbolExtractor for TreeSitterExtractor {
    fn extract_symbols(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<ExtractedSymbol>> {
        let tree = self.parse_source(source, file_path, i18n)?;
        self.extract_symbols_from_tree(&tree, source, file_path, i18n)
    }

    fn extract_imports(
        &self,
        source: &str,
        file_path: &Path,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Vec<String>> {
        let tree = self.parse_source(source, file_path, i18n)?;
        self.extract_imports_from_tree(&tree, source, file_path, i18n)
    }

    fn language(&self) -> SupportedLanguage {
        self.lang
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Find the capture index for a named capture in the query's capture list.
fn capture_index_by_name(capture_names: &[&str], name: &str) -> Option<u32> {
    capture_names
        .iter()
        .position(|n| *n == name)
        .map(|i| i as u32)
}

/// Map Tree-sitter node types to [`SymbolKind`] values.
fn node_type_to_symbol_kind(node_type: &str) -> Option<SymbolKind> {
    match node_type {
        "function_item" | "function_definition" | "function_declaration" => {
            Some(SymbolKind::Function)
        }
        "method_definition" | "method_declaration" => Some(SymbolKind::Method),
        "struct_item" | "struct_specifier" => Some(SymbolKind::Struct),
        "trait_item" => Some(SymbolKind::Trait),
        "class_declaration" | "class_definition" | "class_specifier" => Some(SymbolKind::Class),
        "interface_declaration" => Some(SymbolKind::Interface),
        "enum_item" | "enum_declaration" | "enum_specifier" => Some(SymbolKind::Enum),
        "type_item" | "type_alias_declaration" | "type_definition" => Some(SymbolKind::TypeAlias),
        "const_item" | "const_declaration" => Some(SymbolKind::Constant),
        "mod_item" | "namespace_definition" => Some(SymbolKind::Module),
        _ => None,
    }
}

/// Detect visibility of a definition node based on language conventions.
///
/// - **Rust**: looks for a `visibility_modifier` child (`pub`, `pub(crate)`).
/// - **TypeScript/JavaScript**: checks if the definition's parent is an
///   `export_statement`.
/// - **Python**: checks name prefix (`_` = Private, `__` = Restricted, else Public).
/// - **Go**: checks if first character of the name is uppercase (Public) or not.
/// - **Java**: looks for a `modifiers` child containing access keywords.
/// - **C/C++**: checks for `static` storage class specifier (Private), else Public.
fn detect_visibility(
    definition_node: &tree_sitter::Node,
    source: &[u8],
    lang: SupportedLanguage,
) -> Visibility {
    match lang {
        SupportedLanguage::Rust => detect_rust_visibility(definition_node, source),
        SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
            detect_ts_visibility(definition_node)
        }
        SupportedLanguage::Python => detect_python_visibility(definition_node, source),
        SupportedLanguage::Go => detect_go_visibility(definition_node, source),
        SupportedLanguage::Java => detect_java_visibility(definition_node, source),
        SupportedLanguage::C | SupportedLanguage::Cpp => {
            detect_c_visibility(definition_node, source)
        }
    }
}

/// Rust visibility: look for `visibility_modifier` child.
fn detect_rust_visibility(node: &tree_sitter::Node, source: &[u8]) -> Visibility {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "visibility_modifier" {
                if let Ok(text) = child.utf8_text(source) {
                    if text.contains("pub(crate)") {
                        return Visibility::Crate;
                    }
                    if text.contains("pub(super)") || text.contains("pub(in") {
                        return Visibility::Restricted;
                    }
                    if text.starts_with("pub") {
                        return Visibility::Public;
                    }
                }
            }
        }
    }
    Visibility::Private
}

/// TypeScript/JavaScript visibility: check if parent is `export_statement`.
fn detect_ts_visibility(node: &tree_sitter::Node) -> Visibility {
    if let Some(parent) = node.parent() {
        if parent.kind() == "export_statement" {
            return Visibility::Public;
        }
    }
    Visibility::Private
}

/// Python visibility: based on name prefix convention.
fn detect_python_visibility(node: &tree_sitter::Node, source: &[u8]) -> Visibility {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("");

    if name.starts_with("__") && !name.ends_with("__") {
        Visibility::Restricted
    } else if name.starts_with('_') {
        Visibility::Private
    } else {
        Visibility::Public
    }
}

/// Go visibility: uppercase first letter means exported (public).
fn detect_go_visibility(node: &tree_sitter::Node, source: &[u8]) -> Visibility {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("");

    if name.starts_with(|c: char| c.is_uppercase()) {
        Visibility::Public
    } else {
        Visibility::Private
    }
}

/// Java visibility: look for `modifiers` child with access keywords.
fn detect_java_visibility(node: &tree_sitter::Node, source: &[u8]) -> Visibility {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "modifiers" {
                if let Ok(text) = child.utf8_text(source) {
                    if text.contains("public") {
                        return Visibility::Public;
                    }
                    if text.contains("protected") {
                        return Visibility::Restricted;
                    }
                    if text.contains("private") {
                        return Visibility::Private;
                    }
                }
            }
        }
    }
    Visibility::Private
}

/// C/C++ visibility: check for `static` storage specifier.
fn detect_c_visibility(node: &tree_sitter::Node, source: &[u8]) -> Visibility {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "storage_class_specifier" {
                if let Ok(text) = child.utf8_text(source) {
                    if text == "static" {
                        return Visibility::Private;
                    }
                }
            }
        }
    }
    Visibility::Public
}

/// Extract a signature from the first line of a definition node.
///
/// Returns the first line of the node's text, trimmed. For multi-line
/// definitions, this captures the declaration line (e.g.
/// `fn foo(x: i32) -> bool`, `class MyClass extends Base`).
fn extract_signature(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let text = node.utf8_text(source).ok()?;
    let first_line = text.lines().next()?;
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_owned())
}

//! Symbol classification, visibility, and extracted symbol representation.
//!
//! These types capture the results of tree-sitter AST analysis: what symbols
//! exist in a file, their kind, visibility, location, and optional metadata
//! like signatures and doc comments.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// The kind of a symbol extracted from source code.
///
/// Covers the major categories of named entities found across
/// all supported languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    /// A standalone function definition.
    Function,
    /// A method defined inside a type (impl block, class, etc.).
    Method,
    /// A struct type definition.
    Struct,
    /// A trait definition (Rust-specific).
    Trait,
    /// A class definition (TypeScript, Python, Java, C++, etc.).
    Class,
    /// An interface definition (TypeScript, Java, Go).
    Interface,
    /// An enum type definition.
    Enum,
    /// A variant within an enum.
    EnumVariant,
    /// A type alias definition.
    TypeAlias,
    /// A constant value definition.
    Constant,
    /// A module or namespace definition.
    Module,
    /// A field within a struct, class, or similar container.
    Field,
}

impl SymbolKind {
    /// Returns the kind as a static string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
            Self::Struct => "struct",
            Self::Trait => "trait",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::Enum => "enum",
            Self::EnumVariant => "enum_variant",
            Self::TypeAlias => "type_alias",
            Self::Constant => "constant",
            Self::Module => "module",
            Self::Field => "field",
        }
    }

    /// Returns `true` if this kind represents a type definition.
    ///
    /// Type definitions include structs, traits, classes, interfaces,
    /// enums, and type aliases.
    pub fn is_type_definition(&self) -> bool {
        matches!(
            self,
            Self::Struct
                | Self::Trait
                | Self::Class
                | Self::Interface
                | Self::Enum
                | Self::TypeAlias
        )
    }

    /// Returns `true` if this kind represents a callable entity.
    ///
    /// Callable entities are functions and methods.
    pub fn is_callable(&self) -> bool {
        matches!(self, Self::Function | Self::Method)
    }
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The visibility level of a symbol.
///
/// Maps to language-specific visibility concepts:
/// - Rust: `pub`, `pub(crate)`, `pub(super)`, private
/// - TypeScript: `export`, non-exported
/// - Python: naming conventions (`_private`, `__restricted`)
/// - Go: capitalization convention
/// - Java/C++: `public`, `private`, `protected`
/// - C: `static` (file-local) vs external
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// The symbol is publicly accessible.
    Public,
    /// The symbol is private to its enclosing scope.
    Private,
    /// The symbol is visible within the current crate or module.
    Crate,
    /// The symbol has restricted visibility (e.g., `protected` in Java/C++).
    Restricted,
}

impl Visibility {
    /// Returns the visibility as a static string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Crate => "crate",
            Self::Restricted => "restricted",
        }
    }

    /// Returns `true` if this visibility level is public.
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A symbol extracted from a source file via tree-sitter AST analysis.
///
/// Captures the symbol's name, kind, visibility, file location, and optional
/// metadata such as its signature, doc comment, and parent scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedSymbol {
    /// The symbol's name (e.g., function name, struct name).
    pub name: String,
    /// The kind of symbol (function, struct, trait, etc.).
    pub kind: SymbolKind,
    /// The visibility level of the symbol.
    pub visibility: Visibility,
    /// The file path where this symbol is defined.
    pub file_path: PathBuf,
    /// The starting line number (1-based).
    pub line_start: usize,
    /// The ending line number (1-based, inclusive).
    pub line_end: usize,
    /// The symbol's type signature, if available.
    ///
    /// For functions: `fn name(params) -> ReturnType`
    /// For structs: `struct Name`
    /// For traits: `trait Name`
    pub signature: Option<String>,
    /// The doc comment associated with this symbol, if any.
    pub doc_comment: Option<String>,
    /// The name of the enclosing parent scope, if any.
    ///
    /// For methods, this is the impl target type or class name.
    /// For enum variants, this is the enum name.
    /// For fields, this is the struct or class name.
    pub parent: Option<String>,
}

impl ExtractedSymbol {
    /// Create a new extracted symbol with required fields.
    ///
    /// Optional fields (signature, doc_comment, parent) default to `None`.
    /// Use the builder methods to set them.
    pub fn new(
        name: impl Into<String>,
        kind: SymbolKind,
        visibility: Visibility,
        file_path: impl Into<PathBuf>,
        line_start: usize,
        line_end: usize,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            visibility,
            file_path: file_path.into(),
            line_start,
            line_end,
            signature: None,
            doc_comment: None,
            parent: None,
        }
    }

    /// Set the type signature for this symbol.
    pub fn with_signature(mut self, sig: impl Into<String>) -> Self {
        self.signature = Some(sig.into());
        self
    }

    /// Set the doc comment for this symbol.
    pub fn with_doc_comment(mut self, doc: impl Into<String>) -> Self {
        self.doc_comment = Some(doc.into());
        self
    }

    /// Set the parent scope name for this symbol.
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Returns the qualified name of the symbol.
    ///
    /// If the symbol has a parent, returns `parent::name`.
    /// Otherwise, returns just the symbol name.
    pub fn qualified_name(&self) -> String {
        match &self.parent {
            Some(parent) => format!("{parent}::{}", self.name),
            None => self.name.clone(),
        }
    }

    /// Returns the number of lines this symbol spans.
    ///
    /// Computed as `line_end - line_start + 1`.
    pub fn line_count(&self) -> usize {
        self.line_end - self.line_start + 1
    }

    /// Returns `true` if this symbol has public visibility.
    pub fn is_public(&self) -> bool {
        self.visibility.is_public()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_kind_as_str() {
        assert_eq!(SymbolKind::Function.as_str(), "function");
        assert_eq!(SymbolKind::Method.as_str(), "method");
        assert_eq!(SymbolKind::Struct.as_str(), "struct");
        assert_eq!(SymbolKind::Trait.as_str(), "trait");
        assert_eq!(SymbolKind::Class.as_str(), "class");
        assert_eq!(SymbolKind::Interface.as_str(), "interface");
        assert_eq!(SymbolKind::Enum.as_str(), "enum");
        assert_eq!(SymbolKind::EnumVariant.as_str(), "enum_variant");
        assert_eq!(SymbolKind::TypeAlias.as_str(), "type_alias");
        assert_eq!(SymbolKind::Constant.as_str(), "constant");
        assert_eq!(SymbolKind::Module.as_str(), "module");
        assert_eq!(SymbolKind::Field.as_str(), "field");
    }

    #[test]
    fn symbol_kind_is_type_definition() {
        let type_defs = [
            SymbolKind::Struct,
            SymbolKind::Trait,
            SymbolKind::Class,
            SymbolKind::Interface,
            SymbolKind::Enum,
            SymbolKind::TypeAlias,
        ];
        for kind in &type_defs {
            assert!(
                kind.is_type_definition(),
                "{:?} should be a type definition",
                kind
            );
        }

        let non_type_defs = [
            SymbolKind::Function,
            SymbolKind::Method,
            SymbolKind::EnumVariant,
            SymbolKind::Constant,
            SymbolKind::Module,
            SymbolKind::Field,
        ];
        for kind in &non_type_defs {
            assert!(
                !kind.is_type_definition(),
                "{:?} should not be a type definition",
                kind
            );
        }
    }

    #[test]
    fn symbol_kind_is_callable() {
        assert!(SymbolKind::Function.is_callable());
        assert!(SymbolKind::Method.is_callable());
        assert!(!SymbolKind::Struct.is_callable());
        assert!(!SymbolKind::Trait.is_callable());
        assert!(!SymbolKind::Class.is_callable());
        assert!(!SymbolKind::Interface.is_callable());
        assert!(!SymbolKind::Enum.is_callable());
        assert!(!SymbolKind::EnumVariant.is_callable());
        assert!(!SymbolKind::TypeAlias.is_callable());
        assert!(!SymbolKind::Constant.is_callable());
        assert!(!SymbolKind::Module.is_callable());
        assert!(!SymbolKind::Field.is_callable());
    }

    #[test]
    fn symbol_kind_display() {
        for kind in [
            SymbolKind::Function,
            SymbolKind::Method,
            SymbolKind::Struct,
            SymbolKind::Trait,
            SymbolKind::Class,
            SymbolKind::Interface,
            SymbolKind::Enum,
            SymbolKind::EnumVariant,
            SymbolKind::TypeAlias,
            SymbolKind::Constant,
            SymbolKind::Module,
            SymbolKind::Field,
        ] {
            assert_eq!(format!("{kind}"), kind.as_str());
        }
    }

    #[test]
    fn visibility_as_str() {
        assert_eq!(Visibility::Public.as_str(), "public");
        assert_eq!(Visibility::Private.as_str(), "private");
        assert_eq!(Visibility::Crate.as_str(), "crate");
        assert_eq!(Visibility::Restricted.as_str(), "restricted");
    }

    #[test]
    fn visibility_is_public() {
        assert!(Visibility::Public.is_public());
        assert!(!Visibility::Private.is_public());
        assert!(!Visibility::Crate.is_public());
        assert!(!Visibility::Restricted.is_public());
    }

    #[test]
    fn visibility_display() {
        for vis in [
            Visibility::Public,
            Visibility::Private,
            Visibility::Crate,
            Visibility::Restricted,
        ] {
            assert_eq!(format!("{vis}"), vis.as_str());
        }
    }

    #[test]
    fn extracted_symbol_new() {
        let sym = ExtractedSymbol::new(
            "my_function",
            SymbolKind::Function,
            Visibility::Public,
            "src/lib.rs",
            10,
            25,
        );
        assert_eq!(sym.name, "my_function");
        assert_eq!(sym.kind, SymbolKind::Function);
        assert_eq!(sym.visibility, Visibility::Public);
        assert_eq!(sym.file_path, PathBuf::from("src/lib.rs"));
        assert_eq!(sym.line_start, 10);
        assert_eq!(sym.line_end, 25);
        assert!(sym.signature.is_none());
        assert!(sym.doc_comment.is_none());
        assert!(sym.parent.is_none());
    }

    #[test]
    fn extracted_symbol_builder_methods() {
        let sym = ExtractedSymbol::new(
            "process",
            SymbolKind::Method,
            Visibility::Public,
            "src/engine.rs",
            5,
            15,
        )
        .with_signature("fn process(&self, input: &str) -> Result<()>")
        .with_doc_comment("Processes the given input string.")
        .with_parent("Engine");

        assert_eq!(
            sym.signature.as_deref(),
            Some("fn process(&self, input: &str) -> Result<()>")
        );
        assert_eq!(
            sym.doc_comment.as_deref(),
            Some("Processes the given input string.")
        );
        assert_eq!(sym.parent.as_deref(), Some("Engine"));
    }

    #[test]
    fn extracted_symbol_qualified_name_with_parent() {
        let sym = ExtractedSymbol::new(
            "render",
            SymbolKind::Method,
            Visibility::Public,
            "src/ui.rs",
            1,
            10,
        )
        .with_parent("Widget");

        assert_eq!(sym.qualified_name(), "Widget::render");
    }

    #[test]
    fn extracted_symbol_qualified_name_without_parent() {
        let sym = ExtractedSymbol::new(
            "main",
            SymbolKind::Function,
            Visibility::Public,
            "src/main.rs",
            1,
            5,
        );
        assert_eq!(sym.qualified_name(), "main");
    }

    #[test]
    fn extracted_symbol_line_count() {
        let sym = ExtractedSymbol::new(
            "test",
            SymbolKind::Function,
            Visibility::Private,
            "test.rs",
            10,
            25,
        );
        assert_eq!(sym.line_count(), 16);

        let single_line = ExtractedSymbol::new(
            "CONST",
            SymbolKind::Constant,
            Visibility::Public,
            "consts.rs",
            5,
            5,
        );
        assert_eq!(single_line.line_count(), 1);
    }

    #[test]
    fn extracted_symbol_is_public() {
        let public_sym = ExtractedSymbol::new(
            "pub_fn",
            SymbolKind::Function,
            Visibility::Public,
            "lib.rs",
            1,
            5,
        );
        assert!(public_sym.is_public());

        let private_sym = ExtractedSymbol::new(
            "priv_fn",
            SymbolKind::Function,
            Visibility::Private,
            "lib.rs",
            6,
            10,
        );
        assert!(!private_sym.is_public());
    }

    #[test]
    fn symbol_kind_serde_roundtrip() {
        for kind in [
            SymbolKind::Function,
            SymbolKind::Method,
            SymbolKind::Struct,
            SymbolKind::Trait,
            SymbolKind::Class,
            SymbolKind::Interface,
            SymbolKind::Enum,
            SymbolKind::EnumVariant,
            SymbolKind::TypeAlias,
            SymbolKind::Constant,
            SymbolKind::Module,
            SymbolKind::Field,
        ] {
            let json = serde_json::to_string(&kind).expect("serialize");
            let deserialized: SymbolKind = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(kind, deserialized);
        }
    }

    #[test]
    fn visibility_serde_roundtrip() {
        for vis in [
            Visibility::Public,
            Visibility::Private,
            Visibility::Crate,
            Visibility::Restricted,
        ] {
            let json = serde_json::to_string(&vis).expect("serialize");
            let deserialized: Visibility = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(vis, deserialized);
        }
    }

    #[test]
    fn extracted_symbol_serde_roundtrip() {
        let sym = ExtractedSymbol::new(
            "process",
            SymbolKind::Method,
            Visibility::Public,
            "src/engine.rs",
            5,
            15,
        )
        .with_signature("fn process(&self, input: &str) -> Result<()>")
        .with_doc_comment("Process input data.")
        .with_parent("Engine");

        let json = serde_json::to_string(&sym).expect("serialize");
        let deserialized: ExtractedSymbol = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(sym, deserialized);
    }
}

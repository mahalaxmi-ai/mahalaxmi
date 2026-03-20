//! Integration tests for tree-sitter symbol extraction.
//!
//! Each test creates a [`TreeSitterExtractor`] from the default
//! [`LanguageRegistry`] configuration and feeds it a small source snippet.
//! Assertions verify that the expected symbols are extracted with correct
//! name, kind, visibility, and (where applicable) parent information.
//!
//! The queries are defined in `registry.rs` and the node-type-to-kind
//! mapping lives in `extractor.rs` (`node_type_to_symbol_kind`).

use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_indexing::languages::{LanguageRegistry, SymbolExtractor, TreeSitterExtractor};
use mahalaxmi_indexing::types::{SupportedLanguage, SymbolKind, Visibility};
use std::path::Path;

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

/// Helper: create a [`TreeSitterExtractor`] for the given language from the
/// default registry.
fn extractor_for(lang: SupportedLanguage) -> TreeSitterExtractor {
    let i18n = test_i18n();
    let registry = LanguageRegistry::with_defaults();
    let config = registry
        .get(&lang)
        .expect("default registry should contain all languages");
    TreeSitterExtractor::new(lang, config, &i18n).expect("extractor creation should succeed")
}

// -----------------------------------------------------------------------
// Rust extraction tests
// -----------------------------------------------------------------------

#[test]
fn rust_extract_function() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Rust);

    let source = "fn hello() {}";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.rs"), &i18n)
        .expect("extraction should succeed");

    let func = symbols
        .iter()
        .find(|s| s.name == "hello")
        .expect("should find symbol named 'hello'");
    assert_eq!(func.kind, SymbolKind::Function);
    assert_eq!(func.visibility, Visibility::Private);
    assert!(func.signature.is_some());
}

#[test]
fn rust_extract_pub_struct() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Rust);

    let source = "pub struct Foo { x: i32 }";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.rs"), &i18n)
        .expect("extraction should succeed");

    let st = symbols
        .iter()
        .find(|s| s.name == "Foo")
        .expect("should find symbol named 'Foo'");
    assert_eq!(st.kind, SymbolKind::Struct);
    assert_eq!(st.visibility, Visibility::Public);
}

#[test]
fn rust_extract_impl_method() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Rust);

    // The impl query captures function_item nodes inside impl blocks.
    // The node type `function_item` maps to SymbolKind::Function.
    // The @impl_type capture sets the parent to "S".
    let source = r#"
struct S {}
impl S {
    fn method(&self) {}
}
"#;
    let symbols = extractor
        .extract_symbols(source, Path::new("test.rs"), &i18n)
        .expect("extraction should succeed");

    // The struct "S" should be found.
    let struct_sym = symbols
        .iter()
        .find(|s| s.name == "S" && s.kind == SymbolKind::Struct);
    assert!(
        struct_sym.is_some(),
        "should find struct 'S'; found: {:?}",
        symbols
    );

    // The method inside the impl block should be found as a Function with parent "S".
    let method = symbols
        .iter()
        .find(|s| s.name == "method")
        .expect("should find symbol named 'method'");
    assert_eq!(
        method.kind,
        SymbolKind::Function,
        "impl method node type is function_item, which maps to Function"
    );
    assert_eq!(
        method.parent.as_deref(),
        Some("S"),
        "impl method should have parent set to the impl target type"
    );
}

#[test]
fn rust_extract_enum() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Rust);

    let source = "enum Color { Red, Green, Blue }";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.rs"), &i18n)
        .expect("extraction should succeed");

    let en = symbols
        .iter()
        .find(|s| s.name == "Color")
        .expect("should find symbol named 'Color'");
    assert_eq!(en.kind, SymbolKind::Enum);
    assert_eq!(en.visibility, Visibility::Private);
}

#[test]
fn rust_extract_trait() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Rust);

    let source = "trait Drawable { fn draw(&self); }";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.rs"), &i18n)
        .expect("extraction should succeed");

    let tr = symbols
        .iter()
        .find(|s| s.name == "Drawable")
        .expect("should find symbol named 'Drawable'");
    assert_eq!(tr.kind, SymbolKind::Trait);
    assert_eq!(tr.visibility, Visibility::Private);
}

#[test]
fn rust_extract_const() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Rust);

    let source = "const MAX: u32 = 100;";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.rs"), &i18n)
        .expect("extraction should succeed");

    let c = symbols
        .iter()
        .find(|s| s.name == "MAX")
        .expect("should find symbol named 'MAX'");
    assert_eq!(c.kind, SymbolKind::Constant);
    assert_eq!(c.visibility, Visibility::Private);
}

#[test]
fn rust_extract_mod() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Rust);

    let source = "mod inner {}";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.rs"), &i18n)
        .expect("extraction should succeed");

    let m = symbols
        .iter()
        .find(|s| s.name == "inner")
        .expect("should find symbol named 'inner'");
    assert_eq!(m.kind, SymbolKind::Module);
    assert_eq!(m.visibility, Visibility::Private);
}

#[test]
fn rust_extract_imports() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Rust);

    let source = "use std::collections::HashMap;\nuse crate::types::Foo;";
    let imports = extractor
        .extract_imports(source, Path::new("test.rs"), &i18n)
        .expect("import extraction should succeed");

    assert!(
        imports.len() >= 2,
        "should extract at least 2 imports; got: {:?}",
        imports
    );

    let has_hashmap = imports
        .iter()
        .any(|i| i.contains("std::collections::HashMap"));
    assert!(
        has_hashmap,
        "should contain HashMap import; got: {:?}",
        imports
    );

    let has_foo = imports.iter().any(|i| i.contains("crate::types::Foo"));
    assert!(
        has_foo,
        "should contain crate::types::Foo import; got: {:?}",
        imports
    );
}

// -----------------------------------------------------------------------
// TypeScript extraction tests
// -----------------------------------------------------------------------

#[test]
fn typescript_extract_function() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::TypeScript);

    let source = "function greet(name: string): void {}";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.ts"), &i18n)
        .expect("extraction should succeed");

    let func = symbols
        .iter()
        .find(|s| s.name == "greet")
        .expect("should find symbol named 'greet'");
    assert_eq!(func.kind, SymbolKind::Function);
    // Not exported, so should be private.
    assert_eq!(func.visibility, Visibility::Private);
}

#[test]
fn typescript_extract_class() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::TypeScript);

    let source = r#"
class Animal {
    constructor() {}
    speak(): void {}
}
"#;
    let symbols = extractor
        .extract_symbols(source, Path::new("test.ts"), &i18n)
        .expect("extraction should succeed");

    let cls = symbols
        .iter()
        .find(|s| s.name == "Animal")
        .expect("should find symbol named 'Animal'");
    assert_eq!(cls.kind, SymbolKind::Class);

    // The class is not exported, so visibility should be Private.
    assert_eq!(cls.visibility, Visibility::Private);

    // The method `speak` should also be extracted (method_definition query).
    let speak = symbols.iter().find(|s| s.name == "speak");
    assert!(
        speak.is_some(),
        "should find method 'speak'; found: {:?}",
        symbols.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    if let Some(m) = speak {
        assert_eq!(m.kind, SymbolKind::Method);
    }
}

#[test]
fn typescript_extract_exported_function() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::TypeScript);

    let source = "export function add(a: number, b: number): number { return a + b; }";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.ts"), &i18n)
        .expect("extraction should succeed");

    let func = symbols
        .iter()
        .find(|s| s.name == "add")
        .expect("should find symbol named 'add'");
    assert_eq!(func.kind, SymbolKind::Function);
    // `export` makes the parent node an `export_statement`, detected as Public.
    assert_eq!(func.visibility, Visibility::Public);
}

// -----------------------------------------------------------------------
// Python extraction tests
// -----------------------------------------------------------------------

#[test]
fn python_extract_function() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Python);

    let source = "def greet(name):\n    pass";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.py"), &i18n)
        .expect("extraction should succeed");

    let func = symbols
        .iter()
        .find(|s| s.name == "greet")
        .expect("should find symbol named 'greet'");
    assert_eq!(func.kind, SymbolKind::Function);
    // Python public by convention (no underscore prefix).
    assert_eq!(func.visibility, Visibility::Public);
}

#[test]
fn python_extract_class() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Python);

    let source = "class Dog:\n    def bark(self):\n        pass";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.py"), &i18n)
        .expect("extraction should succeed");

    let cls = symbols
        .iter()
        .find(|s| s.name == "Dog")
        .expect("should find symbol named 'Dog'");
    assert_eq!(cls.kind, SymbolKind::Class);
    assert_eq!(cls.visibility, Visibility::Public);

    // The `bark` method inside the class is a function_definition, so it
    // gets extracted as SymbolKind::Function (not Method, because the
    // Python query uses `function_definition` which maps to Function).
    let bark = symbols.iter().find(|s| s.name == "bark");
    assert!(
        bark.is_some(),
        "should find nested function 'bark'; found: {:?}",
        symbols.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    if let Some(b) = bark {
        assert_eq!(b.kind, SymbolKind::Function);
    }
}

// -----------------------------------------------------------------------
// Go extraction tests
// -----------------------------------------------------------------------

#[test]
fn go_extract_function() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Go);

    let source = "package main\n\nfunc main() {}";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.go"), &i18n)
        .expect("extraction should succeed");

    let func = symbols
        .iter()
        .find(|s| s.name == "main")
        .expect("should find symbol named 'main'");
    assert_eq!(func.kind, SymbolKind::Function);
    // `main` starts with lowercase, so Go convention says Private.
    assert_eq!(func.visibility, Visibility::Private);
}

// -----------------------------------------------------------------------
// Java extraction tests
// -----------------------------------------------------------------------

#[test]
fn java_extract_class() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::Java);

    let source = r#"
public class HelloWorld {
    public static void main(String[] args) {}
}
"#;
    let symbols = extractor
        .extract_symbols(source, Path::new("HelloWorld.java"), &i18n)
        .expect("extraction should succeed");

    let cls = symbols
        .iter()
        .find(|s| s.name == "HelloWorld")
        .expect("should find symbol named 'HelloWorld'");
    assert_eq!(cls.kind, SymbolKind::Class);
    assert_eq!(cls.visibility, Visibility::Public);

    // The `main` method should also be extracted (method_declaration query).
    let main_method = symbols.iter().find(|s| s.name == "main");
    assert!(
        main_method.is_some(),
        "should find method 'main'; found: {:?}",
        symbols.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    if let Some(m) = main_method {
        assert_eq!(m.kind, SymbolKind::Method);
        assert_eq!(m.visibility, Visibility::Public);
    }
}

// -----------------------------------------------------------------------
// C extraction tests
// -----------------------------------------------------------------------

#[test]
fn c_extract_function() {
    let i18n = test_i18n();
    let extractor = extractor_for(SupportedLanguage::C);

    let source = "int add(int a, int b) { return a + b; }";
    let symbols = extractor
        .extract_symbols(source, Path::new("test.c"), &i18n)
        .expect("extraction should succeed");

    let func = symbols
        .iter()
        .find(|s| s.name == "add")
        .expect("should find symbol named 'add'");
    assert_eq!(func.kind, SymbolKind::Function);
    // Non-static C function is Public by convention.
    assert_eq!(func.visibility, Visibility::Public);
}

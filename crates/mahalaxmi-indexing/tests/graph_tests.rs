//! Integration tests for the file dependency graph.
//!
//! Covers import resolution for all supported languages, graph construction
//! from extraction data, topological ordering, and degree calculations.

use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_indexing::graph::{FileDependency, FileDependencyGraph, ImportResolver};
use mahalaxmi_indexing::types::SupportedLanguage;
use std::path::{Path, PathBuf};

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

// ---------------------------------------------------------------------------
// Import resolver: Rust
// ---------------------------------------------------------------------------

#[test]
fn import_resolver_rust_crate_path() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create the target file so .exists() succeeds: src/models/user.rs
    std::fs::create_dir_all(root.join("src/models")).expect("create src/models");
    std::fs::write(root.join("src/models/user.rs"), "pub struct User;").expect("write user.rs");

    let resolver = ImportResolver::new(&root, SupportedLanguage::Rust);
    let from_file = root.join("src/main.rs");

    let resolved = resolver.resolve("crate::models::user", &from_file);
    assert!(resolved.is_some(), "should resolve crate::models::user");
    assert_eq!(resolved.unwrap(), root.join("src/models/user.rs"));
}

#[test]
fn import_resolver_rust_crate_path_mod_rs() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create the mod.rs variant: src/models/user/mod.rs
    std::fs::create_dir_all(root.join("src/models/user")).expect("create src/models/user");
    std::fs::write(root.join("src/models/user/mod.rs"), "pub struct User;").expect("write mod.rs");

    let resolver = ImportResolver::new(&root, SupportedLanguage::Rust);
    let from_file = root.join("src/main.rs");

    let resolved = resolver.resolve("crate::models::user", &from_file);
    assert!(
        resolved.is_some(),
        "should resolve crate::models::user to mod.rs"
    );
    assert_eq!(resolved.unwrap(), root.join("src/models/user/mod.rs"));
}

#[test]
fn import_resolver_rust_crate_path_nonexistent() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Don't create any files
    let resolver = ImportResolver::new(&root, SupportedLanguage::Rust);
    let from_file = root.join("src/main.rs");

    let resolved = resolver.resolve("crate::models::user", &from_file);
    assert!(
        resolved.is_none(),
        "should return None when target does not exist"
    );
}

#[test]
fn import_resolver_rust_module_path() {
    // The Rust resolver only handles `crate::` prefixes. Imports like
    // `super::util` are not supported because `resolve_rust` strips `crate::`
    // and returns None if the prefix is absent.
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    std::fs::create_dir_all(root.join("src/sub")).expect("create src/sub");
    std::fs::write(root.join("src/util.rs"), "pub fn helper() {}").expect("write util.rs");

    let resolver = ImportResolver::new(&root, SupportedLanguage::Rust);
    let from_file = root.join("src/sub/mod.rs");

    let resolved = resolver.resolve("super::util", &from_file);
    assert!(
        resolved.is_none(),
        "super:: imports are not resolved by the current Rust resolver"
    );
}

// ---------------------------------------------------------------------------
// Import resolver: TypeScript
// ---------------------------------------------------------------------------

#[test]
fn import_resolver_typescript_relative() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create target: src/utils.ts
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(root.join("src/utils.ts"), "export function format() {}")
        .expect("write utils.ts");

    let resolver = ImportResolver::new(&root, SupportedLanguage::TypeScript);
    let from_file = root.join("src/main.ts");

    let resolved = resolver.resolve("./utils", &from_file);
    assert!(resolved.is_some(), "should resolve ./utils to src/utils.ts");
    // The resolver joins from_file's parent with "./utils" then appends .ts,
    // resulting in a path like src/./utils.ts — canonicalize to compare.
    let resolved_canonical = resolved
        .unwrap()
        .canonicalize()
        .expect("canonicalize resolved path");
    let expected_canonical = root
        .join("src/utils.ts")
        .canonicalize()
        .expect("canonicalize expected path");
    assert_eq!(resolved_canonical, expected_canonical);
}

#[test]
fn import_resolver_typescript_relative_tsx() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create target: src/Button.tsx (not .ts)
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(
        root.join("src/Button.tsx"),
        "export function Button() { return null; }",
    )
    .expect("write Button.tsx");

    let resolver = ImportResolver::new(&root, SupportedLanguage::TypeScript);
    let from_file = root.join("src/App.tsx");

    let resolved = resolver.resolve("./Button", &from_file);
    assert!(
        resolved.is_some(),
        "should resolve ./Button to src/Button.tsx"
    );
    let resolved_canonical = resolved
        .unwrap()
        .canonicalize()
        .expect("canonicalize resolved path");
    let expected_canonical = root
        .join("src/Button.tsx")
        .canonicalize()
        .expect("canonicalize expected path");
    assert_eq!(resolved_canonical, expected_canonical);
}

#[test]
fn import_resolver_typescript_relative_index() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create a directory with an index.ts
    std::fs::create_dir_all(root.join("src/components")).expect("create src/components");
    std::fs::write(
        root.join("src/components/index.ts"),
        "export * from './Button';",
    )
    .expect("write index.ts");

    let resolver = ImportResolver::new(&root, SupportedLanguage::TypeScript);
    let from_file = root.join("src/App.ts");

    let resolved = resolver.resolve("./components", &from_file);
    assert!(
        resolved.is_some(),
        "should resolve ./components to src/components/index.ts"
    );
    let resolved_canonical = resolved
        .unwrap()
        .canonicalize()
        .expect("canonicalize resolved path");
    let expected_canonical = root
        .join("src/components/index.ts")
        .canonicalize()
        .expect("canonicalize expected path");
    assert_eq!(resolved_canonical, expected_canonical);
}

// ---------------------------------------------------------------------------
// Import resolver: Python
// ---------------------------------------------------------------------------

#[test]
fn import_resolver_python_dotted() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create target: models/user.py
    std::fs::create_dir_all(root.join("models")).expect("create models");
    std::fs::write(root.join("models/user.py"), "class User: pass").expect("write user.py");

    let resolver = ImportResolver::new(&root, SupportedLanguage::Python);
    let from_file = root.join("src/main.py");

    let resolved = resolver.resolve("models.user", &from_file);
    assert!(
        resolved.is_some(),
        "should resolve models.user to models/user.py"
    );
    assert_eq!(resolved.unwrap(), root.join("models/user.py"));
}

#[test]
fn import_resolver_python_init_py() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create target: models/user/__init__.py
    std::fs::create_dir_all(root.join("models/user")).expect("create models/user");
    std::fs::write(root.join("models/user/__init__.py"), "class User: pass")
        .expect("write __init__.py");

    let resolver = ImportResolver::new(&root, SupportedLanguage::Python);
    let from_file = root.join("src/main.py");

    let resolved = resolver.resolve("models.user", &from_file);
    assert!(
        resolved.is_some(),
        "should resolve models.user to models/user/__init__.py"
    );
    assert_eq!(resolved.unwrap(), root.join("models/user/__init__.py"));
}

// ---------------------------------------------------------------------------
// Import resolver: Go
// ---------------------------------------------------------------------------

#[test]
fn import_resolver_go_path() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create target: utils.go
    std::fs::write(root.join("utils.go"), "package main\nfunc Helper() {}")
        .expect("write utils.go");

    let resolver = ImportResolver::new(&root, SupportedLanguage::Go);
    let from_file = root.join("main.go");

    let resolved = resolver.resolve("utils", &from_file);
    assert!(resolved.is_some(), "should resolve utils to utils.go");
    assert_eq!(resolved.unwrap(), root.join("utils.go"));
}

#[test]
fn import_resolver_go_package_directory() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create a package directory
    std::fs::create_dir_all(root.join("internal/util")).expect("create internal/util");
    std::fs::write(
        root.join("internal/util/helpers.go"),
        "package util\nfunc Help() {}",
    )
    .expect("write helpers.go");

    let resolver = ImportResolver::new(&root, SupportedLanguage::Go);
    let from_file = root.join("main.go");

    // The Go resolver checks if the path is a directory first
    let resolved = resolver.resolve("internal/util", &from_file);
    assert!(
        resolved.is_some(),
        "should resolve internal/util as a package directory"
    );
    assert_eq!(resolved.unwrap(), root.join("internal/util"));
}

// ---------------------------------------------------------------------------
// Import resolver: C/C++
// ---------------------------------------------------------------------------

#[test]
fn import_resolver_c_quoted_include() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create the header: src/utils.h
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(root.join("src/utils.h"), "int helper();").expect("write utils.h");

    let resolver = ImportResolver::new(&root, SupportedLanguage::C);
    let from_file = root.join("src/main.c");

    // Quoted include resolves relative to the importing file's directory
    let resolved = resolver.resolve("\"utils.h\"", &from_file);
    assert!(
        resolved.is_some(),
        "should resolve quoted include \"utils.h\""
    );
    assert_eq!(resolved.unwrap(), root.join("src/utils.h"));
}

#[test]
fn import_resolver_c_angle_bracket_returns_none() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    let resolver = ImportResolver::new(&root, SupportedLanguage::C);
    let from_file = root.join("src/main.c");

    // Angle-bracket includes are system headers and should return None
    let resolved = resolver.resolve("<stdio.h>", &from_file);
    assert!(
        resolved.is_none(),
        "angle-bracket includes should return None (system header)"
    );
}

#[test]
fn import_resolver_c_quoted_include_project_root() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Create header at project root level: include/config.h
    std::fs::create_dir_all(root.join("include")).expect("create include");
    std::fs::write(root.join("include/config.h"), "#define VERSION 1").expect("write config.h");

    let resolver = ImportResolver::new(&root, SupportedLanguage::C);
    let from_file = root.join("src/main.c");

    // The file is not relative to from_file's directory (src/), but IS relative
    // to the project root. The resolver falls back to root-relative lookup.
    let resolved = resolver.resolve("\"include/config.h\"", &from_file);
    assert!(
        resolved.is_some(),
        "should resolve quoted include relative to project root"
    );
    assert_eq!(resolved.unwrap(), root.join("include/config.h"));
}

// ---------------------------------------------------------------------------
// Graph construction from extractions
// ---------------------------------------------------------------------------

#[test]
fn build_from_extractions_creates_edges() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path().to_path_buf();

    // Set up a small Rust project with two files:
    // main.rs imports lib.rs via `crate::lib` (which resolves to src/lib.rs)
    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(root.join("src/lib.rs"), "pub fn run() {}").expect("write lib.rs");
    std::fs::write(root.join("src/main.rs"), "fn main() {}").expect("write main.rs");

    let main_path = root.join("src/main.rs");
    let lib_path = root.join("src/lib.rs");

    let extractions: Vec<(PathBuf, Vec<String>)> = vec![
        (main_path.clone(), vec!["crate::lib".to_string()]),
        (lib_path.clone(), vec![]),
    ];

    let graph =
        FileDependencyGraph::build_from_extractions(&extractions, &root, SupportedLanguage::Rust);

    // Both files should be in the graph
    assert_eq!(graph.file_count(), 2);

    // main.rs imports lib.rs, so edge main.rs -> lib.rs should exist
    assert_eq!(graph.edge_count(), 1);
    let deps = graph.dependencies_of(&main_path);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0], &lib_path);

    // lib.rs has no imports
    let lib_deps = graph.dependencies_of(&lib_path);
    assert!(lib_deps.is_empty());
}

// ---------------------------------------------------------------------------
// Topological sort
// ---------------------------------------------------------------------------

#[test]
fn topological_sort_linear_chain() {
    let i18n = test_i18n();

    // Create a linear chain: A imports B, B imports C.
    // A -> B -> C
    let mut graph = FileDependencyGraph::new();
    let a = PathBuf::from("/project/a.rs");
    let b = PathBuf::from("/project/b.rs");
    let c = PathBuf::from("/project/c.rs");

    graph.add_dependency(FileDependency::new(&a, &b, "crate::b"));
    graph.add_dependency(FileDependency::new(&b, &c, "crate::c"));

    let sorted = graph
        .topological_sort(&i18n)
        .expect("should succeed on acyclic graph");
    assert_eq!(sorted.len(), 3);

    // Dependencies come first in topological order.
    // C has no dependencies, so it should appear first.
    // B depends only on C, so B should come after C.
    // A depends on B, so A should come last.
    let c_pos = sorted.iter().position(|p| p == &c).expect("C in sorted");
    let b_pos = sorted.iter().position(|p| p == &b).expect("B in sorted");
    let a_pos = sorted.iter().position(|p| p == &a).expect("A in sorted");

    assert!(c_pos < b_pos, "C should come before B");
    assert!(b_pos < a_pos, "B should come before A");
}

#[test]
fn topological_sort_cycle_returns_error() {
    let i18n = test_i18n();

    let mut graph = FileDependencyGraph::new();
    let a = PathBuf::from("/project/a.rs");
    let b = PathBuf::from("/project/b.rs");

    graph.add_dependency(FileDependency::new(&a, &b, "b"));
    graph.add_dependency(FileDependency::new(&b, &a, "a"));

    let result = graph.topological_sort(&i18n);
    assert!(result.is_err(), "cyclic graph should return an error");
    assert!(result.unwrap_err().is_indexing());
}

// ---------------------------------------------------------------------------
// Degree calculations
// ---------------------------------------------------------------------------

#[test]
fn degree_calculations() {
    // Build a fan-in graph: B and C both import A.
    // B -> A, C -> A
    let mut graph = FileDependencyGraph::new();
    let a = PathBuf::from("/a.rs");
    let b = PathBuf::from("/b.rs");
    let c = PathBuf::from("/c.rs");

    graph.add_dependency(FileDependency::new(&b, &a, "a"));
    graph.add_dependency(FileDependency::new(&c, &a, "a"));

    // A: in_degree = 2 (imported by B and C), out_degree = 0
    assert_eq!(graph.in_degree(Path::new("/a.rs")), 2);
    assert_eq!(graph.out_degree(Path::new("/a.rs")), 0);

    // B: in_degree = 0, out_degree = 1 (imports A)
    assert_eq!(graph.in_degree(Path::new("/b.rs")), 0);
    assert_eq!(graph.out_degree(Path::new("/b.rs")), 1);

    // C: in_degree = 0, out_degree = 1 (imports A)
    assert_eq!(graph.in_degree(Path::new("/c.rs")), 0);
    assert_eq!(graph.out_degree(Path::new("/c.rs")), 1);

    // Total edges: 2
    assert_eq!(graph.edge_count(), 2);
    // Total files: 3
    assert_eq!(graph.file_count(), 3);
}

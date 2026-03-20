//! End-to-end pipeline tests for the indexing system.
//!
//! Each test creates a temporary directory with real source files, builds a
//! `CodebaseIndex`, and verifies symbol extraction, incremental updates, and
//! repo map generation.

use mahalaxmi_core::config::IndexingConfig;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_indexing::index::CodebaseIndex;
use mahalaxmi_indexing::repomap::{GroupBy, RepoMapConfig};
use mahalaxmi_indexing::types::SymbolKind;
use std::path::PathBuf;

fn test_i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

/// Helper to create a file inside a temp directory, creating parent dirs as
/// needed. Returns the absolute path to the created file.
fn write_file(root: &std::path::Path, relative: &str, content: &str) -> PathBuf {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent directories");
    }
    std::fs::write(&path, content).expect("write file");
    path
}

// ---------------------------------------------------------------------------
// Full index build: Rust project
// ---------------------------------------------------------------------------

#[test]
fn index_rust_project() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path();

    write_file(root, "src/main.rs", "fn main() {}\nfn helper() {}");
    write_file(root, "src/lib.rs", "pub struct App {}\npub fn run() {}");

    let i18n = test_i18n();
    let config = IndexingConfig::default();
    let index = CodebaseIndex::build(root, &config, &i18n).expect("build should succeed");

    // Should have indexed 2 files
    assert_eq!(index.file_count(), 2, "should index 2 Rust files");

    // Verify symbols extracted
    assert!(
        index.symbol_count() >= 4,
        "should extract at least 4 symbols (main, helper, App, run); got {}",
        index.symbol_count()
    );

    let main_syms = index.find_symbol_exact("main");
    assert!(!main_syms.is_empty(), "should find symbol 'main'");

    let helper_syms = index.find_symbol_exact("helper");
    assert!(!helper_syms.is_empty(), "should find symbol 'helper'");

    let app_syms = index.find_symbol_exact("App");
    assert!(!app_syms.is_empty(), "should find symbol 'App'");
    assert_eq!(app_syms[0].kind, SymbolKind::Struct);

    let run_syms = index.find_symbol_exact("run");
    assert!(!run_syms.is_empty(), "should find symbol 'run'");
    assert_eq!(run_syms[0].kind, SymbolKind::Function);
}

// ---------------------------------------------------------------------------
// Mixed-language project
// ---------------------------------------------------------------------------

#[test]
fn index_mixed_language_project() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path();

    write_file(root, "src/app.rs", "pub fn start() {}");
    write_file(
        root,
        "src/utils.ts",
        "export function format(s: string): string { return s; }",
    );

    let i18n = test_i18n();
    let config = IndexingConfig::default();
    let index = CodebaseIndex::build(root, &config, &i18n).expect("build should succeed");

    assert_eq!(
        index.file_count(),
        2,
        "should index both Rust and TypeScript files"
    );

    let start_syms = index.find_symbol_exact("start");
    assert!(!start_syms.is_empty(), "should find Rust symbol 'start'");

    let format_syms = index.find_symbol_exact("format");
    assert!(
        !format_syms.is_empty(),
        "should find TypeScript symbol 'format'"
    );
}

// ---------------------------------------------------------------------------
// Excluded directories
// ---------------------------------------------------------------------------

#[test]
fn index_respects_excluded_dirs() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path();

    write_file(root, "src/main.rs", "fn main() {}");
    write_file(root, "node_modules/pkg/index.js", "function internal() {}");

    let i18n = test_i18n();
    let config = IndexingConfig::default(); // node_modules is excluded by default
    let index = CodebaseIndex::build(root, &config, &i18n).expect("build should succeed");

    // Only the src file should be indexed
    assert_eq!(
        index.file_count(),
        1,
        "should only index src files, not node_modules"
    );

    let main_syms = index.find_symbol_exact("main");
    assert!(!main_syms.is_empty(), "should find symbol 'main' from src/");

    let internal_syms = index.find_symbol_exact("internal");
    assert!(
        internal_syms.is_empty(),
        "should NOT find symbol 'internal' from node_modules/"
    );
}

// ---------------------------------------------------------------------------
// Incremental update: file added
// ---------------------------------------------------------------------------

#[test]
fn index_incremental_update() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path();

    write_file(root, "src/main.rs", "fn main() {}");

    let i18n = test_i18n();
    let config = IndexingConfig::default();
    let mut index =
        CodebaseIndex::build(root, &config, &i18n).expect("initial build should succeed");

    assert_eq!(index.file_count(), 1);

    // Add a new file
    write_file(root, "src/lib.rs", "pub fn new_function() {}");

    let stats = index.update(root, &i18n).expect("update should succeed");

    assert_eq!(stats.files_added, 1, "should detect 1 added file");
    assert_eq!(index.file_count(), 2, "should now have 2 indexed files");

    let new_fn = index.find_symbol_exact("new_function");
    assert!(
        !new_fn.is_empty(),
        "should find the newly added symbol 'new_function'"
    );
}

// ---------------------------------------------------------------------------
// Incremental update: file modified
// ---------------------------------------------------------------------------

#[test]
fn index_update_detects_modifications() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path();

    let file_path = write_file(root, "src/main.rs", "fn original() {}");

    let i18n = test_i18n();
    let config = IndexingConfig::default();
    let mut index =
        CodebaseIndex::build(root, &config, &i18n).expect("initial build should succeed");

    let orig_syms = index.find_symbol_exact("original");
    assert!(
        !orig_syms.is_empty(),
        "should find 'original' before update"
    );

    // Modify the file with different content
    std::fs::write(&file_path, "fn modified() {}\nfn extra() {}").expect("overwrite file");

    let stats = index.update(root, &i18n).expect("update should succeed");

    assert_eq!(stats.files_modified, 1, "should detect 1 modified file");

    let mod_syms = index.find_symbol_exact("modified");
    assert!(
        !mod_syms.is_empty(),
        "should find the new symbol 'modified' after update"
    );

    let extra_syms = index.find_symbol_exact("extra");
    assert!(
        !extra_syms.is_empty(),
        "should find the new symbol 'extra' after update"
    );
}

// ---------------------------------------------------------------------------
// Incremental update: file removed
// ---------------------------------------------------------------------------

#[test]
fn index_update_detects_removals() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path();

    write_file(root, "src/main.rs", "fn main() {}");
    let lib_path = write_file(root, "src/lib.rs", "pub fn helper() {}");

    let i18n = test_i18n();
    let config = IndexingConfig::default();
    let mut index =
        CodebaseIndex::build(root, &config, &i18n).expect("initial build should succeed");

    assert_eq!(index.file_count(), 2);

    // Remove a file
    std::fs::remove_file(&lib_path).expect("delete lib.rs");

    let stats = index.update(root, &i18n).expect("update should succeed");

    assert_eq!(stats.files_removed, 1, "should detect 1 removed file");
    assert_eq!(index.file_count(), 1, "should now have 1 indexed file");

    let helper_syms = index.find_symbol_exact("helper");
    assert!(
        helper_syms.is_empty(),
        "should no longer find symbol 'helper' after removal"
    );
}

// ---------------------------------------------------------------------------
// Repo map generation
// ---------------------------------------------------------------------------

#[test]
fn repo_map_generation() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path();

    write_file(root, "src/main.rs", "fn main() {}\npub fn entry_point() {}");
    write_file(
        root,
        "src/models.rs",
        "pub struct User {}\npub struct Config {}",
    );

    let i18n = test_i18n();
    let config = IndexingConfig::default();
    let index = CodebaseIndex::build(root, &config, &i18n).expect("build should succeed");

    let map_config = RepoMapConfig {
        max_tokens: 2048,
        include_signatures: false,
        include_doc_comments: false,
        group_by: GroupBy::File,
    };

    let repo_map = index.repo_map(&map_config);

    assert!(
        !repo_map.is_empty(),
        "repo map should not be empty for a project with symbols"
    );

    // Verify key symbols appear in the output
    assert!(
        repo_map.contains("main") || repo_map.contains("entry_point"),
        "repo map should contain at least one of the function names"
    );
    assert!(
        repo_map.contains("User") || repo_map.contains("Config"),
        "repo map should contain at least one of the struct names"
    );

    // Verify the output stays within the token budget
    let estimated_tokens = repo_map.len() / 4;
    assert!(
        estimated_tokens <= map_config.max_tokens,
        "repo map ({estimated_tokens} estimated tokens) should stay within budget ({})",
        map_config.max_tokens
    );
}

// ---------------------------------------------------------------------------
// Repo map: group by kind
// ---------------------------------------------------------------------------

#[test]
fn repo_map_group_by_kind() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let root = tmp.path();

    write_file(
        root,
        "src/lib.rs",
        r#"
pub fn compute() {}
pub struct Engine {}
pub trait Runnable {
    fn run(&self);
}
pub enum Status { Active, Inactive }
"#,
    );

    let i18n = test_i18n();
    let config = IndexingConfig::default();
    let index = CodebaseIndex::build(root, &config, &i18n).expect("build should succeed");

    let map_config = RepoMapConfig {
        max_tokens: 4096,
        include_signatures: false,
        include_doc_comments: false,
        group_by: GroupBy::Kind,
    };

    let repo_map = index.repo_map(&map_config);

    assert!(
        !repo_map.is_empty(),
        "repo map (by kind) should not be empty"
    );

    // When grouped by kind, the output should have section headers for the kinds
    // present in the source. At minimum we expect "Functions:" and "Structs:".
    let has_kind_section = repo_map.contains("Functions:")
        || repo_map.contains("Structs:")
        || repo_map.contains("Traits:")
        || repo_map.contains("Enums:");

    assert!(
        has_kind_section,
        "repo map grouped by kind should contain kind section headers; got:\n{repo_map}"
    );

    // Symbols should still be present
    assert!(
        repo_map.contains("compute"),
        "repo map should contain function 'compute'"
    );
    assert!(
        repo_map.contains("Engine"),
        "repo map should contain struct 'Engine'"
    );
}

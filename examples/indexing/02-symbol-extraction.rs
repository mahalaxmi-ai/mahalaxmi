// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Symbol extraction and dependency graph
//
// Demonstrates how to use ExtractorFactory to extract symbols from a
// source file, and FileDependencyGraph to inspect import relationships.
// These lower-level APIs are used by CodebaseIndex internally and are
// available for custom context routing logic.

use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
use mahalaxmi_indexing::{
    ExtractorFactory, FileDependency, FileDependencyGraph, LanguageRegistry, SymbolKind,
    SupportedLanguage, Visibility,
};
use std::path::{Path, PathBuf};

fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);

    // Detect the language from a file extension.
    // SupportedLanguage::from_extension requires the leading dot.
    let path = Path::new("src/main.rs");
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();

    if let Some(lang) = SupportedLanguage::from_extension(&extension) {
        println!("Detected language: {:?}", lang);

        // Build the registry with default Tree-sitter grammars, then create
        // an extractor for the detected language.
        let registry = LanguageRegistry::with_defaults();
        match ExtractorFactory::create(lang, &registry, &i18n) {
            Ok(extractor) => {
                // Extract symbols from inline source code.
                let source = r#"
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self { host: host.into(), port }
    }
}
"#;
                match extractor.extract_symbols(source, path, &i18n) {
                    Ok(symbols) => {
                        println!("\nExtracted {} symbols:", symbols.len());
                        for sym in &symbols {
                            println!(
                                "  {:?} `{}` ({:?}) at line {}",
                                sym.kind, sym.name, sym.visibility, sym.line_start
                            );
                        }

                        // Filter to only public symbols.
                        let public: Vec<_> = symbols
                            .iter()
                            .filter(|s| s.visibility == Visibility::Public)
                            .collect();
                        println!("\nPublic symbols: {}", public.len());

                        // Filter to only structs.
                        let structs: Vec<_> = symbols
                            .iter()
                            .filter(|s| s.kind == SymbolKind::Struct)
                            .collect();
                        println!("Structs: {}", structs.len());
                    }
                    Err(e) => eprintln!("Extraction failed: {e}"),
                }
            }
            Err(e) => eprintln!("Could not create extractor: {e}"),
        }
    } else {
        println!("Language not recognized for: {}", path.display());
    }

    // FileDependencyGraph — tracks import relationships between files.
    // In a full index, this is populated automatically during indexing.
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("src/main.rs", "src/config.rs", "crate::config"));
    graph.add_dependency(FileDependency::new("src/main.rs", "src/handler.rs", "crate::handler"));
    graph.add_dependency(FileDependency::new("src/handler.rs", "src/config.rs", "crate::config"));

    // BFS distance: how many import hops from any source to the target?
    // Signature: bfs_distance(target, sources, max_depth)
    let sources = vec![PathBuf::from("src/main.rs")];
    let dist = graph.bfs_distance(
        Path::new("src/config.rs"),
        &sources,
        4,
    );
    println!("\nBFS distance main.rs → config.rs: {:?}", dist);

    // Which files depend on config.rs?  (Used for impact analysis.)
    let dependents = graph.dependents_of(Path::new("src/config.rs"));
    println!("Files importing config.rs: {}", dependents.len());
    for f in dependents {
        println!("  {}", f.display());
    }
}

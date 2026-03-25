// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Building a codebase index
//
// Demonstrates how to build a full CodebaseIndex from a directory,
// inspect update statistics, and retrieve the generated repo map.
// The repo map is the token-budgeted summary injected into AI agent
// context at the start of each orchestration cycle.

use mahalaxmi_core::{
    config::{IndexingConfig, MahalaxmiConfig},
    i18n::{locale::SupportedLocale, I18nService},
};
use mahalaxmi_indexing::{CodebaseIndex, GroupBy, RepoMapConfig};
use std::path::PathBuf;

fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);

    // Point at a real directory. We use the current working directory here.
    // In production this is the project_root from CycleConfig.
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    println!("Indexing: {}", project_root.display());

    // IndexingConfig controls which files are indexed and resource limits.
    let config = MahalaxmiConfig {
        indexing: IndexingConfig {
            // Skip files larger than 500 KB.
            max_file_size_bytes: 500 * 1024,
            // Exclude common non-source directories.
            excluded_dirs: vec![
                "target".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    // Build a full index. For large codebases, prefer incremental update.
    match CodebaseIndex::build(&project_root, &config.indexing, &i18n) {
        Ok(index) => {
            println!("\nIndex built:");
            println!("  files:   {}", index.file_count());
            println!("  symbols: {}", index.symbol_count());

            // Generate a repo map with a 2 000-token budget.
            let map_config = RepoMapConfig {
                max_tokens: 2_000,
                group_by: GroupBy::File,
                ..Default::default()
            };
            let repo_map = index.repo_map(&map_config);
            let preview: String = repo_map.chars().take(500).collect();
            println!("\nRepo map preview ({} chars total):", repo_map.len());
            println!("{}", preview);
            if repo_map.len() > 500 {
                println!("... (truncated)");
            }
        }
        Err(e) => {
            // Expected in environments without Tree-sitter grammar libraries.
            eprintln!("Index build failed (may need Tree-sitter grammars): {e}");
        }
    }
}

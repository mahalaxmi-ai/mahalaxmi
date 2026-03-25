//! Codebase index with full build and incremental update support.
//!
//! The `CodebaseIndex` ties together symbol extraction, dependency graphs,
//! ranking, and repo map generation into a single indexed representation
//! of a codebase. It supports incremental updates via content hash
//! fingerprinting, only re-extracting files that have changed.

use crate::extractors::ExtractorFactory;
use crate::graph::FileDependencyGraph;
use crate::languages::LanguageRegistry;
use crate::ranking::{RankingConfig, SymbolRanking};
use crate::repomap::{RepoMap, RepoMapConfig};
use crate::types::{ExtractedSymbol, FileFingerprint, SupportedLanguage, SymbolKind};
use mahalaxmi_core::config::IndexingConfig;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Maximum number of symbols the index will retain.
///
/// For very large codebases, the symbols vector can grow unboundedly.
/// This cap keeps memory usage predictable. When exceeded, the lowest-ranked
/// symbols are evicted.
const MAX_SYMBOLS: usize = 500_000;

/// Statistics from an incremental index update.
///
/// Tracks how many files were added, modified, removed, and unchanged
/// during the update, along with the total number of symbols extracted
/// and the time taken.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpdateStats {
    /// Number of newly discovered files added to the index.
    pub files_added: usize,
    /// Number of files whose content changed and were re-extracted.
    pub files_modified: usize,
    /// Number of files that no longer exist and were removed.
    pub files_removed: usize,
    /// Number of files whose content hash matched (no re-extraction needed).
    pub files_unchanged: usize,
    /// Total number of symbols extracted from added and modified files.
    pub symbols_extracted: usize,
    /// Time taken for the update in milliseconds.
    pub duration_ms: u64,
}

impl UpdateStats {
    /// Returns the total number of files that were processed (added + modified + removed).
    pub fn total_processed(&self) -> usize {
        self.files_added + self.files_modified + self.files_removed
    }

    /// Returns true if any files were added, modified, or removed.
    pub fn has_changes(&self) -> bool {
        self.files_added > 0 || self.files_modified > 0 || self.files_removed > 0
    }
}

/// A complete index of a codebase including symbols, dependency graph, and rankings.
///
/// The index is built by walking the directory tree, extracting symbols and imports
/// from each supported source file, constructing a dependency graph, and computing
/// importance rankings. It supports incremental updates that only re-extract files
/// whose content has changed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseIndex {
    /// All extracted symbols from the indexed codebase.
    symbols: Vec<ExtractedSymbol>,
    /// Content hash fingerprints for change detection.
    fingerprints: HashMap<PathBuf, FileFingerprint>,
    /// File-level dependency graph built from import analysis.
    graph: FileDependencyGraph,
    /// Importance rankings for symbols and files.
    ranking: SymbolRanking,
    /// The configuration used to build this index.
    config: IndexingConfig,
}

impl CodebaseIndex {
    /// Builds a complete codebase index from the given root directory.
    ///
    /// Algorithm:
    /// 1. Walks the directory tree, respecting excluded dirs and extensions.
    /// 2. Skips files exceeding `max_file_size_bytes`.
    /// 3. Detects language from file extension, skips unsupported languages.
    /// 4. If `enabled_languages` is set, skips languages not in the list.
    /// 5. Computes a `FileFingerprint` for each file.
    /// 6. Creates extractors for each language via `ExtractorFactory`.
    /// 7. Extracts symbols and imports from each file.
    /// 8. Builds a `FileDependencyGraph` from import data.
    /// 9. Computes `SymbolRanking` from symbols and graph.
    /// 10. Returns the assembled index.
    pub fn build(
        root: &Path,
        config: &IndexingConfig,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Self> {
        let start = Instant::now();

        let files = walk_directory(root, config);
        let registry = LanguageRegistry::with_defaults();
        let mut extractors = ExtractorFactory::create_all(&registry, i18n)?;

        let enabled_languages: Option<Vec<String>> = config.enabled_languages.clone();

        let mut all_symbols: Vec<ExtractedSymbol> = Vec::new();
        let mut fingerprints: HashMap<PathBuf, FileFingerprint> = HashMap::new();
        let mut import_data: Vec<(PathBuf, Vec<String>)> = Vec::new();

        for file_path in &files {
            // Detect language from extension (prepend dot for from_extension)
            let extension = file_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            let language = match SupportedLanguage::from_extension(&extension) {
                Some(lang) => lang,
                None => continue,
            };

            // Check if language is in the enabled list
            if let Some(ref enabled) = enabled_languages {
                if !enabled.iter().any(|l| l == language.as_str()) {
                    continue;
                }
            }

            // Compute fingerprint
            let fingerprint = FileFingerprint::compute(file_path, i18n)?;
            fingerprints.insert(file_path.clone(), fingerprint);

            // Read file content
            let content = std::fs::read_to_string(file_path).map_err(|e| {
                MahalaxmiError::indexing(
                    i18n,
                    keys::indexing::FILE_READ_FAILED,
                    &[
                        ("file", &file_path.display().to_string()),
                        ("reason", &e.to_string()),
                    ],
                )
            })?;

            // Get or skip if no extractor for this language
            let extractor = match extractors.get_mut(&language) {
                Some(ext) => ext,
                None => continue,
            };

            // Extract symbols
            let symbols = extractor.extract_symbols(&content, file_path, i18n)?;
            let imports = extractor.extract_imports(&content, file_path, i18n)?;

            import_data.push((file_path.clone(), imports));
            all_symbols.extend(symbols);
        }

        // Build dependency graph from imports
        // Group by language for resolution; for simplicity, use the first file's language
        // or build per-language. Since files can be mixed languages, we build per-language groups.
        let mut graph = FileDependencyGraph::new();
        let mut language_imports: HashMap<SupportedLanguage, Vec<(PathBuf, Vec<String>)>> =
            HashMap::new();

        for (file_path, imports) in &import_data {
            graph.add_file(file_path.clone());
            let extension = file_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            if let Some(lang) = SupportedLanguage::from_extension(&extension) {
                language_imports
                    .entry(lang)
                    .or_default()
                    .push((file_path.clone(), imports.clone()));
            }
        }

        for (lang, lang_import_data) in &language_imports {
            let lang_graph =
                FileDependencyGraph::build_from_extractions(lang_import_data, root, *lang);
            // Merge edges into the main graph
            for file in lang_graph.files() {
                graph.add_file(file.clone());
            }
            // Re-add edges by looking at dependencies
            for file in lang_graph.files() {
                for dep in lang_graph.dependencies_of(file) {
                    let edge =
                        crate::graph::FileDependency::new(file.clone(), dep.clone(), String::new());
                    graph.add_dependency(edge);
                }
            }
        }

        // Compute ranking
        let ranking_config = RankingConfig::from_indexing_config(config);
        let ranking = SymbolRanking::compute(&all_symbols, &graph, &ranking_config);

        let duration = start.elapsed();
        tracing::info!(
            files = fingerprints.len(),
            symbols = all_symbols.len(),
            duration_ms = duration.as_millis() as u64,
            "Codebase index built"
        );

        // Enforce symbol cap: keep the highest-ranked symbols
        if all_symbols.len() > MAX_SYMBOLS {
            tracing::warn!(
                total = all_symbols.len(),
                cap = MAX_SYMBOLS,
                "Symbol count exceeds cap — truncating lowest-ranked symbols"
            );
            // Sort by rank descending (highest rank first), then truncate
            let ranked_names: std::collections::HashSet<String> = ranking
                .top_n(MAX_SYMBOLS)
                .into_iter()
                .map(|(name, _)| name.to_string())
                .collect();
            all_symbols.retain(|s| ranked_names.contains(&s.qualified_name()));
        }

        Ok(Self {
            symbols: all_symbols,
            fingerprints,
            graph,
            ranking,
            config: config.clone(),
        })
    }

    /// Incrementally updates the index by detecting changed files.
    ///
    /// Algorithm:
    /// 1. Walks the directory tree with the same filters as `build`.
    /// 2. Compares fingerprints: new files are added, changed hashes trigger
    ///    re-extraction, and missing files are removed.
    /// 3. Only re-extracts changed and new files.
    /// 4. Rebuilds the dependency graph and re-ranks all symbols.
    /// 5. Returns statistics about what changed.
    pub fn update(&mut self, root: &Path, i18n: &I18nService) -> MahalaxmiResult<UpdateStats> {
        let start = Instant::now();
        let mut stats = UpdateStats::default();

        let current_files = walk_directory(root, &self.config);
        let current_set: std::collections::HashSet<PathBuf> =
            current_files.iter().cloned().collect();

        // Detect removed files
        let old_files: Vec<PathBuf> = self.fingerprints.keys().cloned().collect();
        for old_file in &old_files {
            if !current_set.contains(old_file) {
                self.fingerprints.remove(old_file);
                self.symbols.retain(|s| s.file_path != *old_file);
                stats.files_removed += 1;
            }
        }

        let registry = LanguageRegistry::with_defaults();
        let mut extractors = ExtractorFactory::create_all(&registry, i18n)?;
        let enabled_languages: Option<Vec<String>> = self.config.enabled_languages.clone();

        let mut import_data: Vec<(PathBuf, Vec<String>)> = Vec::new();

        for file_path in &current_files {
            // Detect language (prepend dot for from_extension)
            let extension = file_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            let language = match SupportedLanguage::from_extension(&extension) {
                Some(lang) => lang,
                None => continue,
            };

            // Check enabled languages
            if let Some(ref enabled) = enabled_languages {
                if !enabled.iter().any(|l| l == language.as_str()) {
                    continue;
                }
            }

            // Compute new fingerprint
            let new_fingerprint = FileFingerprint::compute(file_path, i18n)?;

            // Check if file is new or modified
            let needs_extraction = if let Some(old_fingerprint) = self.fingerprints.get(file_path) {
                if old_fingerprint.matches_hash(&new_fingerprint.content_hash) {
                    stats.files_unchanged += 1;
                    // Still collect imports for graph rebuild
                    false
                } else {
                    stats.files_modified += 1;
                    true
                }
            } else {
                stats.files_added += 1;
                true
            };

            self.fingerprints.insert(file_path.clone(), new_fingerprint);

            if needs_extraction {
                // Remove old symbols for this file
                self.symbols.retain(|s| s.file_path != *file_path);

                // Read and extract
                let content = std::fs::read_to_string(file_path).map_err(|e| {
                    MahalaxmiError::indexing(
                        i18n,
                        keys::indexing::FILE_READ_FAILED,
                        &[
                            ("file", &file_path.display().to_string()),
                            ("reason", &e.to_string()),
                        ],
                    )
                })?;

                let extractor = match extractors.get_mut(&language) {
                    Some(ext) => ext,
                    None => continue,
                };

                let symbols = extractor.extract_symbols(&content, file_path, i18n)?;
                let imports = extractor.extract_imports(&content, file_path, i18n)?;

                stats.symbols_extracted += symbols.len();
                import_data.push((file_path.clone(), imports));
                self.symbols.extend(symbols);
            } else {
                // Unchanged file: still need imports for graph rebuild
                let content = std::fs::read_to_string(file_path).map_err(|e| {
                    MahalaxmiError::indexing(
                        i18n,
                        keys::indexing::FILE_READ_FAILED,
                        &[
                            ("file", &file_path.display().to_string()),
                            ("reason", &e.to_string()),
                        ],
                    )
                })?;

                let extractor = match extractors.get_mut(&language) {
                    Some(ext) => ext,
                    None => continue,
                };

                let imports = extractor.extract_imports(&content, file_path, i18n)?;
                import_data.push((file_path.clone(), imports));
            }
        }

        // Rebuild graph
        let mut graph = FileDependencyGraph::new();
        let mut language_imports: HashMap<SupportedLanguage, Vec<(PathBuf, Vec<String>)>> =
            HashMap::new();

        for (file_path, imports) in &import_data {
            graph.add_file(file_path.clone());
            let extension = file_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            if let Some(lang) = SupportedLanguage::from_extension(&extension) {
                language_imports
                    .entry(lang)
                    .or_default()
                    .push((file_path.clone(), imports.clone()));
            }
        }

        for (lang, lang_import_data) in &language_imports {
            let lang_graph =
                FileDependencyGraph::build_from_extractions(lang_import_data, root, *lang);
            for file in lang_graph.files() {
                graph.add_file(file.clone());
            }
            for file in lang_graph.files() {
                for dep in lang_graph.dependencies_of(file) {
                    let edge =
                        crate::graph::FileDependency::new(file.clone(), dep.clone(), String::new());
                    graph.add_dependency(edge);
                }
            }
        }

        self.graph = graph;

        // Re-rank
        let ranking_config = RankingConfig::from_indexing_config(&self.config);
        self.ranking = SymbolRanking::compute(&self.symbols, &self.graph, &ranking_config);

        // Enforce symbol cap after update
        if self.symbols.len() > MAX_SYMBOLS {
            tracing::warn!(
                total = self.symbols.len(),
                cap = MAX_SYMBOLS,
                "Symbol count exceeds cap after update — truncating"
            );
            let ranked_names: std::collections::HashSet<String> = self
                .ranking
                .top_n(MAX_SYMBOLS)
                .into_iter()
                .map(|(name, _)| name.to_string())
                .collect();
            self.symbols
                .retain(|s| ranked_names.contains(&s.qualified_name()));
        }

        let duration = start.elapsed();
        stats.duration_ms = duration.as_millis() as u64;

        tracing::info!(
            added = stats.files_added,
            modified = stats.files_modified,
            removed = stats.files_removed,
            unchanged = stats.files_unchanged,
            duration_ms = stats.duration_ms,
            "Incremental update completed"
        );

        Ok(stats)
    }

    /// Returns all symbols defined in the given file.
    pub fn symbols_in_file(&self, path: &Path) -> Vec<&ExtractedSymbol> {
        self.symbols
            .iter()
            .filter(|s| s.file_path == path)
            .collect()
    }

    /// Searches for symbols whose qualified name contains the given substring.
    pub fn find_symbol(&self, name: &str) -> Vec<&ExtractedSymbol> {
        self.symbols
            .iter()
            .filter(|s| s.qualified_name().contains(name))
            .collect()
    }

    /// Searches for symbols with an exact name match.
    pub fn find_symbol_exact(&self, name: &str) -> Vec<&ExtractedSymbol> {
        self.symbols.iter().filter(|s| s.name == name).collect()
    }

    /// Returns all symbols of the given kind.
    pub fn symbols_of_kind(&self, kind: SymbolKind) -> Vec<&ExtractedSymbol> {
        self.symbols.iter().filter(|s| s.kind == kind).collect()
    }

    /// Generates a repo map from the current index with the given configuration.
    pub fn repo_map(&self, config: &RepoMapConfig) -> String {
        RepoMap::generate(&self.symbols, &self.ranking, config)
    }

    /// Returns the number of indexed files.
    pub fn file_count(&self) -> usize {
        self.fingerprints.len()
    }

    /// Returns the total number of extracted symbols.
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Returns a reference to the file dependency graph.
    pub fn graph(&self) -> &FileDependencyGraph {
        &self.graph
    }

    /// Returns a reference to the symbol rankings.
    pub fn ranking(&self) -> &SymbolRanking {
        &self.ranking
    }

    /// Returns the fingerprint for a specific file, if indexed.
    pub fn fingerprint(&self, path: &Path) -> Option<&FileFingerprint> {
        self.fingerprints.get(path)
    }
}

/// Recursively walks a directory tree, returning paths to files that pass
/// the indexing configuration filters.
///
/// Skips:
/// - Directories listed in `config.excluded_dirs`
/// - Files with extensions listed in `config.excluded_extensions`
/// - Files larger than `config.max_file_size_bytes`
fn walk_directory(root: &Path, config: &IndexingConfig) -> Vec<PathBuf> {
    let mut result = Vec::new();
    walk_directory_recursive(root, config, &mut result);
    result
}

/// Recursive helper for directory walking.
fn walk_directory_recursive(dir: &Path, config: &IndexingConfig, result: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        if path.is_dir() {
            // Check if this directory should be excluded
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if config.excluded_dirs.iter().any(|d| d == dir_name) {
                continue;
            }
            walk_directory_recursive(&path, config, result);
        } else if path.is_file() {
            // Check file size
            let metadata = match std::fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if metadata.len() > config.max_file_size_bytes {
                continue;
            }

            // Check excluded extensions
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let excluded = config
                .excluded_extensions
                .iter()
                .any(|ext| file_name.ends_with(ext.as_str()));
            if excluded {
                continue;
            }

            result.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_stats_default() {
        let stats = UpdateStats::default();
        assert_eq!(stats.files_added, 0);
        assert_eq!(stats.files_modified, 0);
        assert_eq!(stats.files_removed, 0);
        assert_eq!(stats.files_unchanged, 0);
        assert_eq!(stats.symbols_extracted, 0);
        assert_eq!(stats.duration_ms, 0);
    }

    #[test]
    fn update_stats_total_processed() {
        let stats = UpdateStats {
            files_added: 3,
            files_modified: 2,
            files_removed: 1,
            files_unchanged: 10,
            symbols_extracted: 50,
            duration_ms: 100,
        };
        assert_eq!(stats.total_processed(), 6);
    }

    #[test]
    fn update_stats_has_changes() {
        let no_changes = UpdateStats::default();
        assert!(!no_changes.has_changes());

        let with_added = UpdateStats {
            files_added: 1,
            ..Default::default()
        };
        assert!(with_added.has_changes());

        let with_modified = UpdateStats {
            files_modified: 1,
            ..Default::default()
        };
        assert!(with_modified.has_changes());

        let with_removed = UpdateStats {
            files_removed: 1,
            ..Default::default()
        };
        assert!(with_removed.has_changes());
    }

    #[test]
    fn walk_directory_respects_exclusions() {
        let temp_dir = std::env::temp_dir().join("mahalaxmi_walk_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("src")).unwrap();
        std::fs::create_dir_all(temp_dir.join("node_modules")).unwrap();
        std::fs::write(temp_dir.join("src").join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(
            temp_dir.join("node_modules").join("pkg.js"),
            "module.exports = {}",
        )
        .unwrap();
        std::fs::write(temp_dir.join("src").join("bundle.min.js"), "minified").unwrap();

        let config = IndexingConfig::default();
        let files = walk_directory(&temp_dir, &config);

        // Should include main.rs but not node_modules/pkg.js or bundle.min.js
        assert!(files.iter().any(|f| f.ends_with("main.rs")));
        assert!(!files
            .iter()
            .any(|f| f.to_string_lossy().contains("node_modules")));
        assert!(!files.iter().any(|f| f.ends_with("bundle.min.js")));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn walk_directory_respects_max_file_size() {
        let temp_dir = std::env::temp_dir().join("mahalaxmi_size_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Small file - should be included
        std::fs::write(temp_dir.join("small.rs"), "fn small() {}").unwrap();

        // Large file - should be excluded
        let large_content = "x".repeat(2_000_000);
        std::fs::write(temp_dir.join("large.rs"), large_content).unwrap();

        let config = IndexingConfig {
            max_file_size_bytes: 1_048_576,
            ..Default::default()
        };
        let files = walk_directory(&temp_dir, &config);

        assert!(files.iter().any(|f| f.ends_with("small.rs")));
        assert!(!files.iter().any(|f| f.ends_with("large.rs")));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

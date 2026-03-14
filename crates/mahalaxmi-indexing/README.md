# mahalaxmi-indexing

Tree-sitter AST symbol extraction, file dependency graph, PageRank-style importance ranking, and token-budgeted repo map generation for Mahalaxmi.

## Overview

`mahalaxmi-indexing` gives AI coding agents a structured map of a codebase. AI agents operate under strict token limits; sending an entire repository is impractical. The repo map solves this by extracting the most important symbols from every file and generating a compact, ranked summary that fits within a configurable token budget — giving agents the context they need without overwhelming their context window.

The crate uses Tree-sitter grammars to parse source files and extract symbols (functions, structs, classes, methods, interfaces, enums, traits) along with their visibility, kind, and file location. Seven languages are supported out of the box: Rust, TypeScript, Python, Go, Java, C, and C++. The `FileDependencyGraph` infers import relationships between files from `use`/`import`/`include` statements, enabling graph-proximity scoring (files that import a target file are more relevant context for it). `SymbolRanking` applies a PageRank-style algorithm over the dependency graph to score symbol importance. `CodebaseIndex` ties everything together with incremental update support — only files whose content hash has changed are re-extracted.

Orchestration engine developers use `CodebaseIndex` when building context routers and repo map injectors. End users configure indexing through `~/.mahalaxmi/config.toml` and see the results in the AI context preparation step of each cycle.

## Key Types

| Type | Kind | Description |
|------|------|-------------|
| `CodebaseIndex` | Struct | Full indexed representation of a codebase; supports incremental updates |
| `UpdateStats` | Struct | Files added/modified/removed/unchanged and symbols extracted per update |
| `ExtractedSymbol` | Struct | A single symbol: name, kind, file path, line range, visibility, confidence |
| `SymbolKind` | Enum | `Function`, `Struct`, `Class`, `Method`, `Interface`, `Enum`, `Trait`, `Constant`, `Variable` |
| `SupportedLanguage` | Enum | `Rust`, `TypeScript`, `Python`, `Go`, `Java`, `C`, `Cpp` |
| `Visibility` | Enum | `Public`, `Private`, `Internal` |
| `FileFingerprint` | Struct | Content hash + metadata for incremental update detection |
| `FileDependencyGraph` | Struct | Directed graph of file import relationships |
| `FileDependency` | Struct | A single import relationship between two files |
| `SymbolRanking` | Struct | PageRank scores for symbols based on the dependency graph |
| `RepoMap` | Struct | Token-budgeted, ranked textual summary of a codebase |
| `RepoMapConfig` | Struct | Budget (token limit), grouping, and ranking configuration |
| `LanguageRegistry` | Struct | Maps file extensions to Tree-sitter grammars |
| `ExtractorFactory` | Struct | Creates language-specific extractors by file extension |

## Key Functions / Methods

| Function | Description |
|----------|-------------|
| `CodebaseIndex::build(root, config, i18n)` | Full index build from a directory tree |
| `CodebaseIndex::update(root, config, i18n)` | Incremental update; re-extracts only changed files |
| `CodebaseIndex::repo_map(config)` | Generate a token-budgeted repo map from the current index |
| `CodebaseIndex::symbols_for_file(path)` | Return all symbols extracted from a specific file |
| `FileDependencyGraph::bfs_distance(from, to)` | Shortest import-graph path between two files |
| `FileDependencyGraph::files_importing(path)` | All files that directly import the given file |
| `SymbolRanking::rank(symbols, graph, config)` | Score symbols by graph centrality |
| `RepoMap::generate(symbols, ranking, config)` | Build a ranked, token-budgeted map string |
| `LanguageRegistry::detect_language(path)` | Detect language from file extension |
| `ExtractorFactory::create(language)` | Create the appropriate language extractor |

## Feature Flags

No feature flags.

## Dependencies

| Dependency | Why |
|-----------|-----|
| `tree-sitter` | Core AST parsing runtime |
| `tree-sitter-rust` / `-typescript` / `-python` / `-go` / `-java` / `-c` / `-cpp` | Language grammars |
| `petgraph` | Directed graph for the file dependency graph and PageRank |
| `serde` | Index serialization for caching |
| `mahalaxmi-core` | Shared types, config (`IndexingConfig`), errors, i18n |

## Stability

**Unstable** — API may change in minor versions during the pre-1.0 period.

## License

MIT — Copyright 2026 ThriveTech Services LLC

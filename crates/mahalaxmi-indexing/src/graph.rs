//! File-level dependency graph for codebase indexing.
//!
//! Builds a directed graph of file dependencies from extracted import statements.
//! Supports topological sorting, cycle detection, and bidirectional traversal.
//! Used by the ranking system to score file importance and by context preparation
//! to understand which files are related.

use crate::types::SupportedLanguage;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

/// Represents a dependency edge between two files.
///
/// Tracks which file imports from which other file, along with the raw import
/// string as it appeared in the source code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileDependency {
    /// The file that contains the import statement.
    pub from_file: PathBuf,
    /// The file being imported.
    pub to_file: PathBuf,
    /// The raw import string (e.g., `crate::types::Symbol` or `./bar`).
    pub import_name: String,
}

impl FileDependency {
    /// Creates a new file dependency edge.
    ///
    /// # Arguments
    /// * `from` - The file containing the import statement.
    /// * `to` - The file being imported.
    /// * `import_name` - The raw import string from the source code.
    pub fn new(
        from: impl Into<PathBuf>,
        to: impl Into<PathBuf>,
        import_name: impl Into<String>,
    ) -> Self {
        Self {
            from_file: from.into(),
            to_file: to.into(),
            import_name: import_name.into(),
        }
    }
}

/// Resolves raw import strings to concrete file paths on disk.
///
/// Handles language-specific import resolution conventions:
/// - Rust: `crate::foo::bar` resolves to `src/foo/bar.rs` or `src/foo/bar/mod.rs`
/// - TypeScript/JavaScript: `./foo` tries `.ts`, `.tsx`, `/index.ts`, `.js`, `/index.js`
/// - Python: `foo.bar` tries `foo/bar.py` or `foo/bar/__init__.py`
/// - Go: package paths resolved relative to module root
/// - Java: `com.example.Foo` resolves to `com/example/Foo.java`
/// - C/C++: quoted includes resolve relatively, angle-bracket includes are skipped
pub struct ImportResolver {
    /// The project root directory for resolving paths.
    root: PathBuf,
    /// The language whose resolution conventions to use.
    language: SupportedLanguage,
}

impl ImportResolver {
    /// Creates a new import resolver for the given root directory and language.
    pub fn new(root: impl Into<PathBuf>, language: SupportedLanguage) -> Self {
        Self {
            root: root.into(),
            language,
        }
    }

    /// Resolves a raw import string to a file path on disk.
    ///
    /// Returns `None` if the import cannot be resolved to an existing file.
    /// Resolution logic is language-specific.
    ///
    /// # Arguments
    /// * `import_name` - The raw import string from the source code.
    /// * `from_file` - The file containing the import (used for relative resolution).
    pub fn resolve(&self, import_name: &str, from_file: &Path) -> Option<PathBuf> {
        match self.language {
            SupportedLanguage::Rust => self.resolve_rust(import_name),
            SupportedLanguage::TypeScript | SupportedLanguage::JavaScript => {
                self.resolve_typescript(import_name, from_file)
            }
            SupportedLanguage::Python => self.resolve_python(import_name),
            SupportedLanguage::Go => self.resolve_go(import_name),
            SupportedLanguage::Java => self.resolve_java(import_name),
            SupportedLanguage::C | SupportedLanguage::Cpp => {
                self.resolve_c_cpp(import_name, from_file)
            }
        }
    }

    /// Resolves a Rust import path.
    ///
    /// Converts `crate::foo::bar` to `src/foo/bar.rs` or `src/foo/bar/mod.rs`.
    fn resolve_rust(&self, import_name: &str) -> Option<PathBuf> {
        let path_str = import_name.strip_prefix("crate::")?;
        let parts: Vec<&str> = path_str.split("::").collect();
        if parts.is_empty() {
            return None;
        }

        let relative: PathBuf = parts.iter().collect();

        // Try src/foo/bar.rs
        let candidate = self.root.join("src").join(&relative).with_extension("rs");
        if candidate.exists() {
            return Some(candidate);
        }

        // Try src/foo/bar/mod.rs
        let candidate = self.root.join("src").join(&relative).join("mod.rs");
        if candidate.exists() {
            return Some(candidate);
        }

        None
    }

    /// Resolves a TypeScript/JavaScript import path.
    ///
    /// Tries extensions in order: `.ts`, `.tsx`, `/index.ts`, `.js`, `/index.js`.
    fn resolve_typescript(&self, import_name: &str, from_file: &Path) -> Option<PathBuf> {
        let base = if import_name.starts_with('.') {
            // Relative import - resolve from the importing file's directory
            let from_dir = from_file.parent()?;
            from_dir.join(import_name)
        } else {
            // Non-relative import - resolve from root
            self.root.join(import_name)
        };

        let extensions = [".ts", ".tsx", ".js"];
        for ext in &extensions {
            let candidate = base.with_extension(ext.trim_start_matches('.'));
            if candidate.exists() {
                return Some(candidate);
            }
        }

        // Try index files in a directory
        let index_names = ["index.ts", "index.js"];
        for index_name in &index_names {
            let candidate = base.join(index_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        None
    }

    /// Resolves a Python import path.
    ///
    /// Converts `foo.bar` to `foo/bar.py` or `foo/bar/__init__.py`.
    fn resolve_python(&self, import_name: &str) -> Option<PathBuf> {
        let parts: Vec<&str> = import_name.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let relative: PathBuf = parts.iter().collect();

        // Try foo/bar.py
        let candidate = self.root.join(&relative).with_extension("py");
        if candidate.exists() {
            return Some(candidate);
        }

        // Try foo/bar/__init__.py
        let candidate = self.root.join(&relative).join("__init__.py");
        if candidate.exists() {
            return Some(candidate);
        }

        None
    }

    /// Resolves a Go import path.
    ///
    /// Resolves package paths relative to the module root directory.
    fn resolve_go(&self, import_name: &str) -> Option<PathBuf> {
        let candidate = self.root.join(import_name);
        if candidate.exists() && candidate.is_dir() {
            return Some(candidate);
        }

        // Try as a .go file
        let candidate = self.root.join(import_name).with_extension("go");
        if candidate.exists() {
            return Some(candidate);
        }

        None
    }

    /// Resolves a Java import path.
    ///
    /// Converts `com.example.Foo` to `com/example/Foo.java`.
    fn resolve_java(&self, import_name: &str) -> Option<PathBuf> {
        let parts: Vec<&str> = import_name.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let relative: PathBuf = parts.iter().collect();
        let candidate = self.root.join(&relative).with_extension("java");
        if candidate.exists() {
            return Some(candidate);
        }

        None
    }

    /// Resolves a C/C++ include path.
    ///
    /// Quoted includes (`"foo.h"`) are resolved relative to the importing file.
    /// Angle-bracket includes (`<foo.h>`) are system headers and return `None`.
    fn resolve_c_cpp(&self, import_name: &str, from_file: &Path) -> Option<PathBuf> {
        // System headers (angle brackets) cannot be resolved locally
        if import_name.starts_with('<') {
            return None;
        }

        // Strip surrounding quotes if present
        let cleaned = import_name.trim_start_matches('"').trim_end_matches('"');

        // Resolve relative to the importing file's directory
        if let Some(from_dir) = from_file.parent() {
            let candidate = from_dir.join(cleaned);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        // Also try relative to the project root
        let candidate = self.root.join(cleaned);
        if candidate.exists() {
            return Some(candidate);
        }

        None
    }
}

/// A directed graph of file-level dependencies.
///
/// Tracks which files import from which other files, supporting bidirectional
/// traversal, topological sorting, and cycle detection. Used by the ranking
/// system to compute file importance scores.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileDependencyGraph {
    /// All dependency edges in the graph.
    edges: Vec<FileDependency>,
    /// Forward adjacency: file -> files it imports (dependencies).
    adjacency: HashMap<PathBuf, Vec<PathBuf>>,
    /// Reverse adjacency: file -> files that import it (dependents).
    reverse_adjacency: HashMap<PathBuf, Vec<PathBuf>>,
}

impl FileDependencyGraph {
    /// Creates an empty file dependency graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a dependency edge to the graph.
    ///
    /// Updates both forward and reverse adjacency maps.
    pub fn add_dependency(&mut self, dep: FileDependency) {
        self.adjacency
            .entry(dep.from_file.clone())
            .or_default()
            .push(dep.to_file.clone());
        self.reverse_adjacency
            .entry(dep.to_file.clone())
            .or_default()
            .push(dep.from_file.clone());

        // Ensure both files appear in both maps (even with empty lists)
        self.adjacency.entry(dep.to_file.clone()).or_default();
        self.reverse_adjacency
            .entry(dep.from_file.clone())
            .or_default();

        self.edges.push(dep);
    }

    /// Registers a file in the graph with no dependencies.
    ///
    /// Ensures the file appears in both adjacency maps even if it has
    /// no imports and nothing imports it.
    pub fn add_file(&mut self, file: impl Into<PathBuf>) {
        let file = file.into();
        self.adjacency.entry(file.clone()).or_default();
        self.reverse_adjacency.entry(file).or_default();
    }

    /// Returns the files that this file imports (forward dependencies).
    pub fn dependencies_of(&self, file: &Path) -> Vec<&PathBuf> {
        self.adjacency
            .get(file)
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    /// Returns the files that import this file (reverse dependencies / dependents).
    pub fn dependents_of(&self, file: &Path) -> Vec<&PathBuf> {
        self.reverse_adjacency
            .get(file)
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    /// Produces a topological ordering of files using Kahn's algorithm.
    ///
    /// Returns an error if the graph contains a cycle.
    pub fn topological_sort(&self, i18n: &I18nService) -> MahalaxmiResult<Vec<PathBuf>> {
        let all_files: HashSet<&PathBuf> = self
            .adjacency
            .keys()
            .chain(self.reverse_adjacency.keys())
            .collect();

        // Compute in-degrees (number of dependencies each file has)
        let mut in_degree: HashMap<&PathBuf, usize> = HashMap::new();
        for file in &all_files {
            in_degree.insert(file, 0);
        }
        for deps in self.adjacency.values() {
            for dep in deps {
                if let Some(count) = in_degree.get_mut(&dep) {
                    *count += 1;
                }
            }
        }

        // Start with files that have no incoming edges (no dependents in forward direction)
        // Wait - topological sort for dependencies: if A imports B, edge is A -> B.
        // In-degree here means: how many files point TO this file via adjacency (forward).
        // Actually, for Kahn's on a DAG where A->B means A depends on B:
        // We want to process B before A. So in-degree = number of files that depend on this file? No.
        // Standard topological sort: process nodes with in-degree 0 first.
        // If edge A->B means "A depends on B", then in-degree of B = number of things depending on B.
        // That would output B first (since nothing depends on... wait, B has high in-degree).
        //
        // Let me reconsider: In our adjacency map, adjacency[A] contains B means "A imports B".
        // For topological sort where we want to list dependencies before dependents:
        // We need B to come before A. In the adjacency graph A->B, this is a reverse topological sort
        // of the standard definition. OR we can think of it as: in-degree of a node = how many
        // edges point to it. In adjacency, B is pointed to by A. So in-degree(B) = 1.
        // Kahn's processes in-degree 0 first. A has in-degree 0 (nothing points to A).
        // So Kahn's would output A first, then B. But A depends on B, so B should come first.
        //
        // Solution: use reverse_adjacency for Kahn's, or compute in-degree from reverse direction.
        // Let's use the standard approach: in-degree = number of dependencies (out-degree of adjacency
        // that point TO this node). Actually let me just compute it correctly.
        //
        // For topological sort where dependencies come first:
        // in-degree of a node = number of its own dependencies (how many files it imports).
        // Nodes with 0 dependencies get processed first. That's out-degree of adjacency... no.
        // Actually: in-degree = number of edges pointing to the node.
        // In our graph, adjacency[A] = [B] means A->B (A imports B).
        // So edge goes from A to B. in-degree(B) = 1, in-degree(A) = 0.
        // Kahn's: process A first (in-degree 0), then B. But A depends on B!
        //
        // The issue is direction. If A imports B, then B should be listed first.
        // So we need edges in the opposite direction for topological sort to work as "deps first".
        // Use reverse graph: B->A (B is depended upon by A). Then in-degree(A) = 1, in-degree(B) = 0.
        // Kahn's on reverse graph: B first, then A. Correct!

        // Recompute using reverse_adjacency as the "forward" direction for Kahn's
        let mut in_degree_map: HashMap<&PathBuf, usize> = HashMap::new();
        for file in &all_files {
            in_degree_map.insert(file, 0);
        }
        // reverse_adjacency[B] = [A] means A imports B, i.e. B is depended on by A.
        // We want edges: for each file F, its "successors" are reverse_adjacency[F] (files that depend on F).
        // In-degree of A = number of files that A depends on = adjacency[A].len()
        for (file, deps) in &self.adjacency {
            if let Some(f) = all_files.get(file) {
                in_degree_map.insert(f, deps.len());
            }
        }

        let mut queue: VecDeque<&PathBuf> = VecDeque::new();
        for (file, &degree) in &in_degree_map {
            if degree == 0 {
                queue.push_back(file);
            }
        }

        let mut result: Vec<PathBuf> = Vec::new();

        while let Some(file) = queue.pop_front() {
            result.push(file.clone());

            // For each file that depends on `file`, decrement its in-degree
            if let Some(dependents) = self.reverse_adjacency.get(file.as_path()) {
                for dependent in dependents {
                    if let Some(degree) = in_degree_map.get_mut(&dependent) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 {
                            queue.push_back(dependent);
                        }
                    }
                }
            }
        }

        if result.len() != all_files.len() {
            return Err(MahalaxmiError::indexing(
                i18n,
                keys::indexing::GRAPH_CYCLE_DETECTED,
                &[("files", &format!("{}", all_files.len() - result.len()))],
            ));
        }

        Ok(result)
    }

    /// Checks whether the dependency graph contains any cycles using DFS.
    pub fn has_cycle(&self) -> bool {
        let all_files: HashSet<&PathBuf> = self
            .adjacency
            .keys()
            .chain(self.reverse_adjacency.keys())
            .collect();

        let mut visited: HashSet<&PathBuf> = HashSet::new();
        let mut in_stack: HashSet<&PathBuf> = HashSet::new();

        for file in &all_files {
            if !visited.contains(file)
                && Self::dfs_has_cycle(file, &self.adjacency, &mut visited, &mut in_stack)
            {
                return true;
            }
        }

        false
    }

    /// DFS helper for cycle detection.
    fn dfs_has_cycle<'a>(
        node: &'a PathBuf,
        adjacency: &'a HashMap<PathBuf, Vec<PathBuf>>,
        visited: &mut HashSet<&'a PathBuf>,
        in_stack: &mut HashSet<&'a PathBuf>,
    ) -> bool {
        visited.insert(node);
        in_stack.insert(node);

        if let Some(neighbors) = adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if Self::dfs_has_cycle(neighbor, adjacency, visited, in_stack) {
                        return true;
                    }
                } else if in_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        in_stack.remove(node);
        false
    }

    /// Returns the number of unique files in the graph.
    pub fn file_count(&self) -> usize {
        let all_files: HashSet<&PathBuf> = self
            .adjacency
            .keys()
            .chain(self.reverse_adjacency.keys())
            .collect();
        all_files.len()
    }

    /// Returns the number of dependency edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns the in-degree of a file (number of files that import it).
    pub fn in_degree(&self, file: &Path) -> usize {
        self.reverse_adjacency
            .get(file)
            .map(|deps| deps.len())
            .unwrap_or(0)
    }

    /// Returns the out-degree of a file (number of files it imports).
    pub fn out_degree(&self, file: &Path) -> usize {
        self.adjacency.get(file).map(|deps| deps.len()).unwrap_or(0)
    }

    /// Returns all files in the graph.
    pub fn files(&self) -> Vec<&PathBuf> {
        let all_files: HashSet<&PathBuf> = self
            .adjacency
            .keys()
            .chain(self.reverse_adjacency.keys())
            .collect();
        all_files.into_iter().collect()
    }

    /// Compute the minimum BFS hop-distance from any file in `sources` to `target`,
    /// following forward import edges up to `max_depth` hops.
    ///
    /// Returns `Some(0)` when `target` is itself in `sources`.
    /// Returns `Some(n)` with the minimum hop count when reachable within `max_depth`.
    /// Returns `None` when `target` is not reachable within the depth limit.
    pub fn bfs_distance(
        &self,
        target: &Path,
        sources: &[PathBuf],
        max_depth: usize,
    ) -> Option<usize> {
        if sources.iter().any(|s| s.as_path() == target) {
            return Some(0);
        }

        let mut visited: HashSet<PathBuf> = HashSet::new();
        let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();

        for src in sources {
            if visited.insert(src.clone()) {
                queue.push_back((src.clone(), 0));
            }
        }

        while let Some((node, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            if let Some(deps) = self.adjacency.get(&node) {
                for dep in deps {
                    if dep.as_path() == target {
                        return Some(depth + 1);
                    }
                    if visited.insert(dep.clone()) {
                        queue.push_back((dep.clone(), depth + 1));
                    }
                }
            }
        }

        None
    }

    /// Builds a dependency graph from extracted (file_path, imports) pairs.
    ///
    /// Uses `ImportResolver` to resolve raw import strings to concrete file paths.
    /// Imports that cannot be resolved (external dependencies, system headers) are skipped.
    ///
    /// # Arguments
    /// * `extractions` - Pairs of (file_path, import_strings) from symbol extraction.
    /// * `root` - The project root directory for import resolution.
    /// * `language` - The language to use for import resolution conventions.
    pub fn build_from_extractions(
        extractions: &[(PathBuf, Vec<String>)],
        root: &Path,
        language: SupportedLanguage,
    ) -> Self {
        let resolver = ImportResolver::new(root, language);
        let mut graph = Self::new();

        for (file_path, imports) in extractions {
            graph.add_file(file_path.clone());

            for import_name in imports {
                if let Some(resolved_path) = resolver.resolve(import_name, file_path) {
                    let dep =
                        FileDependency::new(file_path.clone(), resolved_path, import_name.clone());
                    graph.add_dependency(dep);
                }
            }
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_dependency_new() {
        let dep = FileDependency::new("/a.rs", "/b.rs", "crate::b");
        assert_eq!(dep.from_file, PathBuf::from("/a.rs"));
        assert_eq!(dep.to_file, PathBuf::from("/b.rs"));
        assert_eq!(dep.import_name, "crate::b");
    }

    #[test]
    fn empty_graph() {
        let graph = FileDependencyGraph::new();
        assert_eq!(graph.file_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn add_file_and_dependency() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/a.rs");
        graph.add_file("/b.rs");
        graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "crate::b"));

        assert_eq!(graph.file_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.out_degree(Path::new("/a.rs")), 1);
        assert_eq!(graph.in_degree(Path::new("/b.rs")), 1);
    }

    #[test]
    fn dependencies_and_dependents() {
        let mut graph = FileDependencyGraph::new();
        graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "crate::b"));
        graph.add_dependency(FileDependency::new("/a.rs", "/c.rs", "crate::c"));

        let deps = graph.dependencies_of(Path::new("/a.rs"));
        assert_eq!(deps.len(), 2);

        let dependents = graph.dependents_of(Path::new("/b.rs"));
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], &PathBuf::from("/a.rs"));
    }

    #[test]
    fn no_cycle_detection() {
        let mut graph = FileDependencyGraph::new();
        graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "b"));
        graph.add_dependency(FileDependency::new("/b.rs", "/c.rs", "c"));
        assert!(!graph.has_cycle());
    }

    #[test]
    fn cycle_detection() {
        let mut graph = FileDependencyGraph::new();
        graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "b"));
        graph.add_dependency(FileDependency::new("/b.rs", "/c.rs", "c"));
        graph.add_dependency(FileDependency::new("/c.rs", "/a.rs", "a"));
        assert!(graph.has_cycle());
    }

    #[test]
    fn topological_sort_acyclic() {
        use mahalaxmi_core::i18n::locale::SupportedLocale;
        let i18n = I18nService::new(SupportedLocale::EnUs);

        let mut graph = FileDependencyGraph::new();
        graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "b"));
        graph.add_dependency(FileDependency::new("/b.rs", "/c.rs", "c"));

        let sorted = graph.topological_sort(&i18n).unwrap();
        assert_eq!(sorted.len(), 3);

        // c has no dependencies, so it should come first
        let c_pos = sorted.iter().position(|p| p == Path::new("/c.rs")).unwrap();
        let b_pos = sorted.iter().position(|p| p == Path::new("/b.rs")).unwrap();
        let a_pos = sorted.iter().position(|p| p == Path::new("/a.rs")).unwrap();
        assert!(c_pos < b_pos);
        assert!(b_pos < a_pos);
    }

    #[test]
    fn topological_sort_cycle_error() {
        use mahalaxmi_core::i18n::locale::SupportedLocale;
        let i18n = I18nService::new(SupportedLocale::EnUs);

        let mut graph = FileDependencyGraph::new();
        graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "b"));
        graph.add_dependency(FileDependency::new("/b.rs", "/a.rs", "a"));

        let result = graph.topological_sort(&i18n);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_indexing());
    }

    #[test]
    fn files_returns_all() {
        let mut graph = FileDependencyGraph::new();
        graph.add_file("/x.rs");
        graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "b"));

        let files = graph.files();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn bfs_distance_target_in_sources_returns_zero() {
        let mut graph = FileDependencyGraph::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        graph.add_dependency(FileDependency::new(a.clone(), b.clone(), "b"));

        let result = graph.bfs_distance(&a, &[a.clone()], 4);
        assert_eq!(result, Some(0), "source file itself should be distance 0");
    }

    #[test]
    fn bfs_distance_direct_dependency_returns_one() {
        let mut graph = FileDependencyGraph::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        graph.add_dependency(FileDependency::new(a.clone(), b.clone(), "b"));

        let result = graph.bfs_distance(&b, &[a.clone()], 4);
        assert_eq!(result, Some(1), "direct dependency should be distance 1");
    }

    #[test]
    fn bfs_distance_two_hop_dependency_returns_two() {
        let mut graph = FileDependencyGraph::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        let c = PathBuf::from("/c.rs");
        graph.add_dependency(FileDependency::new(a.clone(), b.clone(), "b"));
        graph.add_dependency(FileDependency::new(b.clone(), c.clone(), "c"));

        let result = graph.bfs_distance(&c, &[a.clone()], 4);
        assert_eq!(result, Some(2), "two-hop dependency should be distance 2");
    }

    #[test]
    fn bfs_distance_returns_none_when_unreachable() {
        let mut graph = FileDependencyGraph::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        let c = PathBuf::from("/c.rs"); // no edge to c
        graph.add_dependency(FileDependency::new(a.clone(), b.clone(), "b"));
        graph.add_file(c.clone());

        let result = graph.bfs_distance(&c, &[a.clone()], 4);
        assert_eq!(result, None, "unreachable file should return None");
    }

    #[test]
    fn bfs_distance_respects_max_depth() {
        let mut graph = FileDependencyGraph::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        let c = PathBuf::from("/c.rs");
        let d = PathBuf::from("/d.rs");
        graph.add_dependency(FileDependency::new(a.clone(), b.clone(), "b"));
        graph.add_dependency(FileDependency::new(b.clone(), c.clone(), "c"));
        graph.add_dependency(FileDependency::new(c.clone(), d.clone(), "d"));

        // max_depth = 2 means d (at distance 3) is not reachable.
        let result = graph.bfs_distance(&d, &[a.clone()], 2);
        assert_eq!(result, None, "file beyond max_depth should return None");
    }

    #[test]
    fn bfs_distance_terminates_on_cycle() {
        let mut graph = FileDependencyGraph::new();
        let a = PathBuf::from("/a.rs");
        let b = PathBuf::from("/b.rs");
        let c = PathBuf::from("/c.rs");
        let d = PathBuf::from("/d.rs");
        // Cycle: a -> b -> c -> a
        graph.add_dependency(FileDependency::new(a.clone(), b.clone(), "b"));
        graph.add_dependency(FileDependency::new(b.clone(), c.clone(), "c"));
        graph.add_dependency(FileDependency::new(c.clone(), a.clone(), "a"));
        // d is not connected
        graph.add_file(d.clone());

        // Must terminate (not loop forever); d is unreachable.
        let result = graph.bfs_distance(&d, &[a.clone()], 10);
        assert_eq!(result, None, "visited set should prevent infinite loops");
    }
}

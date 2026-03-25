//! File relevance scoring for intelligent context preparation.
//!
//! Scores files by their relevance to a worker task using multiple
//! heuristics: direct mentions, affected files, imports, and test
//! relationships.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::models::plan::WorkerTask;

#[cfg(feature = "context")]
use mahalaxmi_indexing::FileDependencyGraph;

/// Reason a file is considered relevant to a task.
#[derive(Debug, Clone)]
pub enum RelevanceReason {
    /// File path appears directly in the task description or title.
    DirectlyMentioned {
        /// Which field mentioned it (e.g. "description", "title").
        in_field: String,
    },
    /// File is listed in the task's affected_files.
    AffectedFile {
        /// Relationship type (e.g. "primary target").
        relationship: String,
    },
    /// File is imported by an affected file (transitive dependency).
    ImportedByAffected {
        /// The affected file that imports this one.
        imported_by: PathBuf,
    },
    /// File is a test file for a relevant source file.
    TestFile {
        /// The source file this test covers.
        tests_for: PathBuf,
    },
}

impl RelevanceReason {
    /// Returns the base relevance score for this reason type.
    pub fn score(&self) -> f64 {
        match self {
            Self::DirectlyMentioned { .. } => 1.0,
            Self::AffectedFile { .. } => 0.8,
            Self::ImportedByAffected { .. } => 0.5,
            Self::TestFile { .. } => 0.3,
        }
    }
}

impl fmt::Display for RelevanceReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectlyMentioned { in_field } => {
                write!(f, "directly mentioned in {}", in_field)
            }
            Self::AffectedFile { relationship } => {
                write!(f, "affected file ({})", relationship)
            }
            Self::ImportedByAffected { imported_by } => {
                write!(f, "imported by {}", imported_by.display())
            }
            Self::TestFile { tests_for } => {
                write!(f, "test for {}", tests_for.display())
            }
        }
    }
}

/// A file's relevance score and reasons for a particular task.
#[derive(Debug, Clone)]
pub struct FileRelevance {
    /// Path to the file.
    pub file_path: PathBuf,
    /// Aggregate relevance score (max of all reasons).
    pub score: f64,
    /// All reasons this file is considered relevant.
    pub reasons: Vec<RelevanceReason>,
}

impl FileRelevance {
    /// Create a new file relevance entry.
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
            score: 0.0,
            reasons: Vec::new(),
        }
    }

    /// Add a reason and update the aggregate score (max wins).
    pub fn add_reason(&mut self, reason: RelevanceReason) {
        let reason_score = reason.score();
        if reason_score > self.score {
            self.score = reason_score;
        }
        self.reasons.push(reason);
    }

    /// Returns the primary (highest-scoring) reason.
    pub fn primary_reason(&self) -> Option<&RelevanceReason> {
        self.reasons.iter().max_by(|a, b| {
            a.score()
                .partial_cmp(&b.score())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Returns true if the file was directly mentioned in the task.
    pub fn is_directly_mentioned(&self) -> bool {
        self.reasons
            .iter()
            .any(|r| matches!(r, RelevanceReason::DirectlyMentioned { .. }))
    }
}

/// Score files for relevance to a worker task.
///
/// When a `FileDependencyGraph` is available (via the `context` feature),
/// walks imports and dependents for transitive scoring. Without it, uses
/// path-based heuristics (same directory, test directory patterns).
#[cfg(feature = "context")]
pub fn score_files(task: &WorkerTask, graph: Option<&FileDependencyGraph>) -> Vec<FileRelevance> {
    score_files_inner(task, graph)
}

/// Score files for relevance to a worker task (without dependency graph).
#[cfg(not(feature = "context"))]
pub fn score_files(task: &WorkerTask) -> Vec<FileRelevance> {
    score_files_inner(task, None::<&()>)
}

fn score_files_inner<G>(task: &WorkerTask, graph: Option<&G>) -> Vec<FileRelevance>
where
    G: DependencyLookup,
{
    let mut file_map: HashMap<PathBuf, FileRelevance> = HashMap::new();

    // 1. Files mentioned in description
    let desc_paths = extract_file_paths(&task.description);
    for path in &desc_paths {
        let entry = file_map
            .entry(path.clone())
            .or_insert_with(|| FileRelevance::new(path));
        entry.add_reason(RelevanceReason::DirectlyMentioned {
            in_field: "description".to_owned(),
        });
    }

    // 2. Files mentioned in title
    let title_paths = extract_file_paths(&task.title);
    for path in &title_paths {
        let entry = file_map
            .entry(path.clone())
            .or_insert_with(|| FileRelevance::new(path));
        entry.add_reason(RelevanceReason::DirectlyMentioned {
            in_field: "title".to_owned(),
        });
    }

    // 3. Affected files
    for affected in &task.affected_files {
        let path = PathBuf::from(affected);
        let entry = file_map
            .entry(path.clone())
            .or_insert_with(|| FileRelevance::new(&path));
        entry.add_reason(RelevanceReason::AffectedFile {
            relationship: "primary target".to_owned(),
        });
    }

    // 4. Transitive dependencies (if graph available)
    let primary_paths: Vec<PathBuf> = file_map.keys().cloned().collect();
    if let Some(g) = graph {
        for primary in &primary_paths {
            // Dependencies of the primary file
            for dep in g.lookup_dependencies(primary) {
                let entry = file_map
                    .entry(dep.clone())
                    .or_insert_with(|| FileRelevance::new(&dep));
                entry.add_reason(RelevanceReason::ImportedByAffected {
                    imported_by: primary.clone(),
                });
            }
            // Dependents (reverse deps) could be test files
            for dep in g.lookup_dependents(primary) {
                if is_test_path(&dep) {
                    let entry = file_map
                        .entry(dep.clone())
                        .or_insert_with(|| FileRelevance::new(&dep));
                    entry.add_reason(RelevanceReason::TestFile {
                        tests_for: primary.clone(),
                    });
                }
            }
        }
    } else {
        // Path-based test file heuristic
        for primary in &primary_paths {
            if let Some(test_path) = infer_test_path(primary) {
                let entry = file_map
                    .entry(test_path.clone())
                    .or_insert_with(|| FileRelevance::new(&test_path));
                entry.add_reason(RelevanceReason::TestFile {
                    tests_for: primary.clone(),
                });
            }
        }
    }

    let mut results: Vec<FileRelevance> = file_map.into_values().collect();
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

/// Extract file paths from text using regex.
///
/// Recognizes Unix-style paths (`src/foo/bar.rs`) and Windows-style
/// paths (`src\foo\bar.rs`). Paths must contain at least one separator
/// and have a file extension.
pub fn extract_file_paths(text: &str) -> Vec<PathBuf> {
    let re = Regex::new(r#"(?:^|[\s"'`(,])([a-zA-Z0-9_.][a-zA-Z0-9_./\\-]*\.[a-zA-Z0-9]+)"#)
        .expect("file path regex is valid");

    let mut paths = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for cap in re.captures_iter(text) {
        let path_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        // Must contain at least one path separator
        if path_str.contains('/') || path_str.contains('\\') {
            let path = PathBuf::from(path_str);
            if seen.insert(path.clone()) {
                paths.push(path);
            }
        }
    }

    paths
}

/// Trait for abstracting dependency graph lookups.
trait DependencyLookup {
    fn lookup_dependencies(&self, file: &Path) -> Vec<PathBuf>;
    fn lookup_dependents(&self, file: &Path) -> Vec<PathBuf>;
}

#[cfg(feature = "context")]
impl DependencyLookup for FileDependencyGraph {
    fn lookup_dependencies(&self, file: &Path) -> Vec<PathBuf> {
        self.dependencies_of(file).into_iter().cloned().collect()
    }

    fn lookup_dependents(&self, file: &Path) -> Vec<PathBuf> {
        self.dependents_of(file).into_iter().cloned().collect()
    }
}

// Fallback for when no graph is available — never actually used but
// satisfies the generic constraint.
impl DependencyLookup for () {
    fn lookup_dependencies(&self, _file: &Path) -> Vec<PathBuf> {
        Vec::new()
    }
    fn lookup_dependents(&self, _file: &Path) -> Vec<PathBuf> {
        Vec::new()
    }
}

/// Checks if a path looks like a test file.
fn is_test_path(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("test") || s.contains("spec") || s.contains("_test.") || s.contains(".test.")
}

/// Infer a test file path from a source file path using common conventions.
fn infer_test_path(source: &Path) -> Option<PathBuf> {
    let stem = source.file_stem()?.to_str()?;
    let ext = source.extension()?.to_str()?;
    let parent = source.parent()?;

    // Try tests/ sibling directory
    if let Some(grandparent) = parent.parent() {
        let test_dir = grandparent.join("tests");
        let test_file = test_dir.join(format!("{}_test.{}", stem, ext));
        return Some(test_file);
    }

    None
}

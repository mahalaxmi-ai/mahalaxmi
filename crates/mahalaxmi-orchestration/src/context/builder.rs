//! Context builder — assembles the final context preamble for a worker.
//!
//! Combines repo map, relevant file chunks, shared memory entries, and
//! task description into a formatted preamble string within token budget.

use std::fmt;
use std::path::PathBuf;

use mahalaxmi_core::config::{ContextConfig, ContextFormat};
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;

use super::budget::{estimate_tokens, TokenBudget, TokenUsage};
#[cfg(feature = "context")]
use super::chunker::{select_chunks, CodeChunker};
use super::relevance::score_files;
use crate::models::plan::WorkerTask;

#[cfg(feature = "context")]
use mahalaxmi_indexing::{CodebaseIndex, RepoMapConfig};
#[cfg(feature = "context")]
use mahalaxmi_memory::{MemoryInjector, MemoryStore};

/// The kind of section in the context preamble.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextSectionKind {
    /// Repository structure map.
    RepoMap,
    /// Relevant file contents.
    RelevantFiles,
    /// Shared memory entries.
    SharedMemory,
    /// Task description and instructions.
    TaskDescription,
    /// Output from a dependency worker (cross-provider handoff).
    DependencyOutput,
}

impl ContextSectionKind {
    /// Returns the section name as a plain string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RepoMap => "repo_map",
            Self::RelevantFiles => "relevant_files",
            Self::SharedMemory => "shared_memory",
            Self::TaskDescription => "task_description",
            Self::DependencyOutput => "dependency_output",
        }
    }

    /// Returns the XML tag name for this section.
    pub fn xml_tag(&self) -> &'static str {
        self.as_str()
    }

    /// Returns the Markdown header text for this section.
    pub fn markdown_header(&self) -> &'static str {
        match self {
            Self::RepoMap => "Repo Map",
            Self::RelevantFiles => "Relevant Files",
            Self::SharedMemory => "Shared Memory",
            Self::TaskDescription => "Task Description",
            Self::DependencyOutput => "Dependency Output",
        }
    }
}

impl fmt::Display for ContextSectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single section of the assembled context.
#[derive(Debug, Clone)]
pub struct ContextSection {
    /// What kind of section this is.
    pub kind: ContextSectionKind,
    /// The section content.
    pub content: String,
    /// Estimated token count for this section.
    pub token_count: usize,
}

impl ContextSection {
    /// Create a new context section, auto-computing token count.
    pub fn new(kind: ContextSectionKind, content: impl Into<String>) -> Self {
        let content = content.into();
        let token_count = estimate_tokens(&content);
        Self {
            kind,
            content,
            token_count,
        }
    }
}

/// The assembled worker context, ready for injection.
#[derive(Debug, Clone)]
pub struct WorkerContext {
    /// The formatted preamble string.
    pub preamble: String,
    /// Token usage breakdown.
    pub token_usage: TokenUsage,
    /// Files included in the context.
    pub files_included: Vec<PathBuf>,
    /// Number of memory entries included.
    pub memory_entries_included: usize,
}

impl WorkerContext {
    /// Create an empty worker context.
    pub fn new() -> Self {
        Self {
            preamble: String::new(),
            token_usage: TokenUsage::new(0),
            files_included: Vec::new(),
            memory_entries_included: 0,
        }
    }

    /// Returns true if no context was assembled.
    pub fn is_empty(&self) -> bool {
        self.preamble.is_empty()
    }

    /// Returns the number of files included.
    pub fn file_count(&self) -> usize {
        self.files_included.len()
    }
}

impl Default for WorkerContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds context preambles for worker tasks.
pub struct ContextBuilder {
    config: ContextConfig,
}

impl ContextBuilder {
    /// Create a new context builder.
    pub fn new(config: ContextConfig) -> Self {
        Self { config }
    }

    /// Build context for a worker task (with optional index and memory).
    #[cfg(feature = "context")]
    pub fn build(
        &self,
        task: &WorkerTask,
        budget: &TokenBudget,
        index: Option<&CodebaseIndex>,
        memory: Option<&MemoryStore>,
        _i18n: &I18nService,
    ) -> MahalaxmiResult<WorkerContext> {
        if !self.config.enabled {
            return Ok(WorkerContext::new());
        }

        let mut sections = Vec::new();
        let mut token_usage = TokenUsage::new(budget.total());
        let mut files_included = Vec::new();
        let mut memory_entries_included = 0;

        // 1. Repo map section
        if self.config.include_repo_map {
            if let Some(idx) = index {
                let repo_map_config = RepoMapConfig::new();
                let repo_map_content = idx.repo_map(&repo_map_config);
                let repo_map_tokens = estimate_tokens(&repo_map_content);
                let max_repo_tokens = budget.tokens_for_repo_map();

                if repo_map_tokens <= max_repo_tokens && !repo_map_content.is_empty() {
                    token_usage.add_repo_map(repo_map_tokens);
                    sections.push(ContextSection::new(
                        ContextSectionKind::RepoMap,
                        repo_map_content,
                    ));
                }
            }
        }

        // 2. Relevant files section
        let scored = score_files(task, index.map(|idx| idx.graph()));
        let max_files = self.config.max_files;
        let top_files: Vec<_> = scored.into_iter().take(max_files).collect();

        let chunker = CodeChunker::with_default_config();
        let mut all_chunks = Vec::new();

        for file_rel in &top_files {
            // In a real system, we'd read the file content from disk.
            // Here we note the file path for the context.
            files_included.push(file_rel.file_path.clone());

            // If the file content isn't available, we include the path reference
            let placeholder = format!(
                "// File: {} (relevance: {:.1})\n",
                file_rel.file_path.display(),
                file_rel.score
            );
            let chunks = chunker.chunk_file(
                &file_rel.file_path,
                &placeholder,
                budget.tokens_for_files() / max_files.max(1),
            );
            all_chunks.extend(chunks);
        }

        let selected = select_chunks(&all_chunks, budget.tokens_for_files());
        if !selected.is_empty() {
            let file_content: String = selected
                .iter()
                .map(|c| c.content.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let file_tokens = estimate_tokens(&file_content);
            token_usage.add_files(file_tokens);
            sections.push(ContextSection::new(
                ContextSectionKind::RelevantFiles,
                file_content,
            ));
        }

        // 3. Memory section
        if self.config.include_memory {
            if let Some(store) = memory {
                let injector_config = mahalaxmi_memory::InjectorConfig {
                    max_tokens: budget.tokens_for_memory(),
                    include_types: vec![
                        mahalaxmi_memory::MemoryType::CodebaseFact,
                        mahalaxmi_memory::MemoryType::Convention,
                        mahalaxmi_memory::MemoryType::Decision,
                        mahalaxmi_memory::MemoryType::Warning,
                    ],
                    min_confidence: 0.5,
                    format: mahalaxmi_memory::InjectionFormat::Markdown,
                };
                let injector = MemoryInjector::new(injector_config);
                let memory_content = injector.inject(store, None);
                memory_entries_included = injector.inject_count(store, None);

                if !memory_content.is_empty() {
                    let mem_tokens = estimate_tokens(&memory_content);
                    token_usage.add_memory(mem_tokens);
                    sections.push(ContextSection::new(
                        ContextSectionKind::SharedMemory,
                        memory_content,
                    ));
                }
            }
        }

        // 4. Task description section
        let task_desc = format!("## Task: {}\n\n{}", task.title, task.description);
        let task_tokens = estimate_tokens(&task_desc);
        if task_tokens <= budget.tokens_for_task() {
            token_usage.add_task(task_tokens);
            sections.push(ContextSection::new(
                ContextSectionKind::TaskDescription,
                task_desc,
            ));
        }

        let preamble = format_preamble(&sections, self.config.format, None);

        Ok(WorkerContext {
            preamble,
            token_usage,
            files_included,
            memory_entries_included,
        })
    }

    /// Build context for a worker task (without optional dependencies).
    #[cfg(not(feature = "context"))]
    pub fn build(
        &self,
        task: &WorkerTask,
        budget: &TokenBudget,
        _i18n: &I18nService,
    ) -> MahalaxmiResult<WorkerContext> {
        if !self.config.enabled {
            return Ok(WorkerContext::new());
        }

        let mut sections = Vec::new();
        let mut token_usage = TokenUsage::new(budget.total());
        let mut files_included = Vec::new();

        // Without context feature, we can only score files from task metadata
        let scored = score_files(task);
        let max_files = self.config.max_files;
        let top_files: Vec<_> = scored.into_iter().take(max_files).collect();

        for file_rel in &top_files {
            files_included.push(file_rel.file_path.clone());
        }

        if !files_included.is_empty() {
            let file_content: String = files_included
                .iter()
                .map(|p| format!("// File: {}\n", p.display()))
                .collect();
            let file_tokens = estimate_tokens(&file_content);
            token_usage.add_files(file_tokens);
            sections.push(ContextSection::new(
                ContextSectionKind::RelevantFiles,
                file_content,
            ));
        }

        // Task description
        let task_desc = format!("## Task: {}\n\n{}", task.title, task.description);
        let task_tokens = estimate_tokens(&task_desc);
        if task_tokens <= budget.tokens_for_task() {
            token_usage.add_task(task_tokens);
            sections.push(ContextSection::new(
                ContextSectionKind::TaskDescription,
                task_desc,
            ));
        }

        let preamble = format_preamble(&sections, self.config.format, None);

        Ok(WorkerContext {
            preamble,
            token_usage,
            files_included,
            memory_entries_included: 0,
        })
    }
}

/// Description of a dependency worker's output for cross-provider handoff.
pub struct DependencyHandoff {
    /// The task title of the dependency worker.
    pub task_title: String,
    /// The provider that produced this output.
    pub source_provider_id: String,
    /// The output content from the dependency worker.
    pub output: String,
}

/// Build a provenance-annotated context section from dependency worker outputs.
///
/// When a worker depends on output from another worker that used a different
/// provider, this function annotates the output with provenance information
/// so the target worker knows the origin of the content.
///
/// The output is formatted for the target provider (XML for Claude, Markdown for others).
pub fn build_dependency_sections(
    handoffs: &[DependencyHandoff],
    target_format: ContextFormat,
    target_provider_id: Option<&str>,
) -> Vec<ContextSection> {
    if handoffs.is_empty() {
        return Vec::new();
    }

    let effective_format = match target_format {
        ContextFormat::Auto => {
            if target_provider_id.is_some_and(|p| p.contains("claude")) {
                ContextFormat::Xml
            } else {
                ContextFormat::Markdown
            }
        }
        other => other,
    };

    handoffs
        .iter()
        .map(|h| {
            let annotated = match effective_format {
                ContextFormat::Xml | ContextFormat::Auto => {
                    format!(
                        "<provenance source_provider=\"{}\" task=\"{}\">\n{}\n</provenance>",
                        h.source_provider_id, h.task_title, h.output
                    )
                }
                ContextFormat::Markdown => {
                    format!(
                        "> **Provenance**: Output generated by `{}` for task \"{}\"\n\n{}",
                        h.source_provider_id, h.task_title, h.output
                    )
                }
                ContextFormat::PlainText => {
                    format!(
                        "[Provenance: generated by {} for task \"{}\"]\n{}",
                        h.source_provider_id, h.task_title, h.output
                    )
                }
            };
            ContextSection::new(ContextSectionKind::DependencyOutput, annotated)
        })
        .collect()
}

/// Format sections into a preamble string.
///
/// - `Xml` / `Auto` with Claude-like provider: XML tags
/// - `Markdown` / `Auto` with other provider: Markdown headers
/// - `PlainText`: Simple labeled sections with `---` separators
pub fn format_preamble(
    sections: &[ContextSection],
    format: ContextFormat,
    provider_id: Option<&str>,
) -> String {
    if sections.is_empty() {
        return String::new();
    }

    let effective_format = match format {
        ContextFormat::Auto => {
            if provider_id.is_some_and(|p| p.contains("claude")) {
                ContextFormat::Xml
            } else {
                ContextFormat::Markdown
            }
        }
        other => other,
    };

    let mut output = String::new();

    for section in sections {
        if section.content.is_empty() {
            continue;
        }
        match effective_format {
            ContextFormat::Xml | ContextFormat::Auto => {
                output.push_str(&format!(
                    "<{}>\n{}\n</{}>\n\n",
                    section.kind.xml_tag(),
                    section.content,
                    section.kind.xml_tag()
                ));
            }
            ContextFormat::Markdown => {
                output.push_str(&format!(
                    "## {}\n\n{}\n\n",
                    section.kind.markdown_header(),
                    section.content
                ));
            }
            ContextFormat::PlainText => {
                output.push_str(&format!(
                    "=== {} ===\n{}\n---\n\n",
                    section.kind.markdown_header(),
                    section.content
                ));
            }
        }
    }

    output.trim_end().to_owned()
}

//! Code chunking for intelligent context preparation.
//!
//! Splits file content into chunks that fit within token budgets,
//! preferring natural boundaries (blank lines, function boundaries).

use std::path::{Path, PathBuf};

use super::budget::estimate_tokens;

/// Configuration for the code chunker.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    /// Maximum tokens per chunk.
    pub max_chunk_tokens: usize,
    /// Whether to prefer AST boundaries when splitting.
    pub prefer_ast_boundaries: bool,
    /// Number of context lines to include before/after each chunk.
    pub include_context_lines: usize,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            max_chunk_tokens: 512,
            prefer_ast_boundaries: true,
            include_context_lines: 2,
        }
    }
}

/// A chunk of code extracted from a file.
#[derive(Debug, Clone)]
pub struct CodeChunk {
    /// Source file path.
    pub file_path: PathBuf,
    /// The chunk content.
    pub content: String,
    /// Start line in the original file (1-indexed).
    pub start_line: usize,
    /// End line in the original file (1-indexed, inclusive).
    pub end_line: usize,
    /// Symbols (function/struct names) found in this chunk.
    pub symbols: Vec<String>,
    /// Estimated token count for this chunk.
    pub token_estimate: usize,
}

impl CodeChunk {
    /// Create a new code chunk.
    pub fn new(
        file_path: impl Into<PathBuf>,
        content: impl Into<String>,
        start_line: usize,
        end_line: usize,
    ) -> Self {
        let content = content.into();
        let token_estimate = estimate_tokens(&content);
        Self {
            file_path: file_path.into(),
            content,
            start_line,
            end_line,
            symbols: Vec::new(),
            token_estimate,
        }
    }

    /// Returns the number of lines in this chunk.
    pub fn line_count(&self) -> usize {
        if self.end_line >= self.start_line {
            self.end_line - self.start_line + 1
        } else {
            0
        }
    }

    /// Returns true if the chunk has no content.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Add a symbol name found in this chunk.
    pub fn add_symbol(&mut self, symbol: impl Into<String>) {
        self.symbols.push(symbol.into());
    }
}

/// Splits files into token-budget-aware chunks.
pub struct CodeChunker {
    config: ChunkerConfig,
}

impl CodeChunker {
    /// Create a chunker with the given configuration.
    pub fn new(config: ChunkerConfig) -> Self {
        Self { config }
    }

    /// Create a chunker with default configuration.
    pub fn with_default_config() -> Self {
        Self {
            config: ChunkerConfig::default(),
        }
    }

    /// Returns a reference to the chunker configuration.
    pub fn config(&self) -> &ChunkerConfig {
        &self.config
    }

    /// Chunk a file's content into pieces that fit within `max_tokens`.
    ///
    /// If the entire file fits within the budget, returns a single chunk.
    /// Otherwise, splits at blank lines (natural paragraph boundaries).
    /// When `max_tokens` is 0, uses the configured `max_chunk_tokens`.
    pub fn chunk_file(&self, path: &Path, content: &str, max_tokens: usize) -> Vec<CodeChunk> {
        let max_tokens = if max_tokens == 0 {
            self.config.max_chunk_tokens
        } else {
            max_tokens
        };
        if content.is_empty() {
            return Vec::new();
        }

        let file_tokens = estimate_tokens(content);
        if file_tokens <= max_tokens {
            let line_count = content.lines().count().max(1);
            return vec![CodeChunk::new(path, content, 1, line_count)];
        }

        self.chunk_by_lines(path, content, max_tokens)
    }

    /// Split content into chunks by lines, preferring blank-line boundaries.
    pub fn chunk_by_lines(&self, path: &Path, content: &str, max_tokens: usize) -> Vec<CodeChunk> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Vec::new();
        }

        let effective_max = max_tokens.max(1);
        let mut chunks = Vec::new();
        let mut chunk_start = 0;
        let mut current_content = String::new();

        for (i, line) in lines.iter().enumerate() {
            let candidate = if current_content.is_empty() {
                line.to_string()
            } else {
                format!("{}\n{}", current_content, line)
            };

            let candidate_tokens = estimate_tokens(&candidate);

            if candidate_tokens > effective_max && !current_content.is_empty() {
                // Emit the current chunk before this line pushes us over
                let end_line = i; // 0-indexed, exclusive
                let chunk = CodeChunk::new(
                    path,
                    &current_content,
                    chunk_start + 1,
                    end_line, // 1-indexed
                );
                chunks.push(chunk);
                current_content = line.to_string();
                chunk_start = i;
            } else {
                current_content = candidate;
            }
        }

        // Emit the final chunk
        if !current_content.is_empty() {
            let chunk = CodeChunk::new(path, &current_content, chunk_start + 1, lines.len());
            chunks.push(chunk);
        }

        chunks
    }
}

/// Select chunks greedily that fit within the given token budget.
///
/// Processes chunks in order, adding each if there's room.
pub fn select_chunks(chunks: &[CodeChunk], token_budget: usize) -> Vec<&CodeChunk> {
    let mut selected = Vec::new();
    let mut used = 0;

    for chunk in chunks {
        if used + chunk.token_estimate <= token_budget {
            selected.push(chunk);
            used += chunk.token_estimate;
        }
    }

    selected
}

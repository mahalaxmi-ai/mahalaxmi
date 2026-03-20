//! Document input loading and chunking for domain `InputFormat` variants.
//!
//! `load_document` is the single entry point. Given a domain's `InputFormat`,
//! an optional file path, and optional raw text, it returns a `Vec<String>` of
//! text chunks ready to be passed as manager context.
//!
//! Returning an empty `Vec` signals the caller to fall back to the default
//! codebase repository-map scan.

use mahalaxmi_core::domain::InputFormat;

/// Errors produced during document loading and chunking.
#[derive(Debug, thiserror::Error)]
pub enum InputAdapterError {
    /// The provided input parameters are invalid for the configured format.
    #[error("{0}")]
    InvalidInput(String),
    /// An I/O error occurred while reading the document.
    #[error("I/O error reading document: {0}")]
    Io(#[from] std::io::Error),
    /// The document type is not accepted by the domain configuration.
    #[error("File type '{actual}' not accepted by domain; accepted: {accepted:?}")]
    UnacceptedType {
        actual: String,
        accepted: Vec<String>,
    },
    /// The document exceeds the configured maximum file size.
    #[error("File size {actual_mb}MB exceeds the domain limit of {limit_mb}MB")]
    FileTooLarge { actual_mb: u64, limit_mb: usize },
    /// The document format cannot be parsed by the available adapters.
    #[error("Unsupported document format: {0}")]
    UnsupportedFormat(String),
}

/// Chunk size in characters (≈ 1000 tokens at 4 chars/token).
const CHUNK_SIZE: usize = 4000;

/// Overlap in characters between adjacent chunks.
const CHUNK_OVERLAP: usize = 200;

/// Load and chunk document input based on the domain `InputFormat`.
///
/// Returns `Vec<String>` of text chunks.  An **empty `Vec`** signals the caller
/// that this input format defers to the existing codebase scan (i.e. `Codebase`
/// variant or no domain configured).
///
/// # Arguments
///
/// * `input_format` — the domain's configured input format.
/// * `file_path` — path to the document file; required for `DocumentFile`.
/// * `text_input` — raw text; used when `input_format` is `TextInput`.
pub fn load_document(
    input_format: &InputFormat,
    file_path: Option<&str>,
    text_input: Option<&str>,
) -> Result<Vec<String>, InputAdapterError> {
    match input_format {
        InputFormat::Codebase => {
            // Signal caller to use existing codebase scan.
            Ok(vec![])
        }

        InputFormat::TextInput => {
            let text = text_input.unwrap_or("");
            Ok(chunk_text(text, CHUNK_SIZE, CHUNK_OVERLAP))
        }

        InputFormat::DocumentFile {
            accepted_types,
            max_file_size_mb,
        } => {
            let path_str = file_path.ok_or_else(|| {
                InputAdapterError::InvalidInput(
                    "Domain requires document input but no input_file_path provided".to_owned(),
                )
            })?;

            let path = std::path::Path::new(path_str);

            // Validate file extension against accepted_types.
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let accepted_lower: Vec<String> =
                accepted_types.iter().map(|t| t.to_lowercase()).collect();

            if !accepted_lower.contains(&ext) {
                return Err(InputAdapterError::UnacceptedType {
                    actual: ext,
                    accepted: accepted_types.clone(),
                });
            }

            // Validate file size.
            let metadata = std::fs::metadata(path)?;
            let size_bytes = metadata.len();
            let limit_bytes = (*max_file_size_mb as u64) * 1024 * 1024;
            if size_bytes > limit_bytes {
                return Err(InputAdapterError::FileTooLarge {
                    actual_mb: size_bytes / (1024 * 1024),
                    limit_mb: *max_file_size_mb,
                });
            }

            // Extract text based on file type.
            let text = extract_text(path, &ext)?;
            Ok(chunk_text(&text, CHUNK_SIZE, CHUNK_OVERLAP))
        }
    }
}

/// Extract plain text from a document at `path` with the given extension.
fn extract_text(path: &std::path::Path, ext: &str) -> Result<String, InputAdapterError> {
    match ext {
        "txt" | "md" => {
            let content = std::fs::read_to_string(path)?;
            Ok(content)
        }
        #[cfg(feature = "document-input")]
        "pdf" => {
            pdf_extract::extract_text(path)
                .map_err(|e| InputAdapterError::UnsupportedFormat(e.to_string()))
        }
        _ => Err(InputAdapterError::UnsupportedFormat(format!(
            "No text extractor available for '.{ext}' files"
        ))),
    }
}

/// Split `text` into overlapping chunks of at most `chunk_size` characters
/// with `overlap` characters of shared context between adjacent chunks.
///
/// Returns an empty `Vec` when `text` is empty.
fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    if text.len() <= chunk_size {
        return vec![text.to_owned()];
    }

    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let step = chunk_size.saturating_sub(overlap).max(1);
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < total {
        let end = (start + chunk_size).min(total);
        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);
        if end == total {
            break;
        }
        start += step;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codebase_returns_empty_vec() {
        let result = load_document(&InputFormat::Codebase, None, None);
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_empty(),
            "Codebase should return empty vec to signal codebase scan"
        );
    }

    #[test]
    fn text_input_returns_single_chunk_for_short_text() {
        let result = load_document(&InputFormat::TextInput, None, Some("Hello, world!"));
        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello, world!");
    }

    #[test]
    fn text_input_empty_text_returns_empty_vec() {
        let result = load_document(&InputFormat::TextInput, None, Some(""));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn text_input_none_treated_as_empty() {
        let result = load_document(&InputFormat::TextInput, None, None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn document_file_with_no_path_returns_invalid_input_error() {
        let format = InputFormat::DocumentFile {
            accepted_types: vec!["txt".to_owned(), "pdf".to_owned()],
            max_file_size_mb: 10,
        };
        let result = load_document(&format, None, None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, InputAdapterError::InvalidInput(_)),
            "Expected InvalidInput error when file_path is None, got: {err:?}"
        );
        assert!(
            err.to_string().contains("no input_file_path provided"),
            "Error message should mention missing file path: {err}"
        );
    }

    #[test]
    fn document_file_rejects_unaccepted_extension() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let file_path = dir.path().join("document.xyz");
        std::fs::write(&file_path, "content").expect("write failed");

        let format = InputFormat::DocumentFile {
            accepted_types: vec!["pdf".to_owned(), "txt".to_owned()],
            max_file_size_mb: 10,
        };
        let result = load_document(
            &format,
            Some(file_path.to_str().expect("path to str")),
            None,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            InputAdapterError::UnacceptedType { .. }
        ));
    }

    #[test]
    fn document_file_reads_txt_file() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let file_path = dir.path().join("notes.txt");
        std::fs::write(&file_path, "Sample document content.").expect("write failed");

        let format = InputFormat::DocumentFile {
            accepted_types: vec!["txt".to_owned()],
            max_file_size_mb: 10,
        };
        let result = load_document(
            &format,
            Some(file_path.to_str().expect("path to str")),
            None,
        );
        assert!(result.is_ok(), "Should successfully read .txt file");
        let chunks = result.unwrap();
        assert!(!chunks.is_empty(), "Should return at least one chunk");
        assert!(
            chunks[0].contains("Sample document content."),
            "Chunk should contain file content"
        );
    }

    #[test]
    fn document_file_rejects_oversized_file() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let file_path = dir.path().join("big.txt");
        // Write more than 1 MB to trigger the size check.
        let big_content = "x".repeat(2 * 1024 * 1024 + 1);
        std::fs::write(&file_path, &big_content).expect("write failed");

        let format = InputFormat::DocumentFile {
            accepted_types: vec!["txt".to_owned()],
            max_file_size_mb: 1,
        };
        let result = load_document(
            &format,
            Some(file_path.to_str().expect("path to str")),
            None,
        );
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), InputAdapterError::FileTooLarge { .. }),
            "Should return FileTooLarge error"
        );
    }

    #[test]
    fn chunk_text_single_chunk_for_short_input() {
        let chunks = chunk_text("short", 4000, 200);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "short");
    }

    #[test]
    fn chunk_text_splits_long_input_with_overlap() {
        // Create a string longer than chunk_size.
        let long_text = "a".repeat(5000);
        let chunks = chunk_text(&long_text, 4000, 200);
        assert!(chunks.len() >= 2, "Long text should produce multiple chunks");
        // Each chunk should be at most 4000 chars.
        for chunk in &chunks {
            assert!(chunk.len() <= 4000);
        }
        // The second chunk should start within the overlap region.
        let second_start: String = chunks[1].chars().take(50).collect();
        assert!(
            !second_start.is_empty(),
            "Second chunk should not be empty"
        );
    }
}

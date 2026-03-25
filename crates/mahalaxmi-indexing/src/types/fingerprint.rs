//! File fingerprinting for incremental index updates.
//!
//! A [`FileFingerprint`] captures the content hash, size, modification time,
//! and detected language of a source file. By comparing fingerprints across
//! index runs, the system identifies which files have changed and need
//! re-extraction, enabling efficient incremental updates.

use crate::types::language::SupportedLanguage;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::MahalaxmiResult;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// A fingerprint of a source file used for change detection.
///
/// Stores the content hash (SHA-256), file size, last modification time,
/// and detected programming language. Used by the indexing system to
/// determine which files need re-indexing during incremental updates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileFingerprint {
    /// The path to the source file.
    pub file_path: PathBuf,
    /// The SHA-256 hex digest of the file contents.
    pub content_hash: String,
    /// The file size in bytes.
    pub size_bytes: u64,
    /// The last modification time of the file.
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// The detected programming language, if recognized.
    pub language: Option<SupportedLanguage>,
}

impl FileFingerprint {
    /// Compute a fingerprint for the file at the given path.
    ///
    /// Reads the file content, computes a SHA-256 hash, detects the
    /// language from the file extension, and captures the file metadata.
    ///
    /// # Errors
    ///
    /// Returns `MahalaxmiError::Indexing` if the file cannot be read or
    /// its metadata cannot be obtained.
    pub fn compute(path: &Path, i18n: &I18nService) -> MahalaxmiResult<Self> {
        let path_str = path.display().to_string();

        let content = std::fs::read(path).map_err(|e| {
            MahalaxmiError::indexing(
                i18n,
                keys::indexing::FINGERPRINT_FAILED,
                &[("file", &path_str), ("reason", &e.to_string())],
            )
        })?;

        let metadata = std::fs::metadata(path).map_err(|e| {
            MahalaxmiError::indexing(
                i18n,
                keys::indexing::FINGERPRINT_FAILED,
                &[("file", &path_str), ("reason", &e.to_string())],
            )
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash_bytes = hasher.finalize();
        let content_hash = hex_encode(&hash_bytes);

        let size_bytes = metadata.len();

        let last_modified = metadata
            .modified()
            .map(chrono::DateTime::<chrono::Utc>::from)
            .unwrap_or_else(|_| chrono::Utc::now());

        let language = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext_str| {
                let dotted = format!(".{ext_str}");
                SupportedLanguage::from_extension(&dotted)
            });

        Ok(Self {
            file_path: path.to_path_buf(),
            content_hash,
            size_bytes,
            last_modified,
            language,
        })
    }

    /// Check if this fingerprint's hash matches the given hash string.
    pub fn matches_hash(&self, other_hash: &str) -> bool {
        self.content_hash == other_hash
    }

    /// Check if the file on disk has changed since this fingerprint was taken.
    ///
    /// Compares the current file size and modification time against the
    /// fingerprint's stored values. If the file metadata cannot be read,
    /// the file is assumed to be stale (returns `true`).
    pub fn is_stale(&self, path: &Path) -> bool {
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return true,
        };

        if metadata.len() != self.size_bytes {
            return true;
        }

        match metadata.modified() {
            Ok(modified) => {
                let disk_time = chrono::DateTime::<chrono::Utc>::from(modified);
                disk_time != self.last_modified
            }
            Err(_) => true,
        }
    }
}

/// Encode bytes as a lowercase hexadecimal string.
fn hex_encode(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahalaxmi_core::i18n::locale::SupportedLocale;
    use std::io::Write;

    fn test_i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    #[test]
    fn compute_fingerprint_for_rust_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.rs");
        {
            let mut f = std::fs::File::create(&file_path).expect("create file");
            f.write_all(b"fn main() {}").expect("write");
        }

        let i18n = test_i18n();
        let fp = FileFingerprint::compute(&file_path, &i18n).expect("compute fingerprint");

        assert_eq!(fp.file_path, file_path);
        assert!(!fp.content_hash.is_empty());
        assert_eq!(fp.content_hash.len(), 64); // SHA-256 hex = 64 chars
        assert_eq!(fp.size_bytes, 12); // "fn main() {}" = 12 bytes
        assert_eq!(fp.language, Some(SupportedLanguage::Rust));
    }

    #[test]
    fn compute_fingerprint_for_unknown_extension() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("readme.txt");
        {
            let mut f = std::fs::File::create(&file_path).expect("create file");
            f.write_all(b"hello").expect("write");
        }

        let i18n = test_i18n();
        let fp = FileFingerprint::compute(&file_path, &i18n).expect("compute fingerprint");

        assert!(fp.language.is_none());
    }

    #[test]
    fn compute_fingerprint_nonexistent_file_returns_error() {
        let i18n = test_i18n();
        let result = FileFingerprint::compute(Path::new("/nonexistent/file.rs"), &i18n);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_indexing());
    }

    #[test]
    fn matches_hash_positive() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.py");
        {
            let mut f = std::fs::File::create(&file_path).expect("create file");
            f.write_all(b"print('hello')").expect("write");
        }

        let i18n = test_i18n();
        let fp = FileFingerprint::compute(&file_path, &i18n).expect("compute fingerprint");
        let hash = fp.content_hash.clone();
        assert!(fp.matches_hash(&hash));
    }

    #[test]
    fn matches_hash_negative() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.go");
        {
            let mut f = std::fs::File::create(&file_path).expect("create file");
            f.write_all(b"package main").expect("write");
        }

        let i18n = test_i18n();
        let fp = FileFingerprint::compute(&file_path, &i18n).expect("compute fingerprint");
        assert!(
            !fp.matches_hash("0000000000000000000000000000000000000000000000000000000000000000")
        );
    }

    #[test]
    fn is_stale_unchanged_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.rs");
        {
            let mut f = std::fs::File::create(&file_path).expect("create file");
            f.write_all(b"fn main() {}").expect("write");
        }

        let i18n = test_i18n();
        let fp = FileFingerprint::compute(&file_path, &i18n).expect("compute fingerprint");
        assert!(!fp.is_stale(&file_path));
    }

    #[test]
    fn is_stale_missing_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.rs");
        {
            let mut f = std::fs::File::create(&file_path).expect("create file");
            f.write_all(b"fn main() {}").expect("write");
        }

        let i18n = test_i18n();
        let fp = FileFingerprint::compute(&file_path, &i18n).expect("compute fingerprint");

        std::fs::remove_file(&file_path).expect("remove file");
        assert!(fp.is_stale(&file_path));
    }

    #[test]
    fn is_stale_modified_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.rs");
        {
            let mut f = std::fs::File::create(&file_path).expect("create file");
            f.write_all(b"fn main() {}").expect("write");
        }

        let i18n = test_i18n();
        let fp = FileFingerprint::compute(&file_path, &i18n).expect("compute fingerprint");

        // Modify the file to change its size (and likely its mtime).
        {
            let mut f = std::fs::File::create(&file_path).expect("rewrite file");
            f.write_all(b"fn main() { println!(\"hello\"); }")
                .expect("write");
        }

        assert!(fp.is_stale(&file_path));
    }

    #[test]
    fn same_content_produces_same_hash() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("b.rs");
        let content = b"fn foo() -> i32 { 42 }";

        {
            let mut f = std::fs::File::create(&file_a).expect("create a");
            f.write_all(content).expect("write a");
        }
        {
            let mut f = std::fs::File::create(&file_b).expect("create b");
            f.write_all(content).expect("write b");
        }

        let i18n = test_i18n();
        let fp_a = FileFingerprint::compute(&file_a, &i18n).expect("compute a");
        let fp_b = FileFingerprint::compute(&file_b, &i18n).expect("compute b");

        assert_eq!(fp_a.content_hash, fp_b.content_hash);
    }

    #[test]
    fn different_content_produces_different_hash() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("b.rs");

        {
            let mut f = std::fs::File::create(&file_a).expect("create a");
            f.write_all(b"fn foo() {}").expect("write a");
        }
        {
            let mut f = std::fs::File::create(&file_b).expect("create b");
            f.write_all(b"fn bar() {}").expect("write b");
        }

        let i18n = test_i18n();
        let fp_a = FileFingerprint::compute(&file_a, &i18n).expect("compute a");
        let fp_b = FileFingerprint::compute(&file_b, &i18n).expect("compute b");

        assert_ne!(fp_a.content_hash, fp_b.content_hash);
    }

    #[test]
    fn fingerprint_serde_roundtrip() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.ts");
        {
            let mut f = std::fs::File::create(&file_path).expect("create file");
            f.write_all(b"export function greet() {}").expect("write");
        }

        let i18n = test_i18n();
        let fp = FileFingerprint::compute(&file_path, &i18n).expect("compute fingerprint");

        let json = serde_json::to_string(&fp).expect("serialize");
        let deserialized: FileFingerprint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(fp, deserialized);
    }

    #[test]
    fn hex_encode_correctness() {
        let bytes: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];
        assert_eq!(super::hex_encode(&bytes), "deadbeef");
    }
}

//! Language identification and tree-sitter grammar integration.
//!
//! Maps file extensions to programming languages and provides access
//! to the corresponding tree-sitter grammars for AST parsing.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A programming language supported by the indexing system.
///
/// Each variant maps to a tree-sitter grammar crate that provides
/// the parsing infrastructure for that language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportedLanguage {
    /// Rust (.rs)
    Rust,
    /// TypeScript (.ts, .tsx)
    TypeScript,
    /// JavaScript (.js, .jsx)
    JavaScript,
    /// Python (.py)
    Python,
    /// Go (.go)
    Go,
    /// Java (.java)
    Java,
    /// C (.c, .h)
    C,
    /// C++ (.cpp, .cc, .cxx, .hpp)
    Cpp,
}

impl SupportedLanguage {
    /// Detect the language from a file extension.
    ///
    /// The extension should include the leading dot (e.g., `.rs`, `.ts`).
    /// Returns `None` if the extension is not recognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mahalaxmi_indexing::types::SupportedLanguage;
    ///
    /// assert_eq!(SupportedLanguage::from_extension(".rs"), Some(SupportedLanguage::Rust));
    /// assert_eq!(SupportedLanguage::from_extension(".ts"), Some(SupportedLanguage::TypeScript));
    /// assert_eq!(SupportedLanguage::from_extension(".txt"), None);
    /// ```
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            ".rs" => Some(Self::Rust),
            ".ts" => Some(Self::TypeScript),
            ".tsx" => Some(Self::TypeScript),
            ".js" => Some(Self::JavaScript),
            ".jsx" => Some(Self::JavaScript),
            ".py" => Some(Self::Python),
            ".go" => Some(Self::Go),
            ".java" => Some(Self::Java),
            ".c" | ".h" => Some(Self::C),
            ".cpp" | ".cc" | ".cxx" | ".hpp" => Some(Self::Cpp),
            _ => None,
        }
    }

    /// Get the tree-sitter [`Language`](tree_sitter::Language) for this language.
    ///
    /// Each language variant maps to its corresponding tree-sitter grammar crate.
    /// For JavaScript, the TSX grammar is used since it is a superset that handles JSX syntax.
    pub fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            Self::Rust => tree_sitter::Language::new(tree_sitter_rust::LANGUAGE),
            Self::TypeScript => {
                tree_sitter::Language::new(tree_sitter_typescript::LANGUAGE_TYPESCRIPT)
            }
            Self::JavaScript => tree_sitter::Language::new(tree_sitter_typescript::LANGUAGE_TSX),
            Self::Python => tree_sitter::Language::new(tree_sitter_python::LANGUAGE),
            Self::Go => tree_sitter::Language::new(tree_sitter_go::LANGUAGE),
            Self::Java => tree_sitter::Language::new(tree_sitter_java::LANGUAGE),
            Self::C => tree_sitter::Language::new(tree_sitter_c::LANGUAGE),
            Self::Cpp => tree_sitter::Language::new(tree_sitter_cpp::LANGUAGE),
        }
    }

    /// Returns the language name as a static string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::Java => "java",
            Self::C => "c",
            Self::Cpp => "cpp",
        }
    }

    /// Returns all file extensions recognized for this language.
    ///
    /// Each extension includes the leading dot.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Rust => &[".rs"],
            Self::TypeScript => &[".ts", ".tsx"],
            Self::JavaScript => &[".js", ".jsx"],
            Self::Python => &[".py"],
            Self::Go => &[".go"],
            Self::Java => &[".java"],
            Self::C => &[".c", ".h"],
            Self::Cpp => &[".cpp", ".cc", ".cxx", ".hpp"],
        }
    }
}

impl fmt::Display for SupportedLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_extension_rust() {
        assert_eq!(
            SupportedLanguage::from_extension(".rs"),
            Some(SupportedLanguage::Rust)
        );
    }

    #[test]
    fn from_extension_typescript() {
        assert_eq!(
            SupportedLanguage::from_extension(".ts"),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension(".tsx"),
            Some(SupportedLanguage::TypeScript)
        );
    }

    #[test]
    fn from_extension_javascript() {
        assert_eq!(
            SupportedLanguage::from_extension(".js"),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension(".jsx"),
            Some(SupportedLanguage::JavaScript)
        );
    }

    #[test]
    fn from_extension_python() {
        assert_eq!(
            SupportedLanguage::from_extension(".py"),
            Some(SupportedLanguage::Python)
        );
    }

    #[test]
    fn from_extension_go() {
        assert_eq!(
            SupportedLanguage::from_extension(".go"),
            Some(SupportedLanguage::Go)
        );
    }

    #[test]
    fn from_extension_java() {
        assert_eq!(
            SupportedLanguage::from_extension(".java"),
            Some(SupportedLanguage::Java)
        );
    }

    #[test]
    fn from_extension_c() {
        assert_eq!(
            SupportedLanguage::from_extension(".c"),
            Some(SupportedLanguage::C)
        );
        assert_eq!(
            SupportedLanguage::from_extension(".h"),
            Some(SupportedLanguage::C)
        );
    }

    #[test]
    fn from_extension_cpp() {
        assert_eq!(
            SupportedLanguage::from_extension(".cpp"),
            Some(SupportedLanguage::Cpp)
        );
        assert_eq!(
            SupportedLanguage::from_extension(".cc"),
            Some(SupportedLanguage::Cpp)
        );
        assert_eq!(
            SupportedLanguage::from_extension(".cxx"),
            Some(SupportedLanguage::Cpp)
        );
        assert_eq!(
            SupportedLanguage::from_extension(".hpp"),
            Some(SupportedLanguage::Cpp)
        );
    }

    #[test]
    fn from_extension_unknown() {
        assert_eq!(SupportedLanguage::from_extension(".txt"), None);
        assert_eq!(SupportedLanguage::from_extension(".md"), None);
        assert_eq!(SupportedLanguage::from_extension(".toml"), None);
        assert_eq!(SupportedLanguage::from_extension(""), None);
    }

    #[test]
    fn as_str_returns_lowercase() {
        assert_eq!(SupportedLanguage::Rust.as_str(), "rust");
        assert_eq!(SupportedLanguage::TypeScript.as_str(), "typescript");
        assert_eq!(SupportedLanguage::JavaScript.as_str(), "javascript");
        assert_eq!(SupportedLanguage::Python.as_str(), "python");
        assert_eq!(SupportedLanguage::Go.as_str(), "go");
        assert_eq!(SupportedLanguage::Java.as_str(), "java");
        assert_eq!(SupportedLanguage::C.as_str(), "c");
        assert_eq!(SupportedLanguage::Cpp.as_str(), "cpp");
    }

    #[test]
    fn display_matches_as_str() {
        for lang in [
            SupportedLanguage::Rust,
            SupportedLanguage::TypeScript,
            SupportedLanguage::JavaScript,
            SupportedLanguage::Python,
            SupportedLanguage::Go,
            SupportedLanguage::Java,
            SupportedLanguage::C,
            SupportedLanguage::Cpp,
        ] {
            assert_eq!(format!("{lang}"), lang.as_str());
        }
    }

    #[test]
    fn extensions_include_all_recognized() {
        assert_eq!(SupportedLanguage::Rust.extensions(), &[".rs"]);
        assert_eq!(SupportedLanguage::TypeScript.extensions(), &[".ts", ".tsx"]);
        assert_eq!(SupportedLanguage::JavaScript.extensions(), &[".js", ".jsx"]);
        assert_eq!(SupportedLanguage::Python.extensions(), &[".py"]);
        assert_eq!(SupportedLanguage::Go.extensions(), &[".go"]);
        assert_eq!(SupportedLanguage::Java.extensions(), &[".java"]);
        assert_eq!(SupportedLanguage::C.extensions(), &[".c", ".h"]);
        assert_eq!(
            SupportedLanguage::Cpp.extensions(),
            &[".cpp", ".cc", ".cxx", ".hpp"]
        );
    }

    #[test]
    fn extensions_roundtrip_through_from_extension() {
        for lang in [
            SupportedLanguage::Rust,
            SupportedLanguage::TypeScript,
            SupportedLanguage::JavaScript,
            SupportedLanguage::Python,
            SupportedLanguage::Go,
            SupportedLanguage::Java,
            SupportedLanguage::C,
            SupportedLanguage::Cpp,
        ] {
            for ext in lang.extensions() {
                let detected = SupportedLanguage::from_extension(ext);
                assert_eq!(
                    detected,
                    Some(lang),
                    "Extension '{}' should map back to {:?}",
                    ext,
                    lang
                );
            }
        }
    }

    #[test]
    fn tree_sitter_language_loads_rust() {
        let lang = SupportedLanguage::Rust.tree_sitter_language();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang)
            .expect("Failed to load Rust grammar");
    }

    #[test]
    fn tree_sitter_language_loads_typescript() {
        let lang = SupportedLanguage::TypeScript.tree_sitter_language();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang)
            .expect("Failed to load TypeScript grammar");
    }

    #[test]
    fn tree_sitter_language_loads_javascript() {
        let lang = SupportedLanguage::JavaScript.tree_sitter_language();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang)
            .expect("Failed to load JavaScript grammar");
    }

    #[test]
    fn tree_sitter_language_loads_python() {
        let lang = SupportedLanguage::Python.tree_sitter_language();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang)
            .expect("Failed to load Python grammar");
    }

    #[test]
    fn tree_sitter_language_loads_go() {
        let lang = SupportedLanguage::Go.tree_sitter_language();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang)
            .expect("Failed to load Go grammar");
    }

    #[test]
    fn tree_sitter_language_loads_java() {
        let lang = SupportedLanguage::Java.tree_sitter_language();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang)
            .expect("Failed to load Java grammar");
    }

    #[test]
    fn tree_sitter_language_loads_c() {
        let lang = SupportedLanguage::C.tree_sitter_language();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang)
            .expect("Failed to load C grammar");
    }

    #[test]
    fn tree_sitter_language_loads_cpp() {
        let lang = SupportedLanguage::Cpp.tree_sitter_language();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang)
            .expect("Failed to load C++ grammar");
    }

    #[test]
    fn serde_roundtrip() {
        for lang in [
            SupportedLanguage::Rust,
            SupportedLanguage::TypeScript,
            SupportedLanguage::JavaScript,
            SupportedLanguage::Python,
            SupportedLanguage::Go,
            SupportedLanguage::Java,
            SupportedLanguage::C,
            SupportedLanguage::Cpp,
        ] {
            let json = serde_json::to_string(&lang).expect("serialize");
            let deserialized: SupportedLanguage = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(lang, deserialized);
        }
    }
}

//! Task type classification for multi-provider routing.
//!
//! Classifies worker tasks into `TaskType` categories using a hybrid approach:
//! keyword matching takes priority, then AI-suggested type, then default fallback.

use mahalaxmi_providers::TaskType;

/// Keyword groups mapped to task types, ordered by specificity.
const TESTING_KEYWORDS: &[&str] = &[
    "test",
    "spec",
    "coverage",
    "unittest",
    "e2e",
    "integration test",
];
const CODE_REVIEW_KEYWORDS: &[&str] =
    &["review", "audit", "validate", "code review", "peer review"];
const DOCUMENTATION_KEYWORDS: &[&str] = &[
    "document",
    "readme",
    "docs",
    "docstring",
    "jsdoc",
    "rustdoc",
];
const PLANNING_KEYWORDS: &[&str] = &[
    "architecture",
    "design",
    "plan",
    "rfc",
    "proposal",
    "blueprint",
];
const REFACTORING_KEYWORDS: &[&str] = &[
    "refactor",
    "restructure",
    "reorganize",
    "clean up",
    "simplify",
];
const DEBUGGING_KEYWORDS: &[&str] = &[
    "debug", "fix", "bug", "issue", "hotfix", "patch", "diagnose",
];
const CODE_GEN_KEYWORDS: &[&str] = &[
    "implement",
    "create",
    "build",
    "add",
    "develop",
    "scaffold",
    "generate",
];

/// Classify a task based on its title, description, and optional AI suggestion.
///
/// Priority order:
/// 1. Keyword matches in title/description (most specific category wins)
/// 2. AI-suggested task type string (if parseable)
/// 3. Fallback to `TaskType::General`
pub fn classify_task(title: &str, description: &str, ai_suggestion: Option<&str>) -> TaskType {
    let text = format!("{} {}", title, description).to_lowercase();

    // Keyword matching — check most specific categories first
    if contains_any(&text, TESTING_KEYWORDS) {
        return TaskType::Testing;
    }
    if contains_any(&text, CODE_REVIEW_KEYWORDS) {
        return TaskType::CodeReview;
    }
    if contains_any(&text, DOCUMENTATION_KEYWORDS) {
        return TaskType::Documentation;
    }
    if contains_any(&text, PLANNING_KEYWORDS) {
        return TaskType::Planning;
    }
    if contains_any(&text, REFACTORING_KEYWORDS) {
        return TaskType::Refactoring;
    }
    if contains_any(&text, DEBUGGING_KEYWORDS) {
        return TaskType::Debugging;
    }
    if contains_any(&text, CODE_GEN_KEYWORDS) {
        return TaskType::CodeGeneration;
    }

    // AI suggestion fallback
    if let Some(suggestion) = ai_suggestion {
        if let Some(task_type) = parse_task_type(suggestion) {
            return task_type;
        }
    }

    TaskType::General
}

/// Parse a task type string (as might come from AI JSON output) into a `TaskType`.
pub fn parse_task_type(s: &str) -> Option<TaskType> {
    match s.trim().to_lowercase().replace('-', "_").as_str() {
        "code_generation" | "codegeneration" | "implementation" => Some(TaskType::CodeGeneration),
        "code_review" | "codereview" | "review" => Some(TaskType::CodeReview),
        "debugging" | "debug" | "bugfix" => Some(TaskType::Debugging),
        "refactoring" | "refactor" => Some(TaskType::Refactoring),
        "testing" | "test" | "tests" => Some(TaskType::Testing),
        "documentation" | "docs" => Some(TaskType::Documentation),
        "planning" | "design" | "architecture" => Some(TaskType::Planning),
        "general" => Some(TaskType::General),
        _ => None,
    }
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| text.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_testing_keywords() {
        assert_eq!(
            classify_task("Write unit tests", "Add coverage", None),
            TaskType::Testing
        );
        assert_eq!(
            classify_task("Create spec files", "", None),
            TaskType::Testing
        );
    }

    #[test]
    fn classify_code_review_keywords() {
        assert_eq!(
            classify_task("Code review the PR", "", None),
            TaskType::CodeReview
        );
        assert_eq!(
            classify_task("Audit security", "Validate inputs", None),
            TaskType::CodeReview
        );
    }

    #[test]
    fn classify_documentation_keywords() {
        assert_eq!(
            classify_task("Update README", "", None),
            TaskType::Documentation
        );
        assert_eq!(
            classify_task("Write docs", "Add docstring to API", None),
            TaskType::Documentation
        );
    }

    #[test]
    fn classify_planning_keywords() {
        assert_eq!(
            classify_task("Design architecture", "", None),
            TaskType::Planning
        );
        assert_eq!(
            classify_task("Write RFC", "Create a proposal", None),
            TaskType::Planning
        );
    }

    #[test]
    fn classify_refactoring_keywords() {
        assert_eq!(
            classify_task("Refactor auth module", "", None),
            TaskType::Refactoring
        );
        assert_eq!(
            classify_task("Restructure code", "", None),
            TaskType::Refactoring
        );
    }

    #[test]
    fn classify_debugging_keywords() {
        assert_eq!(
            classify_task("Fix null pointer bug", "", None),
            TaskType::Debugging
        );
        assert_eq!(
            classify_task("Debug crash", "Diagnose issue", None),
            TaskType::Debugging
        );
    }

    #[test]
    fn classify_code_generation_keywords() {
        assert_eq!(
            classify_task("Implement auth system", "", None),
            TaskType::CodeGeneration
        );
        assert_eq!(
            classify_task("Create user API", "Build endpoints", None),
            TaskType::CodeGeneration
        );
    }

    #[test]
    fn classify_ai_suggestion_fallback() {
        assert_eq!(
            classify_task("Do something", "generic task", Some("code_review")),
            TaskType::CodeReview
        );
        assert_eq!(
            classify_task("Do something", "generic task", Some("testing")),
            TaskType::Testing
        );
    }

    #[test]
    fn classify_keyword_overrides_ai_suggestion() {
        // Keywords take priority over AI suggestion
        assert_eq!(
            classify_task("Fix the bug", "", Some("code_generation")),
            TaskType::Debugging
        );
    }

    #[test]
    fn classify_invalid_ai_suggestion_falls_to_general() {
        assert_eq!(
            classify_task("Do something", "misc", Some("not_a_type")),
            TaskType::General
        );
    }

    #[test]
    fn classify_empty_inputs_returns_general() {
        assert_eq!(classify_task("", "", None), TaskType::General);
    }

    #[test]
    fn parse_task_type_variants() {
        assert_eq!(
            parse_task_type("code_generation"),
            Some(TaskType::CodeGeneration)
        );
        assert_eq!(parse_task_type("CodeReview"), Some(TaskType::CodeReview));
        assert_eq!(parse_task_type("  debugging  "), Some(TaskType::Debugging));
        assert_eq!(parse_task_type("code-review"), Some(TaskType::CodeReview));
        assert_eq!(parse_task_type("unknown"), None);
    }
}

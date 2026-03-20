//! Integration tests — security domain configuration and end-to-end report shape.
//!
//! These tests codify the three acceptance criteria from the security domain
//! requirements:
//!
//! 1. Manager prompt contains a RoleBased decomposition hint listing all 4 specialist
//!    roles, and the hint appears before the quality mandate's HARD CONSTRAINTS block.
//! 2. The security domain YAML configures exactly 4 roles in the expected order, and
//!    the output format is StructuredReport targeting security-report.md.
//! 3. `format_cycle_output` generates security-report.md containing all 5 severity
//!    sections with real worker content and no placeholder text.

use std::path::PathBuf;
use std::sync::Arc;

use mahalaxmi_core::config::ContextFormat;
use mahalaxmi_core::domain::{DecompositionStrategy, DomainConfig, LoadedDomain, OutputFormat};
use mahalaxmi_orchestration::output_format::format_cycle_output;
use mahalaxmi_orchestration::prompt::builder::{ManagerPromptBuilder, ManagerPromptConfig};

/// Absolute path to `terminalAutomation/domains/security/`.
fn security_domain_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..") // crates/mahalaxmi-orchestration → crates/
        .join("..") // crates/ → terminalAutomation/
        .join("domains")
        .join("security")
}


/// Test 1 — Manager prompt contains RoleBased hint listing 4 specialist roles.
///
/// Loads the actual `domains/security/config.yaml` via `LoadedDomain`, builds the
/// manager prompt with that domain, and asserts:
/// - All 4 role names are present.
/// - The prompt contains "RoleBased" or "exactly 4 tasks" from the decomposition hint.
/// - The decomposition hint appears before the output format specification.
///
/// Note on ordering: the builder injects the decomposition hint immediately after the
/// system role, before any constraint sections and before the output format. When a
/// domain is provided, it overrides the quality mandate with domain constraint sections,
/// so the sentinel used for ordering is the output format header (always the last
/// logical section in the prompt).
#[test]
fn security_domain_manager_prompt_contains_role_based_hint() {
    let domain_dir = security_domain_dir();
    let domain = Arc::new(
        LoadedDomain::load(&domain_dir)
            .expect("failed to load security domain for prompt test"),
    ) as Arc<dyn DomainConfig>;

    let config = ManagerPromptConfig {
        requirements: "Perform a comprehensive security analysis of this codebase.".to_owned(),
        repo_map: "src/\n  main.rs\n  auth.rs".to_owned(),
        shared_memory: String::new(),
        // Non-Claude provider → Markdown format; avoids XML tag noise in substring assertions.
        provider_id: "openai-foundry".to_owned(),
        worker_count: 4,
        format: ContextFormat::Markdown,
        include_quality_mandate: true,
        previous_cycle_report: None,
        previous_validation_verdict: None,
        domain: Some(domain),
    };

    let prompt = ManagerPromptBuilder::build(&config);

    // All 4 specialist role names must be present in the prompt.
    assert!(
        prompt.contains("vulnerability_analyst"),
        "prompt must contain 'vulnerability_analyst'"
    );
    assert!(
        prompt.contains("dependency_auditor"),
        "prompt must contain 'dependency_auditor'"
    );
    assert!(
        prompt.contains("secrets_scanner"),
        "prompt must contain 'secrets_scanner'"
    );
    assert!(
        prompt.contains("authentication_reviewer"),
        "prompt must contain 'authentication_reviewer'"
    );

    // The decomposition hint must signal RoleBased strategy and exactly 4 tasks.
    assert!(
        prompt.contains("RoleBased") || prompt.contains("exactly 4 tasks"),
        "prompt must contain 'RoleBased' or 'exactly 4 tasks' from the decomposition hint"
    );

    // The decomposition hint must appear before the output format specification.
    // The builder injects the hint immediately after the system role — before context
    // sections and before the output format (which is always the last section).
    let hint_pos = prompt
        .find("RoleBased")
        .or_else(|| prompt.find("exactly 4 tasks"))
        .expect("decomposition hint text ('RoleBased' or 'exactly 4 tasks') not found in prompt");
    // Markdown output format header is "## Output Format".
    let output_format_pos = prompt
        .find("## Output Format")
        .or_else(|| prompt.find("output_format"))
        .expect("output format section not found in prompt");
    assert!(
        hint_pos < output_format_pos,
        "decomposition hint (pos {hint_pos}) must appear before the output format section (pos {output_format_pos})"
    );
}

/// Test 2 — Security domain config has exactly 4 roles.
///
/// Loads the actual `domains/security/config.yaml` via `LoadedDomain::load` and
/// verifies the decomposition strategy and output format match the acceptance
/// criteria exactly.
#[test]
fn security_domain_config_has_exactly_4_roles() {
    let domain_dir = security_domain_dir();
    let domain = LoadedDomain::load(&domain_dir)
        .expect("failed to load security domain — ensure domains/security/config.yaml exists");

    // Decomposition strategy must be RoleBased with the 4 specialist roles in order.
    match domain.decomposition_strategy() {
        DecompositionStrategy::RoleBased { roles } => {
            assert_eq!(
                roles.len(),
                4,
                "security domain must have exactly 4 roles, found {}: {:?}",
                roles.len(),
                roles
            );
            assert_eq!(
                roles,
                vec![
                    "vulnerability_analyst",
                    "dependency_auditor",
                    "secrets_scanner",
                    "authentication_reviewer",
                ],
                "roles must match the expected identifiers in the specified order"
            );
        }
        other => panic!(
            "expected DecompositionStrategy::RoleBased, got {:?}",
            other
        ),
    }

    // Output format must be StructuredReport targeting security-report.md.
    match domain.output_format() {
        OutputFormat::StructuredReport { sections, output_file } => {
            assert_eq!(
                output_file, "security-report.md",
                "output_file must be 'security-report.md'"
            );
            assert!(
                !sections.is_empty(),
                "StructuredReport sections must not be empty"
            );
            assert_eq!(
                sections[0], "Executive Summary",
                "first section must be 'Executive Summary'"
            );
        }
        other => panic!(
            "expected OutputFormat::StructuredReport, got {:?}",
            other
        ),
    }
}

/// Test 3 — StructuredReport produces security-report.md with correct shape.
///
/// Calls `format_cycle_output` with mock worker outputs covering all 5 severity
/// sections and verifies the generated file:
/// - Begins with a markdown H1 report header.
/// - Contains all 5 section headings.
/// - Contains real worker content in every section.
/// - Contains no "No findings in this category." placeholder text.
#[test]
fn security_domain_structured_report_produces_security_report_md() {
    let dir = tempfile::TempDir::new().expect("create tempdir for output");

    let format = OutputFormat::StructuredReport {
        sections: vec![
            "Executive Summary".to_owned(),
            "Critical Findings".to_owned(),
            "High Findings".to_owned(),
            "Medium Findings".to_owned(),
            "Low Findings".to_owned(),
        ],
        output_file: "security-report.md".to_owned(),
    };

    // Each worker output covers one or more sections by including the section heading.
    let worker_outputs = vec![
        "## Executive Summary\nNo critical issues found.\n## Critical Findings\nCVE-1234.\n"
            .to_owned(),
        "## High Findings\nInsecure deserialization.\n".to_owned(),
        "## Medium Findings\nMissing rate limiting.\n".to_owned(),
        "## Low Findings\nVerbose error messages.\n".to_owned(),
    ];

    let result = format_cycle_output(&format, &worker_outputs, dir.path());
    assert!(
        result.is_ok(),
        "format_cycle_output must not error: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap().as_deref(),
        Some("security-report.md"),
        "format_cycle_output must return the relative path 'security-report.md'"
    );

    let content = std::fs::read_to_string(dir.path().join("security-report.md"))
        .expect("security-report.md must have been written to the worktree path");

    // File must open with a markdown H1 report header.
    assert!(
        content.starts_with("# "),
        "security-report.md must begin with a markdown H1 header, first line was: {:?}",
        content.lines().next()
    );

    // All 5 configured section headings must be present.
    assert!(
        content.contains("## Executive Summary"),
        "report must contain '## Executive Summary'"
    );
    assert!(
        content.contains("## Critical Findings"),
        "report must contain '## Critical Findings'"
    );
    assert!(
        content.contains("## High Findings"),
        "report must contain '## High Findings'"
    );
    assert!(
        content.contains("## Medium Findings"),
        "report must contain '## Medium Findings'"
    );
    assert!(
        content.contains("## Low Findings"),
        "report must contain '## Low Findings'"
    );

    // Every section must contain real worker output content.
    assert!(
        content.contains("No critical issues found."),
        "Executive Summary section must contain worker content"
    );
    assert!(
        content.contains("CVE-1234."),
        "Critical Findings section must contain worker content"
    );
    assert!(
        content.contains("Insecure deserialization."),
        "High Findings section must contain worker content"
    );
    assert!(
        content.contains("Missing rate limiting."),
        "Medium Findings section must contain worker content"
    );
    assert!(
        content.contains("Verbose error messages."),
        "Low Findings section must contain worker content"
    );

    // No placeholder text must appear for any of the 5 configured sections.
    assert!(
        !content.contains("No findings in this category."),
        "report must not contain placeholder text — all 5 sections have real worker content"
    );
}

/// Verify `domains/security/config.yaml` contains the required field identifiers.
///
/// This smoke test confirms the YAML source file itself is intact and contains
/// the expected role identifiers and output filename. Complementary to the
/// `LoadedDomain`-based tests above.
#[test]
fn security_domain_yaml_file_contains_required_fields() {
    let yaml_path = security_domain_dir().join("config.yaml");
    let yaml = std::fs::read_to_string(&yaml_path)
        .expect("domains/security/config.yaml must be readable");

    assert!(
        yaml.contains("role_based"),
        "config.yaml must contain 'role_based'"
    );
    assert!(
        yaml.contains("vulnerability_analyst"),
        "config.yaml must contain 'vulnerability_analyst'"
    );
    assert!(
        yaml.contains("dependency_auditor"),
        "config.yaml must contain 'dependency_auditor'"
    );
    assert!(
        yaml.contains("secrets_scanner"),
        "config.yaml must contain 'secrets_scanner'"
    );
    assert!(
        yaml.contains("authentication_reviewer"),
        "config.yaml must contain 'authentication_reviewer'"
    );
    assert!(
        yaml.contains("security-report.md"),
        "config.yaml must contain 'security-report.md'"
    );
}

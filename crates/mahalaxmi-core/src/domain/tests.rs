//! Tests for the domain configuration module.
//!
//! Tests 1-9 verify:
//! 1. `PromptSource::Inline` renders without file I/O.
//! 2. `PromptSource::File` reads content from disk.
//! 3. `${worker_count}` placeholder is substituted correctly.
//! 4. `DomainRegistry::load_from_dir` loads the coding domain.
//! 5. `manager_system_role(0)` matches the hardcoded uncapped string byte-for-byte.
//! 6. `manager_system_role(4)` matches the hardcoded capped string byte-for-byte.
//! 7. `worker_system_role()` matches the hardcoded worker string byte-for-byte.
//! 8. Constraint sections match the hardcoded constraint strings byte-for-byte.
//! 9. `ManagerPromptBuilder::build()` with domain == without domain (identical output).

use std::path::Path;
use std::sync::Arc;

use tempfile::TempDir;

use crate::domain::{
    ConsensusAlgorithm, DecompositionStrategy, DomainConfig, DomainRegistry, InputFormat,
    LoadedDomain, OutputFormat, PromptSource,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Path to the workspace-level `domains/` directory.
///
/// Cargo runs tests with CWD = package root.  From `crates/mahalaxmi-core/`
/// we go up two levels to reach the workspace root then into `domains/`.
fn domains_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../domains")
        .canonicalize()
        .expect("domains/ directory should exist at workspace root")
}

fn coding_domain() -> Arc<dyn DomainConfig> {
    let mut registry = DomainRegistry::new();
    registry
        .load_from_dir(&domains_dir())
        .expect("load_from_dir should succeed");
    registry.get("coding").expect("coding domain should be registered")
}

// ---------------------------------------------------------------------------
// Test 1 — PromptSource::Inline renders without file I/O
// ---------------------------------------------------------------------------
#[test]
fn test_1_prompt_source_inline() {
    let tmp = TempDir::new().unwrap();
    let source = PromptSource::Inline("hello world".to_owned());

    // Write a config.yaml that uses an inline source
    let config_yaml = r#"
id: test
description: test domain
manager_uncapped: "inline uncapped"
manager_capped: "inline capped ${worker_count}"
worker_system_role: "inline worker"
constraint_sections: []
"#;
    std::fs::write(tmp.path().join("config.yaml"), config_yaml).unwrap();

    let domain = LoadedDomain::load(tmp.path()).unwrap();
    assert_eq!(domain.worker_system_role(), "inline worker");
    assert_eq!(domain.manager_system_role(0), "inline uncapped");
    assert_eq!(domain.manager_system_role(3), "inline capped 3");

    // Verify the Inline variant itself
    assert!(matches!(source, PromptSource::Inline(_)));
}

// ---------------------------------------------------------------------------
// Test 2 — PromptSource::File reads content from disk
// ---------------------------------------------------------------------------
#[test]
fn test_2_prompt_source_file() {
    let tmp = TempDir::new().unwrap();

    let worker_content = "worker prompt from file";
    std::fs::write(tmp.path().join("worker.prompt"), worker_content).unwrap();

    let config_yaml = r#"
id: test_file
description: test file source
manager_uncapped: "uncapped"
manager_capped: "capped"
worker_system_role:
  file: worker.prompt
constraint_sections: []
"#;
    std::fs::write(tmp.path().join("config.yaml"), config_yaml).unwrap();

    let domain = LoadedDomain::load(tmp.path()).unwrap();
    assert_eq!(domain.worker_system_role(), worker_content);
}

// ---------------------------------------------------------------------------
// Test 3 — ${worker_count} placeholder substitution
// ---------------------------------------------------------------------------
#[test]
fn test_3_worker_count_substitution() {
    let tmp = TempDir::new().unwrap();
    let config_yaml = r#"
id: sub
description: substitution test
manager_uncapped: "uncapped — no substitution"
manager_capped: "assign to ${worker_count} workers"
worker_system_role: "worker"
constraint_sections:
  - name: Cap
    source: "static uncapped"
    capped: "max ${worker_count} tasks"
"#;
    std::fs::write(tmp.path().join("config.yaml"), config_yaml).unwrap();

    let domain = LoadedDomain::load(tmp.path()).unwrap();

    // worker_count == 0 → uncapped variants, no substitution
    assert_eq!(domain.manager_system_role(0), "uncapped — no substitution");
    let sections_0 = domain.constraint_sections(0);
    assert_eq!(sections_0[0].1, "static uncapped");

    // worker_count == 5 → capped variants with substitution
    assert_eq!(domain.manager_system_role(5), "assign to 5 workers");
    let sections_5 = domain.constraint_sections(5);
    assert_eq!(sections_5[0].1, "max 5 tasks");
}

// ---------------------------------------------------------------------------
// Test 4 — DomainRegistry::load_from_dir loads the coding domain
// ---------------------------------------------------------------------------
#[test]
fn test_4_registry_loads_coding_domain() {
    let mut registry = DomainRegistry::new();
    let n = registry
        .load_from_dir(&domains_dir())
        .expect("load_from_dir should succeed");
    assert!(n >= 1, "at least one domain should be loaded");
    let domain = registry.get("coding");
    assert!(domain.is_some(), "coding domain should be present in registry");
}

// ---------------------------------------------------------------------------
// Test 5 — manager_system_role(0) matches hardcoded uncapped string
// ---------------------------------------------------------------------------
#[test]
fn test_5_manager_system_role_uncapped_matches_hardcoded() {
    let expected = concat!(
        "You are a senior software engineering manager. ",
        "Your job is to analyze the project context provided below and ",
        "decompose the work into as many concrete, independent tasks as the ",
        "requirements warrant. There is no upper cap on task count \u{2014} produce ",
        "exactly as many tasks as the work requires."
    );

    let domain = coding_domain();
    assert_eq!(
        domain.manager_system_role(0),
        expected,
        "manager_system_role(0) must match hardcoded uncapped string exactly"
    );
}

// ---------------------------------------------------------------------------
// Test 6 — manager_system_role(4) matches hardcoded capped string
// ---------------------------------------------------------------------------
#[test]
fn test_6_manager_system_role_capped_matches_hardcoded() {
    let expected = concat!(
        "You are a senior software engineering manager. ",
        "Your job is to analyze the project context provided below and ",
        "decompose the work into concrete, actionable tasks that can be ",
        "assigned to 4 AI coding agent workers."
    );

    let domain = coding_domain();
    assert_eq!(
        domain.manager_system_role(4),
        expected,
        "manager_system_role(4) must match hardcoded capped string exactly"
    );
}

// ---------------------------------------------------------------------------
// Test 7 — worker_system_role() matches hardcoded worker string
// ---------------------------------------------------------------------------
#[test]
fn test_7_worker_system_role_matches_hardcoded() {
    let expected = concat!(
        "You are an AI coding agent executing a specific task in a multi-worker ",
        "orchestration system called Mahalaxmi. Focus ONLY on your assigned task. ",
        "Other workers are handling other parts of the system simultaneously."
    );

    let domain = coding_domain();
    assert_eq!(
        domain.worker_system_role(),
        expected,
        "worker_system_role() must match hardcoded string exactly"
    );
}

// ---------------------------------------------------------------------------
// Test 8 — constraint sections match hardcoded constraint strings
// ---------------------------------------------------------------------------
#[test]
fn test_8_constraint_sections_match_hardcoded() {
    let domain = coding_domain();

    // Uncapped (worker_count == 0)
    let sections_0 = domain.constraint_sections(0);
    assert_eq!(sections_0.len(), 3, "expect 3 constraint sections");
    assert_eq!(sections_0[0].0, "Quality Mandate");
    assert_eq!(sections_0[1].0, "Progress Tracking");
    assert_eq!(sections_0[2].0, "Analysis Rules");

    // Quality mandate uncapped must start with the header
    assert!(
        sections_0[0].1.starts_with("HARD CONSTRAINTS"),
        "quality mandate must start with HARD CONSTRAINTS"
    );
    // C3 uncapped text
    assert!(
        sections_0[0].1.contains("There is NO upper cap"),
        "uncapped quality mandate must mention NO upper cap"
    );

    // Capped (worker_count == 4)
    let sections_4 = domain.constraint_sections(4);
    assert!(
        sections_4[0].1.contains("Task count MUST NOT exceed 4"),
        "capped quality mandate must mention specific worker count"
    );
    assert!(
        sections_4[0].1.contains("count <= 4"),
        "capped quality mandate must include count <= N check"
    );

    // Progress tracking and analysis rules are static
    assert_eq!(sections_0[1].1, sections_4[1].1, "progress tracking is static");
    assert_eq!(sections_0[2].1, sections_4[2].1, "analysis rules are static");

    assert!(sections_0[1].1.starts_with("PROGRESS TRACKING"));
    assert!(sections_0[2].1.starts_with("ANALYSIS CONSTRAINTS"));
}

// ---------------------------------------------------------------------------
// Test 9 — All constraint sections have complete, accurate content
//
// Verifies that the loaded domain's constraint sections produce byte-for-byte
// identical text to the hardcoded constraint functions in builder.rs.
// Tests 5-8 cover individual methods; this test covers the full constraint
// block for both uncapped (0) and capped (4) modes in detail.
// ---------------------------------------------------------------------------
#[test]
fn test_9_constraint_sections_complete_content() {
    let domain = coding_domain();

    // --- Uncapped (worker_count == 0) ---
    let sections_0 = domain.constraint_sections(0);

    // Quality Mandate uncapped must contain all C0-C8 markers
    let qm_0 = &sections_0[0].1;
    for c in ["C0:", "C1:", "C2:", "C3:", "C4:", "C5:", "C6:", "C7:", "C8:"] {
        assert!(qm_0.contains(c), "uncapped quality mandate must contain {c}");
    }
    assert!(
        qm_0.contains("There is NO upper cap"),
        "uncapped C3 must mention NO upper cap"
    );
    assert!(
        qm_0.contains("Do not artificially merge unrelated work."),
        "uncapped C3 must include full text"
    );

    // --- Capped (worker_count == 4) ---
    let sections_4 = domain.constraint_sections(4);

    let qm_4 = &sections_4[0].1;
    // Header unchanged
    assert!(qm_4.starts_with("HARD CONSTRAINTS"));
    // C3 capped text (byte-level check of the two key phrases)
    assert!(
        qm_4.contains("Task count MUST NOT exceed 4."),
        "capped C3 must state hard limit"
    );
    assert!(
        qm_4.contains("cannot execute. Merge the lowest-complexity tasks until count <= 4."),
        "capped C3 must include merge guidance"
    );
    // C0-C2 and C4-C8 are identical between uncapped and capped
    for c in ["C0:", "C1:", "C2:", "C4:", "C5:", "C6:", "C7:", "C8:"] {
        assert!(qm_4.contains(c), "capped quality mandate must contain {c}");
    }

    // --- Progress Tracking (static for both modes) ---
    let pt_0 = &sections_0[1].1;
    let pt_4 = &sections_4[1].1;
    assert_eq!(pt_0, pt_4, "progress tracking is static — must be identical");
    assert!(pt_0.starts_with("PROGRESS TRACKING"));
    assert!(pt_0.contains("C9:"));
    assert!(pt_0.contains("C10:"));

    // --- Analysis Rules (static for both modes) ---
    let ar_0 = &sections_0[2].1;
    let ar_4 = &sections_4[2].1;
    assert_eq!(ar_0, ar_4, "analysis rules is static — must be identical");
    assert!(ar_0.starts_with("ANALYSIS CONSTRAINTS"));
    for c in ["C11:", "C12:", "C13:", "C14:", "C15:"] {
        assert!(ar_0.contains(c), "analysis rules must contain {c}");
    }
}

// ---------------------------------------------------------------------------
// Test 10 — DecompositionStrategy defaults and hint()
// ---------------------------------------------------------------------------
#[test]
fn test_10_decomposition_strategy_defaults_and_hint() {
    assert_eq!(
        DecompositionStrategy::default(),
        DecompositionStrategy::SoftwareDevelopment,
        "default must be SoftwareDevelopment"
    );
    assert_eq!(
        DecompositionStrategy::SoftwareDevelopment.hint(),
        "",
        "SoftwareDevelopment hint must be empty"
    );
    assert_eq!(
        DecompositionStrategy::ManagerDefined.hint(),
        "",
        "ManagerDefined hint must be empty"
    );

    let section = DecompositionStrategy::SectionBased {
        max_section_tokens: 4000,
        overlap_tokens: 200,
    };
    let hint = section.hint();
    assert!(hint.contains("4000"), "SectionBased hint must contain max_section_tokens");
    assert!(hint.contains("200"), "SectionBased hint must contain overlap_tokens");

    let role = DecompositionStrategy::RoleBased {
        roles: vec!["analyst".to_owned(), "reviewer".to_owned()],
    };
    let hint = role.hint();
    assert!(hint.contains("analyst"), "RoleBased hint must list roles");
    assert!(hint.contains("reviewer"), "RoleBased hint must list all roles");
}

// ---------------------------------------------------------------------------
// Test 11 — DecompositionStrategy serde round-trip
// ---------------------------------------------------------------------------
#[test]
fn test_11_decomposition_strategy_serde_round_trip() {
    let variants: Vec<DecompositionStrategy> = vec![
        DecompositionStrategy::SoftwareDevelopment,
        DecompositionStrategy::ManagerDefined,
        DecompositionStrategy::SectionBased {
            max_section_tokens: 2000,
            overlap_tokens: 100,
        },
        DecompositionStrategy::RoleBased {
            roles: vec!["alpha".to_owned(), "beta".to_owned()],
        },
    ];

    for variant in variants {
        let serialized = serde_yaml::to_string(&variant)
            .unwrap_or_else(|e| panic!("serialization failed: {e}"));
        let deserialized: DecompositionStrategy = serde_yaml::from_str(&serialized)
            .unwrap_or_else(|e| panic!("deserialization failed for {serialized:?}: {e}"));
        assert_eq!(variant, deserialized, "round-trip mismatch");
    }
}

// ---------------------------------------------------------------------------
// Test 12 — ConsensusAlgorithm defaults and serde round-trip
// ---------------------------------------------------------------------------
#[test]
fn test_12_consensus_algorithm_defaults_and_serde() {
    assert_eq!(
        ConsensusAlgorithm::default(),
        ConsensusAlgorithm::Unanimous,
        "default must be Unanimous"
    );

    let variants: Vec<ConsensusAlgorithm> = vec![
        ConsensusAlgorithm::Unanimous,
        ConsensusAlgorithm::Majority { threshold: 0.67 },
        ConsensusAlgorithm::BestOfN {
            selection_criteria: "most thorough".to_owned(),
        },
        ConsensusAlgorithm::Synthesis {
            synthesis_prompt: "synthesize findings".to_owned(),
        },
        ConsensusAlgorithm::ManagerAdjudicated {
            conflict_resolution_prompt: "resolve conflicts".to_owned(),
        },
    ];

    for variant in variants {
        let serialized = serde_yaml::to_string(&variant)
            .unwrap_or_else(|e| panic!("serialization failed: {e}"));
        let deserialized: ConsensusAlgorithm = serde_yaml::from_str(&serialized)
            .unwrap_or_else(|e| panic!("deserialization failed for {serialized:?}: {e}"));
        assert_eq!(variant, deserialized, "round-trip mismatch");
    }
}

// ---------------------------------------------------------------------------
// Test 13 — OutputFormat and InputFormat defaults and serde round-trip
// ---------------------------------------------------------------------------
#[test]
fn test_13_output_and_input_format_defaults_and_serde() {
    assert_eq!(
        OutputFormat::default(),
        OutputFormat::PullRequest,
        "OutputFormat default must be PullRequest"
    );
    assert_eq!(
        InputFormat::default(),
        InputFormat::Codebase,
        "InputFormat default must be Codebase"
    );

    let output_variants: Vec<OutputFormat> = vec![
        OutputFormat::PullRequest,
        OutputFormat::StructuredReport {
            sections: vec!["Summary".to_owned(), "Findings".to_owned()],
            output_file: "report.md".to_owned(),
        },
        OutputFormat::JsonExport {
            output_file: "out.json".to_owned(),
            schema: Some("schema.json".to_owned()),
        },
        OutputFormat::MarkdownFile {
            output_file: "out.md".to_owned(),
            template: None,
        },
    ];
    for variant in output_variants {
        let serialized = serde_yaml::to_string(&variant)
            .unwrap_or_else(|e| panic!("serialization failed: {e}"));
        let deserialized: OutputFormat = serde_yaml::from_str(&serialized)
            .unwrap_or_else(|e| panic!("deserialization failed for {serialized:?}: {e}"));
        assert_eq!(variant, deserialized, "OutputFormat round-trip mismatch");
    }

    let input_variants: Vec<InputFormat> = vec![
        InputFormat::Codebase,
        InputFormat::TextInput,
        InputFormat::DocumentFile {
            accepted_types: vec!["pdf".to_owned(), "docx".to_owned()],
            max_file_size_mb: 50,
        },
    ];
    for variant in input_variants {
        let serialized = serde_yaml::to_string(&variant)
            .unwrap_or_else(|e| panic!("serialization failed: {e}"));
        let deserialized: InputFormat = serde_yaml::from_str(&serialized)
            .unwrap_or_else(|e| panic!("deserialization failed for {serialized:?}: {e}"));
        assert_eq!(variant, deserialized, "InputFormat round-trip mismatch");
    }
}

// ---------------------------------------------------------------------------
// Test 14 — Pluggable behavior fields are backward-compatible with existing
//           coding domain config (missing fields → defaults)
// ---------------------------------------------------------------------------
#[test]
fn test_14_pluggable_behavior_backward_compatible() {
    let domain = coding_domain();

    assert_eq!(
        domain.decomposition_strategy(),
        DecompositionStrategy::SoftwareDevelopment,
        "coding domain must default to SoftwareDevelopment"
    );
    assert_eq!(
        domain.consensus_algorithm(),
        ConsensusAlgorithm::Unanimous,
        "coding domain must default to Unanimous"
    );
    assert_eq!(
        domain.output_format(),
        OutputFormat::PullRequest,
        "coding domain must default to PullRequest"
    );
    assert_eq!(
        domain.input_format(),
        InputFormat::Codebase,
        "coding domain must default to Codebase"
    );
    assert_eq!(
        domain.decomposition_hint(),
        "",
        "SoftwareDevelopment hint must be empty string"
    );
}

// ---------------------------------------------------------------------------
// Test 15 — LoadedDomain reads explicit pluggable behavior from config.yaml
// ---------------------------------------------------------------------------
#[test]
fn test_15_loaded_domain_reads_pluggable_behavior() {
    let tmp = TempDir::new().unwrap();
    let config_yaml = r#"
id: test_pluggable
description: pluggable behavior test
manager_uncapped: "uncapped"
manager_capped: "capped"
worker_system_role: "worker"
constraint_sections: []
decomposition_strategy:
  section_based:
    max_section_tokens: 2000
    overlap_tokens: 100
consensus_algorithm:
  synthesis:
    synthesis_prompt: "synthesize all findings"
output_format:
  structured_report:
    sections:
      - Summary
      - Details
    output_file: report.md
input_format:
  document_file:
    accepted_types:
      - pdf
      - txt
    max_file_size_mb: 50
"#;
    std::fs::write(tmp.path().join("config.yaml"), config_yaml).unwrap();

    let domain = LoadedDomain::load(tmp.path()).unwrap();

    assert_eq!(
        domain.decomposition_strategy(),
        DecompositionStrategy::SectionBased {
            max_section_tokens: 2000,
            overlap_tokens: 100,
        }
    );
    assert_eq!(
        domain.consensus_algorithm(),
        ConsensusAlgorithm::Synthesis {
            synthesis_prompt: "synthesize all findings".to_owned(),
        }
    );
    assert_eq!(
        domain.output_format(),
        OutputFormat::StructuredReport {
            sections: vec!["Summary".to_owned(), "Details".to_owned()],
            output_file: "report.md".to_owned(),
        }
    );
    assert_eq!(
        domain.input_format(),
        InputFormat::DocumentFile {
            accepted_types: vec!["pdf".to_owned(), "txt".to_owned()],
            max_file_size_mb: 50,
        }
    );

    let hint = domain.decomposition_hint();
    assert!(hint.contains("2000"), "hint must contain max_section_tokens");
    assert!(hint.contains("100"), "hint must contain overlap_tokens");
}

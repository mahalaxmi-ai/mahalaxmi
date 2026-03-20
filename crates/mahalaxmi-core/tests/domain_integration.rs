//! Phase 2 integration tests for pluggable domain behavior.
//!
//! These tests verify:
//! 1. `DomainBehavior` deserializes `DecompositionStrategy::SectionBased` and
//!    `ConsensusAlgorithm::Synthesis` from a YAML string correctly.
//! 2. The actual coding domain's `LoadedDomain` returns the correct default
//!    values for all four pluggable behavior fields.

use std::path::PathBuf;

use mahalaxmi_core::domain::{
    ConsensusAlgorithm, DecompositionStrategy, DomainConfig, InputFormat, LoadedDomain,
    OutputFormat,
};

// ---------------------------------------------------------------------------
// test_domain_config_drives_decomposition_and_consensus
//
// Parses a YAML domain config with explicit SectionBased decomposition and
// Synthesis consensus, then asserts the deserialized values and hint text.
// ---------------------------------------------------------------------------
#[test]
fn test_domain_config_drives_decomposition_and_consensus() {
    use std::io::Write as _;

    let tmp = tempfile::TempDir::new()
        .expect("creating temp dir must succeed");

    let config_yaml = r#"
id: integration_test
description: "Integration test domain"
manager_uncapped: "manager uncapped prompt"
manager_capped: "manager capped prompt"
worker_system_role: "worker prompt"
constraint_sections: []
decomposition_strategy:
  section_based:
    max_section_tokens: 2000
    overlap_tokens: 150
consensus_algorithm:
  synthesis:
    synthesis_prompt: "Synthesize all findings."
"#;

    let config_path = tmp.path().join("config.yaml");
    std::fs::File::create(&config_path)
        .expect("creating config.yaml must succeed")
        .write_all(config_yaml.as_bytes())
        .expect("writing config.yaml must succeed");

    let domain = LoadedDomain::load(tmp.path())
        .expect("LoadedDomain::load must succeed for valid config");

    assert_eq!(
        domain.decomposition_strategy(),
        DecompositionStrategy::SectionBased {
            max_section_tokens: 2000,
            overlap_tokens: 150,
        },
        "decomposition_strategy must deserialize to SectionBased with correct token values"
    );

    assert_eq!(
        domain.consensus_algorithm(),
        ConsensusAlgorithm::Synthesis {
            synthesis_prompt: "Synthesize all findings.".to_owned(),
        },
        "consensus_algorithm must deserialize to Synthesis with correct prompt"
    );

    let hint = domain.decomposition_hint();
    assert!(
        hint.contains("2000"),
        "decomposition_hint() must contain max_section_tokens value '2000'; got: {hint:?}"
    );
    assert!(
        hint.contains("150"),
        "decomposition_hint() must contain overlap_tokens value '150'; got: {hint:?}"
    );

    assert!(
        matches!(domain.consensus_algorithm(), ConsensusAlgorithm::Synthesis { .. }),
        "consensus_algorithm() must be the Synthesis variant"
    );
}

// ---------------------------------------------------------------------------
// test_coding_domain_full_chain_unchanged
//
// Loads the actual domains/coding/ directory and asserts that all four
// pluggable behavior methods return their default values, confirming
// byte-for-byte behavioral parity with the pre-Phase-2 coding domain.
// ---------------------------------------------------------------------------
#[test]
fn test_coding_domain_full_chain_unchanged() {
    let coding_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../domains/coding")
        .canonicalize()
        .expect("domains/coding directory must exist relative to mahalaxmi-core");

    let domain = LoadedDomain::load(&coding_dir)
        .expect("LoadedDomain::load must succeed for coding domain");

    assert_eq!(
        domain.decomposition_hint(),
        "",
        "coding domain decomposition_hint() must return empty string for SoftwareDevelopment"
    );

    assert_eq!(
        domain.consensus_algorithm(),
        ConsensusAlgorithm::Unanimous,
        "coding domain consensus_algorithm() must return Unanimous"
    );

    assert_eq!(
        domain.output_format(),
        OutputFormat::PullRequest,
        "coding domain output_format() must return PullRequest"
    );

    assert_eq!(
        domain.input_format(),
        InputFormat::Codebase,
        "coding domain input_format() must return Codebase"
    );
}

//! Integration tests — legal domain synthesis pipeline end-to-end.
//!
//! Validates the full synthesis pipeline:
//! 1. `synthesis_context()` builds a prompt aggregating all worker outputs when the
//!    domain's consensus algorithm is `ConsensusAlgorithm::Synthesis`.
//! 2. `synthesis_context()` returns `None` for non-Synthesis domain algorithms.
//! 3. `format_cycle_output` produces `legal-analysis.md` with all 6 structured sections
//!    from `domains/legal/config.yaml`.
//! 4. Missing sections in worker output are filled with the expected placeholder text.
//! 5. `ConsensusEngine::evaluate_with_domain_algorithm` delegates to standard evaluation
//!    for the `Synthesis` algorithm arm (plan can still be built before synthesis runs).

use std::path::PathBuf;
use std::sync::Arc;

use mahalaxmi_core::config::VerificationConfig;
use mahalaxmi_core::domain::{
    ConsensusAlgorithm, DecompositionStrategy, DomainConfig, InputFormat, LoadedDomain,
    OutputFormat,
};
use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
use mahalaxmi_core::types::{ConsensusStrategy, GitMergeStrategy, GitPrPlatform};
use mahalaxmi_orchestration::consensus::engine::ConsensusEngine;
use mahalaxmi_orchestration::models::proposal::{ManagerProposal, ProposedTask};
use mahalaxmi_orchestration::models::ConsensusConfiguration;
use mahalaxmi_orchestration::output_format::format_cycle_output;
use mahalaxmi_orchestration::service::{CycleConfig, OrchestrationService};
use tokio::sync::broadcast;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Absolute path to `terminalAutomation/domains/legal/`.
fn legal_domain_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..") // crates/mahalaxmi-orchestration → crates/
        .join("..") // crates/ → terminalAutomation/
        .join("domains")
        .join("legal")
}

/// Build a minimal `CycleConfig`, optionally with an `active_domain`.
fn make_cycle_config(active_domain: Option<Arc<dyn DomainConfig>>) -> CycleConfig {
    CycleConfig {
        project_root: "/tmp/test-legal".into(),
        provider_id: "claude-code".into(),
        manager_count: 1,
        worker_count: 2,
        max_retries: 0,
        consensus_config: ConsensusConfiguration {
            strategy: ConsensusStrategy::Union,
            ..ConsensusConfiguration::default()
        },
        requirements: "Analyze this legal document.".into(),
        repo_map: String::new(),
        shared_memory: String::new(),
        provider_ids: Vec::new(),
        routing_strategy: String::new(),
        manager_provider_id: None,
        enable_review_chain: false,
        review_provider_id: None,
        accept_partial_progress: false,
        git_strategy: GitMergeStrategy::DirectMerge,
        git_target_branch: String::new(),
        git_auto_merge_pr: false,
        git_pr_platform: GitPrPlatform::GitHub,
        enable_validation: false,
        validator_provider_id: None,
        active_domain,
    }
}

/// Create an `OrchestrationService` with the given domain.
fn make_service(active_domain: Option<Arc<dyn DomainConfig>>) -> OrchestrationService {
    let (tx, _rx) = broadcast::channel(16);
    OrchestrationService::new(
        make_cycle_config(active_domain),
        I18nService::new(SupportedLocale::EnUs),
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    )
}

/// A minimal domain stub whose consensus algorithm is `Unanimous`.
///
/// Used to verify that `synthesis_context()` returns `None` for any
/// non-Synthesis algorithm.
struct UnanimousDomain;

impl DomainConfig for UnanimousDomain {
    fn id(&self) -> &str {
        "mock-unanimous"
    }

    fn manager_system_role(&self, _worker_count: u32) -> String {
        String::new()
    }

    fn worker_system_role(&self) -> String {
        String::new()
    }

    fn constraint_sections(&self, _worker_count: u32) -> Vec<(String, String)> {
        Vec::new()
    }

    fn decomposition_strategy(&self) -> DecompositionStrategy {
        DecompositionStrategy::default()
    }

    fn consensus_algorithm(&self) -> ConsensusAlgorithm {
        ConsensusAlgorithm::Unanimous
    }

    fn output_format(&self) -> OutputFormat {
        OutputFormat::default()
    }

    fn input_format(&self) -> InputFormat {
        InputFormat::default()
    }
}

// ── legal section constants ───────────────────────────────────────────────────

const LEGAL_SECTIONS: &[&str] = &[
    "Executive Summary",
    "Key Obligations",
    "Risk Areas",
    "Unusual Clauses",
    "Missing Protections",
    "Negotiation Recommendations",
];

/// Build worker output covering all 6 legal sections with non-empty content.
fn full_legal_synthesis_result() -> String {
    "## Executive Summary\n\nThis contract governs a software licensing agreement.\n\n\
     ## Key Obligations\n\nVendor must deliver quarterly updates.\n\n\
     ## Risk Areas\n\nIndemnification clause is one-sided in favor of vendor.\n\n\
     ## Unusual Clauses\n\nClause 14.3 restricts assignment without prior written consent.\n\n\
     ## Missing Protections\n\nNo data breach notification timeline specified.\n\n\
     ## Negotiation Recommendations\n\nSeek mutual indemnification and a 72-hour breach notification window.\n"
        .to_owned()
}

/// Build worker output with all 6 legal sections EXCEPT `Risk Areas`.
fn synthesis_result_missing_risk_areas() -> String {
    "## Executive Summary\n\nThis contract governs a software licensing agreement.\n\n\
     ## Key Obligations\n\nVendor must deliver quarterly updates.\n\n\
     ## Unusual Clauses\n\nClause 14.3 restricts assignment without prior written consent.\n\n\
     ## Missing Protections\n\nNo data breach notification timeline specified.\n\n\
     ## Negotiation Recommendations\n\nSeek mutual indemnification and a 72-hour breach notification window.\n"
        .to_owned()
}

// ── test 1 ───────────────────────────────────────────────────────────────────

/// `synthesis_context()` aggregates all recorded worker outputs into a prompt
/// when the active domain uses `ConsensusAlgorithm::Synthesis`.
///
/// Loads the actual `domains/legal/config.yaml`, records two mock worker outputs,
/// and asserts that:
/// - `synthesis_context()` returns `Some`.
/// - The returned prompt contains `"## Worker 1 Output"` and `"## Worker 2 Output"`.
#[test]
fn legal_domain_synthesis_context_builds_prompt_from_worker_outputs() {
    let domain_dir = legal_domain_dir();
    let domain = Arc::new(
        LoadedDomain::load(&domain_dir).expect("failed to load legal domain for synthesis test"),
    ) as Arc<dyn DomainConfig>;

    // Verify the loaded domain does in fact use Synthesis.
    assert!(
        matches!(domain.consensus_algorithm(), ConsensusAlgorithm::Synthesis { .. }),
        "legal domain must use ConsensusAlgorithm::Synthesis, got: {:?}",
        domain.consensus_algorithm()
    );

    let mut service = make_service(Some(domain));

    service.record_worker_output(
        "Section 3 obligations: The licensor agrees to provide support.".to_owned(),
    );
    service.record_worker_output(
        "Identified risk: indemnification clause is asymmetric.".to_owned(),
    );

    let ctx = service.synthesis_context();

    assert!(
        ctx.is_some(),
        "synthesis_context() must return Some for a domain with ConsensusAlgorithm::Synthesis"
    );

    let prompt = ctx.expect("synthesis_context returned None");

    assert!(
        prompt.contains("## Worker 1 Output"),
        "synthesis prompt must contain '## Worker 1 Output', got: {prompt:?}"
    );
    assert!(
        prompt.contains("## Worker 2 Output"),
        "synthesis prompt must contain '## Worker 2 Output', got: {prompt:?}"
    );
}

// ── test 2 ───────────────────────────────────────────────────────────────────

/// `synthesis_context()` returns `None` when the active domain does not use
/// `ConsensusAlgorithm::Synthesis`.
///
/// Uses a minimal in-memory domain stub with `ConsensusAlgorithm::Unanimous`.
#[test]
fn legal_domain_synthesis_context_returns_none_for_non_synthesis_domain() {
    let domain = Arc::new(UnanimousDomain) as Arc<dyn DomainConfig>;
    let mut service = make_service(Some(domain));

    service.record_worker_output("worker output alpha".to_owned());
    service.record_worker_output("worker output beta".to_owned());

    let ctx = service.synthesis_context();

    assert!(
        ctx.is_none(),
        "synthesis_context() must return None for ConsensusAlgorithm::Unanimous; got Some"
    );
}

// ── test 3 ───────────────────────────────────────────────────────────────────

/// `format_cycle_output` produces `legal-analysis.md` containing all 6 legal
/// section headings when given a synthesis result that covers every section.
#[test]
fn legal_domain_format_cycle_output_produces_legal_analysis_md() {
    let dir = tempfile::tempdir().expect("failed to create tempdir for legal-analysis.md test");

    let format = OutputFormat::StructuredReport {
        sections: LEGAL_SECTIONS
            .iter()
            .map(|s| s.to_string())
            .collect(),
        output_file: "legal-analysis.md".to_owned(),
    };

    let worker_outputs = vec![full_legal_synthesis_result()];

    let result = format_cycle_output(&format, &worker_outputs, dir.path());
    assert!(
        result.is_ok(),
        "format_cycle_output must not error for legal-analysis.md: {:?}",
        result.err()
    );
    assert_eq!(
        result
            .expect("format_cycle_output returned Err")
            .as_deref(),
        Some("legal-analysis.md"),
        "format_cycle_output must return 'legal-analysis.md'"
    );

    let content = std::fs::read_to_string(dir.path().join("legal-analysis.md"))
        .expect("legal-analysis.md must have been written to the temp directory");

    // File must begin with a markdown H1 report title.
    assert!(
        content.starts_with("# "),
        "legal-analysis.md must begin with a markdown H1 header; first line: {:?}",
        content.lines().next()
    );

    // Every configured section heading must be present.
    for section in LEGAL_SECTIONS {
        assert!(
            content.contains(&format!("## {section}")),
            "legal-analysis.md must contain '## {section}'"
        );
    }

    // Each section must contain real content — no placeholder text.
    assert!(
        !content.contains("No findings in this category."),
        "legal-analysis.md must not contain placeholder text when all sections have content"
    );
}

// ── test 4 ───────────────────────────────────────────────────────────────────

/// `format_cycle_output` fills sections absent from the synthesis result with
/// placeholder text so that every configured section heading still appears in
/// the output file.
#[test]
fn legal_domain_format_cycle_output_handles_missing_section_gracefully() {
    let dir = tempfile::tempdir()
        .expect("failed to create tempdir for missing-section graceful-handling test");

    let format = OutputFormat::StructuredReport {
        sections: LEGAL_SECTIONS
            .iter()
            .map(|s| s.to_string())
            .collect(),
        output_file: "legal-analysis.md".to_owned(),
    };

    // Synthesis result is missing the "Risk Areas" section.
    let worker_outputs = vec![synthesis_result_missing_risk_areas()];

    let result = format_cycle_output(&format, &worker_outputs, dir.path());
    assert!(
        result.is_ok(),
        "format_cycle_output must not error when a section is absent: {:?}",
        result.err()
    );

    let content = std::fs::read_to_string(dir.path().join("legal-analysis.md"))
        .expect("legal-analysis.md must be written even when a section is missing");

    // All 6 section headings must still appear in the output file.
    for section in LEGAL_SECTIONS {
        assert!(
            content.contains(&format!("## {section}")),
            "legal-analysis.md must still contain '## {section}' even when section body is absent"
        );
    }

    // The Risk Areas section must be filled with placeholder text.
    let risk_area_pos = content
        .find("## Risk Areas")
        .expect("'## Risk Areas' heading must be present in the output");

    // Find the next section heading after Risk Areas to isolate the body.
    let after_risk_area = &content[risk_area_pos + "## Risk Areas".len()..];
    let risk_area_body = match after_risk_area.find("## ") {
        Some(next) => &after_risk_area[..next],
        None => after_risk_area,
    };

    assert!(
        risk_area_body.contains("No findings in this category."),
        "Risk Areas section body must contain placeholder text 'No findings in this category.' \
         when no worker output covered that section; got: {risk_area_body:?}"
    );
}

// ── test 5 ───────────────────────────────────────────────────────────────────

/// `ConsensusEngine::evaluate_with_domain_algorithm` delegates to standard
/// evaluation when the domain algorithm is `ConsensusAlgorithm::Synthesis`.
///
/// This ensures the consensus engine does NOT no-op for Synthesis domains —
/// a regular execution plan can still be built before the synthesis manager
/// session runs post-worker-completion.
#[test]
fn legal_domain_engine_synthesis_arm_delegates_to_standard_evaluation() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let config = ConsensusConfiguration {
        strategy: ConsensusStrategy::Union,
        ..ConsensusConfiguration::default()
    };
    let engine = ConsensusEngine::new(config);

    let task_a = ProposedTask::new("Analyze contract sections", "Review each clause in detail");
    let task_b = ProposedTask::new("Identify risk areas", "Flag any unfavorable terms");

    let proposals = vec![
        ManagerProposal::new(
            mahalaxmi_core::types::ManagerId::new("manager-0"),
            vec![task_a.clone(), task_b.clone()],
            100,
        ),
        ManagerProposal::new(
            mahalaxmi_core::types::ManagerId::new("manager-1"),
            vec![task_a.clone(), task_b.clone()],
            100,
        ),
    ];

    let algo = ConsensusAlgorithm::Synthesis {
        synthesis_prompt: "Synthesize all worker outputs into a unified legal review.".to_owned(),
    };

    let result = engine
        .evaluate_with_domain_algorithm(&proposals, &i18n, Some(&algo))
        .expect("evaluate_with_domain_algorithm must not error for Synthesis algorithm");

    assert!(
        !result.agreed_tasks.is_empty(),
        "evaluate_with_domain_algorithm with Synthesis must return a non-empty task list \
         (standard evaluation must run, not a no-op); got 0 tasks"
    );
}

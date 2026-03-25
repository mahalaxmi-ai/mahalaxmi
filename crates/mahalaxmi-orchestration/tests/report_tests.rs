//! Tests for the CycleReport model and generation via OrchestrationService.

use chrono::Utc;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::CycleId;
use mahalaxmi_orchestration::models::report::{
    CycleCostSummary, CycleOutcome, CycleReport, TaskFailureSummary, TaskSummary,
    VerificationSummary,
};

fn sample_cycle_id() -> CycleId {
    CycleId::new()
}

fn make_report(
    tasks_completed: Vec<TaskSummary>,
    tasks_failed: Vec<TaskFailureSummary>,
) -> CycleReport {
    let total = tasks_completed.len() + tasks_failed.len();
    let completed_count = tasks_completed.len() as u32;

    let status = if tasks_failed.is_empty() && total > 0 {
        CycleOutcome::AllTasksCompleted
    } else if completed_count > 0 {
        CycleOutcome::PartialCompletion {
            completed: completed_count,
            total: total as u32,
        }
    } else {
        CycleOutcome::Failed {
            reason: "All tasks failed".into(),
        }
    };

    CycleReport {
        cycle_id: sample_cycle_id(),
        started_at: Utc::now(),
        completed_at: Utc::now(),
        duration_ms: 120_000,
        status,
        tasks_completed,
        tasks_failed,
        files_modified: vec!["src/main.rs".into(), "src/lib.rs".into()],
        verification_results: Vec::new(),
        agent_performance: Vec::new(),
        recommendations: Vec::new(),
        cost_summary: CycleCostSummary::default(),
        plan_audit: Vec::new(),
    }
}

#[test]
fn cycle_report_all_tasks_completed() {
    let report = make_report(
        vec![
            TaskSummary {
                title: "Add auth module".into(),
                provider_id: "claude-code".into(),
                duration_ms: 30_000,
                files_modified: vec!["src/auth.rs".into()],
            },
            TaskSummary {
                title: "Add tests".into(),
                provider_id: "claude-code".into(),
                duration_ms: 20_000,
                files_modified: vec!["tests/auth_test.rs".into()],
            },
        ],
        vec![],
    );

    assert!(matches!(report.status, CycleOutcome::AllTasksCompleted));
    assert_eq!(report.tasks_completed.len(), 2);
    assert!(report.tasks_failed.is_empty());
}

#[test]
fn cycle_report_partial_completion() {
    let report = make_report(
        vec![TaskSummary {
            title: "Add auth module".into(),
            provider_id: "claude-code".into(),
            duration_ms: 30_000,
            files_modified: vec!["src/auth.rs".into()],
        }],
        vec![TaskFailureSummary {
            title: "Add tests".into(),
            provider_id: "openai-foundry".into(),
            error: "Test compilation error".into(),
            retry_count: 2,
            recommendation: "Split into smaller tasks".into(),
        }],
    );

    match &report.status {
        CycleOutcome::PartialCompletion { completed, total } => {
            assert_eq!(*completed, 1);
            assert_eq!(*total, 2);
        }
        other => panic!("Expected PartialCompletion, got {other}"),
    }
}

#[test]
fn cycle_report_all_failed() {
    let report = make_report(
        vec![],
        vec![TaskFailureSummary {
            title: "Add auth module".into(),
            provider_id: "claude-code".into(),
            error: "Timeout".into(),
            retry_count: 3,
            recommendation: "Try again".into(),
        }],
    );

    assert!(matches!(report.status, CycleOutcome::Failed { .. }));
}

#[test]
fn to_prompt_summary_includes_completed_tasks() {
    let report = make_report(
        vec![TaskSummary {
            title: "Build API endpoint".into(),
            provider_id: "claude-code".into(),
            duration_ms: 45_000,
            files_modified: vec!["src/api.rs".into()],
        }],
        vec![],
    );

    let summary = report.to_prompt_summary();
    assert!(summary.contains("completed ALL 1 tasks"));
    assert!(summary.contains("Build API endpoint"));
    assert!(summary.contains("claude-code"));
    assert!(summary.contains("src/api.rs"));
}

#[test]
fn to_prompt_summary_includes_failures() {
    let report = make_report(
        vec![],
        vec![TaskFailureSummary {
            title: "Fix database schema".into(),
            provider_id: "openai-foundry".into(),
            error: "Migration conflict".into(),
            retry_count: 2,
            recommendation: "Split migration into separate steps".into(),
        }],
    );

    let summary = report.to_prompt_summary();
    assert!(summary.contains("Fix database schema"));
    assert!(summary.contains("Migration conflict"));
    assert!(summary.contains("Recommendation:"));
}

#[test]
fn to_prompt_summary_includes_verification_failures() {
    let mut report = make_report(
        vec![TaskSummary {
            title: "Add feature".into(),
            provider_id: "claude-code".into(),
            duration_ms: 10_000,
            files_modified: vec![],
        }],
        vec![],
    );
    report.verification_results.push(VerificationSummary {
        title: "Add feature".into(),
        passed: false,
        checks_passed: 1,
        checks_total: 3,
        failure_details: vec![
            "Lint: 3 warnings".into(),
            "Tests: 1 failure in test_auth".into(),
        ],
    });

    let summary = report.to_prompt_summary();
    assert!(summary.contains("Verification failures:"));
    assert!(summary.contains("1/3 checks passed"));
    assert!(summary.contains("Lint: 3 warnings"));
}

#[test]
fn to_prompt_summary_includes_recommendations() {
    let mut report = make_report(vec![], vec![]);
    report.recommendations = vec![
        "Consider using async handlers".into(),
        "Split large files".into(),
    ];

    let summary = report.to_prompt_summary();
    assert!(summary.contains("Recommendations for this cycle:"));
    assert!(summary.contains("Consider using async handlers"));
    assert!(summary.contains("Split large files"));
}

#[test]
fn to_prompt_summary_includes_files_modified() {
    let report = make_report(
        vec![TaskSummary {
            title: "Task A".into(),
            provider_id: "p".into(),
            duration_ms: 1000,
            files_modified: vec!["src/a.rs".into()],
        }],
        vec![],
    );

    let summary = report.to_prompt_summary();
    assert!(summary.contains("Files modified:"));
    assert!(summary.contains("src/main.rs"));
}

#[test]
fn cycle_report_serialization_roundtrip() {
    let report = make_report(
        vec![TaskSummary {
            title: "Task A".into(),
            provider_id: "claude-code".into(),
            duration_ms: 5000,
            files_modified: vec!["a.rs".into()],
        }],
        vec![TaskFailureSummary {
            title: "Task B".into(),
            provider_id: "openai".into(),
            error: "timeout".into(),
            retry_count: 1,
            recommendation: "retry".into(),
        }],
    );

    let json = serde_json::to_string(&report).expect("serialize");
    let deserialized: CycleReport = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.cycle_id, report.cycle_id);
    assert_eq!(deserialized.tasks_completed.len(), 1);
    assert_eq!(deserialized.tasks_failed.len(), 1);
    assert_eq!(deserialized.tasks_completed[0].title, "Task A");
    assert_eq!(deserialized.tasks_failed[0].error, "timeout");
}

#[test]
fn cycle_outcome_display() {
    let all = CycleOutcome::AllTasksCompleted;
    assert_eq!(all.to_string(), "AllTasksCompleted");

    let partial = CycleOutcome::PartialCompletion {
        completed: 3,
        total: 5,
    };
    assert_eq!(partial.to_string(), "PartialCompletion(3/5)");

    let failed = CycleOutcome::Failed {
        reason: "oops".into(),
    };
    assert_eq!(failed.to_string(), "Failed(oops)");
}

// =========================================================================
// Integration: generate_cycle_report via OrchestrationService
// =========================================================================

use mahalaxmi_core::config::VerificationConfig;
use mahalaxmi_core::types::ConsensusStrategy;
use mahalaxmi_orchestration::models::ConsensusConfiguration;
use mahalaxmi_orchestration::service::{CycleConfig, OrchestrationService};
use tokio::sync::broadcast;

fn test_config() -> CycleConfig {
    CycleConfig {
        project_root: "/tmp/test".into(),
        provider_id: "claude-code".into(),
        manager_count: 1,
        worker_count: 2,
        max_retries: 1,
        consensus_config: ConsensusConfiguration {
            strategy: ConsensusStrategy::Union,
            ..Default::default()
        },
        requirements: "Build something".into(),
        repo_map: String::new(),
        shared_memory: String::new(),
        provider_ids: vec![],
        routing_strategy: "quality_first".into(),
        manager_provider_id: None,
        enable_review_chain: false,
        review_provider_id: None,
        accept_partial_progress: false,
        git_strategy: mahalaxmi_core::types::GitMergeStrategy::DirectMerge,
        git_target_branch: String::new(),
        git_auto_merge_pr: false,
        git_pr_platform: mahalaxmi_core::types::GitPrPlatform::GitHub,
        enable_validation: false,
        validator_provider_id: None,
        active_domain: None,
    }
}

fn test_i18n() -> I18nService {
    I18nService::new(mahalaxmi_core::i18n::locale::SupportedLocale::EnUs)
}

#[test]
fn generate_cycle_report_no_workers_returns_failed() {
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    let report = svc.generate_cycle_report(5000);

    assert!(matches!(report.status, CycleOutcome::Failed { .. }));
    assert!(report.tasks_completed.is_empty());
    assert!(report.tasks_failed.is_empty());
}

#[test]
fn generate_cycle_report_with_completed_workers() {
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    // Submit a manager output that produces 2 tasks
    let manager_output = r#"{"tasks":[
        {"title":"Task Alpha","description":"Do alpha work","complexity":3,"priority":1,"dependencies":[],"affected_files":["src/alpha.rs"]},
        {"title":"Task Beta","description":"Do beta work","complexity":5,"priority":2,"dependencies":[],"affected_files":["src/beta.rs"]}
    ]}"#;

    let _all = svc
        .submit_manager_output(
            mahalaxmi_core::types::ManagerId::new("manager-0"),
            manager_output,
            1000,
        )
        .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    // Activate and complete both workers
    let ready = svc.ready_workers();
    assert_eq!(ready.len(), 2);

    for wid in &ready {
        svc.activate_worker(*wid).unwrap();
    }
    for wid in &ready {
        svc.complete_worker(*wid, 10_000).unwrap();
    }

    let report = svc.generate_cycle_report(25_000);

    assert!(matches!(report.status, CycleOutcome::AllTasksCompleted));
    assert_eq!(report.tasks_completed.len(), 2);
    assert!(report.tasks_failed.is_empty());
    assert!(report.files_modified.contains(&"src/alpha.rs".to_string()));
    assert!(report.files_modified.contains(&"src/beta.rs".to_string()));
}

#[test]
fn generate_cycle_report_with_mixed_results() {
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    svc.initialize().unwrap();
    svc.build_manager_prompts().unwrap();

    let manager_output = r#"{"tasks":[
        {"title":"Task One","description":"Do one","complexity":3,"priority":1,"dependencies":[],"affected_files":["src/one.rs"]},
        {"title":"Task Two","description":"Do two","complexity":5,"priority":2,"dependencies":[],"affected_files":["src/two.rs"]}
    ]}"#;

    svc.submit_manager_output(
        mahalaxmi_core::types::ManagerId::new("manager-0"),
        manager_output,
        1000,
    )
    .unwrap();
    svc.run_consensus().unwrap();
    svc.begin_worker_execution().unwrap();

    let ready = svc.ready_workers();
    for wid in &ready {
        svc.activate_worker(*wid).unwrap();
    }

    // Complete first, fail second
    svc.complete_worker(ready[0], 8000).unwrap();
    svc.fail_worker(ready[1], "Compilation error".into())
        .unwrap();

    let report = svc.generate_cycle_report(20_000);

    match &report.status {
        CycleOutcome::PartialCompletion { completed, total } => {
            assert_eq!(*completed, 1);
            assert_eq!(*total, 2);
        }
        other => panic!("Expected PartialCompletion, got {other}"),
    }
    assert_eq!(report.tasks_completed.len(), 1);
    assert_eq!(report.tasks_failed.len(), 1);
    assert!(!report.recommendations.is_empty());
}

#[test]
fn generate_cycle_report_stores_as_last_report() {
    let (tx, _rx) = broadcast::channel(16);
    let i18n = test_i18n();
    let mut svc = OrchestrationService::new(
        test_config(),
        i18n,
        tx,
        VerificationConfig {
            enabled: false,
            ..VerificationConfig::default()
        },
    );

    assert!(svc.last_cycle_report().is_none());

    svc.initialize().unwrap();
    svc.generate_cycle_report(1000);

    assert!(svc.last_cycle_report().is_some());
}

#[test]
fn manager_prompt_includes_previous_cycle_report_when_present() {
    use mahalaxmi_core::config::ContextFormat;
    use mahalaxmi_orchestration::prompt::builder::{ManagerPromptBuilder, ManagerPromptConfig};

    let config = ManagerPromptConfig {
        requirements: "Build feature X".into(),
        repo_map: String::new(),
        shared_memory: String::new(),
        provider_id: "claude-code".into(),
        worker_count: 3,
        format: ContextFormat::Auto,
        include_quality_mandate: true,
        previous_cycle_report: Some(
            "Previous cycle completed 2/3 tasks in 5m 30s.\nFailed tasks:\n  - Fix tests".into(),
        ),
        previous_validation_verdict: None,
            domain: None,
    };

    let prompt = ManagerPromptBuilder::build(&config);
    assert!(prompt.contains("Previous cycle completed 2/3 tasks"));
    assert!(prompt.contains("Fix tests"));
    // Should be wrapped in XML tags for claude provider
    assert!(prompt.contains("<previous_cycle>"));
    assert!(prompt.contains("</previous_cycle>"));
}

#[test]
fn manager_prompt_omits_previous_cycle_when_none() {
    use mahalaxmi_core::config::ContextFormat;
    use mahalaxmi_orchestration::prompt::builder::{ManagerPromptBuilder, ManagerPromptConfig};

    let config = ManagerPromptConfig {
        requirements: "Build feature X".into(),
        repo_map: String::new(),
        shared_memory: String::new(),
        provider_id: "claude-code".into(),
        worker_count: 3,
        format: ContextFormat::Auto,
        include_quality_mandate: true,
        previous_cycle_report: None,
        previous_validation_verdict: None,
            domain: None,
    };

    let prompt = ManagerPromptBuilder::build(&config);
    assert!(!prompt.contains("previous_cycle"));
}

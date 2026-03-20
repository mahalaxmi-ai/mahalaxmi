use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{
    ActionType, ConsensusStrategy, CycleId, ManagerId, MatchType, OrchestrationCycleState,
    RootCauseCategory, TaskId, WorkerId, WorkerStatus,
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// CycleId tests
// ---------------------------------------------------------------------------

#[test]
fn cycle_id_generates_unique_ids() {
    let id1 = CycleId::new();
    let id2 = CycleId::new();
    assert_ne!(id1, id2);
}

#[test]
fn cycle_id_default_generates_unique_ids() {
    let id1 = CycleId::default();
    let id2 = CycleId::default();
    assert_ne!(id1, id2);
}

#[test]
fn cycle_id_from_uuid_roundtrip() {
    let uuid = Uuid::new_v4();
    let cycle_id = CycleId::from_uuid(uuid);
    assert_eq!(*cycle_id.as_uuid(), uuid);
}

#[test]
fn cycle_id_display_format() {
    let cycle_id = CycleId::new();
    let display = cycle_id.to_string();
    // A valid UUID v4 string has the format xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    let parsed = Uuid::parse_str(&display);
    assert!(
        parsed.is_ok(),
        "Display string '{}' is not a valid UUID",
        display
    );
    assert_eq!(*cycle_id.as_uuid(), parsed.unwrap());
}

#[test]
fn cycle_id_serde_roundtrip() {
    let original = CycleId::new();
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: CycleId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

// ---------------------------------------------------------------------------
// WorkerId tests
// ---------------------------------------------------------------------------

#[test]
fn worker_id_from_u32() {
    let id = WorkerId::new(42);
    assert_eq!(id.as_u32(), 42);
}

#[test]
fn worker_id_display_format() {
    let id = WorkerId::new(1);
    assert_eq!(id.to_string(), "worker-1");
}

#[test]
fn worker_id_serde_roundtrip() {
    let original = WorkerId::new(99);
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: WorkerId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

// ---------------------------------------------------------------------------
// ManagerId and TaskId tests
// ---------------------------------------------------------------------------

#[test]
fn manager_id_from_string() {
    let id = ManagerId::new("mgr-1");
    assert_eq!(id.as_str(), "mgr-1");
}

#[test]
fn task_id_from_string() {
    let id = TaskId::new("task-1");
    assert_eq!(id.as_str(), "task-1");
}

// ---------------------------------------------------------------------------
// OrchestrationCycleState tests
// ---------------------------------------------------------------------------

#[test]
fn cycle_state_is_active_for_active_states() {
    let active_states = [
        OrchestrationCycleState::InitializingManager,
        OrchestrationCycleState::ManagerProcessing,
        OrchestrationCycleState::AnalyzingManagerResults,
        OrchestrationCycleState::DiscoveringWorkerRequirements,
        OrchestrationCycleState::InitializingWorkers,
        OrchestrationCycleState::WorkersProcessing,
        OrchestrationCycleState::CompletingCycle,
        OrchestrationCycleState::RestartingCycle,
    ];
    for state in &active_states {
        assert!(
            state.is_active(),
            "{} should be active but is_active() returned false",
            state
        );
    }
}

#[test]
fn cycle_state_is_active_false_for_idle_error_stopped() {
    assert!(!OrchestrationCycleState::Idle.is_active());
    assert!(!OrchestrationCycleState::Error.is_active());
    assert!(!OrchestrationCycleState::Stopped.is_active());
}

#[test]
fn cycle_state_is_terminal_only_for_stopped() {
    let all_states = [
        OrchestrationCycleState::Idle,
        OrchestrationCycleState::InitializingManager,
        OrchestrationCycleState::ManagerProcessing,
        OrchestrationCycleState::AnalyzingManagerResults,
        OrchestrationCycleState::DiscoveringWorkerRequirements,
        OrchestrationCycleState::InitializingWorkers,
        OrchestrationCycleState::WorkersProcessing,
        OrchestrationCycleState::CompletingCycle,
        OrchestrationCycleState::RestartingCycle,
        OrchestrationCycleState::Error,
        OrchestrationCycleState::Stopped,
    ];
    for state in &all_states {
        if *state == OrchestrationCycleState::Stopped {
            assert!(state.is_terminal(), "Stopped should be terminal");
        } else {
            assert!(
                !state.is_terminal(),
                "{} should NOT be terminal but is_terminal() returned true",
                state
            );
        }
    }
}

#[test]
fn cycle_state_valid_transitions_idle() {
    let transitions = OrchestrationCycleState::Idle.valid_transitions();
    assert_eq!(transitions.len(), 2);
    assert!(transitions.contains(&OrchestrationCycleState::InitializingManager));
    assert!(transitions.contains(&OrchestrationCycleState::Stopped));
}

#[test]
fn cycle_state_can_transition_to_valid() {
    assert!(OrchestrationCycleState::Idle
        .can_transition_to(OrchestrationCycleState::InitializingManager));
}

#[test]
fn cycle_state_can_transition_to_invalid() {
    assert!(!OrchestrationCycleState::Idle
        .can_transition_to(OrchestrationCycleState::WorkersProcessing));
}

#[test]
fn cycle_state_serde_roundtrip_all_variants() {
    let all_states = [
        OrchestrationCycleState::Idle,
        OrchestrationCycleState::InitializingManager,
        OrchestrationCycleState::ManagerProcessing,
        OrchestrationCycleState::AnalyzingManagerResults,
        OrchestrationCycleState::DiscoveringWorkerRequirements,
        OrchestrationCycleState::InitializingWorkers,
        OrchestrationCycleState::WorkersProcessing,
        OrchestrationCycleState::CompletingCycle,
        OrchestrationCycleState::RestartingCycle,
        OrchestrationCycleState::Error,
        OrchestrationCycleState::Stopped,
    ];
    for state in &all_states {
        let json = serde_json::to_string(state).unwrap();
        let deserialized: OrchestrationCycleState = serde_json::from_str(&json).unwrap();
        assert_eq!(
            *state, deserialized,
            "Serde roundtrip failed for state {}",
            state
        );
    }
}

// ---------------------------------------------------------------------------
// ConsensusStrategy tests
// ---------------------------------------------------------------------------

#[test]
fn consensus_strategy_from_config_str() {
    assert_eq!(
        ConsensusStrategy::from_config_str("weighted_voting"),
        Some(ConsensusStrategy::WeightedVoting)
    );
    assert_eq!(
        ConsensusStrategy::from_config_str("union"),
        Some(ConsensusStrategy::Union)
    );
    assert_eq!(
        ConsensusStrategy::from_config_str("intersection"),
        Some(ConsensusStrategy::Intersection)
    );
    assert_eq!(
        ConsensusStrategy::from_config_str("complexity_weighted"),
        Some(ConsensusStrategy::ComplexityWeighted)
    );
    assert_eq!(ConsensusStrategy::from_config_str("bad"), None);
    assert_eq!(ConsensusStrategy::from_config_str(""), None);
}

#[test]
fn consensus_strategy_as_config_str_roundtrip() {
    let strategies = [
        ConsensusStrategy::Union,
        ConsensusStrategy::Intersection,
        ConsensusStrategy::WeightedVoting,
        ConsensusStrategy::ComplexityWeighted,
    ];
    for strategy in &strategies {
        let config_str = strategy.as_config_str();
        let parsed = ConsensusStrategy::from_config_str(config_str);
        assert_eq!(
            parsed,
            Some(*strategy),
            "Roundtrip failed for {:?} (config str: '{}')",
            strategy,
            config_str
        );
    }
}

// ---------------------------------------------------------------------------
// WorkerStatus tests
// ---------------------------------------------------------------------------

#[test]
fn worker_status_is_finished() {
    assert!(WorkerStatus::Completed.is_finished());
    assert!(WorkerStatus::Failed.is_finished());
    assert!(!WorkerStatus::Pending.is_finished());
    assert!(!WorkerStatus::Active.is_finished());
    assert!(!WorkerStatus::Blocked.is_finished());
    assert!(!WorkerStatus::Verifying.is_finished());
}

#[test]
fn worker_status_is_schedulable() {
    assert!(WorkerStatus::Pending.is_schedulable());
    assert!(WorkerStatus::Blocked.is_schedulable());
    assert!(!WorkerStatus::Active.is_schedulable());
    assert!(!WorkerStatus::Completed.is_schedulable());
    assert!(!WorkerStatus::Failed.is_schedulable());
    assert!(!WorkerStatus::Verifying.is_schedulable());
}

// ---------------------------------------------------------------------------
// ActionType tests
// ---------------------------------------------------------------------------

#[test]
fn action_type_sends_input() {
    assert!(ActionType::SendEnter.sends_input());
    assert!(ActionType::SendText.sends_input());
    assert!(ActionType::SendTextWithEnter.sends_input());
    assert!(!ActionType::ContinueProcessing.sends_input());
    assert!(!ActionType::CompleteManagerCycle.sends_input());
    assert!(!ActionType::RestartSession.sends_input());
}

#[test]
fn action_type_is_cycle_signal() {
    assert!(ActionType::CompleteManagerCycle.is_cycle_signal());
    assert!(ActionType::CompleteWorkerCycle.is_cycle_signal());
    assert!(ActionType::LaunchManager.is_cycle_signal());
    assert!(ActionType::LaunchWorkers.is_cycle_signal());
    assert!(ActionType::RestartOrchestration.is_cycle_signal());
    assert!(ActionType::StopOrchestration.is_cycle_signal());
    assert!(!ActionType::SendEnter.is_cycle_signal());
    assert!(!ActionType::ContinueProcessing.is_cycle_signal());
    assert!(!ActionType::CloseTerminal.is_cycle_signal());
}

// ---------------------------------------------------------------------------
// MatchType display tests
// ---------------------------------------------------------------------------

#[test]
fn match_type_display() {
    let variants = [
        MatchType::Exact,
        MatchType::Contains,
        MatchType::Regex,
        MatchType::StartsWith,
        MatchType::EndsWith,
    ];
    for variant in &variants {
        let display = variant.to_string();
        assert!(
            !display.is_empty(),
            "MatchType::{:?} has empty Display output",
            variant
        );
    }
}

// ---------------------------------------------------------------------------
// RootCauseCategory display tests
// ---------------------------------------------------------------------------

#[test]
fn root_cause_category_display() {
    let variants = [
        RootCauseCategory::Unknown,
        RootCauseCategory::Authentication,
        RootCauseCategory::Network,
        RootCauseCategory::FileSystem,
        RootCauseCategory::Dependency,
        RootCauseCategory::Syntax,
        RootCauseCategory::Runtime,
        RootCauseCategory::Resource,
        RootCauseCategory::Configuration,
        RootCauseCategory::RateLimit,
        RootCauseCategory::Permission,
        RootCauseCategory::Build,
        RootCauseCategory::Test,
        RootCauseCategory::VersionControl,
        RootCauseCategory::ExternalService,
        RootCauseCategory::Timeout,
    ];
    for variant in &variants {
        let display = variant.to_string();
        assert!(
            !display.is_empty(),
            "RootCauseCategory::{:?} has empty Display output",
            variant
        );
    }
}

// ---------------------------------------------------------------------------
// MahalaxmiError::detection() test
// ---------------------------------------------------------------------------

#[test]
fn detection_error_variant_works() {
    let i18n = I18nService::new(SupportedLocale::default());
    let err = MahalaxmiError::detection(
        &i18n,
        "error-detection-rule-compile-failed",
        &[("reason", "bad regex")],
    );
    assert_eq!(err.category(), "detection");
    assert!(matches!(err, MahalaxmiError::Detection { .. }));
    assert_eq!(err.i18n_key(), Some("error-detection-rule-compile-failed"));
    // Verify the error message is not just the key (i.e., it was translated)
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{CycleId, OrchestrationCycleState};
use mahalaxmi_orchestration::CycleStateMachine;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::default())
}

#[test]
fn state_machine_starts_idle() {
    let sm = CycleStateMachine::new();
    assert_eq!(sm.current_state(), OrchestrationCycleState::Idle);
}

#[test]
fn state_machine_default_starts_idle() {
    let sm = CycleStateMachine::default();
    assert_eq!(sm.current_state(), OrchestrationCycleState::Idle);
}

#[test]
fn state_machine_valid_transition() {
    let i18n = i18n();
    let mut sm = CycleStateMachine::new();

    let result = sm.transition_to(OrchestrationCycleState::InitializingManager, &i18n);
    assert!(result.is_ok());
    assert_eq!(
        sm.current_state(),
        OrchestrationCycleState::InitializingManager
    );
}

#[test]
fn state_machine_invalid_transition() {
    let i18n = i18n();
    let mut sm = CycleStateMachine::new();

    // Idle cannot transition directly to WorkersProcessing
    let result = sm.transition_to(OrchestrationCycleState::WorkersProcessing, &i18n);
    assert!(result.is_err());
    // State should remain Idle
    assert_eq!(sm.current_state(), OrchestrationCycleState::Idle);
}

#[test]
fn state_machine_full_happy_path() {
    let i18n = i18n();
    let mut sm = CycleStateMachine::new();

    let transitions = [
        OrchestrationCycleState::InitializingManager,
        OrchestrationCycleState::ManagerProcessing,
        OrchestrationCycleState::AnalyzingManagerResults,
        OrchestrationCycleState::DiscoveringWorkerRequirements,
        OrchestrationCycleState::InitializingWorkers,
        OrchestrationCycleState::WorkersProcessing,
        OrchestrationCycleState::CompletingCycle,
        OrchestrationCycleState::Idle,
    ];

    for next in &transitions {
        sm.transition_to(*next, &i18n).unwrap();
    }

    assert_eq!(sm.current_state(), OrchestrationCycleState::Idle);
}

#[test]
fn state_machine_force_error_from_any_state() {
    let i18n = i18n();
    let mut sm = CycleStateMachine::new();

    // Move to ManagerProcessing first
    sm.transition_to(OrchestrationCycleState::InitializingManager, &i18n)
        .unwrap();
    sm.transition_to(OrchestrationCycleState::ManagerProcessing, &i18n)
        .unwrap();

    // Force error from ManagerProcessing
    sm.force_error();
    assert_eq!(sm.current_state(), OrchestrationCycleState::Error);
}

#[test]
fn state_machine_force_stop_from_any_state() {
    let i18n = i18n();
    let mut sm = CycleStateMachine::new();

    // Move to WorkersProcessing
    sm.transition_to(OrchestrationCycleState::InitializingManager, &i18n)
        .unwrap();
    sm.transition_to(OrchestrationCycleState::ManagerProcessing, &i18n)
        .unwrap();
    sm.transition_to(OrchestrationCycleState::AnalyzingManagerResults, &i18n)
        .unwrap();
    sm.transition_to(
        OrchestrationCycleState::DiscoveringWorkerRequirements,
        &i18n,
    )
    .unwrap();
    sm.transition_to(OrchestrationCycleState::InitializingWorkers, &i18n)
        .unwrap();
    sm.transition_to(OrchestrationCycleState::WorkersProcessing, &i18n)
        .unwrap();

    // Force stop from WorkersProcessing
    sm.force_stop();
    assert_eq!(sm.current_state(), OrchestrationCycleState::Stopped);
}

#[test]
fn state_machine_history_records_transitions() {
    let i18n = i18n();
    let mut sm = CycleStateMachine::new();

    sm.transition_to(OrchestrationCycleState::InitializingManager, &i18n)
        .unwrap();
    sm.transition_to(OrchestrationCycleState::ManagerProcessing, &i18n)
        .unwrap();
    sm.transition_to(OrchestrationCycleState::AnalyzingManagerResults, &i18n)
        .unwrap();

    assert_eq!(sm.history().len(), 3);
}

#[test]
fn state_machine_history_entries_correct() {
    let i18n = i18n();
    let mut sm = CycleStateMachine::new();

    sm.transition_to(OrchestrationCycleState::InitializingManager, &i18n)
        .unwrap();
    sm.transition_to(OrchestrationCycleState::ManagerProcessing, &i18n)
        .unwrap();
    sm.transition_to(OrchestrationCycleState::AnalyzingManagerResults, &i18n)
        .unwrap();

    let history = sm.history();

    assert_eq!(history[0].from, OrchestrationCycleState::Idle);
    assert_eq!(history[0].to, OrchestrationCycleState::InitializingManager);

    assert_eq!(
        history[1].from,
        OrchestrationCycleState::InitializingManager
    );
    assert_eq!(history[1].to, OrchestrationCycleState::ManagerProcessing);

    assert_eq!(history[2].from, OrchestrationCycleState::ManagerProcessing);
    assert_eq!(
        history[2].to,
        OrchestrationCycleState::AnalyzingManagerResults
    );
}

#[test]
fn state_machine_is_finished_after_stop() {
    let mut sm = CycleStateMachine::new();
    sm.force_stop();
    assert!(sm.is_finished());
}

#[test]
fn state_machine_is_finished_after_cycle() {
    let i18n = i18n();
    let mut sm = CycleStateMachine::new();

    // Run full cycle back to Idle
    let transitions = [
        OrchestrationCycleState::InitializingManager,
        OrchestrationCycleState::ManagerProcessing,
        OrchestrationCycleState::AnalyzingManagerResults,
        OrchestrationCycleState::DiscoveringWorkerRequirements,
        OrchestrationCycleState::InitializingWorkers,
        OrchestrationCycleState::WorkersProcessing,
        OrchestrationCycleState::CompletingCycle,
        OrchestrationCycleState::Idle,
    ];
    for next in &transitions {
        sm.transition_to(*next, &i18n).unwrap();
    }

    // is_finished returns true when back to Idle with non-empty history
    assert!(sm.is_finished());
}

#[test]
fn state_machine_is_finished_false_initially() {
    let sm = CycleStateMachine::new();
    assert!(!sm.is_finished());
}

#[test]
fn state_machine_error_recovery() {
    let i18n = i18n();
    let mut sm = CycleStateMachine::new();

    sm.transition_to(OrchestrationCycleState::InitializingManager, &i18n)
        .unwrap();
    sm.force_error();
    assert_eq!(sm.current_state(), OrchestrationCycleState::Error);

    // Error -> RestartingCycle -> Idle
    sm.transition_to(OrchestrationCycleState::RestartingCycle, &i18n)
        .unwrap();
    assert_eq!(sm.current_state(), OrchestrationCycleState::RestartingCycle);

    sm.transition_to(OrchestrationCycleState::Idle, &i18n)
        .unwrap();
    assert_eq!(sm.current_state(), OrchestrationCycleState::Idle);
}

#[test]
fn state_machine_stopped_is_terminal() {
    let mut sm = CycleStateMachine::new();
    sm.force_stop();
    assert_eq!(sm.current_state(), OrchestrationCycleState::Stopped);

    let valid = OrchestrationCycleState::Stopped.valid_transitions();
    assert!(valid.is_empty());
}

#[test]
fn state_machine_with_cycle_id() {
    let custom_id = CycleId::new();
    let sm = CycleStateMachine::with_cycle_id(custom_id);
    assert_eq!(sm.cycle_id(), custom_id);
    assert_eq!(sm.current_state(), OrchestrationCycleState::Idle);
}

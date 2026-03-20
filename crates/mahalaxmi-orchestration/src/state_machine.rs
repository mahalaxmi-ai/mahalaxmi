use chrono::{DateTime, Utc};
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{CycleId, OrchestrationCycleState};
use mahalaxmi_core::MahalaxmiResult;
use serde::{Deserialize, Serialize};

use crate::models::events::OrchestrationEvent;

/// A record of a single state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// The state before the transition.
    pub from: OrchestrationCycleState,
    /// The state after the transition.
    pub to: OrchestrationCycleState,
    /// When the transition occurred.
    pub timestamp: DateTime<Utc>,
}

/// Manages the orchestration cycle state machine with validated transitions.
pub struct CycleStateMachine {
    /// Current cycle ID.
    cycle_id: CycleId,
    /// Current state.
    current_state: OrchestrationCycleState,
    /// History of state transitions.
    history: Vec<StateTransition>,
}

impl CycleStateMachine {
    /// Create a new state machine in the Idle state.
    pub fn new() -> Self {
        Self {
            cycle_id: CycleId::new(),
            current_state: OrchestrationCycleState::Idle,
            history: Vec::new(),
        }
    }

    /// Create a new state machine with a specific cycle ID.
    pub fn with_cycle_id(cycle_id: CycleId) -> Self {
        Self {
            cycle_id,
            current_state: OrchestrationCycleState::Idle,
            history: Vec::new(),
        }
    }

    /// Get the current state.
    pub fn current_state(&self) -> OrchestrationCycleState {
        self.current_state
    }

    /// Get the cycle ID.
    pub fn cycle_id(&self) -> CycleId {
        self.cycle_id
    }

    /// Attempt to transition to the next state.
    ///
    /// Returns an `OrchestrationEvent::StateChanged` on success, or an error
    /// if the transition is invalid.
    pub fn transition_to(
        &mut self,
        next: OrchestrationCycleState,
        i18n: &I18nService,
    ) -> MahalaxmiResult<OrchestrationEvent> {
        if !self.current_state.can_transition_to(next) {
            return Err(MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::INVALID_TRANSITION,
                &[
                    ("from", &self.current_state.to_string()),
                    ("to", &next.to_string()),
                ],
            ));
        }

        let old_state = self.current_state;
        let now = Utc::now();

        self.history.push(StateTransition {
            from: old_state,
            to: next,
            timestamp: now,
        });

        self.current_state = next;

        Ok(OrchestrationEvent::StateChanged {
            cycle_id: self.cycle_id,
            old_state,
            new_state: next,
            timestamp: now,
        })
    }

    /// Force transition to Error state from any state.
    ///
    /// This bypasses normal transition validation — errors can happen at any time.
    pub fn force_error(&mut self) -> OrchestrationEvent {
        let old_state = self.current_state;
        let now = Utc::now();

        self.history.push(StateTransition {
            from: old_state,
            to: OrchestrationCycleState::Error,
            timestamp: now,
        });

        self.current_state = OrchestrationCycleState::Error;

        OrchestrationEvent::StateChanged {
            cycle_id: self.cycle_id,
            old_state,
            new_state: OrchestrationCycleState::Error,
            timestamp: now,
        }
    }

    /// Force transition to Stopped state from any state.
    ///
    /// This bypasses normal transition validation — stopping can happen at any time.
    pub fn force_stop(&mut self) -> OrchestrationEvent {
        let old_state = self.current_state;
        let now = Utc::now();

        self.history.push(StateTransition {
            from: old_state,
            to: OrchestrationCycleState::Stopped,
            timestamp: now,
        });

        self.current_state = OrchestrationCycleState::Stopped;

        OrchestrationEvent::StateChanged {
            cycle_id: self.cycle_id,
            old_state,
            new_state: OrchestrationCycleState::Stopped,
            timestamp: now,
        }
    }

    /// Get the transition history.
    pub fn history(&self) -> &[StateTransition] {
        &self.history
    }

    /// Returns true if the state machine is in a finished state (Idle after running, or Stopped).
    pub fn is_finished(&self) -> bool {
        self.current_state == OrchestrationCycleState::Stopped
            || (self.current_state == OrchestrationCycleState::Idle && !self.history.is_empty())
    }
}

impl Default for CycleStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahalaxmi_core::types::OrchestrationCycleState;

    #[test]
    fn discovering_worker_requirements_to_awaiting_plan_approval_is_valid() {
        assert!(
            OrchestrationCycleState::DiscoveringWorkerRequirements
                .can_transition_to(OrchestrationCycleState::AwaitingPlanApproval),
            "DiscoveringWorkerRequirements → AwaitingPlanApproval must be a valid transition"
        );
    }

    #[test]
    fn awaiting_plan_approval_to_initializing_workers_is_valid() {
        assert!(
            OrchestrationCycleState::AwaitingPlanApproval
                .can_transition_to(OrchestrationCycleState::InitializingWorkers),
            "AwaitingPlanApproval → InitializingWorkers must be a valid transition"
        );
    }

    #[test]
    fn awaiting_plan_approval_to_error_is_valid() {
        assert!(
            OrchestrationCycleState::AwaitingPlanApproval
                .can_transition_to(OrchestrationCycleState::Error),
            "AwaitingPlanApproval → Error must be a valid transition"
        );
    }

    #[test]
    fn awaiting_plan_approval_is_active() {
        assert!(
            OrchestrationCycleState::AwaitingPlanApproval.is_active(),
            "AwaitingPlanApproval must be an active state"
        );
    }

    #[test]
    fn state_machine_transitions_through_awaiting_plan_approval() {
        let i18n = mahalaxmi_core::i18n::I18nService::new(
            mahalaxmi_core::i18n::locale::SupportedLocale::EnUs,
        );
        let mut sm = CycleStateMachine::new();
        sm.transition_to(OrchestrationCycleState::InitializingManager, &i18n)
            .expect("idle → initializing_manager");
        sm.transition_to(OrchestrationCycleState::ManagerProcessing, &i18n)
            .expect("initializing_manager → manager_processing");
        sm.transition_to(OrchestrationCycleState::AnalyzingManagerResults, &i18n)
            .expect("manager_processing → analyzing");
        sm.transition_to(
            OrchestrationCycleState::DiscoveringWorkerRequirements,
            &i18n,
        )
        .expect("analyzing → discovering");
        sm.transition_to(OrchestrationCycleState::AwaitingPlanApproval, &i18n)
            .expect("discovering → awaiting_plan_approval");
        sm.transition_to(OrchestrationCycleState::InitializingWorkers, &i18n)
            .expect("awaiting_plan_approval → initializing_workers");
        assert_eq!(
            sm.current_state(),
            OrchestrationCycleState::InitializingWorkers
        );
    }
}

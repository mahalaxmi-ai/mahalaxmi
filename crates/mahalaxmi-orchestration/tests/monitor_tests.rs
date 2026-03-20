//! Tests for Step 05: StreamMonitor.

use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{ActionType, TerminalId, WorkerId};
use mahalaxmi_detection::DetectionRule;
use mahalaxmi_orchestration::monitor::{MonitorAction, StreamMonitor, TerminalRole};

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

fn tid() -> TerminalId {
    TerminalId::new()
}

/// Create a simple rule set for testing.
fn test_rules() -> Vec<DetectionRule> {
    vec![
        // Worker completion: shell prompt detected
        DetectionRule::new("shell-prompt", ActionType::CompleteWorkerCycle)
            .with_regex_pattern(r"\$\s*$")
            .with_priority(90),
        // Auto-confirm: "Do you want to continue? [y/n]"
        DetectionRule::new("auto-confirm", ActionType::SendTextWithEnter)
            .with_contains_pattern("Do you want to continue")
            .with_response_text("y")
            .with_priority(20),
        // Error escalation: segfault
        DetectionRule::new("segfault", ActionType::EscalateToManager)
            .with_contains_pattern("Segmentation fault")
            .with_priority(10),
        // Stop on auth error
        DetectionRule::new("auth-error", ActionType::StopOrchestration)
            .with_contains_pattern("Authentication failed")
            .with_priority(5),
        // Close terminal
        DetectionRule::new("close-terminal", ActionType::CloseTerminal)
            .with_contains_pattern("SESSION_DONE")
            .with_priority(50),
        // Restart session
        DetectionRule::new("restart-session", ActionType::RestartSession)
            .with_contains_pattern("Connection reset")
            .with_priority(15),
        // Send enter on blank prompt
        DetectionRule::new("blank-prompt", ActionType::SendEnter)
            .with_contains_pattern("Press Enter to continue")
            .with_priority(60),
        // Continue processing (cost warning — informational only)
        DetectionRule::new("cost-warning", ActionType::ContinueProcessing)
            .with_contains_pattern("WARNING: cost exceeded")
            .with_priority(80),
        // Send text without enter
        DetectionRule::new("send-text-only", ActionType::SendText)
            .with_contains_pattern("Enter password:")
            .with_response_text("secret123")
            .with_priority(25),
    ]
}

fn make_monitor() -> StreamMonitor {
    StreamMonitor::new(test_rules(), "claude-code", &i18n()).unwrap()
}

// ===========================================================================
// Construction tests
// ===========================================================================

#[test]
fn monitor_creates_with_rules() {
    let monitor = make_monitor();
    assert!(monitor.rule_count() > 0);
    assert_eq!(monitor.terminal_count(), 0);
}

#[test]
fn monitor_with_defaults_compiles() {
    let monitor = StreamMonitor::with_defaults("claude-code", &i18n()).unwrap();
    assert!(monitor.rule_count() > 0);
}

// ===========================================================================
// Terminal registration tests
// ===========================================================================

#[test]
fn register_and_unregister_terminal() {
    let mut monitor = make_monitor();
    let term_id = tid();
    let worker_id = WorkerId::new(0);

    monitor.register_terminal(term_id, TerminalRole::Worker, Some(worker_id));
    assert_eq!(monitor.terminal_count(), 1);
    assert_eq!(monitor.worker_for_terminal(&term_id), Some(worker_id));
    assert_eq!(monitor.terminal_role(&term_id), Some(TerminalRole::Worker));

    monitor.unregister_terminal(&term_id);
    assert_eq!(monitor.terminal_count(), 0);
    assert_eq!(monitor.worker_for_terminal(&term_id), None);
}

#[test]
fn register_manager_terminal_no_worker_id() {
    let mut monitor = make_monitor();
    let term_id = tid();

    monitor.register_terminal(term_id, TerminalRole::Manager, None);
    assert_eq!(monitor.terminal_count(), 1);
    assert_eq!(monitor.worker_for_terminal(&term_id), None);
    assert_eq!(monitor.terminal_role(&term_id), Some(TerminalRole::Manager));
}

// ===========================================================================
// Evaluation tests — action dispatch
// ===========================================================================

#[test]
fn unregistered_terminal_returns_continue() {
    let mut monitor = make_monitor();
    let unknown_id = tid();
    let action = monitor.evaluate(&unknown_id, "some output");
    assert_eq!(action, MonitorAction::Continue);
}

#[test]
fn worker_completion_on_shell_prompt() {
    let mut monitor = make_monitor();
    let term_id = tid();
    let worker_id = WorkerId::new(0);
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(worker_id));

    let action = monitor.evaluate(&term_id, "user@host:~/project$ ");
    assert_eq!(
        action,
        MonitorAction::WorkerCompleted {
            worker_id,
            terminal_id: term_id,
        }
    );
}

#[test]
fn auto_confirm_sends_text_with_enter() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(1)));

    let action = monitor.evaluate(&term_id, "Do you want to continue? [y/n]");
    assert_eq!(
        action,
        MonitorAction::SendInput {
            terminal_id: term_id,
            text: "y".into(),
            append_enter: true,
        }
    );
}

#[test]
fn segfault_escalates_to_worker_failed() {
    let mut monitor = make_monitor();
    let term_id = tid();
    let worker_id = WorkerId::new(2);
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(worker_id));

    let action = monitor.evaluate(&term_id, "Segmentation fault (core dumped)");
    assert_eq!(
        action,
        MonitorAction::WorkerFailed {
            worker_id,
            terminal_id: term_id,
            error_summary: "segfault".into(),
        }
    );
}

#[test]
fn auth_error_stops_orchestration() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(3)));

    let action = monitor.evaluate(&term_id, "Authentication failed: invalid token");
    assert_eq!(action, MonitorAction::StopOrchestration);
}

#[test]
fn close_terminal_action() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(4)));

    let action = monitor.evaluate(&term_id, "SESSION_DONE");
    assert_eq!(
        action,
        MonitorAction::CloseTerminal {
            terminal_id: term_id
        }
    );
}

#[test]
fn restart_session_action() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(5)));

    let action = monitor.evaluate(&term_id, "Connection reset by peer");
    assert_eq!(
        action,
        MonitorAction::RestartSession {
            terminal_id: term_id,
        }
    );
}

#[test]
fn send_enter_action() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(6)));

    let action = monitor.evaluate(&term_id, "Press Enter to continue...");
    assert_eq!(
        action,
        MonitorAction::SendInput {
            terminal_id: term_id,
            text: String::new(),
            append_enter: true,
        }
    );
}

#[test]
fn continue_processing_returns_continue() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(7)));

    let action = monitor.evaluate(&term_id, "WARNING: cost exceeded $5.00");
    assert_eq!(action, MonitorAction::Continue);
}

#[test]
fn send_text_without_enter() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(8)));

    let action = monitor.evaluate(&term_id, "Enter password:");
    assert_eq!(
        action,
        MonitorAction::SendInput {
            terminal_id: term_id,
            text: "secret123".into(),
            append_enter: false,
        }
    );
}

#[test]
fn no_match_returns_continue() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(9)));

    let action = monitor.evaluate(&term_id, "Compiling mahalaxmi-core v0.1.0...");
    assert_eq!(action, MonitorAction::Continue);
}

// ===========================================================================
// Manager terminal tests
// ===========================================================================

#[test]
fn manager_completion_without_worker_returns_continue() {
    let mut monitor = make_monitor();
    let term_id = tid();
    // Manager terminal has no worker_id
    monitor.register_terminal(term_id, TerminalRole::Manager, None);

    // Shell prompt triggers CompleteWorkerCycle but no worker_id → Continue
    let action = monitor.evaluate(&term_id, "user@host:~/project$ ");
    assert_eq!(action, MonitorAction::Continue);
}

#[test]
fn manager_escalation_without_worker_returns_continue() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Manager, None);

    let action = monitor.evaluate(&term_id, "Segmentation fault (core dumped)");
    // No worker to fail → Continue
    assert_eq!(action, MonitorAction::Continue);
}

// ===========================================================================
// Multiple terminals
// ===========================================================================

#[test]
fn multiple_terminals_tracked_independently() {
    let mut monitor = make_monitor();
    let term1 = tid();
    let term2 = tid();
    let w1 = WorkerId::new(10);
    let w2 = WorkerId::new(11);

    monitor.register_terminal(term1, TerminalRole::Worker, Some(w1));
    monitor.register_terminal(term2, TerminalRole::Worker, Some(w2));

    assert_eq!(monitor.terminal_count(), 2);
    assert_eq!(monitor.worker_for_terminal(&term1), Some(w1));
    assert_eq!(monitor.worker_for_terminal(&term2), Some(w2));

    // Complete only one worker
    let action1 = monitor.evaluate(&term1, "user@host:~/project$ ");
    assert_eq!(
        action1,
        MonitorAction::WorkerCompleted {
            worker_id: w1,
            terminal_id: term1,
        }
    );

    // Other terminal gets a different output
    let action2 = monitor.evaluate(&term2, "Building project...");
    assert_eq!(action2, MonitorAction::Continue);
}

// ===========================================================================
// Reset cooldowns
// ===========================================================================

#[test]
fn reset_cooldowns_clears_state() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(12)));

    // Trigger a rule
    monitor.evaluate(&term_id, "Do you want to continue? [y/n]");

    // Reset cooldowns
    monitor.reset_cooldowns();
    // Should still work (no panic)
    assert_eq!(monitor.rule_count(), test_rules().len());
}

// ===========================================================================
// Provider-filtered rules
// ===========================================================================

#[test]
fn provider_filtered_rules_apply() {
    let rules = vec![
        DetectionRule::new("claude-only", ActionType::SendTextWithEnter)
            .with_contains_pattern("claude-specific-prompt")
            .with_provider_filter(vec!["claude-code".into()])
            .with_response_text("yes")
            .with_priority(10),
    ];
    let mut monitor = StreamMonitor::new(rules, "claude-code", &i18n()).unwrap();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(13)));

    let action = monitor.evaluate(&term_id, "claude-specific-prompt");
    assert_eq!(
        action,
        MonitorAction::SendInput {
            terminal_id: term_id,
            text: "yes".into(),
            append_enter: true,
        }
    );
}

#[test]
fn provider_filtered_rules_skip_wrong_provider() {
    let rules = vec![
        DetectionRule::new("openai-only", ActionType::SendTextWithEnter)
            .with_contains_pattern("openai-specific")
            .with_provider_filter(vec!["openai-foundry".into()])
            .with_response_text("ok")
            .with_priority(10),
    ];
    let mut monitor = StreamMonitor::new(rules, "claude-code", &i18n()).unwrap();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(14)));

    let action = monitor.evaluate(&term_id, "openai-specific prompt here");
    assert_eq!(action, MonitorAction::Continue);
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[test]
fn empty_text_returns_continue() {
    let mut monitor = make_monitor();
    let term_id = tid();
    monitor.register_terminal(term_id, TerminalRole::Worker, Some(WorkerId::new(15)));

    let action = monitor.evaluate(&term_id, "");
    assert_eq!(action, MonitorAction::Continue);
}

#[test]
fn terminal_role_equality() {
    assert_eq!(TerminalRole::Manager, TerminalRole::Manager);
    assert_eq!(TerminalRole::Worker, TerminalRole::Worker);
    assert_ne!(TerminalRole::Manager, TerminalRole::Worker);
}

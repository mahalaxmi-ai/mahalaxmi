//! Tests for Step 12: Cross-Provider Handoff.

use mahalaxmi_core::types::{ProviderId, WorkerId};
use mahalaxmi_providers::handoff::{HandoffContext, HandoffResult, HandoffTracker};

// ===========================================================================
// HandoffContext
// ===========================================================================

#[test]
fn handoff_context_new() {
    let ctx = HandoffContext::new(
        WorkerId::new(1),
        ProviderId::new("claude-code"),
        ProviderId::new("deepseek"),
    );
    assert_eq!(ctx.worker_id, WorkerId::new(1));
    assert_eq!(ctx.source_provider, ProviderId::new("claude-code"));
    assert_eq!(ctx.target_provider, ProviderId::new("deepseek"));
    assert!(ctx.progress_summary.is_empty());
    assert!(ctx.modified_files.is_empty());
    assert!(ctx.discoveries.is_empty());
    assert!(ctx.errors.is_empty());
}

#[test]
fn handoff_context_builder() {
    let ctx = HandoffContext::new(WorkerId::new(2), ProviderId::new("a"), ProviderId::new("b"))
        .with_progress("Completed 3 of 5 subtasks")
        .with_modified_file("src/main.rs")
        .with_modified_file("src/lib.rs")
        .with_discovery("Uses custom error type")
        .with_error("Compilation failed on line 42")
        .with_metadata("attempt", "2");

    assert_eq!(ctx.progress_summary, "Completed 3 of 5 subtasks");
    assert_eq!(ctx.modified_files.len(), 2);
    assert_eq!(ctx.discoveries.len(), 1);
    assert!(ctx.has_errors());
    assert_eq!(ctx.metadata["attempt"], "2");
}

#[test]
fn handoff_context_no_errors() {
    let ctx = HandoffContext::new(WorkerId::new(0), ProviderId::new("a"), ProviderId::new("b"));
    assert!(!ctx.has_errors());
}

#[test]
fn handoff_context_build_prompt_empty() {
    let ctx = HandoffContext::new(
        WorkerId::new(0),
        ProviderId::new("source"),
        ProviderId::new("target"),
    );
    let prompt = ctx.build_handoff_prompt();
    assert!(prompt.contains("Handoff Context"));
    assert!(prompt.contains("source"));
}

#[test]
fn handoff_context_build_prompt_full() {
    let ctx = HandoffContext::new(
        WorkerId::new(0),
        ProviderId::new("claude-code"),
        ProviderId::new("deepseek"),
    )
    .with_progress("Implemented auth module")
    .with_modified_file("src/auth.rs")
    .with_discovery("Project uses axum framework")
    .with_error("Test failure in auth_tests");

    let prompt = ctx.build_handoff_prompt();
    assert!(prompt.contains("Progress So Far"));
    assert!(prompt.contains("Implemented auth module"));
    assert!(prompt.contains("Files Modified"));
    assert!(prompt.contains("src/auth.rs"));
    assert!(prompt.contains("Discoveries"));
    assert!(prompt.contains("axum framework"));
    assert!(prompt.contains("Previous Errors"));
    assert!(prompt.contains("auth_tests"));
}

#[test]
fn handoff_context_serialization() {
    let ctx = HandoffContext::new(WorkerId::new(3), ProviderId::new("a"), ProviderId::new("b"))
        .with_progress("done");

    let json = serde_json::to_string(&ctx).unwrap();
    let deserialized: HandoffContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.worker_id, WorkerId::new(3));
    assert_eq!(deserialized.progress_summary, "done");
}

// ===========================================================================
// HandoffResult
// ===========================================================================

#[test]
fn handoff_result_variants() {
    assert_eq!(HandoffResult::Success, HandoffResult::Success);
    assert_ne!(HandoffResult::Success, HandoffResult::NoTarget);
    assert_ne!(HandoffResult::TargetUnavailable, HandoffResult::NoTarget);
}

// ===========================================================================
// HandoffTracker
// ===========================================================================

#[test]
fn tracker_new_empty() {
    let tracker = HandoffTracker::new(3);
    assert_eq!(tracker.total_handoffs(), 0);
    assert!(tracker.history().is_empty());
}

#[test]
fn tracker_record_and_count() {
    let mut tracker = HandoffTracker::new(3);
    let ctx = HandoffContext::new(WorkerId::new(1), ProviderId::new("a"), ProviderId::new("b"));
    tracker.record(ctx);

    assert_eq!(tracker.total_handoffs(), 1);
    assert_eq!(tracker.handoff_count(&WorkerId::new(1)), 1);
    assert_eq!(tracker.handoff_count(&WorkerId::new(2)), 0);
}

#[test]
fn tracker_can_handoff_limit() {
    let mut tracker = HandoffTracker::new(2);
    let worker = WorkerId::new(5);

    assert!(tracker.can_handoff(&worker));

    tracker.record(HandoffContext::new(
        worker,
        ProviderId::new("a"),
        ProviderId::new("b"),
    ));
    assert!(tracker.can_handoff(&worker)); // 1 < 2

    tracker.record(HandoffContext::new(
        worker,
        ProviderId::new("b"),
        ProviderId::new("c"),
    ));
    assert!(!tracker.can_handoff(&worker)); // 2 >= 2
}

#[test]
fn tracker_last_handoff() {
    let mut tracker = HandoffTracker::new(5);
    let worker = WorkerId::new(1);

    tracker.record(
        HandoffContext::new(worker, ProviderId::new("a"), ProviderId::new("b"))
            .with_progress("first"),
    );
    tracker.record(
        HandoffContext::new(worker, ProviderId::new("b"), ProviderId::new("c"))
            .with_progress("second"),
    );

    let last = tracker.last_handoff(&worker).unwrap();
    assert_eq!(last.progress_summary, "second");
    assert_eq!(last.target_provider, ProviderId::new("c"));
}

#[test]
fn tracker_last_handoff_none_for_unknown_worker() {
    let tracker = HandoffTracker::new(3);
    assert!(tracker.last_handoff(&WorkerId::new(99)).is_none());
}

#[test]
fn tracker_independent_workers() {
    let mut tracker = HandoffTracker::new(1);

    tracker.record(HandoffContext::new(
        WorkerId::new(1),
        ProviderId::new("a"),
        ProviderId::new("b"),
    ));

    assert!(!tracker.can_handoff(&WorkerId::new(1)));
    assert!(tracker.can_handoff(&WorkerId::new(2))); // different worker, not limited
}

//! Tests for Step 14: Performance Tracking — metrics collection and comparison.

use mahalaxmi_core::types::ProviderId;
use mahalaxmi_providers::metrics::{PerformanceTracker, ProviderMetrics};
use mahalaxmi_providers::types::TaskType;

// ===========================================================================
// ProviderMetrics
// ===========================================================================

#[test]
fn metrics_avg_latency_no_requests() {
    let m = ProviderMetrics {
        provider_id: ProviderId::new("test"),
        total_requests: 0,
        successful_requests: 0,
        failed_requests: 0,
        total_latency_ms: 0,
        min_latency_ms: u64::MAX,
        max_latency_ms: 0,
        total_tokens: 0,
        task_counts: Default::default(),
    };
    assert_eq!(m.avg_latency_ms(), 0);
    assert_eq!(m.success_rate(), 1.0); // no failures = 100%
}

// ===========================================================================
// PerformanceTracker
// ===========================================================================

#[test]
fn tracker_new_empty() {
    let tracker = PerformanceTracker::new();
    assert_eq!(tracker.tracked_count(), 0);
    assert_eq!(tracker.total_events(), 0);
}

#[test]
fn tracker_record_request() {
    let mut tracker = PerformanceTracker::new();
    tracker.record_request(
        ProviderId::new("claude-code"),
        TaskType::CodeGeneration,
        true,
        5000,
        1500,
    );

    assert_eq!(tracker.tracked_count(), 1);
    assert_eq!(tracker.total_events(), 1);

    let metrics = tracker
        .get_metrics(&ProviderId::new("claude-code"))
        .unwrap();
    assert_eq!(metrics.total_requests, 1);
    assert_eq!(metrics.successful_requests, 1);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(metrics.avg_latency_ms(), 5000);
    assert_eq!(metrics.total_tokens, 1500);
}

#[test]
fn tracker_multiple_requests() {
    let mut tracker = PerformanceTracker::new();
    let pid = ProviderId::new("deepseek");

    tracker.record_request(pid.clone(), TaskType::Debugging, true, 2000, 500);
    tracker.record_request(pid.clone(), TaskType::Debugging, true, 4000, 700);
    tracker.record_request(pid.clone(), TaskType::Debugging, false, 1000, 100);

    let m = tracker.get_metrics(&pid).unwrap();
    assert_eq!(m.total_requests, 3);
    assert_eq!(m.successful_requests, 2);
    assert_eq!(m.failed_requests, 1);
    assert_eq!(m.avg_latency_ms(), 2333); // 7000 / 3
    assert_eq!(m.min_latency_ms, 1000);
    assert_eq!(m.max_latency_ms, 4000);
    assert_eq!(m.total_tokens, 1300);
}

#[test]
fn tracker_success_rate() {
    let mut tracker = PerformanceTracker::new();
    let pid = ProviderId::new("test");

    tracker.record_request(pid.clone(), TaskType::Testing, true, 100, 0);
    tracker.record_request(pid.clone(), TaskType::Testing, true, 100, 0);
    tracker.record_request(pid.clone(), TaskType::Testing, false, 100, 0);
    tracker.record_request(pid.clone(), TaskType::Testing, true, 100, 0);

    let m = tracker.get_metrics(&pid).unwrap();
    assert!((m.success_rate() - 0.75).abs() < f64::EPSILON);
}

#[test]
fn tracker_task_counts() {
    let mut tracker = PerformanceTracker::new();
    let pid = ProviderId::new("cody");

    tracker.record_request(pid.clone(), TaskType::CodeReview, true, 100, 0);
    tracker.record_request(pid.clone(), TaskType::CodeReview, true, 100, 0);
    tracker.record_request(pid.clone(), TaskType::Documentation, true, 100, 0);

    let m = tracker.get_metrics(&pid).unwrap();
    assert_eq!(m.task_counts[&TaskType::CodeReview], 2);
    assert_eq!(m.task_counts[&TaskType::Documentation], 1);
}

#[test]
fn tracker_multiple_providers() {
    let mut tracker = PerformanceTracker::new();

    tracker.record_request(ProviderId::new("a"), TaskType::General, true, 1000, 100);
    tracker.record_request(ProviderId::new("b"), TaskType::General, true, 2000, 200);

    assert_eq!(tracker.tracked_count(), 2);
    assert_eq!(tracker.total_events(), 2);
}

#[test]
fn tracker_compare_sorted_by_success_then_latency() {
    let mut tracker = PerformanceTracker::new();
    let fast = ProviderId::new("fast");
    let slow = ProviderId::new("slow");
    let flaky = ProviderId::new("flaky");

    tracker.record_request(fast.clone(), TaskType::General, true, 1000, 0);
    tracker.record_request(slow.clone(), TaskType::General, true, 5000, 0);
    tracker.record_request(flaky.clone(), TaskType::General, false, 500, 0);

    let comp = tracker.compare();
    assert_eq!(comp.len(), 3);
    // fast and slow both have 100% success rate, fast has lower latency
    assert_eq!(comp[0].provider_id, fast);
    assert_eq!(comp[1].provider_id, slow);
    assert_eq!(comp[2].provider_id, flaky);
}

#[test]
fn tracker_history() {
    let mut tracker = PerformanceTracker::new();
    tracker.record_request(
        ProviderId::new("p"),
        TaskType::CodeGeneration,
        true,
        100,
        50,
    );

    let history = tracker.history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].latency_ms, 100);
    assert_eq!(history[0].tokens_used, 50);
}

#[test]
fn tracker_max_history_bounded() {
    let mut tracker = PerformanceTracker::with_max_history(5);
    for i in 0..10 {
        tracker.record_request(ProviderId::new("p"), TaskType::General, true, i * 100, 0);
    }
    assert_eq!(tracker.total_events(), 5);
    // Metrics should still reflect all 10 requests
    let m = tracker.get_metrics(&ProviderId::new("p")).unwrap();
    assert_eq!(m.total_requests, 10);
}

#[test]
fn tracker_export_metrics() {
    let mut tracker = PerformanceTracker::new();
    tracker.record_request(ProviderId::new("a"), TaskType::General, true, 100, 50);

    let exported = tracker.export_metrics();
    assert_eq!(exported.len(), 1);
    assert!(exported.contains_key(&ProviderId::new("a")));
}

#[test]
fn tracker_export_history() {
    let mut tracker = PerformanceTracker::new();
    tracker.record_request(ProviderId::new("a"), TaskType::Debugging, true, 100, 50);

    let exported = tracker.export_history();
    assert_eq!(exported.len(), 1);
    assert!(exported[0].success);
}

#[test]
fn tracker_clear() {
    let mut tracker = PerformanceTracker::new();
    tracker.record_request(ProviderId::new("p"), TaskType::General, true, 100, 0);
    tracker.clear();

    assert_eq!(tracker.tracked_count(), 0);
    assert_eq!(tracker.total_events(), 0);
}

#[test]
fn tracker_all_metrics() {
    let mut tracker = PerformanceTracker::new();
    tracker.record_request(ProviderId::new("a"), TaskType::General, true, 100, 0);
    tracker.record_request(ProviderId::new("b"), TaskType::General, true, 200, 0);

    let all = tracker.all_metrics();
    assert_eq!(all.len(), 2);
}

#[test]
fn tracker_get_nonexistent() {
    let tracker = PerformanceTracker::new();
    assert!(tracker.get_metrics(&ProviderId::new("missing")).is_none());
}

#[test]
fn metrics_serialization() {
    let mut tracker = PerformanceTracker::new();
    tracker.record_request(
        ProviderId::new("test"),
        TaskType::CodeGeneration,
        true,
        3000,
        500,
    );

    let metrics = tracker.get_metrics(&ProviderId::new("test")).unwrap();
    let json = serde_json::to_string(metrics).unwrap();
    let deserialized: mahalaxmi_providers::ProviderMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_requests, 1);
    assert_eq!(deserialized.total_latency_ms, 3000);
}

#[test]
fn avg_tokens_per_request() {
    let mut tracker = PerformanceTracker::new();
    let pid = ProviderId::new("p");
    tracker.record_request(pid.clone(), TaskType::General, true, 100, 1000);
    tracker.record_request(pid.clone(), TaskType::General, true, 100, 3000);

    let m = tracker.get_metrics(&pid).unwrap();
    assert_eq!(m.avg_tokens_per_request(), 2000); // 4000 / 2
}

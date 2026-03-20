//! Tests for Step 13: Fallback & Resilience — circuit breakers and health tracking.

use mahalaxmi_core::types::ProviderId;
use mahalaxmi_providers::resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, ProviderHealthTracker,
};

// ===========================================================================
// CircuitBreaker
// ===========================================================================

#[test]
fn breaker_starts_closed() {
    let breaker = CircuitBreaker::with_defaults();
    assert_eq!(breaker.state(), CircuitState::Closed);
    assert!(breaker.allows_request());
}

#[test]
fn breaker_opens_after_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        cooldown_seconds: 60,
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);

    breaker.record_failure();
    assert_eq!(breaker.state(), CircuitState::Closed);

    breaker.record_failure();
    assert_eq!(breaker.state(), CircuitState::Closed);

    breaker.record_failure();
    assert_eq!(breaker.state(), CircuitState::Open);
    assert!(!breaker.allows_request());
}

#[test]
fn breaker_success_resets_consecutive_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        ..Default::default()
    };
    let mut breaker = CircuitBreaker::new(config);

    breaker.record_failure();
    breaker.record_failure();
    breaker.record_success(); // resets consecutive count
    breaker.record_failure();
    breaker.record_failure();

    assert_eq!(breaker.state(), CircuitState::Closed); // 2 consecutive, not 3
}

#[test]
fn breaker_tracks_totals() {
    let mut breaker = CircuitBreaker::with_defaults();
    breaker.record_success();
    breaker.record_success();
    breaker.record_failure();

    assert_eq!(breaker.total_successes(), 2);
    assert_eq!(breaker.total_failures(), 1);
}

#[test]
fn breaker_reset() {
    let mut breaker = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 1,
        ..Default::default()
    });
    breaker.record_failure(); // opens circuit
    assert_eq!(breaker.state(), CircuitState::Open);

    breaker.reset();
    assert_eq!(breaker.state(), CircuitState::Closed);
    assert!(breaker.allows_request());
    assert_eq!(breaker.consecutive_failures(), 0);
}

#[test]
fn breaker_half_open_after_cooldown() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0, // immediate cooldown for testing
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);
    breaker.record_failure();

    // With 0 cooldown, state() immediately returns HalfOpen
    // since (now - last_failure) > Duration::seconds(0) is true
    assert_eq!(breaker.state(), CircuitState::HalfOpen);
    assert!(breaker.allows_request());
}

#[test]
fn breaker_half_open_closes_on_success() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);
    breaker.record_failure(); // Open
    std::thread::sleep(std::time::Duration::from_millis(10));

    assert_eq!(breaker.state(), CircuitState::HalfOpen);
    breaker.record_success();
    assert_eq!(breaker.state(), CircuitState::Closed);
}

// ===========================================================================
// ProviderHealthTracker
// ===========================================================================

#[test]
fn tracker_unknown_provider_available() {
    let tracker = ProviderHealthTracker::new();
    assert!(tracker.is_available(&ProviderId::new("unknown")));
}

#[test]
fn tracker_records_and_checks() {
    let mut tracker = ProviderHealthTracker::new();
    let pid = ProviderId::new("test");

    tracker.record_success(&pid);
    assert!(tracker.is_available(&pid));

    let health = tracker.health(&pid);
    assert_eq!(health.total_successes, 1);
    assert_eq!(health.total_failures, 0);
}

#[test]
fn tracker_failure_threshold_opens_circuit() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        cooldown_seconds: 300,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let pid = ProviderId::new("flaky");

    tracker.record_failure(&pid);
    assert!(tracker.is_available(&pid));

    tracker.record_failure(&pid);
    assert!(!tracker.is_available(&pid));

    let health = tracker.health(&pid);
    assert_eq!(health.circuit_state, CircuitState::Open);
    assert!(!health.available);
}

#[test]
fn tracker_multiple_providers_independent() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 300,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let good = ProviderId::new("good");
    let bad = ProviderId::new("bad");

    tracker.record_success(&good);
    tracker.record_failure(&bad);

    assert!(tracker.is_available(&good));
    assert!(!tracker.is_available(&bad));
}

#[test]
fn tracker_all_health() {
    let mut tracker = ProviderHealthTracker::new();
    tracker.record_success(&ProviderId::new("a"));
    tracker.record_success(&ProviderId::new("b"));

    let health = tracker.all_health();
    assert_eq!(health.len(), 2);
}

#[test]
fn tracker_reset_all() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 300,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let pid = ProviderId::new("p");

    tracker.record_failure(&pid);
    assert!(!tracker.is_available(&pid));

    tracker.reset_all();
    assert!(tracker.is_available(&pid));
}

#[test]
fn tracker_reset_single() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 300,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let a = ProviderId::new("a");
    let b = ProviderId::new("b");

    tracker.record_failure(&a);
    tracker.record_failure(&b);

    tracker.reset(&a);
    assert!(tracker.is_available(&a));
    assert!(!tracker.is_available(&b));
}

#[test]
fn tracker_tracked_count() {
    let mut tracker = ProviderHealthTracker::new();
    assert_eq!(tracker.tracked_count(), 0);

    tracker.record_success(&ProviderId::new("a"));
    assert_eq!(tracker.tracked_count(), 1);

    tracker.record_failure(&ProviderId::new("b"));
    assert_eq!(tracker.tracked_count(), 2);
}

#[test]
fn tracker_health_for_untracked() {
    let tracker = ProviderHealthTracker::new();
    let health = tracker.health(&ProviderId::new("new"));
    assert_eq!(health.circuit_state, CircuitState::Closed);
    assert!(health.available);
    assert_eq!(health.total_failures, 0);
}

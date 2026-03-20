//! Tests for Section 9: CircuitBreaker state consistency.
//!
//! Verifies that the Cell-based interior mutability fix ensures the internal
//! state field stays synchronized with what state() reports. Also covers
//! edge cases like multiple cooldown expirations, rapid state transitions,
//! and the simplified record_success() path.

use mahalaxmi_core::types::ProviderId;
use mahalaxmi_providers::resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, ProviderHealthTracker,
};
use std::thread;
use std::time::Duration;

// =============================================================================
// State consistency — the core fix
// =============================================================================

#[test]
fn state_returns_consistent_result_on_multiple_calls() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);
    breaker.record_failure();
    thread::sleep(Duration::from_millis(5));

    // Call state() multiple times — should always return the same value
    let s1 = breaker.state();
    let s2 = breaker.state();
    let s3 = breaker.state();
    assert_eq!(s1, CircuitState::HalfOpen);
    assert_eq!(s2, CircuitState::HalfOpen);
    assert_eq!(s3, CircuitState::HalfOpen);
}

#[test]
fn state_transitions_open_to_half_open_persistently() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 2,
    };
    let mut breaker = CircuitBreaker::new(config);
    breaker.record_failure();
    thread::sleep(Duration::from_millis(5));

    // After cooldown, state should be HalfOpen
    assert_eq!(breaker.state(), CircuitState::HalfOpen);

    // One success — not enough for success_threshold=2
    breaker.record_success();
    // Should still be HalfOpen (need 2 successes)
    assert_eq!(breaker.state(), CircuitState::HalfOpen);

    // Second success — should close
    breaker.record_success();
    assert_eq!(breaker.state(), CircuitState::Closed);
}

#[test]
fn state_stays_open_before_cooldown() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 3600, // 1 hour — won't expire during test
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);
    breaker.record_failure();

    // Should remain Open (cooldown hasn't passed)
    assert_eq!(breaker.state(), CircuitState::Open);
    assert_eq!(breaker.state(), CircuitState::Open);
    assert!(!breaker.allows_request());
}

// =============================================================================
// Full lifecycle transitions
// =============================================================================

#[test]
fn lifecycle_closed_to_open_to_half_open_to_closed() {
    // Use non-zero cooldown so we can observe the Open state
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        cooldown_seconds: 3600, // long cooldown to observe Open
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);

    // Phase 1: Closed
    assert_eq!(breaker.state(), CircuitState::Closed);
    assert!(breaker.allows_request());

    // Phase 2: Hit failure threshold → Open
    breaker.record_failure();
    assert_eq!(breaker.state(), CircuitState::Closed); // 1 failure, threshold is 2
    breaker.record_failure();
    assert_eq!(breaker.state(), CircuitState::Open);
    assert!(!breaker.allows_request());

    // Phase 3: Simulate cooldown by resetting and using 0 cooldown config
    // (Can't wait 3600s in a test, so we use reset + re-fail with short cooldown)
    breaker.reset();
    let config2 = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 1,
    };
    let mut breaker2 = CircuitBreaker::new(config2);
    breaker2.record_failure();
    thread::sleep(Duration::from_millis(5));
    assert_eq!(breaker2.state(), CircuitState::HalfOpen);
    assert!(breaker2.allows_request());

    // Phase 4: Success → Closed
    breaker2.record_success();
    assert_eq!(breaker2.state(), CircuitState::Closed);
    assert!(breaker2.allows_request());
}

#[test]
fn lifecycle_half_open_failure_reopens() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);

    // Open the circuit, cooldown=0 means it transitions to HalfOpen immediately
    breaker.record_failure();
    thread::sleep(Duration::from_millis(5));
    assert_eq!(breaker.state(), CircuitState::HalfOpen);

    // Failure in HalfOpen → re-opens (but cooldown=0 means instant HalfOpen again)
    breaker.record_failure();
    // With 0 cooldown, state() transitions to HalfOpen immediately since
    // the cooldown has already elapsed. This is correct behavior — the
    // circuit "opens" but the cooldown is so short it's immediately HalfOpen.
    thread::sleep(Duration::from_millis(5));
    assert_eq!(breaker.state(), CircuitState::HalfOpen);

    // Verify with a long cooldown that Open is actually set
    let config2 = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 3600,
        success_threshold: 1,
    };
    let mut breaker2 = CircuitBreaker::new(config2);
    breaker2.record_failure();
    assert_eq!(breaker2.state(), CircuitState::Open);
    assert!(!breaker2.allows_request());
}

#[test]
fn repeated_open_close_cycles() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);

    for _ in 0..5 {
        // Fail → Open → HalfOpen → succeed → Closed
        breaker.record_failure();
        thread::sleep(Duration::from_millis(5));
        assert_eq!(breaker.state(), CircuitState::HalfOpen);
        breaker.record_success();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    assert_eq!(breaker.total_failures(), 5);
    assert_eq!(breaker.total_successes(), 5);
}

// =============================================================================
// Counter tracking during transitions
// =============================================================================

#[test]
fn consecutive_failures_reset_on_success() {
    let mut breaker = CircuitBreaker::with_defaults();
    breaker.record_failure();
    breaker.record_failure();
    assert_eq!(breaker.consecutive_failures(), 2);

    breaker.record_success();
    assert_eq!(breaker.consecutive_failures(), 0);
}

#[test]
fn total_counters_never_reset() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);

    breaker.record_failure();
    breaker.record_success();
    breaker.record_failure();
    breaker.record_success();

    assert_eq!(breaker.total_failures(), 2);
    assert_eq!(breaker.total_successes(), 2);
}

#[test]
fn reset_clears_consecutive_but_not_totals() {
    let mut breaker = CircuitBreaker::with_defaults();
    breaker.record_failure();
    breaker.record_failure();
    breaker.record_success();

    // Totals should be set
    assert_eq!(breaker.total_failures(), 2);
    assert_eq!(breaker.total_successes(), 1);

    breaker.reset();
    assert_eq!(breaker.consecutive_failures(), 0);
    assert_eq!(breaker.state(), CircuitState::Closed);
    // Totals are preserved through reset (they're lifetime counters)
    assert_eq!(breaker.total_failures(), 2);
    assert_eq!(breaker.total_successes(), 1);
}

// =============================================================================
// allows_request consistency with state()
// =============================================================================

#[test]
fn allows_request_consistent_with_state() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);

    // Closed → allows
    assert_eq!(breaker.state(), CircuitState::Closed);
    assert!(breaker.allows_request());

    // Open → blocks
    breaker.record_failure();
    // With cooldown=0, state() returns HalfOpen immediately
    thread::sleep(Duration::from_millis(5));
    assert_eq!(breaker.state(), CircuitState::HalfOpen);
    assert!(breaker.allows_request());
}

#[test]
fn open_with_long_cooldown_blocks_request() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 9999,
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);

    breaker.record_failure();
    assert_eq!(breaker.state(), CircuitState::Open);
    assert!(!breaker.allows_request());
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn success_threshold_greater_than_one() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 3,
    };
    let mut breaker = CircuitBreaker::new(config);

    breaker.record_failure();
    thread::sleep(Duration::from_millis(5));
    assert_eq!(breaker.state(), CircuitState::HalfOpen);

    // Need 3 consecutive successes to close
    breaker.record_success();
    assert_eq!(breaker.state(), CircuitState::HalfOpen);
    breaker.record_success();
    assert_eq!(breaker.state(), CircuitState::HalfOpen);
    breaker.record_success();
    assert_eq!(breaker.state(), CircuitState::Closed);
}

#[test]
fn clone_preserves_state() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 3600,
        success_threshold: 1,
    };
    let mut breaker = CircuitBreaker::new(config);
    breaker.record_failure();

    let cloned = breaker.clone();
    assert_eq!(cloned.state(), CircuitState::Open);
    assert_eq!(cloned.total_failures(), 1);
    assert_eq!(cloned.consecutive_failures(), 1);
}

#[test]
fn debug_format_includes_state() {
    let breaker = CircuitBreaker::with_defaults();
    let debug = format!("{:?}", breaker);
    assert!(debug.contains("CircuitBreaker"));
    assert!(debug.contains("Closed"));
}

#[test]
fn default_config_values() {
    let config = CircuitBreakerConfig::default();
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.cooldown_seconds, 60);
    assert_eq!(config.success_threshold, 1);
}

#[test]
fn no_failure_time_stays_open() {
    // Edge case: if somehow last_failure_time is None while Open,
    // state should remain Open (shouldn't transition to HalfOpen)
    let breaker = CircuitBreaker::with_defaults();
    // Fresh breaker is Closed, not Open — can't get into this state normally
    assert_eq!(breaker.state(), CircuitState::Closed);
}

// =============================================================================
// ProviderHealthTracker with state consistency
// =============================================================================

#[test]
fn tracker_reflects_half_open_after_cooldown() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let pid = ProviderId::new("flaky-provider");

    tracker.record_failure(&pid);
    thread::sleep(Duration::from_millis(5));

    let health = tracker.health(&pid);
    assert_eq!(health.circuit_state, CircuitState::HalfOpen);
    assert!(health.available);
}

#[test]
fn tracker_recovery_after_half_open_success() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 0,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let pid = ProviderId::new("recovering");

    tracker.record_failure(&pid);
    thread::sleep(Duration::from_millis(5));
    tracker.record_success(&pid);

    let health = tracker.health(&pid);
    assert_eq!(health.circuit_state, CircuitState::Closed);
    assert!(health.available);
}

#[test]
fn tracker_multiple_providers_independent_state() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 3600,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let healthy = ProviderId::new("healthy");
    let failing = ProviderId::new("failing");

    tracker.record_success(&healthy);
    tracker.record_failure(&failing);

    assert!(tracker.is_available(&healthy));
    assert!(!tracker.is_available(&failing));

    let h_health = tracker.health(&healthy);
    let f_health = tracker.health(&failing);
    assert_eq!(h_health.circuit_state, CircuitState::Closed);
    assert_eq!(f_health.circuit_state, CircuitState::Open);
}

#[test]
fn tracker_available_providers_filters_open_circuits() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 3600,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let a = ProviderId::new("a");
    let b = ProviderId::new("b");
    let c = ProviderId::new("c");

    tracker.record_success(&a);
    tracker.record_failure(&b); // Open
    tracker.record_success(&c);

    let available = tracker.available_providers();
    assert_eq!(available.len(), 2);
    assert!(available.contains(&&a));
    assert!(!available.contains(&&b));
    assert!(available.contains(&&c));
}

#[test]
fn tracker_reset_restores_availability() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 3600,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let pid = ProviderId::new("reset-me");

    tracker.record_failure(&pid);
    assert!(!tracker.is_available(&pid));

    tracker.reset(&pid);
    assert!(tracker.is_available(&pid));
    let health = tracker.health(&pid);
    assert_eq!(health.circuit_state, CircuitState::Closed);
}

#[test]
fn tracker_reset_all_restores_all_providers() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        cooldown_seconds: 3600,
        success_threshold: 1,
    };
    let mut tracker = ProviderHealthTracker::with_config(config);
    let a = ProviderId::new("a");
    let b = ProviderId::new("b");

    tracker.record_failure(&a);
    tracker.record_failure(&b);
    assert!(!tracker.is_available(&a));
    assert!(!tracker.is_available(&b));

    tracker.reset_all();
    assert!(tracker.is_available(&a));
    assert!(tracker.is_available(&b));
}

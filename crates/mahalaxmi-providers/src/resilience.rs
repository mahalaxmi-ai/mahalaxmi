//! Provider resilience — health checks, circuit breakers, and automatic failover.
//!
//! The `CircuitBreaker` tracks provider failures and automatically opens
//! the circuit (blocking requests) when a failure threshold is exceeded.
//! After a cooldown period, the circuit transitions to half-open, allowing
//! a single test request before fully closing or re-opening.
//!
//! The `ProviderHealthTracker` aggregates circuit breakers for all providers
//! and provides a unified interface for health checks and failover decisions.

use std::cell::Cell;
use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use mahalaxmi_core::types::ProviderId;

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed — requests flow normally.
    Closed,
    /// Circuit is open — requests are blocked (provider is considered down).
    Open,
    /// Circuit is half-open — one test request is allowed.
    HalfOpen,
}

/// Circuit breaker configuration.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit.
    pub failure_threshold: u32,
    /// Time to wait before transitioning from Open to HalfOpen (seconds).
    pub cooldown_seconds: u64,
    /// Number of consecutive successes in HalfOpen to close the circuit.
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            cooldown_seconds: 60,
            success_threshold: 1,
        }
    }
}

/// Circuit breaker for a single provider.
///
/// Uses `Cell<CircuitState>` for interior mutability so that the `state()`
/// method can transparently transition Open → HalfOpen after cooldown
/// while only requiring `&self`.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: Cell<CircuitState>,
    consecutive_failures: u32,
    consecutive_successes: u32,
    total_failures: u64,
    total_successes: u64,
    last_failure_time: Option<DateTime<Utc>>,
    last_state_change: DateTime<Utc>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Cell::new(CircuitState::Closed),
            consecutive_failures: 0,
            consecutive_successes: 0,
            total_failures: 0,
            total_successes: 0,
            last_failure_time: None,
            last_state_change: Utc::now(),
            config,
        }
    }

    /// Create a circuit breaker with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Get the current circuit state (automatically transitions Open → HalfOpen after cooldown).
    ///
    /// Uses interior mutability (`Cell`) to update the stored state when the
    /// cooldown expires, ensuring consistency between the returned value and
    /// the stored field.
    pub fn state(&self) -> CircuitState {
        let current = self.state.get();
        if current == CircuitState::Open {
            if let Some(last_fail) = self.last_failure_time {
                let cooldown = Duration::seconds(self.config.cooldown_seconds as i64);
                if Utc::now() - last_fail > cooldown {
                    self.state.set(CircuitState::HalfOpen);
                    return CircuitState::HalfOpen;
                }
            }
        }
        current
    }

    /// Check if the circuit allows a request.
    pub fn allows_request(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => false,
        }
    }

    /// Record a successful request.
    pub fn record_success(&mut self) {
        self.total_successes += 1;
        self.consecutive_failures = 0;
        self.consecutive_successes += 1;

        // state() handles Open → HalfOpen transition via Cell,
        // so we only need to handle HalfOpen → Closed here.
        if self.state() == CircuitState::HalfOpen
            && self.consecutive_successes >= self.config.success_threshold
        {
            self.state.set(CircuitState::Closed);
            self.last_state_change = Utc::now();
            self.consecutive_successes = 0;
        }
    }

    /// Record a failed request.
    pub fn record_failure(&mut self) {
        self.total_failures += 1;
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;
        self.last_failure_time = Some(Utc::now());

        if self.consecutive_failures >= self.config.failure_threshold {
            self.state.set(CircuitState::Open);
            self.last_state_change = Utc::now();
        }
    }

    /// Reset the circuit breaker to closed state.
    pub fn reset(&mut self) {
        self.state.set(CircuitState::Closed);
        self.consecutive_failures = 0;
        self.consecutive_successes = 0;
        self.last_failure_time = None;
        self.last_state_change = Utc::now();
    }

    /// Total failures since creation.
    pub fn total_failures(&self) -> u64 {
        self.total_failures
    }

    /// Total successes since creation.
    pub fn total_successes(&self) -> u64 {
        self.total_successes
    }

    /// Consecutive failures.
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }
}

/// Health status of a provider.
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Provider ID.
    pub provider_id: ProviderId,
    /// Circuit breaker state.
    pub circuit_state: CircuitState,
    /// Whether the provider can accept requests.
    pub available: bool,
    /// Total failures recorded.
    pub total_failures: u64,
    /// Total successes recorded.
    pub total_successes: u64,
}

/// Tracks health and circuit breaker state for all providers.
#[derive(Debug)]
pub struct ProviderHealthTracker {
    breakers: HashMap<ProviderId, CircuitBreaker>,
    config: CircuitBreakerConfig,
}

impl ProviderHealthTracker {
    /// Create a new health tracker with default circuit breaker config.
    pub fn new() -> Self {
        Self {
            breakers: HashMap::new(),
            config: CircuitBreakerConfig::default(),
        }
    }

    /// Create a health tracker with custom circuit breaker config.
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: HashMap::new(),
            config,
        }
    }

    /// Get or create the circuit breaker for a provider.
    fn get_or_create(&mut self, provider_id: &ProviderId) -> &mut CircuitBreaker {
        if !self.breakers.contains_key(provider_id) {
            self.breakers.insert(
                provider_id.clone(),
                CircuitBreaker::new(self.config.clone()),
            );
        }
        self.breakers.get_mut(provider_id).unwrap()
    }

    /// Record a successful request for a provider.
    pub fn record_success(&mut self, provider_id: &ProviderId) {
        self.get_or_create(provider_id).record_success();
    }

    /// Record a failed request for a provider.
    pub fn record_failure(&mut self, provider_id: &ProviderId) {
        self.get_or_create(provider_id).record_failure();
    }

    /// Check if a provider is available (circuit allows requests).
    pub fn is_available(&self, provider_id: &ProviderId) -> bool {
        self.breakers
            .get(provider_id)
            .is_none_or(|b| b.allows_request())
    }

    /// Get the health status of a provider.
    pub fn health(&self, provider_id: &ProviderId) -> ProviderHealth {
        match self.breakers.get(provider_id) {
            Some(breaker) => ProviderHealth {
                provider_id: provider_id.clone(),
                circuit_state: breaker.state(),
                available: breaker.allows_request(),
                total_failures: breaker.total_failures(),
                total_successes: breaker.total_successes(),
            },
            None => ProviderHealth {
                provider_id: provider_id.clone(),
                circuit_state: CircuitState::Closed,
                available: true,
                total_failures: 0,
                total_successes: 0,
            },
        }
    }

    /// Get health status for all tracked providers.
    pub fn all_health(&self) -> Vec<ProviderHealth> {
        self.breakers.keys().map(|id| self.health(id)).collect()
    }

    /// Get list of available providers.
    pub fn available_providers(&self) -> Vec<&ProviderId> {
        self.breakers
            .iter()
            .filter(|(_, b)| b.allows_request())
            .map(|(id, _)| id)
            .collect()
    }

    /// Reset all circuit breakers.
    pub fn reset_all(&mut self) {
        for breaker in self.breakers.values_mut() {
            breaker.reset();
        }
    }

    /// Reset a specific provider's circuit breaker.
    pub fn reset(&mut self, provider_id: &ProviderId) {
        if let Some(breaker) = self.breakers.get_mut(provider_id) {
            breaker.reset();
        }
    }

    /// Number of tracked providers.
    pub fn tracked_count(&self) -> usize {
        self.breakers.len()
    }
}

impl Default for ProviderHealthTracker {
    fn default() -> Self {
        Self::new()
    }
}

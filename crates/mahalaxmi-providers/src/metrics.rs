//! Provider performance metrics — latency, cost, and success rate tracking.
//!
//! The `PerformanceTracker` collects per-provider metrics for routing decisions,
//! cost estimation, and performance comparison. Metrics are accumulated in memory
//! and can be serialized for persistence.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use mahalaxmi_core::types::ProviderId;
use serde::{Deserialize, Serialize};

use crate::types::TaskType;

/// A single recorded metric event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEvent {
    /// Provider that handled the request.
    pub provider_id: ProviderId,
    /// Type of task performed.
    pub task_type: TaskType,
    /// Whether the request succeeded.
    pub success: bool,
    /// Latency in milliseconds.
    pub latency_ms: u64,
    /// Estimated tokens consumed (0 if unknown).
    pub tokens_used: u64,
    /// When the event was recorded.
    pub timestamp: DateTime<Utc>,
}

/// Aggregated metrics for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    /// Provider ID.
    pub provider_id: ProviderId,
    /// Total number of requests.
    pub total_requests: u64,
    /// Number of successful requests.
    pub successful_requests: u64,
    /// Number of failed requests.
    pub failed_requests: u64,
    /// Total latency across all requests (ms).
    pub total_latency_ms: u64,
    /// Minimum observed latency (ms).
    pub min_latency_ms: u64,
    /// Maximum observed latency (ms).
    pub max_latency_ms: u64,
    /// Total tokens consumed.
    pub total_tokens: u64,
    /// Per-task-type request counts.
    pub task_counts: HashMap<TaskType, u64>,
}

impl ProviderMetrics {
    fn new(provider_id: ProviderId) -> Self {
        Self {
            provider_id,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_latency_ms: 0,
            min_latency_ms: u64::MAX,
            max_latency_ms: 0,
            total_tokens: 0,
            task_counts: HashMap::new(),
        }
    }

    /// Average latency in milliseconds. Returns 0 if no requests recorded.
    pub fn avg_latency_ms(&self) -> u64 {
        if self.total_requests == 0 {
            0
        } else {
            self.total_latency_ms / self.total_requests
        }
    }

    /// Success rate as a fraction (0.0 to 1.0).
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    /// Average tokens per request.
    pub fn avg_tokens_per_request(&self) -> u64 {
        if self.total_requests == 0 {
            0
        } else {
            self.total_tokens / self.total_requests
        }
    }

    fn record(&mut self, event: &MetricEvent) {
        self.total_requests += 1;
        if event.success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        self.total_latency_ms += event.latency_ms;
        self.min_latency_ms = self.min_latency_ms.min(event.latency_ms);
        self.max_latency_ms = self.max_latency_ms.max(event.latency_ms);
        self.total_tokens += event.tokens_used;
        *self.task_counts.entry(event.task_type).or_insert(0) += 1;
    }
}

/// Provider performance comparison summary.
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// Provider ID.
    pub provider_id: ProviderId,
    /// Average latency (ms).
    pub avg_latency_ms: u64,
    /// Success rate (0.0 to 1.0).
    pub success_rate: f64,
    /// Total requests handled.
    pub total_requests: u64,
    /// Average tokens per request.
    pub avg_tokens: u64,
}

/// Tracks performance metrics for all providers.
#[derive(Debug, Default)]
pub struct PerformanceTracker {
    /// Per-provider aggregated metrics.
    metrics: HashMap<ProviderId, ProviderMetrics>,
    /// Full event history (bounded by max_history).
    history: Vec<MetricEvent>,
    /// Maximum number of events to retain in history.
    max_history: usize,
}

impl PerformanceTracker {
    /// Create a new performance tracker.
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            history: Vec::new(),
            max_history: 10_000,
        }
    }

    /// Create a tracker with a custom history limit.
    pub fn with_max_history(max: usize) -> Self {
        Self {
            metrics: HashMap::new(),
            history: Vec::new(),
            max_history: max,
        }
    }

    /// Record a metric event.
    pub fn record(&mut self, event: MetricEvent) {
        let metrics = self
            .metrics
            .entry(event.provider_id.clone())
            .or_insert_with(|| ProviderMetrics::new(event.provider_id.clone()));
        metrics.record(&event);

        if self.history.len() < self.max_history {
            self.history.push(event);
        }
    }

    /// Record a request completion.
    pub fn record_request(
        &mut self,
        provider_id: ProviderId,
        task_type: TaskType,
        success: bool,
        latency_ms: u64,
        tokens_used: u64,
    ) {
        self.record(MetricEvent {
            provider_id,
            task_type,
            success,
            latency_ms,
            tokens_used,
            timestamp: Utc::now(),
        });
    }

    /// Get aggregated metrics for a provider.
    pub fn get_metrics(&self, provider_id: &ProviderId) -> Option<&ProviderMetrics> {
        self.metrics.get(provider_id)
    }

    /// Get metrics for all tracked providers.
    pub fn all_metrics(&self) -> Vec<&ProviderMetrics> {
        self.metrics.values().collect()
    }

    /// Get the full event history.
    pub fn history(&self) -> &[MetricEvent] {
        &self.history
    }

    /// Compare performance across all tracked providers.
    pub fn compare(&self) -> Vec<PerformanceComparison> {
        let mut comparisons: Vec<_> = self
            .metrics
            .values()
            .map(|m| PerformanceComparison {
                provider_id: m.provider_id.clone(),
                avg_latency_ms: m.avg_latency_ms(),
                success_rate: m.success_rate(),
                total_requests: m.total_requests,
                avg_tokens: m.avg_tokens_per_request(),
            })
            .collect();
        comparisons.sort_by(|a, b| {
            b.success_rate
                .partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.avg_latency_ms.cmp(&b.avg_latency_ms))
        });
        comparisons
    }

    /// Number of tracked providers.
    pub fn tracked_count(&self) -> usize {
        self.metrics.len()
    }

    /// Total number of recorded events.
    pub fn total_events(&self) -> usize {
        self.history.len()
    }

    /// Export all metrics as serializable data.
    pub fn export_metrics(&self) -> HashMap<ProviderId, ProviderMetrics> {
        self.metrics.clone()
    }

    /// Export event history as serializable data.
    pub fn export_history(&self) -> Vec<MetricEvent> {
        self.history.clone()
    }

    /// Clear all metrics and history.
    pub fn clear(&mut self) {
        self.metrics.clear();
        self.history.clear();
    }
}

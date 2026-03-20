//! Pre-configured tracing spans for Mahalaxmi subsystems.
//!
//! These span helpers ensure consistent structured logging across the application.
//! Each span includes relevant context fields for filtering and analysis.
//!
//! # Usage
//!
//! ```
//! use mahalaxmi_core::logging::spans;
//!
//! let span = spans::worker_span(1, "compile project");
//! let _enter = span.enter();
//! // Work runs inside the span context.
//! ```

use tracing::{info_span, Span};

/// Create a span for an orchestration cycle.
///
/// Fields: `cycle_id`
pub fn orchestration_span(cycle_id: &str) -> Span {
    info_span!("orchestration", cycle_id = %cycle_id)
}

/// Create a span for a terminal session.
///
/// Fields: `terminal_id`, `role` (manager/worker)
pub fn terminal_span(terminal_id: &str, role: &str) -> Span {
    info_span!("terminal", terminal_id = %terminal_id, role = %role)
}

/// Create a span for an AI provider operation.
///
/// Fields: `provider` (e.g., "claude-code", "openai-foundry")
pub fn provider_span(provider_name: &str) -> Span {
    info_span!("provider", provider = %provider_name)
}

/// Create a span for a worker task.
///
/// Fields: `worker_id`, `task`
pub fn worker_span(worker_id: u32, task_summary: &str) -> Span {
    info_span!("worker", worker_id = %worker_id, task = %task_summary)
}

/// Create a span for template operations.
///
/// Fields: `template_id`, `category`
pub fn template_span(template_id: &str, category: &str) -> Span {
    info_span!("template", template_id = %template_id, category = %category)
}

/// Create a span for configuration operations.
///
/// Fields: `source` (e.g., "file", "defaults", "env")
pub fn config_span(source: &str) -> Span {
    info_span!("config", source = %source)
}

/// Create a span for licensing operations.
///
/// Fields: `operation` (e.g., "check_license", "activate_trial", "validate")
pub fn licensing_span(operation: &str) -> Span {
    info_span!("licensing", operation = %operation)
}

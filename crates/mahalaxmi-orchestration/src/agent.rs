//! Agent specifications and performance tracking.
//!
//! Defines reusable agent identities (specialist personas) that can be assigned
//! to workers. Each agent spec carries a system preamble that shapes the AI's
//! behavior, plus domain tags for task matching.
//!
//! The [`AgentRegistry`] tracks accuracy metrics across invocations to measure
//! whether agent specialization improves worker output quality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A reusable agent identity with specialist persona and accumulated metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    /// Unique identifier (e.g., "rust-backend-specialist").
    pub id: String,
    /// Human-readable display name (e.g., "Rust Backend Specialist").
    pub name: String,
    /// System prompt preamble that establishes the agent's persona and expertise.
    pub system_preamble: String,
    /// Preferred provider for this agent type (hint, not hard constraint).
    pub preferred_provider: Option<String>,
    /// Domain tags for matching (e.g., ["rust", "backend", "api"]).
    pub domain_tags: Vec<String>,
}

/// Tracks accuracy metrics for an agent spec across invocations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    /// Agent ID this record belongs to.
    pub agent_id: String,
    /// Total tasks assigned to this agent.
    pub total_tasks: u32,
    /// Tasks completed successfully.
    pub successful_tasks: u32,
    /// Tasks that failed.
    pub failed_tasks: u32,
    /// Cumulative verification score (sum of per-task scores).
    pub total_verification_score: f64,
    /// When the agent was last used.
    pub last_used: DateTime<Utc>,
}

impl AgentRecord {
    /// Create a new record for the given agent.
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            total_tasks: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            total_verification_score: 0.0,
            last_used: Utc::now(),
        }
    }

    /// Returns the success rate as a fraction (0.0 to 1.0).
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            return 0.0;
        }
        self.successful_tasks as f64 / self.total_tasks as f64
    }

    /// Returns the average verification score.
    pub fn avg_verification_score(&self) -> f64 {
        if self.successful_tasks == 0 {
            return 0.0;
        }
        self.total_verification_score / self.successful_tasks as f64
    }

    /// Record a successful task completion.
    pub fn record_success(&mut self, verification_score: f64) {
        self.total_tasks += 1;
        self.successful_tasks += 1;
        self.total_verification_score += verification_score;
        self.last_used = Utc::now();
    }

    /// Record a failed task.
    pub fn record_failure(&mut self) {
        self.total_tasks += 1;
        self.failed_tasks += 1;
        self.last_used = Utc::now();
    }
}

/// In-memory registry of agent specs and their performance records.
///
/// This is intentionally not persistent — specs and records live for the
/// duration of the application session. Persistence is deferred to Phase 3.
pub struct AgentRegistry {
    specs: HashMap<String, AgentSpec>,
    records: HashMap<String, AgentRecord>,
}

impl AgentRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            specs: HashMap::new(),
            records: HashMap::new(),
        }
    }

    /// Register a new agent spec (or update an existing one).
    pub fn register(&mut self, spec: AgentSpec) {
        self.specs.insert(spec.id.clone(), spec);
    }

    /// Get an agent spec by ID.
    pub fn get(&self, id: &str) -> Option<&AgentSpec> {
        self.specs.get(id)
    }

    /// Get a spec if it exists, or register the provided one and return it.
    pub fn get_or_create(&mut self, spec: AgentSpec) -> &AgentSpec {
        let id = spec.id.clone();
        self.specs.entry(id.clone()).or_insert(spec);
        self.specs.get(&id).unwrap()
    }

    /// Record the outcome of a task assigned to an agent.
    pub fn record_result(&mut self, agent_id: &str, success: bool, verification_score: f64) {
        let record = self
            .records
            .entry(agent_id.to_string())
            .or_insert_with(|| AgentRecord::new(agent_id));

        if success {
            record.record_success(verification_score);
        } else {
            record.record_failure();
        }
    }

    /// Get the performance record for an agent.
    pub fn get_record(&self, agent_id: &str) -> Option<&AgentRecord> {
        self.records.get(agent_id)
    }

    /// Find the best agent for a set of domain tags, based on tag overlap and success rate.
    ///
    /// Returns `None` if no agents have matching tags.
    pub fn best_agent_for_tags(&self, tags: &[String]) -> Option<&AgentSpec> {
        self.specs
            .values()
            .filter(|spec| spec.domain_tags.iter().any(|t| tags.contains(t)))
            .max_by(|a, b| {
                let a_overlap = a.domain_tags.iter().filter(|t| tags.contains(t)).count();
                let b_overlap = b.domain_tags.iter().filter(|t| tags.contains(t)).count();

                let a_rate = self
                    .records
                    .get(&a.id)
                    .map(|r| r.success_rate())
                    .unwrap_or(0.5); // Neutral for new agents
                let b_rate = self
                    .records
                    .get(&b.id)
                    .map(|r| r.success_rate())
                    .unwrap_or(0.5);

                // Score = tag_overlap * 10 + success_rate * 5
                let a_score = (a_overlap as f64) * 10.0 + a_rate * 5.0;
                let b_score = (b_overlap as f64) * 10.0 + b_rate * 5.0;

                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// List all registered agent specs.
    pub fn list_specs(&self) -> Vec<&AgentSpec> {
        self.specs.values().collect()
    }

    /// List all performance records.
    pub fn list_records(&self) -> Vec<&AgentRecord> {
        self.records.values().collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

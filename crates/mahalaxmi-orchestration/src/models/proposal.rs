use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::developer::DeveloperId;
use mahalaxmi_core::types::ManagerId;
use mahalaxmi_core::MahalaxmiResult;
use serde::{Deserialize, Serialize};

/// A single task proposed by a manager during the analysis phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedTask {
    /// Human-readable title of the task.
    pub title: String,
    /// Detailed description of the work to be done.
    pub description: String,
    /// Estimated complexity (1-10 scale).
    pub complexity: u32,
    /// Priority within the proposal (lower number = higher priority).
    pub priority: u32,
    /// Task IDs that this task depends on.
    pub dependencies: Vec<String>,
    /// Files this task is expected to modify.
    pub affected_files: Vec<String>,
    /// AI-suggested task type category (used as hint for classifier).
    pub task_type: Option<String>,
    /// Optional agent spec for this task. Manager can "create" a new spec or
    /// "reuse" an existing one by referencing its ID.
    #[serde(default)]
    pub agent_spec: Option<ProposedAgentSpec>,
    /// Testable acceptance criteria that define when this task is complete.
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
}

/// An agent spec proposed by a manager for a specific task.
///
/// When `action` is `"create"`, all fields are required to define a new agent.
/// When `action` is `"reuse"`, only `id` is required to reference an existing agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAgentSpec {
    /// "create" or "reuse".
    pub action: String,
    /// Agent ID (must match existing for "reuse", new for "create").
    pub id: String,
    /// Display name (required for "create").
    pub name: Option<String>,
    /// Persona/expertise preamble (required for "create").
    pub system_preamble: Option<String>,
    /// Domain tags for matching.
    #[serde(default)]
    pub domain_tags: Vec<String>,
}

impl ProposedTask {
    /// Create a new proposed task with the given title and description.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            complexity: 5,
            priority: 100,
            dependencies: Vec::new(),
            affected_files: Vec::new(),
            task_type: None,
            agent_spec: None,
            acceptance_criteria: Vec::new(),
        }
    }

    /// Set the complexity score.
    pub fn with_complexity(mut self, complexity: u32) -> Self {
        self.complexity = complexity;
        self
    }

    /// Set the priority.
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a dependency on another task.
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Add an affected file path.
    pub fn with_affected_file(mut self, path: impl Into<String>) -> Self {
        self.affected_files.push(path.into());
        self
    }

    /// Add a testable acceptance criterion.
    pub fn with_acceptance_criterion(mut self, criterion: impl Into<String>) -> Self {
        self.acceptance_criteria.push(criterion.into());
        self
    }

    /// Generate a normalized key for matching this task across proposals.
    ///
    /// Normalizes the title to lowercase, trims whitespace, and replaces
    /// sequences of whitespace/punctuation with a single hyphen.
    pub fn normalized_key(&self) -> String {
        self.title
            .to_lowercase()
            .trim()
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

/// A complete proposal from a single manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerProposal {
    /// ID of the manager that produced this proposal.
    pub manager_id: ManagerId,
    /// The tasks proposed by this manager.
    pub tasks: Vec<ProposedTask>,
    /// Whether the manager completed successfully.
    pub completed: bool,
    /// Optional error message if the manager failed.
    pub error_message: Option<String>,
    /// Duration in milliseconds the manager took to produce this proposal.
    pub duration_ms: u64,
    /// A 1–3 word, lowercase, hyphen-separated slug describing the overall
    /// work theme for this cycle (e.g. `"security-hardening"`).
    /// Produced by the AI manager and used as the git branch prefix so that
    /// PRs are grouped under `mahalaxmi/{cycle_label}/worker-{n}-{task}`.
    #[serde(default)]
    pub cycle_label: Option<String>,
    /// Developer identifier for the developer who spawned this manager.
    ///
    /// `None` for single-developer cycles (backward-compatible default).
    /// Set by the orchestration service when multi-developer sessions are active.
    #[serde(default)]
    pub developer_id: Option<DeveloperId>,
    /// Consensus weight for this manager's proposals, derived from the
    /// developer's configured weight in the `DeveloperRegistry`.
    ///
    /// Defaults to `1.0` (equal weight for all managers when no explicit
    /// developer association is configured). Set by the orchestration service
    /// at proposal submission time after looking up the developer's weight.
    #[serde(default = "default_developer_weight")]
    pub developer_weight: f32,
}

fn default_developer_weight() -> f32 {
    1.0
}

impl ManagerProposal {
    /// Create a new successful proposal from a manager.
    pub fn new(manager_id: ManagerId, tasks: Vec<ProposedTask>, duration_ms: u64) -> Self {
        Self {
            manager_id,
            tasks,
            completed: true,
            error_message: None,
            duration_ms,
            cycle_label: None,
            developer_id: None,
            developer_weight: 1.0,
        }
    }

    /// Create a failed proposal (manager errored out).
    pub fn failed(manager_id: ManagerId, error: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            manager_id,
            tasks: Vec::new(),
            completed: false,
            error_message: Some(error.into()),
            duration_ms,
            cycle_label: None,
            developer_id: None,
            developer_weight: 1.0,
        }
    }

    /// Parse a `ManagerProposal` from raw JSON output produced by an AI manager.
    ///
    /// The expected JSON format is:
    /// ```json
    /// {
    ///   "tasks": [
    ///     {
    ///       "title": "Task title",
    ///       "description": "Task description",
    ///       "complexity": 5,
    ///       "priority": 1,
    ///       "dependencies": ["other-task-key"],
    ///       "affected_files": ["src/main.rs"]
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// Behavior:
    /// - Missing `complexity` defaults to 5, values are clamped to 1-10
    /// - Missing `priority` defaults to 100
    /// - Missing `dependencies` and `affected_files` default to empty arrays
    /// - Empty task titles produce an error
    /// - Empty tasks array produces an error
    /// - Extra/unknown JSON fields are ignored
    pub fn from_json(
        json: &str,
        manager_id: ManagerId,
        duration_ms: u64,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Self> {
        let raw: RawProposal = serde_json::from_str(json).map_err(|e| {
            MahalaxmiError::orchestration(
                i18n,
                "error-json-parse-failed",
                &[("reason", &e.to_string())],
            )
        })?;

        if raw.tasks.is_empty() {
            return Err(MahalaxmiError::orchestration(
                i18n,
                "error-empty-proposal",
                &[("manager", manager_id.as_str())],
            ));
        }

        let mut tasks = Vec::with_capacity(raw.tasks.len());
        for (i, raw_task) in raw.tasks.into_iter().enumerate() {
            let title = raw_task.title.trim().to_string();
            if title.is_empty() {
                return Err(MahalaxmiError::orchestration(
                    i18n,
                    "error-empty-task-title",
                    &[("index", &i.to_string())],
                ));
            }

            let complexity = raw_task.complexity.unwrap_or(5).clamp(1, 10);
            let priority = raw_task.priority.unwrap_or(100);
            let description = raw_task.description.unwrap_or_default();
            let dependencies = raw_task.dependencies.unwrap_or_default();
            let affected_files = raw_task.affected_files.unwrap_or_default();
            let task_type = raw_task.task_type;
            let agent_spec = raw_task.agent_spec;
            let acceptance_criteria = raw_task.acceptance_criteria.unwrap_or_default();

            tasks.push(ProposedTask {
                title,
                description,
                complexity,
                priority,
                dependencies,
                affected_files,
                task_type,
                agent_spec,
                acceptance_criteria,
            });
        }

        let mut proposal = Self::new(manager_id, tasks, duration_ms);
        proposal.cycle_label = raw.cycle_label.filter(|s| !s.trim().is_empty());
        Ok(proposal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahalaxmi_core::i18n::{locale::SupportedLocale, I18nService};
    use mahalaxmi_core::types::ManagerId;

    fn mgr() -> ManagerId {
        ManagerId::new("manager-0")
    }

    fn i18n() -> I18nService {
        I18nService::new(SupportedLocale::EnUs)
    }

    /// Verifies that `developer_id` round-trips through serde and that old
    /// JSON without the field deserializes with `developer_id = None`
    /// (backward compatibility).
    #[test]
    fn test_manager_proposal_developer_id_serde() {
        let mut proposal = ManagerProposal::new(ManagerId::new("manager-0"), vec![], 100);
        proposal.developer_id = Some(DeveloperId::from("alice"));

        let json = serde_json::to_string(&proposal).unwrap();
        let deserialized: ManagerProposal = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.developer_id, Some(DeveloperId::from("alice")));
        assert!((deserialized.developer_weight - 1.0).abs() < f32::EPSILON);

        // Old JSON without developer_id / developer_weight must deserialize
        // to developer_id = None and developer_weight = 1.0 (defaults).
        let legacy_json = r#"{
            "manager_id": "manager-0",
            "tasks": [],
            "completed": true,
            "error_message": null,
            "duration_ms": 100
        }"#;
        let legacy: ManagerProposal = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(legacy.developer_id, None);
        assert!((legacy.developer_weight - 1.0).abs() < f32::EPSILON);
    }

    // ── Empty-proposal anomaly regression ─────────────────────────────────────

    /// `from_json` must reject the exact 13-byte output observed from manager-0
    /// in production (cycle 14b90837, 2026-03-01).
    #[test]
    fn from_json_empty_tasks_is_error_empty_proposal() {
        let result = ManagerProposal::from_json(r#"{"tasks": []}"#, mgr(), 11_829, &i18n());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("error-empty-proposal"),
            "Expected error-empty-proposal, got: {err}"
        );
    }

    /// A task with an empty (whitespace-only) title is also invalid.
    #[test]
    fn from_json_blank_title_is_error() {
        let json = r#"{"tasks": [{"title": "   "}]}"#;
        let result = ManagerProposal::from_json(json, mgr(), 100, &i18n());
        assert!(result.is_err());
    }

    // ── Happy path ────────────────────────────────────────────────────────────

    /// A minimal valid task (only title required by RawTask) succeeds and
    /// fills in default values for optional fields.
    #[test]
    fn from_json_minimal_task_uses_defaults() {
        let json = r#"{"tasks": [{"title": "Do something"}]}"#;
        let proposal = ManagerProposal::from_json(json, mgr(), 5_000, &i18n()).unwrap();
        assert_eq!(proposal.tasks.len(), 1);
        let task = &proposal.tasks[0];
        assert_eq!(task.title, "Do something");
        assert_eq!(task.complexity, 5); // default
        assert_eq!(task.priority, 100); // default
        assert!(task.dependencies.is_empty());
        assert!(task.affected_files.is_empty());
    }

    /// complexity is clamped to 1-10.
    #[test]
    fn from_json_clamps_complexity() {
        let json = r#"{"tasks": [{"title": "T", "complexity": 99}]}"#;
        let proposal = ManagerProposal::from_json(json, mgr(), 100, &i18n()).unwrap();
        assert_eq!(proposal.tasks[0].complexity, 10);

        let json_low = r#"{"tasks": [{"title": "T", "complexity": 0}]}"#;
        let proposal_low = ManagerProposal::from_json(json_low, mgr(), 100, &i18n()).unwrap();
        assert_eq!(proposal_low.tasks[0].complexity, 1);
    }

    /// Unknown JSON fields in AI output must not cause a parse error.
    #[test]
    fn from_json_ignores_unknown_fields() {
        let json = r#"{"tasks": [{"title": "T", "unknown_future_field": true}]}"#;
        let result = ManagerProposal::from_json(json, mgr(), 100, &i18n());
        assert!(result.is_ok(), "Unknown fields should be silently ignored");
    }
}

/// Intermediate deserialization struct for lenient JSON parsing.
///
/// All fields except `title` are optional to handle incomplete AI output.
/// Uses `#[serde(deny_unknown_fields)]` is intentionally NOT applied —
/// extra fields from AI output are silently ignored.
#[derive(Deserialize)]
struct RawProposal {
    tasks: Vec<RawTask>,
    #[serde(default)]
    cycle_label: Option<String>,
}

#[derive(Deserialize)]
struct RawTask {
    title: String,
    description: Option<String>,
    complexity: Option<u32>,
    priority: Option<u32>,
    dependencies: Option<Vec<String>>,
    affected_files: Option<Vec<String>>,
    task_type: Option<String>,
    agent_spec: Option<ProposedAgentSpec>,
    acceptance_criteria: Option<Vec<String>>,
}

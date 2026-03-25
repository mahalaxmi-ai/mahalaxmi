use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::developer::DeveloperId;
use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_core::MahalaxmiResult;
use mahalaxmi_providers::TaskType;
use serde::{Deserialize, Serialize};

use super::consensus::ConsensusResult;

/// Indicates the origin of a worker task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskSource {
    /// Task was produced by a manager during the planning phase.
    #[default]
    Manager,
    /// Task was carried forward from a previous cycle due to failure.
    CarryForward,
}

/// Priority level for a worker task batch slot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BatchPriority {
    /// Standard execution priority.
    #[default]
    Normal,
    /// Elevated priority; carry-forward tasks use this to run before new work.
    High,
}

/// A single worker task within an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerTask {
    /// Unique task identifier.
    pub task_id: TaskId,
    /// Assigned worker ID.
    pub worker_id: WorkerId,
    /// Human-readable title.
    pub title: String,
    /// Detailed task description / prompt for the AI worker.
    pub description: String,
    /// Task IDs this task depends on (must complete first).
    pub dependencies: Vec<TaskId>,
    /// Estimated complexity (1-10).
    pub complexity: u32,
    /// Files this task is expected to modify.
    pub affected_files: Vec<String>,
    /// Context preamble injected before worker activation (populated lazily).
    pub context_preamble: Option<String>,
    /// Classified task type for multi-provider routing.
    pub task_type: Option<TaskType>,
    /// Testable acceptance criteria carried from consensus.
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    /// Origin of this task (manager-produced vs. carried forward).
    #[serde(default)]
    pub source: TaskSource,
    /// Batch scheduling priority for this task.
    #[serde(default)]
    pub priority: BatchPriority,
}

impl WorkerTask {
    /// Create a new worker task with default source and priority.
    pub fn new(
        task_id: TaskId,
        worker_id: WorkerId,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            worker_id,
            title: title.into(),
            description: description.into(),
            dependencies: Vec::new(),
            complexity: 5,
            affected_files: Vec::new(),
            context_preamble: None,
            task_type: None,
            acceptance_criteria: Vec::new(),
            source: TaskSource::default(),
            priority: BatchPriority::default(),
        }
    }

    /// Set the classified task type for routing.
    pub fn with_task_type(mut self, task_type: TaskType) -> Self {
        self.task_type = Some(task_type);
        self
    }

    /// Add a dependency on another task.
    pub fn with_dependency(mut self, dep: TaskId) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Set the complexity score.
    pub fn with_complexity(mut self, complexity: u32) -> Self {
        self.complexity = complexity;
        self
    }

    /// Add an affected file path.
    pub fn with_affected_file(mut self, path: impl Into<String>) -> Self {
        self.affected_files.push(path.into());
        self
    }

    /// Returns true if this task has no dependencies.
    pub fn has_no_dependencies(&self) -> bool {
        self.dependencies.is_empty()
    }
}

/// A phase of execution — a group of tasks that can run in parallel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPhase {
    /// Phase number (0-indexed).
    pub phase_number: u32,
    /// Tasks in this phase (can run in parallel).
    pub tasks: Vec<WorkerTask>,
}

/// A complete execution plan — the DAG of worker tasks organized into phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Ordered list of execution phases.
    pub phases: Vec<ExecutionPhase>,
    /// Resolved cycle label from manager consensus — used as the git branch
    /// path prefix (`mahalaxmi/{cycle_label}/worker-{n}-{task}`).
    /// `None` when no manager produced a valid label.
    #[serde(default)]
    pub cycle_label: Option<String>,
    /// Developer IDs that contributed at least one adopted task to this plan.
    ///
    /// Populated by the orchestration service after consensus: contains every
    /// `DeveloperId` whose manager proposal(s) contributed a task that made it
    /// into `agreed_tasks`. Empty for single-developer cycles.
    #[serde(default)]
    pub contributing_developers: Vec<DeveloperId>,
}

impl ExecutionPlan {
    /// Create an empty execution plan.
    pub fn new() -> Self {
        Self {
            phases: Vec::new(),
            cycle_label: None,
            contributing_developers: Vec::new(),
        }
    }

    /// Create an execution plan from a list of phases.
    pub fn from_phases(phases: Vec<ExecutionPhase>) -> Self {
        Self {
            phases,
            cycle_label: None,
            contributing_developers: Vec::new(),
        }
    }

    /// Get all worker tasks across all phases.
    pub fn all_workers(&self) -> Vec<&WorkerTask> {
        self.phases.iter().flat_map(|p| &p.tasks).collect()
    }

    /// Validate the execution plan.
    ///
    /// Checks:
    /// - All task dependencies reference tasks that exist in the plan
    /// - Dependencies only reference tasks in earlier phases
    /// - No duplicate task IDs
    /// - No duplicate worker IDs
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let mut seen_task_ids = std::collections::HashSet::new();
        let mut seen_worker_ids = std::collections::HashSet::new();
        let mut completed_task_ids = std::collections::HashSet::new();

        for phase in &self.phases {
            for task in &phase.tasks {
                if !seen_task_ids.insert(task.task_id.clone()) {
                    errors.push(format!("Duplicate task ID: {}", task.task_id));
                }
                if !seen_worker_ids.insert(task.worker_id) {
                    errors.push(format!("Duplicate worker ID: {}", task.worker_id));
                }
                for dep in &task.dependencies {
                    if !completed_task_ids.contains(dep) {
                        errors.push(format!(
                            "Task {} depends on {} which is not in an earlier phase",
                            task.task_id, dep
                        ));
                    }
                }
            }
            for task in &phase.tasks {
                completed_task_ids.insert(task.task_id.clone());
            }
        }

        errors
    }

    /// Get the number of phases.
    pub fn phase_count(&self) -> usize {
        self.phases.len()
    }

    /// Convert a `ConsensusResult` into an `ExecutionPlan`.
    ///
    /// Each agreed task becomes a `WorkerTask` with a unique worker ID.
    /// Tasks are organized into phases based on their dependency graph
    /// using topological ordering — tasks with no dependencies go first,
    /// and tasks whose dependencies are all in earlier phases are grouped
    /// together for parallel execution.
    ///
    /// Returns an empty plan if there are no agreed tasks.
    pub fn from_consensus_result(
        result: &ConsensusResult,
        i18n: &I18nService,
    ) -> MahalaxmiResult<Self> {
        if result.agreed_tasks.is_empty() {
            return Ok(Self::new());
        }

        // Map normalized_key → task_id for dependency resolution
        let key_to_task_id: std::collections::HashMap<String, TaskId> = result
            .agreed_tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                (
                    task.normalized_key.clone(),
                    TaskId::new(format!("task-{}", i)),
                )
            })
            .collect();

        // Convert ConsensusTask → WorkerTask
        let mut worker_tasks: Vec<WorkerTask> = result
            .agreed_tasks
            .iter()
            .enumerate()
            .map(|(i, ct)| {
                let task_id = TaskId::new(format!("task-{}", i));
                let worker_id = WorkerId::new(i as u32);

                let mut wt = WorkerTask::new(task_id, worker_id, &ct.title, &ct.description);
                wt.complexity = ct.average_complexity.round().clamp(1.0, 10.0) as u32;
                wt.affected_files = ct.affected_files.clone();
                wt.acceptance_criteria = ct.acceptance_criteria.clone();

                // Resolve string dependencies to TaskIds
                for dep_key in &ct.dependencies {
                    if let Some(dep_task_id) = key_to_task_id.get(dep_key) {
                        wt.dependencies.push(dep_task_id.clone());
                    }
                }

                wt
            })
            .collect();

        // REQ-005: inject synthetic dependency edges for tasks that share files.
        // Must run before build_phases() so the topological sort sees the edges.
        crate::dag::add_file_conflict_edges(&mut worker_tasks);

        // Build phases using the DAG utilities
        let phases = crate::dag::build_phases(&worker_tasks, i18n)?;
        let mut plan = Self::from_phases(phases);
        plan.cycle_label = result.cycle_label.clone();
        Ok(plan)
    }
}

impl Default for ExecutionPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Path prefixes that belong exclusively to the ThriveTech product platform
/// (Node.js services, database migrations, Stripe scripts, Bull workers).
/// Workers operating inside the `mahalaxmi-terminal-automation` worktree must
/// never touch these paths — they live in a separate repository and require
/// platform credentials that are not present in AI coding agent sessions.
const PLATFORM_PATH_PREFIXES: &[&str] = &[
    "services/",
    "db/migrations/",
    "database/migrations/",
    "packages/",
    "scripts/stripe",
    "scripts/seed",
];

impl ExecutionPlan {
    /// Remove any tasks whose `affected_files` exclusively contain paths that
    /// fall outside the current worktree scope.
    ///
    /// Called after consensus to enforce the routing constraint: tasks touching
    /// platform-only paths (`services/`, `db/migrations/`, etc.) are stripped
    /// before the plan reaches the worker queue.  A task is only filtered if
    /// *all* of its affected files are out-of-scope — tasks that mix in-scope
    /// and out-of-scope files are kept so that valid work is not silently
    /// discarded (the worker will simply skip the unreachable paths).
    ///
    /// Returns the number of tasks removed.
    pub fn filter_platform_scoped_tasks(&mut self) -> usize {
        let before: usize = self.phases.iter().map(|p| p.tasks.len()).sum();

        for phase in &mut self.phases {
            phase.tasks.retain(|task| {
                if task.affected_files.is_empty() {
                    return true;
                }
                // Keep the task if at least one affected file is NOT a platform path.
                task.affected_files
                    .iter()
                    .any(|f| !is_platform_path(f))
            });
        }

        // Drop phases that became empty after filtering.
        self.phases.retain(|p| !p.tasks.is_empty());

        let after: usize = self.phases.iter().map(|p| p.tasks.len()).sum();
        before - after
    }
}

/// Returns `true` when `path` starts with a known platform-only prefix.
fn is_platform_path(path: &str) -> bool {
    PLATFORM_PATH_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ExecutionPlan::default()` must initialise `contributing_developers` as
    /// an empty `Vec` and the field must round-trip through JSON when absent.
    #[test]
    fn test_execution_plan_contributing_developers_default() {
        let plan = ExecutionPlan::default();
        assert!(
            plan.contributing_developers.is_empty(),
            "contributing_developers must default to an empty Vec"
        );

        // Deserialize a JSON payload that omits contributing_developers —
        // backward-compatible plans must still parse correctly.
        let json = r#"{"phases":[]}"#;
        let deserialized: ExecutionPlan = serde_json::from_str(json).expect("deserialize");
        assert!(
            deserialized.contributing_developers.is_empty(),
            "contributing_developers must deserialize as empty when absent from JSON"
        );
    }
}

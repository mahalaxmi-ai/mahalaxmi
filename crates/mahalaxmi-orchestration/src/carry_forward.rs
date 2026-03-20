//! Carry-forward queue for failed tasks across orchestration cycles.
//!
//! When a worker task fails permanently (exhausting its retry budget), it is
//! placed into a [`CarryForwardQueue`].  At the start of the next cycle the
//! queue can be drained into [`WorkerTask`]s that are prepended to the new
//! [`ExecutionPlan`] with elevated priority so they run before fresh manager
//! tasks.

use mahalaxmi_core::types::{TaskId, WorkerId};

use crate::models::plan::{BatchPriority, ExecutionPhase, ExecutionPlan, TaskSource, WorkerTask};

/// A task that failed permanently in a previous orchestration cycle and is
/// eligible to be retried in the next cycle.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FailedTask {
    /// Human-readable description of the work that was attempted.
    #[serde(default)]
    pub task_description: String,
    /// Stable hash derived from the task content — used for deduplication.
    #[serde(default)]
    pub task_hash: String,
    /// Index of the execution batch in which the task last failed.
    #[serde(default)]
    pub failed_in_batch: usize,
    /// Total number of times this task has been recorded as a permanent failure.
    #[serde(default)]
    pub attempt_count: usize,
}

/// A queue of tasks that failed in a previous cycle and should be retried.
///
/// Tasks can be pushed individually via [`CarryForwardQueue::push`] or
/// registered through [`CarryForwardQueue::record_permanent_failure`] which
/// deduplicates by `task_hash` and increments `attempt_count` on repeated
/// failures.  Call [`CarryForwardQueue::drain_as_worker_tasks`] to convert the
/// queue into [`WorkerTask`]s ready for execution, or
/// [`CarryForwardQueue::drain_to_entries`] to retrieve the raw [`FailedTask`]
/// entries.
pub struct CarryForwardQueue {
    tasks: Vec<FailedTask>,
}

impl CarryForwardQueue {
    /// Create an empty carry-forward queue.
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Push a pre-built [`FailedTask`] onto the queue.
    pub fn push(&mut self, task: FailedTask) {
        self.tasks.push(task);
    }

    /// Record a permanent failure, deduplicating by `task_hash`.
    ///
    /// If a task with the same `task_hash` is already in the queue its
    /// `attempt_count` is incremented and `failed_in_batch` is updated.
    /// Otherwise a new [`FailedTask`] is appended with `attempt_count = 1`.
    pub fn record_permanent_failure(
        &mut self,
        task_description: impl Into<String>,
        task_hash: impl Into<String>,
        failed_in_batch: usize,
    ) {
        let description = task_description.into();
        let hash = task_hash.into();

        if let Some(existing) = self.tasks.iter_mut().find(|t| t.task_hash == hash) {
            existing.attempt_count += 1;
            existing.failed_in_batch = failed_in_batch;
        } else {
            self.tasks.push(FailedTask {
                task_description: description,
                task_hash: hash,
                failed_in_batch,
                attempt_count: 1,
            });
        }
    }

    /// Drain the queue and return all entries as [`WorkerTask`]s.
    ///
    /// Each resulting task has:
    /// - `source` set to [`TaskSource::CarryForward`]
    /// - `priority` set to [`BatchPriority::High`]
    /// - `task_id` of the form `carry-{first_8_chars_of_hash}`
    /// - `description` taken from [`FailedTask::task_description`]
    /// - empty `dependencies` and `affected_files`
    ///
    /// After this call the queue is empty.
    pub fn drain_as_worker_tasks(&mut self) -> Vec<WorkerTask> {
        std::mem::take(&mut self.tasks)
            .into_iter()
            .enumerate()
            .map(|(i, task)| {
                let hash_prefix = if task.task_hash.len() >= 8 {
                    &task.task_hash[..8]
                } else {
                    task.task_hash.as_str()
                };
                let task_id = TaskId::new(format!("carry-{hash_prefix}"));
                let worker_id = WorkerId::new(i as u32);

                let mut wt = WorkerTask::new(
                    task_id,
                    worker_id,
                    task.task_description.as_str(),
                    task.task_description.as_str(),
                );
                wt.source = TaskSource::CarryForward;
                wt.priority = BatchPriority::High;
                wt
            })
            .collect()
    }

    /// Drain the queue and return all raw [`FailedTask`] entries.
    ///
    /// Each entry is returned with `source` semantics implied by context; call
    /// sites that need [`WorkerTask`]s should use
    /// [`drain_as_worker_tasks`](CarryForwardQueue::drain_as_worker_tasks)
    /// instead.
    ///
    /// After this call the queue is empty.
    pub fn drain_to_entries(&mut self) -> Vec<FailedTask> {
        std::mem::take(&mut self.tasks)
    }

    /// Returns `true` when the queue contains no tasks.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Returns the number of tasks currently in the queue.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for CarryForwardQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Prepend all tasks in `queue` to the front of `plan` as a new phase 0.
///
/// Each carry-forward task is converted via
/// [`CarryForwardQueue::drain_as_worker_tasks`] and placed into a new
/// [`ExecutionPhase`] at index 0.  Existing phases are renumbered starting
/// from 1.  After this call `queue` is empty.
///
/// If `queue` is empty this function is a no-op.
pub fn prepend_carry_forward(queue: &mut CarryForwardQueue, plan: &mut ExecutionPlan) {
    if queue.is_empty() {
        return;
    }

    let carry_tasks = queue.drain_as_worker_tasks();

    for phase in &mut plan.phases {
        phase.phase_number += 1;
    }

    plan.phases.insert(
        0,
        ExecutionPhase {
            phase_number: 0,
            tasks: carry_tasks,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_failed_task(description: &str, hash: &str) -> FailedTask {
        FailedTask {
            task_description: description.to_owned(),
            task_hash: hash.to_owned(),
            failed_in_batch: 0,
            attempt_count: 1,
        }
    }

    #[test]
    fn drain_produces_carry_forward_source_and_high_priority() {
        let mut queue = CarryForwardQueue::new();
        queue.push(make_failed_task("implement auth", "abcdef1234567890"));
        queue.push(make_failed_task("fix database", "deadbeef00000000"));

        let tasks = queue.drain_as_worker_tasks();

        assert_eq!(tasks.len(), 2);
        for task in &tasks {
            assert_eq!(task.source, TaskSource::CarryForward);
            assert_eq!(task.priority, BatchPriority::High);
        }
    }

    #[test]
    fn drain_clears_the_queue() {
        let mut queue = CarryForwardQueue::new();
        queue.push(make_failed_task("some task", "aaaa1111bbbb2222"));

        assert!(!queue.is_empty());
        let _ = queue.drain_as_worker_tasks();
        assert!(queue.is_empty());
    }

    #[test]
    fn empty_queue_returns_empty_vec() {
        let mut queue = CarryForwardQueue::new();
        let tasks = queue.drain_as_worker_tasks();
        assert!(tasks.is_empty());
    }

    #[test]
    fn record_permanent_failure_increments_attempt_count() {
        let mut queue = CarryForwardQueue::new();

        queue.record_permanent_failure("failing task", "hash12345678", 0);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.tasks[0].attempt_count, 1);

        queue.record_permanent_failure("failing task", "hash12345678", 1);
        assert_eq!(queue.len(), 1, "duplicate hash must not add a second entry");
        assert_eq!(queue.tasks[0].attempt_count, 2);

        queue.record_permanent_failure("failing task", "hash12345678", 2);
        assert_eq!(queue.tasks[0].attempt_count, 3);
    }

    #[test]
    fn drain_to_entries_clears_queue() {
        let mut queue = CarryForwardQueue::new();
        queue.push(make_failed_task("task one", "entry111aaaabbbb"));
        queue.push(make_failed_task("task two", "entry222ccccdddd"));

        let entries = queue.drain_to_entries();
        assert_eq!(entries.len(), 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn prepend_carry_forward_places_tasks_before_manager_tasks() {
        let mut queue = CarryForwardQueue::new();
        queue.push(make_failed_task("retry task", "retry00011112222"));

        let mut plan = ExecutionPlan::new();
        plan.phases.push(ExecutionPhase {
            phase_number: 0,
            tasks: vec![WorkerTask::new(
                TaskId::new("manager-task-1"),
                WorkerId::new(10),
                "Manager task",
                "Do manager work",
            )],
        });

        prepend_carry_forward(&mut queue, &mut plan);

        assert!(queue.is_empty(), "queue must be drained after prepend");
        assert_eq!(plan.phases.len(), 2);

        let phase0 = &plan.phases[0];
        assert_eq!(phase0.phase_number, 0);
        assert_eq!(phase0.tasks.len(), 1);
        assert_eq!(phase0.tasks[0].source, TaskSource::CarryForward);
        assert_eq!(phase0.tasks[0].priority, BatchPriority::High);

        let phase1 = &plan.phases[1];
        assert_eq!(phase1.phase_number, 1);
        assert_eq!(phase1.tasks[0].source, TaskSource::Manager);
    }

    #[test]
    fn task_id_truncated_to_8_chars_of_hash() {
        let mut queue = CarryForwardQueue::new();
        queue.push(make_failed_task("truncation test", "abcdef1234567890"));

        let tasks = queue.drain_as_worker_tasks();
        assert_eq!(tasks[0].task_id.as_str(), "carry-abcdef12");
    }

    #[test]
    fn task_id_short_hash_uses_full_hash() {
        let mut queue = CarryForwardQueue::new();
        queue.push(make_failed_task("short hash", "abc"));

        let tasks = queue.drain_as_worker_tasks();
        assert_eq!(tasks[0].task_id.as_str(), "carry-abc");
    }
}

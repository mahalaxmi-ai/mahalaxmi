use chrono::Utc;
use mahalaxmi_core::config::VerificationConfig;
use mahalaxmi_core::error::MahalaxmiError;
use mahalaxmi_core::i18n::messages::keys;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId, WorkerStatus};
use mahalaxmi_core::MahalaxmiResult;
use std::collections::{HashMap, HashSet};

use crate::models::plan::ExecutionPlan;
use crate::models::queue::{QueueStatistics, QueuedWorker};
use crate::verification::VerificationPipeline;

/// Manages the execution queue of workers, tracking dependencies and status.
pub struct WorkerQueue {
    /// All workers in the queue, keyed by worker ID.
    workers: HashMap<WorkerId, QueuedWorker>,
    /// Set of completed task IDs (for dependency resolution).
    completed_tasks: HashSet<TaskId>,
    /// Maximum concurrent active workers.
    max_concurrent: u32,
    /// Maximum retry attempts for failed workers.
    max_retries: u32,
    /// Verification configuration (opt-in, default disabled).
    verification_config: VerificationConfig,
}

impl WorkerQueue {
    /// Create a worker queue from an execution plan.
    ///
    /// Each `WorkerTask` in the plan becomes a `QueuedWorker`. Workers with
    /// no dependencies start as `Pending`; those with dependencies start as `Blocked`.
    /// Verification is disabled — use `from_plan_with_verification` to enable it.
    pub fn from_plan(plan: &ExecutionPlan, max_concurrent: u32, max_retries: u32) -> Self {
        Self::from_plan_with_verification(
            plan,
            max_concurrent,
            max_retries,
            VerificationConfig {
                enabled: false,
                ..VerificationConfig::default()
            },
        )
    }

    /// Create a worker queue from an execution plan with verification configuration.
    pub fn from_plan_with_verification(
        plan: &ExecutionPlan,
        max_concurrent: u32,
        max_retries: u32,
        verification_config: VerificationConfig,
    ) -> Self {
        let mut workers = HashMap::new();

        for phase in &plan.phases {
            for task in &phase.tasks {
                let queued = QueuedWorker::new(
                    task.worker_id,
                    task.task_id.clone(),
                    task.dependencies.clone(),
                    max_retries,
                );
                workers.insert(task.worker_id, queued);
            }
        }

        Self {
            workers,
            completed_tasks: HashSet::new(),
            max_concurrent,
            max_retries,
            verification_config,
        }
    }

    /// Get the IDs of workers that are ready to be activated.
    ///
    /// A worker is ready if:
    /// - Status is Pending or Blocked with all dependencies satisfied
    /// - There are available concurrent slots
    pub fn ready_worker_ids(&self) -> Vec<WorkerId> {
        let active_count = self
            .workers
            .values()
            .filter(|w| w.status.is_active_slot())
            .count() as u32;

        let available = self.max_concurrent.saturating_sub(active_count);
        if available == 0 {
            return Vec::new();
        }

        let mut ready: Vec<WorkerId> = self
            .workers
            .values()
            .filter(|w| {
                (w.status == WorkerStatus::Pending || w.status == WorkerStatus::Blocked)
                    && w.are_dependencies_satisfied(&self.completed_tasks)
            })
            .map(|w| w.worker_id)
            .collect();

        ready.sort_by_key(|id| id.as_u32());
        ready.truncate(available as usize);
        ready
    }

    /// Activate a worker (transition to Active status).
    pub fn activate_worker(
        &mut self,
        worker_id: WorkerId,
        i18n: &I18nService,
    ) -> MahalaxmiResult<()> {
        let worker = self.workers.get_mut(&worker_id).ok_or_else(|| {
            MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::WORKER_NOT_FOUND,
                &[("worker_id", &worker_id.to_string())],
            )
        })?;
        worker.status = WorkerStatus::Active;
        worker.started_at = Some(Utc::now());
        Ok(())
    }

    /// Mark a worker as completed.
    ///
    /// If verification is enabled and the worker has NOT already been verified
    /// by the driver, transitions to `Verifying` instead of `Completed`.
    /// When the driver has already stored a passing `last_verification` on the
    /// worker, skips the `Verifying` state and finalizes directly.
    pub fn complete_worker(
        &mut self,
        worker_id: WorkerId,
        i18n: &I18nService,
    ) -> MahalaxmiResult<()> {
        let worker = self.workers.get_mut(&worker_id).ok_or_else(|| {
            MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::WORKER_NOT_FOUND,
                &[("worker_id", &worker_id.to_string())],
            )
        })?;

        // If the driver already ran verification and it passed, skip Verifying
        let already_verified = worker
            .last_verification
            .as_ref()
            .map(|v| v.status.is_success())
            .unwrap_or(false);

        if self.verification_config.enabled && !already_verified {
            worker.status = WorkerStatus::Verifying;
        } else {
            worker.status = WorkerStatus::Completed;
            worker.finished_at = Some(Utc::now());
            self.completed_tasks.insert(worker.task_id.clone());
            self.resolve_blocked_workers();
        }
        Ok(())
    }

    /// Mark a worker as completed unconditionally, bypassing the verification gate.
    ///
    /// Used for workers that produced no code changes (e.g. "task already done").
    /// There is nothing to verify, so forcing `Completed` directly avoids a
    /// permanent `Verifying` stall in the dispatch loop.
    pub fn force_complete_worker(
        &mut self,
        worker_id: WorkerId,
        i18n: &I18nService,
    ) -> MahalaxmiResult<()> {
        let worker = self.workers.get_mut(&worker_id).ok_or_else(|| {
            MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::WORKER_NOT_FOUND,
                &[("worker_id", &worker_id.to_string())],
            )
        })?;
        worker.status = WorkerStatus::Completed;
        worker.finished_at = Some(Utc::now());
        self.completed_tasks.insert(worker.task_id.clone());
        self.resolve_blocked_workers();
        Ok(())
    }

    /// Run verification pipeline on the worker's output.
    ///
    /// - If verification passes: Verifying -> Completed (unblocks dependents)
    /// - If verification fails and retries remain: Verifying -> Active (retry)
    /// - If verification fails and no retries remain: Verifying -> Failed
    pub fn verify_worker(
        &mut self,
        worker_id: WorkerId,
        output: &str,
        pipeline: &VerificationPipeline,
        i18n: &I18nService,
    ) -> MahalaxmiResult<crate::verification::VerificationResult> {
        let max_verification_retries = self.verification_config.max_retries as usize;
        let worker = self.workers.get_mut(&worker_id).ok_or_else(|| {
            MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::WORKER_NOT_FOUND,
                &[("worker_id", &worker_id.to_string())],
            )
        })?;

        let result = pipeline.run(output);
        worker.last_verification = Some(result.clone());

        if result.status.is_success() {
            worker.status = WorkerStatus::Completed;
            worker.finished_at = Some(Utc::now());
            let task_id = worker.task_id.clone();
            self.completed_tasks.insert(task_id);
            self.resolve_blocked_workers();
        } else if worker.verification_retries < max_verification_retries {
            worker.verification_retries += 1;
            worker.retry_context = result.retry_context.clone();
            worker.status = WorkerStatus::Active;
            worker.started_at = Some(Utc::now());
        } else {
            worker.status = WorkerStatus::Failed;
            worker.finished_at = Some(Utc::now());
        }

        Ok(result)
    }

    /// Re-activate a worker with error context for retry.
    ///
    /// Increments the retry counter. Fails if max retries exceeded.
    /// The `retry_context` string is stored on the [`QueuedWorker`] for the
    /// orchestration layer to inject into the worker's next session.
    pub fn retry_worker_with_context(
        &mut self,
        worker_id: WorkerId,
        retry_context: String,
        i18n: &I18nService,
    ) -> MahalaxmiResult<()> {
        let max_verification_retries = self.verification_config.max_retries as usize;
        let worker = self.workers.get_mut(&worker_id).ok_or_else(|| {
            MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::WORKER_NOT_FOUND,
                &[("worker_id", &worker_id.to_string())],
            )
        })?;

        if worker.verification_retries >= max_verification_retries {
            return Err(MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::MAX_RETRIES_EXCEEDED,
                &[
                    ("worker_id", &worker_id.to_string()),
                    ("max_retries", &max_verification_retries.to_string()),
                ],
            ));
        }

        worker.verification_retries += 1;
        worker.retry_context = Some(retry_context);
        worker.status = WorkerStatus::Active;
        worker.started_at = Some(Utc::now());
        worker.finished_at = None;
        Ok(())
    }

    /// Mark a worker as failed.
    pub fn fail_worker(&mut self, worker_id: WorkerId, i18n: &I18nService) -> MahalaxmiResult<()> {
        let failed_task_id = {
            let worker = self.workers.get_mut(&worker_id).ok_or_else(|| {
                MahalaxmiError::orchestration(
                    i18n,
                    keys::orchestration::WORKER_NOT_FOUND,
                    &[("worker_id", &worker_id.to_string())],
                )
            })?;
            worker.status = WorkerStatus::Failed;
            worker.failure_count += 1;
            worker.last_failure_time = Some(Utc::now());
            worker.finished_at = Some(Utc::now());
            worker.task_id.clone()
        };

        // Cascade: workers that are Blocked waiting on this task can never
        // proceed — mark them Skipped so the cycle can complete instead of
        // hanging with all_workers_finished() returning false indefinitely.
        self.cascade_skip_dependents(&failed_task_id);
        Ok(())
    }

    /// Recursively mark Blocked workers as Skipped when a required dependency
    /// has permanently failed and can never be added to `completed_tasks`.
    ///
    /// Returns the list of `(WorkerId, TaskId)` for every worker that was
    /// transitioned to `Skipped`, so callers can emit events for each one.
    pub fn cascade_skip_dependents(&mut self, failed_task_id: &TaskId) -> Vec<(WorkerId, TaskId)> {
        let now = Utc::now();
        // Collect IDs and their task_ids in one pass to avoid borrow conflict.
        let dependents: Vec<(WorkerId, TaskId)> = self
            .workers
            .values()
            .filter(|w| {
                w.status == WorkerStatus::Blocked && w.dependencies.contains(failed_task_id)
            })
            .map(|w| (w.worker_id, w.task_id.clone()))
            .collect();

        let mut skipped = Vec::new();
        for (dep_id, dep_task_id) in dependents {
            if let Some(w) = self.workers.get_mut(&dep_id) {
                w.status = WorkerStatus::Skipped;
                w.finished_at = Some(now);
            }
            skipped.push((dep_id, dep_task_id.clone()));
            // Recurse so grandchild dependents are also skipped.
            let children = self.cascade_skip_dependents(&dep_task_id);
            skipped.extend(children);
        }
        skipped
    }

    /// Retry a failed worker (reset to Pending if retries remain).
    pub fn retry_worker(&mut self, worker_id: WorkerId, i18n: &I18nService) -> MahalaxmiResult<()> {
        let worker = self.workers.get_mut(&worker_id).ok_or_else(|| {
            MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::WORKER_NOT_FOUND,
                &[("worker_id", &worker_id.to_string())],
            )
        })?;
        if worker.failure_count >= worker.max_retries {
            return Err(MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::MAX_RETRIES_EXCEEDED,
                &[
                    ("worker_id", &worker_id.to_string()),
                    ("max_retries", &worker.max_retries.to_string()),
                ],
            ));
        }
        worker.status = if worker.are_dependencies_satisfied(&self.completed_tasks) {
            WorkerStatus::Pending
        } else {
            WorkerStatus::Blocked
        };
        worker.started_at = None;
        worker.finished_at = None;
        Ok(())
    }

    /// Directly set a worker's status for stall-detection transitions.
    ///
    /// Used by the driver to toggle `Active ↔ Stalled` without going through
    /// the full `fail_worker` / `complete_worker` lifecycle.  Only valid for
    /// transitions that don't require dependency cascade logic: `Active →
    /// Stalled` and `Stalled → Active`.  Returns `false` if the worker is not
    /// found.
    pub fn set_worker_status(&mut self, worker_id: WorkerId, status: WorkerStatus) -> bool {
        if let Some(w) = self.workers.get_mut(&worker_id) {
            w.status = status;
            true
        } else {
            false
        }
    }

    /// Get current queue statistics.
    pub fn statistics(&self) -> QueueStatistics {
        let mut stats = QueueStatistics {
            total: self.workers.len() as u32,
            ..QueueStatistics::default()
        };
        for worker in self.workers.values() {
            match worker.status {
                WorkerStatus::Completed => stats.completed += 1,
                WorkerStatus::Active | WorkerStatus::Stalled => stats.active += 1,
                WorkerStatus::Pending => stats.pending += 1,
                WorkerStatus::Blocked => stats.blocked += 1,
                WorkerStatus::Verifying => stats.verifying += 1,
                WorkerStatus::Failed => stats.failed += 1,
                WorkerStatus::Skipped => stats.skipped += 1,
            }
        }
        stats
    }

    /// Get the number of available concurrent slots.
    ///
    /// Both Active and Verifying workers occupy a slot.
    pub fn available_slots(&self) -> u32 {
        let occupied = self
            .workers
            .values()
            .filter(|w| w.status.is_active_slot())
            .count() as u32;
        self.max_concurrent.saturating_sub(occupied)
    }

    /// Get a reference to a specific worker.
    pub fn get_worker(&self, worker_id: &WorkerId) -> Option<&QueuedWorker> {
        self.workers.get(worker_id)
    }

    /// Get a mutable reference to a specific worker.
    pub fn get_worker_mut(&mut self, worker_id: &WorkerId) -> Option<&mut QueuedWorker> {
        self.workers.get_mut(worker_id)
    }

    /// Find a worker by its assigned task ID.
    pub fn find_worker_by_task(&self, task_id: &TaskId) -> Option<&QueuedWorker> {
        self.workers.values().find(|w| &w.task_id == task_id)
    }

    /// Get the maximum concurrent worker slots for this queue.
    pub fn max_concurrent(&self) -> u32 {
        self.max_concurrent
    }

    /// Set the maximum concurrent worker slots.
    ///
    /// Used by the driver to clamp auto-scaled counts to the license tier limit.
    pub fn set_max_concurrent(&mut self, max: u32) {
        self.max_concurrent = max;
    }

    /// Get the maximum retry limit configured for this queue.
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Get the verification configuration.
    pub fn verification_config(&self) -> &VerificationConfig {
        &self.verification_config
    }

    /// Iterate over all workers in the queue.
    pub fn workers(&self) -> impl Iterator<Item = &QueuedWorker> {
        self.workers.values()
    }

    /// Set the context preamble for a worker.
    ///
    /// Called before worker activation to inject assembled context.
    pub fn set_worker_context(
        &mut self,
        worker_id: WorkerId,
        preamble: String,
        i18n: &I18nService,
    ) -> MahalaxmiResult<()> {
        let worker = self.workers.get_mut(&worker_id).ok_or_else(|| {
            MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::WORKER_NOT_FOUND,
                &[("worker_id", &worker_id.to_string())],
            )
        })?;
        worker.context_preamble = Some(preamble);
        Ok(())
    }

    /// Get the context preamble for a worker, if set.
    pub fn worker_context(&self, worker_id: &WorkerId) -> Option<&str> {
        self.workers
            .get(worker_id)
            .and_then(|w| w.context_preamble.as_deref())
    }

    /// Set the assembled prompt for a worker.
    ///
    /// Called after `WorkerPromptBuilder::build()` produces the full prompt.
    pub fn set_worker_prompt(
        &mut self,
        worker_id: WorkerId,
        prompt: String,
        i18n: &I18nService,
    ) -> MahalaxmiResult<()> {
        let worker = self.workers.get_mut(&worker_id).ok_or_else(|| {
            MahalaxmiError::orchestration(
                i18n,
                keys::orchestration::WORKER_NOT_FOUND,
                &[("worker_id", &worker_id.to_string())],
            )
        })?;
        worker.assembled_prompt = Some(prompt);
        Ok(())
    }

    /// Get the assembled prompt for a worker, if set.
    pub fn worker_prompt(&self, worker_id: &WorkerId) -> Option<&str> {
        self.workers
            .get(worker_id)
            .and_then(|w| w.assembled_prompt.as_deref())
    }

    /// Transition blocked workers to pending when their dependencies are satisfied.
    fn resolve_blocked_workers(&mut self) {
        let completed = &self.completed_tasks;
        for worker in self.workers.values_mut() {
            if worker.status == WorkerStatus::Blocked
                && worker.are_dependencies_satisfied(completed)
            {
                worker.status = WorkerStatus::Pending;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahalaxmi_core::config::VerificationConfig;
    use mahalaxmi_core::types::{TaskId, WorkerId};

    /// Build a minimal two-worker plan: worker-1 (task-A, no deps) and
    /// worker-2 (task-B, depends on task-A). Permanently fail worker-1 via
    /// `fail_worker` and verify worker-2 transitions to Skipped, not Blocked.
    #[test]
    fn failed_worker_causes_dependent_to_be_skipped() {
        let w1 = WorkerId::new(1);
        let w2 = WorkerId::new(2);
        let task_a = TaskId::new("task-A");
        let task_b = TaskId::new("task-B");

        let mut queue = WorkerQueue {
            workers: {
                let mut m = HashMap::new();
                m.insert(
                    w1,
                    QueuedWorker::new(w1, task_a.clone(), vec![], 2),
                );
                let mut dep_worker = QueuedWorker::new(w2, task_b.clone(), vec![task_a.clone()], 2);
                dep_worker.status = WorkerStatus::Blocked;
                m.insert(w2, dep_worker);
                m
            },
            completed_tasks: HashSet::new(),
            max_concurrent: 4,
            max_retries: 2,
            verification_config: VerificationConfig { enabled: false, ..Default::default() },
        };

        let i18n = I18nService::new(mahalaxmi_core::i18n::SupportedLocale::EnUs);
        queue.fail_worker(w1, &i18n).expect("fail_worker should succeed");

        let w2_status = queue.get_worker(&w2).unwrap().status;
        assert_eq!(
            w2_status,
            WorkerStatus::Skipped,
            "dependent of a failed worker must be Skipped, not {:?}",
            w2_status
        );

        // The queue must now report as finished (no blocked/pending/active).
        let stats = queue.statistics();
        assert_eq!(stats.blocked, 0, "no workers should remain Blocked");
        assert_eq!(stats.skipped, 1, "one worker should be Skipped");
    }
}

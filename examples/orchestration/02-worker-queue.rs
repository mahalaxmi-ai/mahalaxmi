// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Worker queue lifecycle
//
// Demonstrates how WorkerQueue tracks task state through the execution
// lifecycle: Pending → Active → Completed. Shows dependency unblocking:
// tasks with unsatisfied dependencies stay Blocked until their prerequisites
// complete, then become Pending and eligible for dispatch.

use mahalaxmi_core::{
    i18n::{locale::SupportedLocale, I18nService},
    types::{TaskId, WorkerId},
};
use mahalaxmi_orchestration::{
    WorkerQueue,
    models::plan::{ExecutionPhase, ExecutionPlan, WorkerTask},
};

fn make_plan() -> ExecutionPlan {
    // Two phases: task-1 runs first, task-2 is unblocked when task-1 completes.
    let task1 = WorkerTask::new(
        TaskId::new("task-1"),
        WorkerId::new(1),
        "Foundation work",
        "Build the foundation layer.",
    );

    let mut task2 = WorkerTask::new(
        TaskId::new("task-2"),
        WorkerId::new(2),
        "Feature work",
        "Build the feature on top of the foundation.",
    );
    task2.dependencies = vec![TaskId::new("task-1")];

    ExecutionPlan::from_phases(vec![
        ExecutionPhase { phase_number: 0, tasks: vec![task1] },
        ExecutionPhase { phase_number: 1, tasks: vec![task2] },
    ])
}

fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);
    let plan = make_plan();

    // Create the queue: max 2 concurrent workers, 1 retry allowed.
    let mut queue = WorkerQueue::from_plan(&plan, 2, 1);

    println!("Initial stats: {:?}", queue.statistics());

    // Retrieve worker IDs that are ready to run (dependencies satisfied, capacity available).
    let ready = queue.ready_worker_ids();
    println!("Ready workers: {}", ready.len()); // 1: only worker-1 (worker-2 is Blocked)
    for id in &ready {
        println!("  -> worker {}", id.as_u32());
    }

    // Simulate activating and completing worker-1.
    if let Some(&first_id) = ready.first() {
        queue.activate_worker(first_id, &i18n).expect("activate failed");
        println!("Activated worker {}.", first_id.as_u32());

        queue.complete_worker(first_id, &i18n).expect("complete failed");
        println!("Completed worker {}.", first_id.as_u32());
    }

    // Worker-2 should now be unblocked and ready.
    let ready = queue.ready_worker_ids();
    println!("Ready after worker-1 complete: {}", ready.len());
    for id in &ready {
        println!("  -> worker {}", id.as_u32());
    }

    println!("Final stats: {:?}", queue.statistics());
}

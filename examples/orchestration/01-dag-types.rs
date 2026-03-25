// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
//
// Example: Working with DAG types
//
// Demonstrates how to construct WorkerTasks with dependency relationships,
// validate the graph for cycles, and sort tasks into execution phases.
//
// The DAG validation and phase-building functions are the same ones used by
// the ConsensusEngine when turning manager proposals into an ExecutionPlan.

use mahalaxmi_core::{
    i18n::{locale::SupportedLocale, I18nService},
    types::{TaskId, WorkerId},
};
use mahalaxmi_orchestration::{
    build_phases, detect_cycles, validate_dag,
    models::plan::WorkerTask,
};

fn main() {
    let i18n = I18nService::new(SupportedLocale::EnUs);

    // Build a simple four-task DAG:
    //   task-a  (no deps)
    //   task-b  depends on task-a
    //   task-c  depends on task-a
    //   task-d  depends on task-b and task-c
    let task_a = WorkerTask::new(
        TaskId::new("task-a"),
        WorkerId::new(1),
        "Set up project scaffolding",
        "Create the directory structure and base files.",
    );

    let mut task_b = WorkerTask::new(
        TaskId::new("task-b"),
        WorkerId::new(2),
        "Implement core logic",
        "Write the main business logic module.",
    );
    task_b.dependencies = vec![TaskId::new("task-a")];

    let mut task_c = WorkerTask::new(
        TaskId::new("task-c"),
        WorkerId::new(3),
        "Write tests",
        "Write unit tests for the core logic.",
    );
    task_c.dependencies = vec![TaskId::new("task-a")];

    let mut task_d = WorkerTask::new(
        TaskId::new("task-d"),
        WorkerId::new(4),
        "Integration review",
        "Review and integrate all modules.",
    );
    task_d.dependencies = vec![TaskId::new("task-b"), TaskId::new("task-c")];

    let tasks = vec![task_a, task_b, task_c, task_d];

    // Validate: this graph is acyclic so validate_dag should return Ok.
    match validate_dag(&tasks, &i18n) {
        Ok(()) => println!("DAG is valid — no cycles detected."),
        Err(e) => eprintln!("Invalid DAG: {e}"),
    }

    // detect_cycles returns the cycles explicitly (empty = acyclic).
    let cycles = detect_cycles(&tasks);
    println!("Cycles found: {}", cycles.len());

    // build_phases groups tasks into concurrent execution phases.
    // Phase 1: task-a (no deps)
    // Phase 2: task-b, task-c (both depend only on task-a)
    // Phase 3: task-d (depends on task-b and task-c)
    match build_phases(&tasks, &i18n) {
        Ok(phases) => {
            println!("Execution phases: {}", phases.len());
            for (i, phase) in phases.iter().enumerate() {
                let titles: Vec<&str> = phase.tasks.iter().map(|t| t.title.as_str()).collect();
                println!("  Phase {}: {:?}", i + 1, titles);
            }
        }
        Err(e) => eprintln!("build_phases failed: {e}"),
    }
}

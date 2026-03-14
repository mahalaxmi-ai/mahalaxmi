# mahalaxmi-orchestration

Manager-Worker DAG execution engine: consensus strategies, execution plan lifecycle, worker queue, cycle state machine, and prompt building for Mahalaxmi.

## Overview

`mahalaxmi-orchestration` is the brain of the Mahalaxmi platform. It implements the Manager-Worker architecture where AI manager agents analyze a codebase and produce an execution plan — a DAG of worker tasks — and AI worker agents execute those tasks in dependency-ordered phases. The orchestration engine handles the full cycle lifecycle from requirements intake to post-cycle validation.

The consensus engine (`ConsensusEngine`) merges proposals from multiple managers into a single de-duplicated execution plan using one of four strategies: Union (all proposed tasks), Intersection (only unanimously proposed tasks), WeightedVoting (tasks reaching a configurable vote threshold), or ComplexityWeighted (tasks weighted by estimated complexity). Semantic de-duplication via Jaccard similarity prevents CamelCase and snake_case variants of the same task from being treated as different tasks. Ambiguous pairs are resolved by an optional LLM arbitrator.

The `WorkerQueue` tracks each worker task's status through its full lifecycle (`Pending → Blocked → Active → Verifying → Completed/Failed/Retrying`), enforces dependency ordering, and manages retry logic with context injection on failure. The `CycleStateMachine` ensures valid transitions between cycle states (`Idle → Planning → AwaitingPlanApproval → Executing → Validating → Complete`).

Most Mahalaxmi integrators will interact with this crate through `OrchestrationService` (the high-level entry point) or by reading `CycleSnapshot` data through the Tauri command layer. Crate authors building custom orchestration drivers use the lower-level `WorkerQueue`, `ConsensusEngine`, and `CycleStateMachine` directly.

## Key Types

| Type | Kind | Description |
|------|------|-------------|
| `OrchestrationService` | Struct | High-level entry point; owns the full cycle lifecycle |
| `CycleConfig` | Struct | Input to start a cycle: project root, requirements, worker count, strategy |
| `CycleHandle` | Struct | Handle to a running cycle; supports stop, approve, status queries |
| `CycleSnapshot` | Struct | Point-in-time view of a cycle's state and worker statuses |
| `ConsensusEngine` | Struct | Merges manager proposals into an execution plan |
| `ArbitrationConfig` | Struct | LLM arbitration settings for ambiguous task pairs |
| `WorkerQueue` | Struct | Tracks worker task status, dependencies, retries |
| `WorkerTask` | Struct | A single worker task: title, description, dependencies, complexity |
| `ExecutionPlan` | Struct | Ordered phases of `WorkerTask`s produced by the consensus engine |
| `ExecutionPhase` | Struct | A set of tasks that can execute concurrently |
| `CycleStateMachine` | Struct | Validates and drives transitions through cycle states |
| `CycleState` | Enum | `Idle`, `Planning`, `AwaitingPlanApproval`, `Executing`, `Validating`, `Complete` |
| `ManagerPromptBuilder` | Struct | Builds structured prompts for manager AI agents |
| `WorkerPromptBuilder` | Struct | Builds structured task prompts for worker AI agents |
| `ManagerOutputParser` | Struct | Parses manager output into `ManagerProposal` structs |
| `StreamMonitor` | Struct | Monitors PTY output streams and dispatches detection rules |
| `VerificationPipeline` | Struct | Post-task test/lint verification for worker self-verification |
| `SimilarityWeights` | Struct | Configures Jaccard weights for semantic de-duplication |

## Key Functions / Methods

| Function | Description |
|----------|-------------|
| `validate_dag(tasks, i18n)` | Validate that a task set forms a valid acyclic graph |
| `detect_cycles(tasks)` | Return all dependency cycles found in a task set |
| `topological_sort(tasks)` | Sort tasks in dependency order |
| `build_phases(tasks)` | Group tasks into concurrent execution phases |
| `WorkerQueue::from_plan(plan, max_concurrent, max_retries)` | Initialize queue from an execution plan |
| `WorkerQueue::next_runnable()` | Return tasks ready to run (dependencies satisfied, capacity available) |
| `group_matching_tasks(proposals, weights)` | Semantic de-duplication of task proposals from multiple managers |
| `ConsensusEngine::evaluate(proposals, config)` | Run consensus; returns de-duplicated `ExecutionPlan` |
| `VerificationPipeline::run(output, config)` | Parse test/lint output and produce a `VerificationResult` |
| `format_retry_context(result)` | Format verification failures into a context string for retry prompts |

## Feature Flags

| Flag | Description |
|------|-------------|
| `context` (default) | Enables intelligent context routing via `mahalaxmi-memory` and `mahalaxmi-indexing` |
| `arbitration` | Enables LLM-based arbitration for ambiguous task pairs (requires `ANTHROPIC_API_KEY` or `claude` binary) |

## Dependencies

| Dependency | Why |
|-----------|-----|
| `mahalaxmi-core` | Shared types, config, errors, i18n |
| `mahalaxmi-detection` | Stream monitoring and error analysis |
| `mahalaxmi-providers` | Provider routing for multi-provider cycles |
| `mahalaxmi-memory` | Cross-agent memory injection (optional, `context` feature) |
| `mahalaxmi-indexing` | Context routing signal (optional, `context` feature) |
| `tokio` | Async execution of worker cycles |
| `reqwest` | LLM arbitration REST calls (optional) |
| `serde` + `serde_json` | Plan and proposal serialization |

## Stability

**Unstable** — API may change in minor versions during the pre-1.0 period.

## License

MIT — Copyright 2026 ThriveTech Services LLC

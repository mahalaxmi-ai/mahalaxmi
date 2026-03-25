// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { EventEmitter } from 'events';

/** Snapshot of a single worker's state. */
export interface WorkerSnapshotDto {
  worker_id: string;
  task_title: string;
  status: string;
  files_modified: string[];
}

/** Lightweight orchestration snapshot suitable for streaming. */
export interface OrchestrationSnapshotDto {
  cycle_id: string | null;
  phase: string;
  workers: WorkerSnapshotDto[];
  running_cost_usd: number;
  file_acceptances: Record<string, boolean>;
}

/** Response from `GET /v1/projects/status`. */
export interface RepoStatusResponse {
  exists: boolean;
  path: string;
  status: 'clean' | 'dirty' | 'has_worktrees' | 'not_cloned';
}

/** Template available for cycle start. */
export interface TemplateDto {
  id: string;
  name: string;
  category: string;
}

/** PTY output received from a worker terminal. */
export interface OutputReceivedEvent {
  worker_id: string;
  data: string;
}

/** Worker started event payload. */
export interface WorkerStartedEvent {
  worker_id: string;
  task_title: string;
  worker_number: number;
}

/** Cost update event payload. */
export interface CostUpdatedEvent {
  total_cost_usd: number;
  active_worker_count: number;
  worker_task_names: string[];
}

/**
 * Common interface implemented by both the local WebSocket client and the
 * remote REST+SSE client.  All VS Code UI components accept this type so they
 * work identically regardless of which transport is active.
 *
 * Events emitted by all implementations:
 *   - `connected`                          — connection established
 *   - `disconnected`                       — connection lost
 *   - `snapshot`       (OrchestrationSnapshotDto) — full state snapshot
 *   - `orchestration_event` (unknown)      — raw orchestration event
 *   - `templates`      (TemplateDto[])     — available templates list
 *   - `output_received` (OutputReceivedEvent) — worker PTY output
 *   - `worker_started` (WorkerStartedEvent)  — worker began
 *   - `worker_finished` (workerId, passed)   — worker completed/failed
 *   - `cost_updated`   (CostUpdatedEvent)    — live cost update
 *   - `plan_ready`     (cycleId, plan)       — plan awaits approval
 */
export interface IMahalaxmiClient extends EventEmitter {
  /** `true` for the remote REST client; `false` for the local WebSocket client. */
  readonly isRemote: boolean;
  connect(): void;
  dispose(): void;
  startCycle(
    requirements: string,
    projectRoot?: string,
    chainMode?: boolean,
    chainRequirementsPath?: string,
    repoUrl?: string,
    branch?: string,
    repoAction?: string,
  ): void;
  stopCycle(cycleId: string): void;
  approvePlan(cycleId: string): void;
  stopChain(): void;
  setFileAcceptance(filePath: string, accepted: boolean): void;
  queryTemplates(): void;
  /** Check whether the given git repo URL is already cloned on the server. */
  checkRepoStatus(repoUrl: string): Promise<RepoStatusResponse>;
  /**
   * Kill a specific worker by ID.
   *
   * Closes the worker's PTY, marks it as failed, and cascades the failure to
   * any Blocked workers that depend on it.  The cycle continues with remaining
   * workers.
   */
  killWorker(workerId: string): Promise<void>;
}

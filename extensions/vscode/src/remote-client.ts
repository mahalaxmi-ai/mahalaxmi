// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { EventEmitter } from 'events';
import * as http from 'http';
import * as https from 'https';

import type {
  IMahalaxmiClient,
  OrchestrationSnapshotDto,
  RepoStatusResponse,
  WorkerSnapshotDto,
  TemplateDto,
  OutputReceivedEvent,
  WorkerStartedEvent,
  CostUpdatedEvent,
} from './client-interface';

/**
 * REST + SSE client for the `mahalaxmi-service` Axum backend (port 17421).
 *
 * Transport decisions:
 * - REST calls: use `fetch()` — available in modern Node.js / Electron.
 * - SSE stream: use Node.js `http.request` / `https.request` directly so a
 *   Bearer token can be set on every connection (browser EventSource does not
 *   support custom headers).
 * - Polling: `GET /v1/cycles/:id` every 2 s synthesises `snapshot` events for
 *   the sidebar TreeView and status bar while a cycle is active.
 * - File acceptances: maintained client-side; merged into synthesised snapshots.
 */
export class RemoteApiClient extends EventEmitter implements IMahalaxmiClient {
  readonly isRemote = true;

  private currentCycleId: string | null = null;
  private sseRequest: http.ClientRequest | null = null;
  private pollTimer: ReturnType<typeof setInterval> | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private disposed = false;
  private reconnectDelay = 2000;
  private readonly maxReconnectDelay = 30_000;
  private fileAcceptances: Record<string, boolean> = {};

  constructor(
    private readonly baseUrl: string,
    private readonly apiToken: string,
  ) {
    super();
  }

  // ---------------------------------------------------------------------------
  // IMahalaxmiClient — public API
  // ---------------------------------------------------------------------------

  connect(): void {
    if (this.disposed) return;
    void this.checkHealth();
  }

  dispose(): void {
    this.disposed = true;
    this.stopPollTimer();
    this.destroySse();
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.removeAllListeners();
  }

  startCycle(
    requirements: string,
    projectRoot?: string,
    chainMode?: boolean,
    chainRequirementsPath?: string,
    repoUrl?: string,
    branch?: string,
    repoAction?: string,
  ): void {
    this.fetchApi('/v1/cycles', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        requirements,
        project_root: projectRoot,
        chain_mode: chainMode ?? false,
        chain_requirements_path: chainRequirementsPath,
        repo_url: repoUrl,
        branch,
        repo_action: repoAction,
      }),
    })
      .then(async (resp) => {
        if (!resp.ok) return;
        const data = (await resp.json()) as { cycle_id?: string };
        if (data.cycle_id) {
          this.currentCycleId = data.cycle_id;
          this.startPollTimer(data.cycle_id);
        }
      })
      .catch(() => {
        /* network failure — swallow */
      });
  }

  stopCycle(cycleId: string): void {
    const id = cycleId === '__current__' ? (this.currentCycleId ?? cycleId) : cycleId;
    this.fetchApi(`/v1/cycles/${id}`, { method: 'DELETE' }).catch(() => {
      /* swallow */
    });
    this.stopPollTimer();
    this.currentCycleId = null;
  }

  approvePlan(cycleId: string): void {
    const id = cycleId === '__current__' ? (this.currentCycleId ?? cycleId) : cycleId;
    this.fetchApi(`/v1/cycles/${id}/approve`, { method: 'POST' }).catch(() => {
      /* swallow */
    });
  }

  /** Remote service has no chain-mode concept yet — proxy as cycle cancel. */
  stopChain(): void {
    if (this.currentCycleId) {
      this.stopCycle(this.currentCycleId);
    }
  }

  /** Update local acceptance map; merged into synthesised snapshots. */
  setFileAcceptance(filePath: string, accepted: boolean): void {
    this.fileAcceptances[filePath] = accepted;
  }

  /** Check whether the repo at `repoUrl` is already cloned on the server. */
  async checkRepoStatus(repoUrl: string): Promise<RepoStatusResponse> {
    try {
      const resp = await this.fetchApi(
        `/v1/projects/status?repo_url=${encodeURIComponent(repoUrl)}`,
      );
      if (!resp.ok) {
        return { exists: false, path: '', status: 'not_cloned' };
      }
      return (await resp.json()) as RepoStatusResponse;
    } catch {
      return { exists: false, path: '', status: 'not_cloned' };
    }
  }

  /** Kill a specific worker via the REST API DELETE /v1/workers/:id. */
  async killWorker(workerId: string): Promise<void> {
    try {
      await this.fetchApi(`/v1/workers/${encodeURIComponent(workerId)}`, { method: 'DELETE' });
    } catch {
      // Non-fatal — the UI event from the backend will reflect the outcome.
    }
  }

  /** Fetch available templates; emits `templates` with empty array on error. */
  queryTemplates(): void {
    this.fetchApi('/v1/templates')
      .then(async (resp) => {
        if (!resp.ok) {
          this.emit('templates', [] as TemplateDto[]);
          return;
        }
        const data = (await resp.json()) as TemplateDto[];
        this.emit('templates', Array.isArray(data) ? data : []);
      })
      .catch(() => this.emit('templates', [] as TemplateDto[]));
  }

  // ---------------------------------------------------------------------------
  // Health check + reconnect
  // ---------------------------------------------------------------------------

  private async checkHealth(): Promise<void> {
    try {
      const resp = await this.fetchApi('/v1/health');
      if (resp.ok) {
        this.reconnectDelay = 2000;
        this.emit('connected');
        this.startSse();
      } else {
        this.emit('disconnected');
        this.scheduleReconnect();
      }
    } catch {
      this.emit('disconnected');
      this.scheduleReconnect();
    }
  }

  private scheduleReconnect(): void {
    if (this.disposed) return;
    const delay = this.reconnectDelay;
    this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
    this.reconnectTimer = setTimeout(() => void this.checkHealth(), delay);
  }

  // ---------------------------------------------------------------------------
  // SSE stream (Node.js http/https to support Bearer auth header)
  // ---------------------------------------------------------------------------

  private startSse(): void {
    if (this.disposed) return;

    const parsed = new URL('/v1/events', this.baseUrl);
    const lib = parsed.protocol === 'https:' ? https : http;

    const options: http.RequestOptions = {
      hostname: parsed.hostname,
      port: parsed.port || (parsed.protocol === 'https:' ? '443' : '80'),
      path: '/v1/events',
      method: 'GET',
      headers: {
        Accept: 'text/event-stream',
        ...(this.apiToken ? { Authorization: `Bearer ${this.apiToken}` } : {}),
      },
    };

    const req = lib.request(options, (res) => {
      res.setEncoding('utf8');
      let buffer = '';

      res.on('data', (chunk: string) => {
        buffer += chunk;
        // SSE blocks are separated by a blank line (\n\n)
        const parts = buffer.split('\n\n');
        buffer = parts.pop() ?? '';
        for (const block of parts) {
          this.handleSseBlock(block);
        }
      });

      res.on('end', () => {
        this.sseRequest = null;
        if (!this.disposed) {
          this.emit('disconnected');
          this.scheduleReconnect();
        }
      });
    });

    req.on('error', () => {
      this.sseRequest = null;
      if (!this.disposed) {
        this.emit('disconnected');
        this.scheduleReconnect();
      }
    });

    req.end();
    this.sseRequest = req;
  }

  private handleSseBlock(block: string): void {
    let eventType = 'message';
    let data = '';

    for (const line of block.split('\n')) {
      if (line.startsWith('event:')) {
        eventType = line.slice(6).trim();
      } else if (line.startsWith('data:')) {
        data = line.slice(5).trim();
      }
    }

    if (!data) return;

    let payload: unknown;
    try {
      payload = JSON.parse(data) as unknown;
    } catch {
      return;
    }

    switch (eventType) {
      case 'worker_started': {
        const p = payload as { worker_id?: string; task_title?: string; worker_number?: number };
        this.emit('worker_started', {
          worker_id: p.worker_id ?? '',
          task_title: p.task_title ?? '',
          worker_number: p.worker_number ?? 0,
        } satisfies WorkerStartedEvent);
        break;
      }
      case 'worker_output': {
        const p = payload as { worker_id?: string; data?: string };
        this.emit('output_received', {
          worker_id: p.worker_id ?? '',
          data: p.data ?? '',
        } satisfies OutputReceivedEvent);
        break;
      }
      case 'worker_completed':
      case 'worker_failed': {
        const p = payload as { worker_id?: string };
        this.emit('worker_finished', p.worker_id ?? '', eventType === 'worker_completed');
        break;
      }
      case 'cost_updated': {
        this.emit('cost_updated', payload as CostUpdatedEvent);
        break;
      }
      case 'plan_ready': {
        const p = payload as { cycle_id?: string; plan?: unknown };
        if (p.cycle_id) this.currentCycleId = p.cycle_id;
        this.emit('plan_ready', p.cycle_id ?? '', p.plan);
        break;
      }
      case 'cycle_complete': {
        this.stopPollTimer();
        this.destroySse();
        this.currentCycleId = null;
        break;
      }
    }
  }

  private destroySse(): void {
    if (this.sseRequest) {
      this.sseRequest.destroy();
      this.sseRequest = null;
    }
  }

  // ---------------------------------------------------------------------------
  // Polling — synthesises `snapshot` events for the sidebar
  // ---------------------------------------------------------------------------

  private startPollTimer(cycleId: string): void {
    this.stopPollTimer();
    this.pollTimer = setInterval(() => void this.pollCycleState(cycleId), 2000);
  }

  private stopPollTimer(): void {
    if (this.pollTimer !== null) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
  }

  async pollCycleState(cycleId: string): Promise<void> {
    let data: {
      id: string;
      state: string;
      created_at: string;
      worker_statuses: Array<{ worker_id: string; status: unknown }>;
    };

    try {
      const resp = await this.fetchApi(`/v1/cycles/${cycleId}`);
      if (!resp.ok) {
        if (resp.status === 404) {
          this.stopPollTimer();
          this.currentCycleId = null;
        }
        return;
      }
      data = (await resp.json()) as typeof data;
    } catch {
      return; // network error — retry on next tick
    }

    if (data.state === 'awaiting_plan_approval') {
      this.emit('plan_ready', cycleId, null);
    }

    if (data.state === 'completed' || data.state === 'failed' || data.state === 'cancelled') {
      this.stopPollTimer();
      this.currentCycleId = null;
    }

    const workers: WorkerSnapshotDto[] = (data.worker_statuses ?? []).map((ws) => {
      const s = ws.status as Record<string, unknown> | null;
      return {
        worker_id: ws.worker_id,
        task_title: typeof s?.['task_title'] === 'string' ? s['task_title'] : ws.worker_id,
        status: typeof s?.['status'] === 'string' ? s['status'] : String(ws.status),
        files_modified: [],
      };
    });

    const snapshot: OrchestrationSnapshotDto = {
      cycle_id: cycleId,
      phase: data.state,
      workers,
      running_cost_usd: 0,
      file_acceptances: { ...this.fileAcceptances },
    };

    this.emit('snapshot', snapshot);
  }

  // ---------------------------------------------------------------------------
  // Shared fetch helper — adds Bearer auth to every request
  // ---------------------------------------------------------------------------

  private fetchApi(path: string, init?: RequestInit): Promise<Response> {
    const url = new URL(path, this.baseUrl).toString();
    const headers: Record<string, string> = {
      ...((init?.headers as Record<string, string> | undefined) ?? {}),
    };
    if (this.apiToken) {
      headers['Authorization'] = `Bearer ${this.apiToken}`;
    }
    return fetch(url, { ...init, headers });
  }
}

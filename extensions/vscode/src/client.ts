// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { EventEmitter } from 'events';
import WebSocket from 'ws';

// Re-export all shared DTOs and the interface so existing imports from 'client'
// continue to work unchanged after the interface was extracted to client-interface.ts.
export type {
  WorkerSnapshotDto,
  OrchestrationSnapshotDto,
  TemplateDto,
  OutputReceivedEvent,
  WorkerStartedEvent,
  CostUpdatedEvent,
  IMahalaxmiClient,
} from './client-interface';

import type { IMahalaxmiClient, RepoStatusResponse } from './client-interface';
import type {
  OrchestrationSnapshotDto,
  OutputReceivedEvent,
  WorkerStartedEvent,
  CostUpdatedEvent,
  TemplateDto,
} from './client-interface';

interface WsMessage {
  type: string;
  payload: unknown;
}

/** WebSocket client that connects to the Tauri local API at ws://127.0.0.1:17420. */
export class MahalaxmiClient extends EventEmitter implements IMahalaxmiClient {
  readonly isRemote = false;

  private ws: WebSocket | null = null;
  private reconnectDelay = 1000;
  private readonly maxReconnectDelay = 30_000;
  private disposed = false;

  constructor(private readonly url: string) {
    super();
  }

  /** Open (or reopen) the WebSocket connection. */
  connect(): void {
    if (this.disposed) {
      return;
    }

    const socket = new WebSocket(this.url);
    this.ws = socket;

    socket.on('open', () => {
      this.reconnectDelay = 1000;
      this.emit('connected');
    });

    socket.on('message', (data: WebSocket.RawData) => {
      this.handleMessage(data.toString());
    });

    socket.on('close', () => {
      this.emit('disconnected');
      this.scheduleReconnect();
    });

    socket.on('error', () => {
      // close event handles the reconnect
    });
  }

  private handleMessage(text: string): void {
    let msg: WsMessage;
    try {
      msg = JSON.parse(text) as WsMessage;
    } catch {
      return;
    }

    switch (msg.type) {
      case 'snapshot':
        this.emit('snapshot', msg.payload as OrchestrationSnapshotDto);
        break;
      case 'event':
        this.emit('orchestration_event', msg.payload);
        this.dispatchTypedEvent(msg.payload);
        break;
      case 'templates':
        this.emit('templates', msg.payload as TemplateDto[]);
        break;
    }
  }

  private dispatchTypedEvent(payload: unknown): void {
    if (typeof payload !== 'object' || payload === null) return;
    const p = payload as Record<string, unknown>;
    const type = p['type'] as string | undefined;
    switch (type) {
      case 'OutputReceived':
        this.emit('output_received', payload as OutputReceivedEvent);
        break;
      case 'WorkerStarted':
        this.emit('worker_started', payload as WorkerStartedEvent);
        break;
      case 'WorkerCompleted':
      case 'WorkerFailed':
        this.emit('worker_finished', p['worker_id'] as string, type === 'WorkerCompleted');
        break;
      case 'CostUpdated':
        this.emit('cost_updated', payload as CostUpdatedEvent);
        break;
      case 'PlanReady':
        this.emit('plan_ready', p['cycle_id'] as string, p['plan'] as unknown);
        break;
    }
  }

  private scheduleReconnect(): void {
    if (this.disposed) {
      return;
    }
    const delay = this.reconnectDelay;
    this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
    setTimeout(() => this.connect(), delay);
  }

  /** Send a typed message to the Tauri backend. No-op when the socket is not open. */
  send(type: string, payload: unknown): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type, payload }));
    }
  }

  /** Send a start_cycle command to the backend. Remote-only fields are ignored. */
  startCycle(
    requirements: string,
    projectRoot?: string,
    chainMode?: boolean,
    chainRequirementsPath?: string,
    _repoUrl?: string,
    _branch?: string,
    _repoAction?: string,
  ): void {
    this.send('start_cycle', {
      requirements,
      project_root: projectRoot,
      chain_mode: chainMode ?? false,
      chain_requirements_path: chainRequirementsPath,
    });
  }

  /** Local mode has no remote repo concept — always returns not_cloned. */
  checkRepoStatus(_repoUrl: string): Promise<RepoStatusResponse> {
    return Promise.resolve({ exists: false, path: '', status: 'not_cloned' as const });
  }

  /** Kill a specific worker by closing its PTY and marking it as failed. */
  killWorker(workerId: string): Promise<void> {
    this.send('kill_worker', { worker_id: workerId });
    return Promise.resolve();
  }

  /** Send a stop_cycle command to the backend. */
  stopCycle(cycleId: string): void {
    this.send('stop_cycle', { cycle_id: cycleId });
  }

  /** Approve the pending execution plan. */
  approvePlan(cycleId: string): void {
    this.send('approve_execution_plan', { cycle_id: cycleId });
  }

  /** Stop chain mode. */
  stopChain(): void {
    this.send('stop_chain', {});
  }

  /** Accept or reject a specific file in the current cycle. */
  setFileAcceptance(filePath: string, accepted: boolean): void {
    this.send('set_file_acceptance', { file_path: filePath, accepted });
  }

  /** Request the list of available templates. */
  queryTemplates(): void {
    this.send('get_templates', {});
  }

  /** Permanently close the client and prevent further reconnections. */
  dispose(): void {
    this.disposed = true;
    this.ws?.close();
    this.removeAllListeners();
  }
}

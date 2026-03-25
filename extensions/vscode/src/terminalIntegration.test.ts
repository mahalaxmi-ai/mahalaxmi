// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { EventEmitter } from 'events';

// ---------------------------------------------------------------------------
// Mocks
// ---------------------------------------------------------------------------

const createdTerminals: Array<{ name: string; pty: unknown }> = [];

vi.mock('vscode', () => ({
  window: {
    createTerminal: vi.fn((opts: { name: string; pty: unknown }) => {
      createdTerminals.push({ name: opts.name, pty: opts.pty });
      return { dispose: vi.fn() };
    }),
  },
  EventEmitter: class MockEventEmitter<T> {
    private listeners: Array<(e: T) => void> = [];
    event = (l: (e: T) => void) => { this.listeners.push(l); return { dispose: vi.fn() }; };
    fire(e: T): void { for (const l of this.listeners) l(e); }
    dispose(): void { this.listeners = []; }
  },
}));

vi.mock('ws', () => {
  class MockWS {
    static OPEN = 1;
    readyState = 1;
    private readonly _listeners: Map<string, Array<(...args: unknown[]) => void>> = new Map();
    on(event: string, l: (...args: unknown[]) => void): this {
      (this._listeners.get(event) ?? (this._listeners.set(event, []), this._listeners.get(event)!)).push(l);
      return this;
    }
    send(): void {}
    close(): void {}
  }
  return { default: MockWS };
});

// ---------------------------------------------------------------------------
// Stub client
// ---------------------------------------------------------------------------

class StubClient extends EventEmitter {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('WorkerTerminalManager', () => {
  let client: StubClient;

  beforeEach(async () => {
    createdTerminals.length = 0;
    client = new StubClient();
  });

  it('creates a terminal when worker_started fires', async () => {
    const { WorkerTerminalManager } = await import('./workerTerminals');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const mgr = new WorkerTerminalManager(client as any);

    client.emit('worker_started', { worker_id: 'w1', task_title: 'Fix auth bug', worker_number: 1 });
    expect(createdTerminals).toHaveLength(1);
    expect(createdTerminals[0].name).toBe('Mahalaxmi: Worker-1 — Fix auth bug');

    mgr.dispose();
  });

  it('does not create duplicate terminals for the same worker_id', async () => {
    const { WorkerTerminalManager } = await import('./workerTerminals');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const mgr = new WorkerTerminalManager(client as any);

    client.emit('worker_started', { worker_id: 'w1', task_title: 'Fix auth bug', worker_number: 1 });
    client.emit('worker_started', { worker_id: 'w1', task_title: 'Fix auth bug', worker_number: 1 });
    expect(createdTerminals).toHaveLength(1);

    mgr.dispose();
  });

  it('creates separate terminals for different workers', async () => {
    const { WorkerTerminalManager } = await import('./workerTerminals');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const mgr = new WorkerTerminalManager(client as any);

    client.emit('worker_started', { worker_id: 'w1', task_title: 'Task A', worker_number: 1 });
    client.emit('worker_started', { worker_id: 'w2', task_title: 'Task B', worker_number: 2 });
    expect(createdTerminals).toHaveLength(2);

    mgr.dispose();
  });
});

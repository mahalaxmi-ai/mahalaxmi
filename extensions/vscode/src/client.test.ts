// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { MahalaxmiClient } from './client';

// ---------------------------------------------------------------------------
// Shared socket state — must be hoisted so the mock factory can reference it
// ---------------------------------------------------------------------------

interface MockSocketInterface {
  readyState: number;
  sentMessages: string[];
  simulateOpen(): void;
  simulateMessage(data: string): void;
  simulateClose(): void;
  simulateError(): void;
  close(): void;
}

const socketState = vi.hoisted(() => ({ lastSocket: null as unknown }));

// ---------------------------------------------------------------------------
// Mock WebSocket (ws package) — class defined entirely inside the factory
// ---------------------------------------------------------------------------

vi.mock('ws', () => {
  class MockWebSocket {
    static OPEN = 1;
    static CONNECTING = 0;
    static CLOSED = 3;

    readyState = 0;
    sentMessages: string[] = [];

    private readonly _listeners: Map<string, Array<(...args: unknown[]) => void>> = new Map();

    constructor(_url: string) {
      socketState.lastSocket = this;
    }

    on(event: string, listener: (...args: unknown[]) => void): this {
      const list = this._listeners.get(event) ?? [];
      list.push(listener);
      this._listeners.set(event, list);
      return this;
    }

    emit(event: string, ...args: unknown[]): void {
      for (const l of (this._listeners.get(event) ?? [])) {
        l(...args);
      }
    }

    send(data: string): void {
      this.sentMessages.push(data);
    }

    close(): void {
      this.readyState = 3;
      this.emit('close');
    }

    simulateOpen(): void {
      this.readyState = 1;
      this.emit('open');
    }

    simulateMessage(data: string): void {
      this.emit('message', Buffer.from(data));
    }

    simulateClose(): void {
      this.readyState = 3;
      this.emit('close');
    }

    simulateError(): void {
      this.emit('error', new Error('socket error'));
    }
  }

  return { default: MockWebSocket };
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function lastSocket(): MockSocketInterface {
  if (socketState.lastSocket === null) {
    throw new Error('No mock socket created yet');
  }
  return socketState.lastSocket as MockSocketInterface;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('MahalaxmiClient', () => {
  let client: MahalaxmiClient;

  beforeEach(() => {
    socketState.lastSocket = null;
    vi.useFakeTimers();
    client = new MahalaxmiClient('ws://127.0.0.1:17420');
  });

  afterEach(() => {
    client.dispose();
    vi.useRealTimers();
  });

  it("emits 'connected' when the WebSocket opens", () => {
    const handler = vi.fn();
    client.on('connected', handler);
    client.connect();

    lastSocket().simulateOpen();

    expect(handler).toHaveBeenCalledOnce();
  });

  it("emits 'snapshot' when receiving a {type:'snapshot'} frame", () => {
    const handler = vi.fn();
    client.on('snapshot', handler);
    client.connect();
    lastSocket().simulateOpen();

    const payload = { cycle_id: 'c1', phase: 'Running', workers: [] };
    lastSocket().simulateMessage(JSON.stringify({ type: 'snapshot', payload }));

    expect(handler).toHaveBeenCalledWith(payload);
  });

  it("emits 'orchestration_event' when receiving a {type:'event'} frame", () => {
    const handler = vi.fn();
    client.on('orchestration_event', handler);
    client.connect();
    lastSocket().simulateOpen();

    const payload = { type: 'WorkerStarted', worker_id: 'w1' };
    lastSocket().simulateMessage(JSON.stringify({ type: 'event', payload }));

    expect(handler).toHaveBeenCalledWith(payload);
  });

  it('ignores frames with unknown message types', () => {
    const snapshotHandler = vi.fn();
    const eventHandler = vi.fn();
    client.on('snapshot', snapshotHandler);
    client.on('orchestration_event', eventHandler);
    client.connect();
    lastSocket().simulateOpen();

    lastSocket().simulateMessage(JSON.stringify({ type: 'unknown_type', payload: {} }));

    expect(snapshotHandler).not.toHaveBeenCalled();
    expect(eventHandler).not.toHaveBeenCalled();
  });

  it('ignores malformed (non-JSON) frames', () => {
    const handler = vi.fn();
    client.on('snapshot', handler);
    client.connect();
    lastSocket().simulateOpen();

    lastSocket().simulateMessage('not json {{');

    expect(handler).not.toHaveBeenCalled();
  });

  it('doubles the reconnect delay on each disconnection up to maxReconnectDelay', () => {
    client.connect();
    const first = lastSocket();
    first.simulateOpen();

    // First disconnect → reconnect after 1000 ms
    first.simulateClose();
    expect(socketState.lastSocket).toBe(first);
    vi.advanceTimersByTime(1000);
    const second = lastSocket();
    expect(second).not.toBe(first);

    // Second disconnect → reconnect after 2000 ms
    second.simulateOpen();
    second.simulateClose();
    vi.advanceTimersByTime(2000);
    const third = lastSocket();
    expect(third).not.toBe(second);

    // Third disconnect → reconnect after 4000 ms
    third.simulateOpen();
    third.simulateClose();
    vi.advanceTimersByTime(4000);
    expect(socketState.lastSocket).not.toBe(third);
  });

  it('caps the reconnect delay at maxReconnectDelay (30 000 ms)', () => {
    client.connect();

    // Drive the delay well past 30 000 ms by simulating many rapid disconnects
    for (let i = 0; i < 20; i++) {
      lastSocket().simulateOpen();
      lastSocket().simulateClose();
      vi.advanceTimersByTime(30_000);
    }

    // On another disconnect the delay should be at most 30 000 ms
    const before = lastSocket();
    before.simulateClose();
    vi.advanceTimersByTime(30_000);
    expect(socketState.lastSocket).not.toBe(before);
  });

  it('stops reconnecting after dispose()', () => {
    client.connect();
    const socket = lastSocket();
    socket.simulateOpen();

    client.dispose();
    socket.simulateClose();

    vi.advanceTimersByTime(60_000);
    // No new socket should have been created after dispose
    expect(socketState.lastSocket).toBe(socket);
  });

  it("send() is a no-op when the WebSocket is not OPEN", () => {
    client.connect();
    const socket = lastSocket();
    // socket.readyState is still CONNECTING (0)
    client.send('start_cycle', { requirements: 'hello' });
    expect(socket.sentMessages).toHaveLength(0);
  });

  it('send() delivers a JSON message when the socket is OPEN', () => {
    client.connect();
    const socket = lastSocket();
    socket.simulateOpen();

    client.send('start_cycle', { requirements: 'build a thing' });

    expect(socket.sentMessages).toHaveLength(1);
    const parsed = JSON.parse(socket.sentMessages[0]) as {
      type: string;
      payload: { requirements: string };
    };
    expect(parsed.type).toBe('start_cycle');
    expect(parsed.payload.requirements).toBe('build a thing');
  });
});

// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// ---------------------------------------------------------------------------
// Hoist shared mutable state so the vi.mock factory can reference it.
// vi.hoisted runs before all imports and mock factories, making `shared`
// available to the factory even though it is declared "before" the imports.
// ---------------------------------------------------------------------------

const shared = vi.hoisted(() => {
  return {
    windowStateListeners: [] as Array<(state: { focused: boolean }) => void>,
    configListeners: [] as Array<
      (e: { affectsConfiguration: (key: string) => boolean }) => void
    >,
    windowFocused: { value: true as boolean },
    connectionMode: { value: 'remote' as string },
  };
});

// ---------------------------------------------------------------------------
// Mock the vscode module — factory only references hoisted state.
// ---------------------------------------------------------------------------

vi.mock('vscode', () => {
  return {
    window: {
      get state(): { focused: boolean } {
        return { focused: shared.windowFocused.value };
      },
      onDidChangeWindowState(
        cb: (state: { focused: boolean }) => void,
      ): { dispose: () => void } {
        shared.windowStateListeners.push(cb);
        let disposed = false;
        return {
          dispose(): void {
            if (!disposed) {
              disposed = true;
              const idx = shared.windowStateListeners.indexOf(cb);
              if (idx !== -1) {
                shared.windowStateListeners.splice(idx, 1);
              }
            }
          },
        };
      },
    },
    workspace: {
      onDidChangeConfiguration(
        cb: (e: { affectsConfiguration: (key: string) => boolean }) => void,
      ): { dispose: () => void } {
        shared.configListeners.push(cb);
        let disposed = false;
        return {
          dispose(): void {
            if (!disposed) {
              disposed = true;
              const idx = shared.configListeners.indexOf(cb);
              if (idx !== -1) {
                shared.configListeners.splice(idx, 1);
              }
            }
          },
        };
      },
      getConfiguration(_section: string): { get: (key: string, def: string) => string } {
        return {
          get(key: string, def: string): string {
            if (key === 'connectionMode') {
              return shared.connectionMode.value;
            }
            return def;
          },
        };
      },
    },
  };
});

// ---------------------------------------------------------------------------
// Imports come after mocks so vitest can intercept them.
// ---------------------------------------------------------------------------

import { CloudHeartbeatManager } from './cloud-heartbeat';
import type { CloudProvisioningClient } from './heartbeat';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function fireWindowState(focused: boolean): void {
  for (const cb of [...shared.windowStateListeners]) {
    cb({ focused });
  }
}

function fireConfigChange(affectedKey: string): void {
  for (const cb of [...shared.configListeners]) {
    cb({ affectsConfiguration: (k: string) => k === affectedKey });
  }
}

function makeClient(rejectWith?: Error): {
  client: CloudProvisioningClient;
  sendHeartbeat: ReturnType<typeof vi.fn>;
} {
  const sendHeartbeat = vi.fn<[string, string], Promise<void>>();
  if (rejectWith !== undefined) {
    sendHeartbeat.mockRejectedValue(rejectWith);
  } else {
    sendHeartbeat.mockResolvedValue(undefined);
  }
  return { client: { sendHeartbeat }, sendHeartbeat };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('CloudHeartbeatManager', () => {
  let manager: CloudHeartbeatManager;

  beforeEach(() => {
    vi.useFakeTimers();
    shared.windowFocused.value = true;
    shared.connectionMode.value = 'remote';
    shared.windowStateListeners.splice(0);
    shared.configListeners.splice(0);
    // Use a 1 000 ms interval so tests do not need to advance by 120 000 ms.
    manager = new CloudHeartbeatManager(1_000);
  });

  afterEach(() => {
    manager.dispose();
    vi.useRealTimers();
  });

  // -------------------------------------------------------------------------
  // isRunning
  // -------------------------------------------------------------------------

  it('isRunning() returns false before start()', () => {
    expect(manager.isRunning()).toBe(false);
  });

  it('isRunning() returns true after start()', () => {
    const { client } = makeClient();
    manager.start('proj-1', 'key-1', client);
    expect(manager.isRunning()).toBe(true);
  });

  it('isRunning() returns false after stop()', () => {
    const { client } = makeClient();
    manager.start('proj-1', 'key-1', client);
    manager.stop();
    expect(manager.isRunning()).toBe(false);
  });

  it('isRunning() returns true between start() and stop() calls', () => {
    const { client } = makeClient();
    manager.start('proj-1', 'key-1', client);
    expect(manager.isRunning()).toBe(true);
    manager.stop();
    expect(manager.isRunning()).toBe(false);
  });

  // -------------------------------------------------------------------------
  // Immediate heartbeat
  // -------------------------------------------------------------------------

  it('fires an immediate heartbeat before the first interval tick', () => {
    const { client, sendHeartbeat } = makeClient();
    manager.start('proj-1', 'key-1', client);
    expect(sendHeartbeat).toHaveBeenCalledOnce();
    expect(sendHeartbeat).toHaveBeenCalledWith('proj-1', 'key-1');
  });

  // -------------------------------------------------------------------------
  // Interval scheduling
  // -------------------------------------------------------------------------

  it('fires heartbeat at each interval tick', () => {
    const { client, sendHeartbeat } = makeClient();
    manager.start('proj-1', 'key-1', client);

    expect(sendHeartbeat).toHaveBeenCalledTimes(1); // immediate

    vi.advanceTimersByTime(1_000);
    expect(sendHeartbeat).toHaveBeenCalledTimes(2);

    vi.advanceTimersByTime(1_000);
    expect(sendHeartbeat).toHaveBeenCalledTimes(3);
  });

  // -------------------------------------------------------------------------
  // stop() clears the interval
  // -------------------------------------------------------------------------

  it('stop() prevents any further heartbeats after clearing the timer', () => {
    const { client, sendHeartbeat } = makeClient();
    manager.start('proj-1', 'key-1', client);
    expect(sendHeartbeat).toHaveBeenCalledTimes(1);

    manager.stop();
    vi.advanceTimersByTime(10_000);
    expect(sendHeartbeat).toHaveBeenCalledTimes(1); // no additional ticks
  });

  // -------------------------------------------------------------------------
  // Singleton — calling start() twice must not double-fire
  // -------------------------------------------------------------------------

  it('calling start() twice stops the first interval before starting a second', () => {
    const { client, sendHeartbeat } = makeClient();

    manager.start('proj-1', 'key-1', client); // immediate #1
    manager.start('proj-1', 'key-1', client); // stops first, immediate #2

    expect(sendHeartbeat).toHaveBeenCalledTimes(2);

    vi.advanceTimersByTime(1_000);
    // Only one interval is running — exactly one additional tick
    expect(sendHeartbeat).toHaveBeenCalledTimes(3);
  });

  // -------------------------------------------------------------------------
  // Window-state pause / resume
  // -------------------------------------------------------------------------

  it('pauses heartbeat ticks when window loses focus', () => {
    const { client, sendHeartbeat } = makeClient();
    manager.start('proj-1', 'key-1', client);
    expect(sendHeartbeat).toHaveBeenCalledTimes(1);

    fireWindowState(false); // simulate minimize / hide

    vi.advanceTimersByTime(5_000);
    expect(sendHeartbeat).toHaveBeenCalledTimes(1); // no new ticks
  });

  it('fires an immediate heartbeat when window regains focus', () => {
    const { client, sendHeartbeat } = makeClient();
    manager.start('proj-1', 'key-1', client);
    expect(sendHeartbeat).toHaveBeenCalledTimes(1);

    fireWindowState(false);
    vi.advanceTimersByTime(2_000);
    expect(sendHeartbeat).toHaveBeenCalledTimes(1); // still paused

    fireWindowState(true); // regain focus — should fire immediately
    expect(sendHeartbeat).toHaveBeenCalledTimes(2);
  });

  it('resumes interval ticks after window regains focus', () => {
    const { client, sendHeartbeat } = makeClient();
    manager.start('proj-1', 'key-1', client);

    fireWindowState(false);
    vi.advanceTimersByTime(2_000);
    expect(sendHeartbeat).toHaveBeenCalledTimes(1);

    fireWindowState(true); // immediate tick on focus regain
    expect(sendHeartbeat).toHaveBeenCalledTimes(2);

    vi.advanceTimersByTime(1_000);
    expect(sendHeartbeat).toHaveBeenCalledTimes(3); // interval resumes
  });

  // -------------------------------------------------------------------------
  // Error handling — failures must not throw to the caller
  // -------------------------------------------------------------------------

  it('does not throw or stop when sendHeartbeat rejects', async () => {
    const { client } = makeClient(new Error('network failure'));
    manager.start('proj-1', 'key-1', client);

    vi.advanceTimersByTime(1_000);
    await vi.runAllTicks(); // flush microtask queue

    expect(manager.isRunning()).toBe(true);
  });

  // -------------------------------------------------------------------------
  // Configuration change — stop when connectionMode → 'local'
  // -------------------------------------------------------------------------

  it('stops when mahalaxmi.connectionMode changes to local', () => {
    const { client } = makeClient();
    manager.start('proj-1', 'key-1', client);
    expect(manager.isRunning()).toBe(true);

    shared.connectionMode.value = 'local';
    fireConfigChange('mahalaxmi.connectionMode');

    expect(manager.isRunning()).toBe(false);
  });

  it('keeps running when mahalaxmi.connectionMode changes to remote', () => {
    const { client } = makeClient();
    manager.start('proj-1', 'key-1', client);

    shared.connectionMode.value = 'remote';
    fireConfigChange('mahalaxmi.connectionMode');

    expect(manager.isRunning()).toBe(true);
  });

  it('keeps running when an unrelated configuration key changes', () => {
    const { client } = makeClient();
    manager.start('proj-1', 'key-1', client);

    fireConfigChange('mahalaxmi.someOtherSetting');

    expect(manager.isRunning()).toBe(true);
  });

  // -------------------------------------------------------------------------
  // dispose() — stops interval and removes all listeners
  // -------------------------------------------------------------------------

  it('dispose() stops the interval so no further heartbeats are sent', () => {
    const { client, sendHeartbeat } = makeClient();
    manager.start('proj-1', 'key-1', client);

    manager.dispose();

    vi.advanceTimersByTime(10_000);
    expect(sendHeartbeat).toHaveBeenCalledTimes(1); // only the initial immediate one
    expect(manager.isRunning()).toBe(false);
  });

  it('dispose() removes the window state listener', () => {
    const { client } = makeClient();
    manager.start('proj-1', 'key-1', client);
    const listenerCountBefore = shared.windowStateListeners.length;

    manager.dispose();

    expect(shared.windowStateListeners.length).toBeLessThan(listenerCountBefore);
  });
});

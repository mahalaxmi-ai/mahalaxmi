// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { EventEmitter } from 'events';
import { MahalaxmiStatusBar } from './statusbar';
import type { CloudVmState } from './statusbar';

// ---------------------------------------------------------------------------
// Hoist shared mock state — must run before any imports or mock factories
// ---------------------------------------------------------------------------

const shared = vi.hoisted(() => {
  class MockStatusBarItem {
    text = '';
    tooltip = '';
    command: string | undefined = undefined;
    backgroundColor: unknown = undefined;
    disposeCalled = false;

    show(): void { /* noop */ }
    dispose(): void { this.disposeCalled = true; }
  }

  return {
    MockStatusBarItem,
    createStatusBarItem: vi.fn(),
    lastItem: null as InstanceType<typeof MockStatusBarItem> | null,
  };
});

// ---------------------------------------------------------------------------
// Mock vscode — factory only references hoisted mocks
// ---------------------------------------------------------------------------

vi.mock('vscode', () => {
  class ThemeColor {
    id: string;
    constructor(id: string) {
      this.id = id;
    }
  }

  return {
    window: {
      createStatusBarItem: shared.createStatusBarItem,
    },
    workspace: {
      getConfiguration: () => ({
        get: (_key: string, defaultValue: string) => defaultValue,
      }),
    },
    ThemeColor,
    StatusBarAlignment: { Left: 1, Right: 2 },
    l10n: {
      t: (str: string, ...args: unknown[]): string =>
        args.reduce<string>((s, arg, i) => s.replace(`{${i}}`, String(arg)), str),
    },
  };
});

// ---------------------------------------------------------------------------
// Mock ws — no imported base class inside the factory
// ---------------------------------------------------------------------------

vi.mock('ws', () => {
  class MockWS {
    static OPEN = 1;
    readyState = 1;
    private readonly _listeners: Map<string, Array<(...args: unknown[]) => void>> = new Map();
    on(event: string, listener: (...args: unknown[]) => void): this {
      const list = this._listeners.get(event) ?? [];
      list.push(listener);
      this._listeners.set(event, list);
      return this;
    }
    emit(event: string, ...args: unknown[]): void {
      for (const l of (this._listeners.get(event) ?? [])) l(...args);
    }
    send(): void { /* noop */ }
    close(): void { /* noop */ }
  }
  return { default: MockWS };
});

// ---------------------------------------------------------------------------
// Stub MahalaxmiClient and ExtensionContext
// ---------------------------------------------------------------------------

class StubClient extends EventEmitter {}

function makeContext(): { subscriptions: { dispose: () => void }[] } {
  return { subscriptions: [] };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('MahalaxmiStatusBar', () => {
  let client: StubClient;

  beforeEach(() => {
    vi.clearAllMocks();
    shared.lastItem = null;
    client = new StubClient();

    shared.createStatusBarItem.mockImplementation(() => {
      const item = new shared.MockStatusBarItem();
      shared.lastItem = item;
      return item;
    });
  });

  function item(): InstanceType<typeof shared.MockStatusBarItem> {
    if (shared.lastItem === null) throw new Error('No status bar item created');
    return shared.lastItem;
  }

  it('text contains "Offline" variant after construction (before any events)', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    new MahalaxmiStatusBar(client as any, makeContext() as any);
    expect(item().text).toContain('Offline');
  });

  it('text updates to phase name after receiving a snapshot event', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    new MahalaxmiStatusBar(client as any, makeContext() as any);

    client.emit('snapshot', { cycle_id: 'c1', phase: 'Executing', workers: [] });

    expect(item().text).toContain('Executing');
    expect(item().text).not.toContain('Offline');
  });

  it('text updates to "Idle" after connected event', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    new MahalaxmiStatusBar(client as any, makeContext() as any);

    client.emit('connected');

    expect(item().text).toContain('Idle');
  });

  it('text reverts to "Offline" variant after disconnected event', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    new MahalaxmiStatusBar(client as any, makeContext() as any);

    client.emit('connected');
    expect(item().text).toContain('Idle');

    client.emit('disconnected');
    expect(item().text).toContain('Offline');
  });

  it('sets warningBackground color when disconnected', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    new MahalaxmiStatusBar(client as any, makeContext() as any);

    const bg = item().backgroundColor as { id: string };
    expect(bg?.id).toBe('statusBarItem.warningBackground');
  });

  it('clears backgroundColor when showing a phase', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    new MahalaxmiStatusBar(client as any, makeContext() as any);

    client.emit('snapshot', { cycle_id: 'c1', phase: 'Running', workers: [] });

    expect(item().backgroundColor).toBeUndefined();
  });

  it('dispose() calls item.dispose()', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const bar = new MahalaxmiStatusBar(client as any, makeContext() as any);
    bar.dispose();
    expect(item().disposeCalled).toBe(true);
  });

  it('pushes itself to context.subscriptions', () => {
    const context = makeContext();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const bar = new MahalaxmiStatusBar(client as any, context as any);
    expect(context.subscriptions).toContain(bar);
  });

  // ---------------------------------------------------------------------------
  // Cloud VM lifecycle state tests
  // ---------------------------------------------------------------------------

  describe('setCloudVmState', () => {
    let bar: MahalaxmiStatusBar;

    beforeEach(() => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      bar = new MahalaxmiStatusBar(client as any, makeContext() as any);
    });

    it('Offline: text contains CLOUD and Offline, sets warningBackground', () => {
      const state: CloudVmState = { kind: 'Offline' };
      bar.setCloudVmState(state);
      expect(item().text).toContain('CLOUD');
      expect(item().text).toContain('Offline');
      const bg = item().backgroundColor as { id: string };
      expect(bg?.id).toBe('statusBarItem.warningBackground');
    });

    it('Starting: text contains CLOUD, sync~spin icon, and Starting VM...', () => {
      const state: CloudVmState = { kind: 'Starting' };
      bar.setCloudVmState(state);
      expect(item().text).toContain('CLOUD');
      expect(item().text).toContain('$(sync~spin)');
      expect(item().text).toContain('Starting VM...');
      expect(item().backgroundColor).toBeUndefined();
    });

    it('Connected: text contains CLOUD, cloud icon, and Idle', () => {
      const state: CloudVmState = { kind: 'Connected' };
      bar.setCloudVmState(state);
      expect(item().text).toContain('CLOUD');
      expect(item().text).toContain('$(cloud)');
      expect(item().text).toContain('Idle');
      expect(item().backgroundColor).toBeUndefined();
    });

    it('Running: text contains CLOUD, worker count as NW, and cost to 2 decimal places', () => {
      const state: CloudVmState = { kind: 'Running', workerCount: 3, costUsd: 1.23 };
      bar.setCloudVmState(state);
      expect(item().text).toContain('CLOUD');
      expect(item().text).toContain('3W');
      expect(item().text).toContain('$1.23');
      expect(item().backgroundColor).toBeUndefined();
    });

    it('Running: formats cost to exactly 2 decimal places', () => {
      const state: CloudVmState = { kind: 'Running', workerCount: 1, costUsd: 5 };
      bar.setCloudVmState(state);
      expect(item().text).toContain('$5.00');
    });

    it('Idle with countdownSeconds >= 60: shows mm:ss countdown, clears background', () => {
      const state: CloudVmState = { kind: 'Idle', countdownSeconds: 90 };
      bar.setCloudVmState(state);
      expect(item().text).toContain('CLOUD');
      expect(item().text).toContain('01:30');
      expect(item().backgroundColor).toBeUndefined();
    });

    it('Idle with countdownSeconds < 60: shows mm:ss countdown and sets warningBackground', () => {
      const state: CloudVmState = { kind: 'Idle', countdownSeconds: 45 };
      bar.setCloudVmState(state);
      expect(item().text).toContain('CLOUD');
      expect(item().text).toContain('00:45');
      const bg = item().backgroundColor as { id: string };
      expect(bg?.id).toBe('statusBarItem.warningBackground');
    });

    it('Idle with countdownSeconds === 60: does not set warningBackground', () => {
      const state: CloudVmState = { kind: 'Idle', countdownSeconds: 60 };
      bar.setCloudVmState(state);
      expect(item().text).toContain('01:00');
      expect(item().backgroundColor).toBeUndefined();
    });

    it('Stopping: text contains CLOUD, loading~spin icon, and Stopping...', () => {
      const state: CloudVmState = { kind: 'Stopping' };
      bar.setCloudVmState(state);
      expect(item().text).toContain('CLOUD');
      expect(item().text).toContain('$(loading~spin)');
      expect(item().text).toContain('Stopping...');
      expect(item().backgroundColor).toBeUndefined();
    });

    it('Expired: text contains CLOUD, error icon, and Subscription Expired', () => {
      const state: CloudVmState = { kind: 'Expired' };
      bar.setCloudVmState(state);
      expect(item().text).toContain('CLOUD');
      expect(item().text).toContain('$(error)');
      expect(item().text).toContain('Subscription Expired');
      const bg = item().backgroundColor as { id: string };
      expect(bg?.id).toBe('statusBarItem.errorBackground');
    });

    it('Expired: tooltip instructs renewal at mahalaxmi.ai', () => {
      const state: CloudVmState = { kind: 'Expired' };
      bar.setCloudVmState(state);
      expect(String(item().tooltip)).toContain('mahalaxmi.ai');
    });
  });
});

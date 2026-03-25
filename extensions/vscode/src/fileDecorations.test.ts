// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { EventEmitter } from 'events';

// ---------------------------------------------------------------------------
// Hoisted mock state
// ---------------------------------------------------------------------------

const mocks = vi.hoisted(() => ({
  fireArgs: [] as Array<unknown>,
}));

vi.mock('vscode', () => {
  class EventEmitterMock<T> {
    private listeners: Array<(e: T) => void> = [];
    event = (listener: (e: T) => void) => { this.listeners.push(listener); return { dispose: vi.fn() }; };
    fire(e: T): void {
      mocks.fireArgs.push(e);
      for (const l of this.listeners) l(e);
    }
    dispose(): void { this.listeners = []; }
  }

  return {
    EventEmitter: EventEmitterMock,
    Uri: {
      file: (p: string) => ({ fsPath: p, scheme: 'file', path: p }),
    },
    ThemeColor: class ThemeColor {
      id: string;
      constructor(id: string) { this.id = id; }
    },
  };
});

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

class StubClient extends EventEmitter {
  emitSnapshot(acceptances: Record<string, boolean>): void {
    this.emit('snapshot', {
      cycle_id: 'c1', phase: 'Running', workers: [],
      running_cost_usd: 0, file_acceptances: acceptances,
    });
  }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('MahalaxmiFileDecorationProvider', () => {
  let client: StubClient;

  beforeEach(async () => {
    mocks.fireArgs.length = 0;
    client = new StubClient();
  });

  it('provides green ✓ badge for accepted files', async () => {
    const { MahalaxmiFileDecorationProvider } = await import('./fileDecorations');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const provider = new MahalaxmiFileDecorationProvider(client as any);
    client.emitSnapshot({ '/src/foo.rs': true });

    const dec = provider.provideFileDecoration({ fsPath: '/src/foo.rs' } as never);
    expect(dec?.badge).toBe('✓');
    provider.dispose();
  });

  it('provides red ✗ badge for rejected files', async () => {
    const { MahalaxmiFileDecorationProvider } = await import('./fileDecorations');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const provider = new MahalaxmiFileDecorationProvider(client as any);
    client.emitSnapshot({ '/src/bar.rs': false });

    const dec = provider.provideFileDecoration({ fsPath: '/src/bar.rs' } as never);
    expect(dec?.badge).toBe('✗');
    provider.dispose();
  });

  it('returns undefined for files not in acceptances', async () => {
    const { MahalaxmiFileDecorationProvider } = await import('./fileDecorations');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const provider = new MahalaxmiFileDecorationProvider(client as any);
    client.emitSnapshot({});

    const dec = provider.provideFileDecoration({ fsPath: '/src/unknown.rs' } as never);
    expect(dec).toBeUndefined();
    provider.dispose();
  });

  it('fires onDidChangeFileDecorations when snapshot arrives', async () => {
    const { MahalaxmiFileDecorationProvider } = await import('./fileDecorations');
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const provider = new MahalaxmiFileDecorationProvider(client as any);
    client.emitSnapshot({ '/src/foo.rs': true });

    expect(mocks.fireArgs.length).toBeGreaterThan(0);
    provider.dispose();
  });
});

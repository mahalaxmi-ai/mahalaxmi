// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { EventEmitter } from 'events';
import { DiffManager } from './diff';

// ---------------------------------------------------------------------------
// Hoist vi.fn() mocks so mock factories can reference them safely
// ---------------------------------------------------------------------------

const mocks = vi.hoisted(() => ({
  showInformationMessage: vi.fn(),
  showWarningMessage: vi.fn(),
  executeCommand: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
  registerTextDocumentContentProvider: vi.fn().mockReturnValue({ dispose: vi.fn() }),
}));

// ---------------------------------------------------------------------------
// Mock vscode — factory only references hoisted mocks
// ---------------------------------------------------------------------------

vi.mock('vscode', () => {
  class Uri {
    scheme: string;
    path: string;

    constructor(scheme: string, path: string) {
      this.scheme = scheme;
      this.path = path;
    }

    static file(p: string): Uri {
      return new Uri('file', p);
    }

    static from(components: { scheme: string; path: string }): Uri {
      return new Uri(components.scheme, components.path);
    }
  }

  return {
    Uri,
    workspace: {
      registerTextDocumentContentProvider: mocks.registerTextDocumentContentProvider,
    },
    commands: {
      executeCommand: mocks.executeCommand,
    },
    window: {
      showInformationMessage: mocks.showInformationMessage,
      showWarningMessage: mocks.showWarningMessage,
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
// Stub MahalaxmiClient
// ---------------------------------------------------------------------------

class StubClient extends EventEmitter {
  sentMessages: Array<{ type: string; payload: unknown }> = [];

  send(type: string, payload: unknown): void {
    this.sentMessages.push({ type, payload });
  }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('DiffManager', () => {
  let client: StubClient;
  let diffManager: DiffManager;

  beforeEach(() => {
    vi.clearAllMocks();
    client = new StubClient();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    diffManager = new DiffManager(client as any);
  });

  it('accept() calls client.send("accept_file", ...) with correct cycleId and file', () => {
    diffManager.accept('cycle-123', 'src/main.rs');

    expect(client.sentMessages).toHaveLength(1);
    expect(client.sentMessages[0].type).toBe('accept_file');
    expect(client.sentMessages[0].payload).toEqual({ cycle_id: 'cycle-123', file: 'src/main.rs' });
  });

  it('accept() shows an information message', () => {
    diffManager.accept('cycle-123', 'src/main.rs');
    expect(mocks.showInformationMessage).toHaveBeenCalledWith('Accepted: src/main.rs');
  });

  it('reject() calls client.send("reject_file", ...) with correct cycleId and file', () => {
    diffManager.reject('cycle-456', 'src/lib.rs');

    expect(client.sentMessages).toHaveLength(1);
    expect(client.sentMessages[0].type).toBe('reject_file');
    expect(client.sentMessages[0].payload).toEqual({ cycle_id: 'cycle-456', file: 'src/lib.rs' });
  });

  it('reject() shows a warning message', () => {
    diffManager.reject('cycle-456', 'src/lib.rs');
    expect(mocks.showWarningMessage).toHaveBeenCalledWith('Rejected: src/lib.rs');
  });

  it('WorkerFileWritten event stores original_content in the map', async () => {
    const payload = {
      type: 'WorkerFileWritten',
      file: 'src/foo.ts',
      original_content: 'const x = 1;',
    };

    client.emit('orchestration_event', payload);

    // Trigger showDiff — the provider registered should use the stored original content
    await diffManager.showDiff('src/foo.ts');

    expect(mocks.registerTextDocumentContentProvider).toHaveBeenCalled();

    const providerArg = mocks.registerTextDocumentContentProvider.mock.calls[0][1] as {
      provideTextDocumentContent(): string;
    };
    expect(providerArg.provideTextDocumentContent()).toBe('const x = 1;');
  });

  it('showDiff uses empty string as original when file has no stored content', async () => {
    await diffManager.showDiff('src/unknown.ts');

    const providerArg = mocks.registerTextDocumentContentProvider.mock.calls[0][1] as {
      provideTextDocumentContent(): string;
    };
    expect(providerArg.provideTextDocumentContent()).toBe('');
  });

  it('ignores orchestration_event payloads that are not WorkerFileWritten', async () => {
    client.emit('orchestration_event', { type: 'WorkerStarted', worker_id: 'w1' });
    await diffManager.showDiff('src/never-stored.ts');

    const providerArg = mocks.registerTextDocumentContentProvider.mock.calls[0][1] as {
      provideTextDocumentContent(): string;
    };
    expect(providerArg.provideTextDocumentContent()).toBe('');
  });

  it('ignores orchestration_event payloads missing original_content', async () => {
    client.emit('orchestration_event', { type: 'WorkerFileWritten', file: 'src/partial.ts' });
    await diffManager.showDiff('src/partial.ts');

    const providerArg = mocks.registerTextDocumentContentProvider.mock.calls[0][1] as {
      provideTextDocumentContent(): string;
    };
    expect(providerArg.provideTextDocumentContent()).toBe('');
  });
});

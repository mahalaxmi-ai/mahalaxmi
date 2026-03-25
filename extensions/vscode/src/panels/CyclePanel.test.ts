// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { EventEmitter } from 'events';
import { CyclePanel, PlaceholderNode, PhaseNode, WorkerNode, FileNode } from './CyclePanel';
import { OrchestrationSnapshotDto } from '../client';

// ---------------------------------------------------------------------------
// Mock vscode
// ---------------------------------------------------------------------------

vi.mock('vscode', () => {
  class TreeItem {
    label: string;
    collapsibleState: number;
    iconPath?: { id: string };
    description?: string;
    resourceUri?: { path: string };
    command?: { command: string; title: string; arguments?: unknown[] };

    constructor(label: string, collapsibleState: number) {
      this.label = label;
      this.collapsibleState = collapsibleState;
    }
  }

  class ThemeIcon {
    id: string;
    constructor(id: string) {
      this.id = id;
    }
  }

  class EventEmitterVsc {
    private listeners: Array<() => void> = [];
    event = (listener: () => void) => {
      this.listeners.push(listener);
      return { dispose: () => { /* noop */ } };
    };
    fire() {
      for (const l of this.listeners) {
        l();
      }
    }
  }

  class Uri {
    path: string;
    constructor(path: string) {
      this.path = path;
    }
    static file(p: string): Uri {
      return new Uri(p);
    }
  }

  return {
    TreeItem,
    ThemeIcon,
    EventEmitter: EventEmitterVsc,
    Uri,
    TreeItemCollapsibleState: {
      None: 0,
      Collapsed: 1,
      Expanded: 2,
    },
  };
});

// ---------------------------------------------------------------------------
// Mock ws (not used directly in CyclePanel but needed transitively)
// ---------------------------------------------------------------------------

vi.mock('ws', () => {
  // Use inline listener map — no imported EventEmitter in the factory
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
      for (const l of (this._listeners.get(event) ?? [])) {
        l(...args);
      }
    }

    send(): void { /* noop */ }
    close(): void { /* noop */ }
  }
  return { default: MockWS };
});

// ---------------------------------------------------------------------------
// Minimal MahalaxmiClient stub
// ---------------------------------------------------------------------------

class StubClient extends EventEmitter {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('CyclePanel', () => {
  let client: StubClient;
  let panel: CyclePanel;

  beforeEach(() => {
    client = new StubClient();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    panel = new CyclePanel(client as any);
  });

  it('getChildren() returns a single PlaceholderNode when snapshot is null', () => {
    const children = panel.getChildren();
    expect(children).toHaveLength(1);
    expect(children[0]).toBeInstanceOf(PlaceholderNode);
  });

  it('getChildren() returns PhaseNode + WorkerNodes after receiving a snapshot', () => {
    const snapshot: OrchestrationSnapshotDto = {
      cycle_id: 'c1',
      phase: 'Executing',
      workers: [
        { worker_id: 'w1', task_title: 'Task A', status: 'Running', files_modified: [] },
        { worker_id: 'w2', task_title: 'Task B', status: 'Pending', files_modified: [] },
      ],
    };

    client.emit('snapshot', snapshot);

    const children = panel.getChildren();
    expect(children).toHaveLength(3); // PhaseNode + 2 WorkerNodes
    expect(children[0]).toBeInstanceOf(PhaseNode);
    expect(children[1]).toBeInstanceOf(WorkerNode);
    expect(children[2]).toBeInstanceOf(WorkerNode);
  });

  it('WorkerNode with status Completed has icon "check"', () => {
    const snapshot: OrchestrationSnapshotDto = {
      cycle_id: 'c1',
      phase: 'Done',
      workers: [
        { worker_id: 'w1', task_title: 'Done Task', status: 'Completed', files_modified: [] },
      ],
    };

    client.emit('snapshot', snapshot);

    const children = panel.getChildren();
    const workerNode = children[1] as WorkerNode;
    expect(workerNode).toBeInstanceOf(WorkerNode);

    const icon = workerNode.iconPath as { id: string };
    expect(icon.id).toBe('check');
  });

  it('WorkerNode with status Running has icon "loading~spin"', () => {
    const snapshot: OrchestrationSnapshotDto = {
      cycle_id: 'c1',
      phase: 'Executing',
      workers: [
        { worker_id: 'w1', task_title: 'Active Task', status: 'Running', files_modified: [] },
      ],
    };

    client.emit('snapshot', snapshot);

    const children = panel.getChildren();
    const workerNode = children[1] as WorkerNode;
    const icon = workerNode.iconPath as { id: string };
    expect(icon.id).toBe('loading~spin');
  });

  it('WorkerNode with other status has icon "circle-outline"', () => {
    const snapshot: OrchestrationSnapshotDto = {
      cycle_id: 'c1',
      phase: 'Queued',
      workers: [
        { worker_id: 'w1', task_title: 'Pending Task', status: 'Pending', files_modified: [] },
      ],
    };

    client.emit('snapshot', snapshot);

    const children = panel.getChildren();
    const workerNode = children[1] as WorkerNode;
    const icon = workerNode.iconPath as { id: string };
    expect(icon.id).toBe('circle-outline');
  });

  it('getChildren(WorkerNode) returns FileNodes for each modified file', () => {
    const snapshot: OrchestrationSnapshotDto = {
      cycle_id: 'c1',
      phase: 'Executing',
      workers: [
        {
          worker_id: 'w1',
          task_title: 'File Task',
          status: 'Running',
          files_modified: ['/path/to/a.ts', '/path/to/b.ts'],
        },
      ],
    };

    client.emit('snapshot', snapshot);

    const children = panel.getChildren();
    const workerNode = children[1] as WorkerNode;
    const fileChildren = panel.getChildren(workerNode);

    expect(fileChildren).toHaveLength(2);
    expect(fileChildren[0]).toBeInstanceOf(FileNode);
    expect(fileChildren[1]).toBeInstanceOf(FileNode);
  });

  it('reverts to PlaceholderNode after disconnected event', () => {
    const snapshot: OrchestrationSnapshotDto = {
      cycle_id: 'c1',
      phase: 'Executing',
      workers: [],
    };

    client.emit('snapshot', snapshot);
    expect(panel.getChildren()).not.toHaveLength(0);

    client.emit('disconnected');
    const children = panel.getChildren();
    expect(children).toHaveLength(1);
    expect(children[0]).toBeInstanceOf(PlaceholderNode);
  });
});

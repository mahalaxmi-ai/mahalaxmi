// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';
import type { IMahalaxmiClient, OrchestrationSnapshotDto, WorkerSnapshotDto } from '../client-interface';

// ---------------------------------------------------------------------------
// Node types
// ---------------------------------------------------------------------------

/** Base class for all tree nodes in the Mahalaxmi sidebar. */
export abstract class CycleNode extends vscode.TreeItem {}

/** Shown at the root when the client is disconnected. */
export class PlaceholderNode extends CycleNode {
  constructor() {
    super('Not connected to Mahalaxmi', vscode.TreeItemCollapsibleState.None);
    this.iconPath = new vscode.ThemeIcon('circle-slash');
  }
}

/** Root node displaying the current orchestration phase. */
export class PhaseNode extends CycleNode {
  constructor(phase: string) {
    super(`Phase: ${phase}`, vscode.TreeItemCollapsibleState.None);
    this.iconPath = new vscode.ThemeIcon('pulse');
  }
}

/** Node representing a single worker. */
export class WorkerNode extends CycleNode {
  readonly worker: WorkerSnapshotDto;

  constructor(worker: WorkerSnapshotDto) {
    super(worker.task_title, vscode.TreeItemCollapsibleState.Collapsed);
    this.worker = worker;
    this.description = worker.status;

    if (worker.status === 'Completed') {
      this.iconPath = new vscode.ThemeIcon('check');
    } else if (worker.status === 'Running') {
      this.iconPath = new vscode.ThemeIcon('loading~spin');
    } else if (worker.status === 'Stalled') {
      this.iconPath = new vscode.ThemeIcon('warning');
      this.description = `${worker.status} \u26A0`;
    } else {
      this.iconPath = new vscode.ThemeIcon('circle-outline');
    }
  }
}

/** Leaf node showing a modified file path. Clicking opens the file. */
export class FileNode extends CycleNode {
  constructor(filePath: string) {
    super(filePath, vscode.TreeItemCollapsibleState.None);
    this.resourceUri = vscode.Uri.file(filePath);
    this.command = {
      command: 'vscode.open',
      title: 'Open File',
      arguments: [vscode.Uri.file(filePath)],
    };
  }
}

// ---------------------------------------------------------------------------
// Tree data provider
// ---------------------------------------------------------------------------

/** Provides the Mahalaxmi cycle status tree in the VS Code sidebar. */
export class CyclePanel implements vscode.TreeDataProvider<CycleNode> {
  private readonly _onDidChangeTreeData = new vscode.EventEmitter<CycleNode | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<CycleNode | undefined | null | void> =
    this._onDidChangeTreeData.event;

  private snapshot: OrchestrationSnapshotDto | null = null;

  constructor(client: IMahalaxmiClient) {
    this.attachListeners(client);
  }

  /** Hot-swap the underlying transport. The sidebar refreshes automatically. */
  setClient(client: IMahalaxmiClient): void {
    this.snapshot = null;
    this._onDidChangeTreeData.fire();
    this.attachListeners(client);
  }

  private attachListeners(client: IMahalaxmiClient): void {
    client.on('snapshot', (s: OrchestrationSnapshotDto) => {
      this.snapshot = s;
      this._onDidChangeTreeData.fire();
    });

    client.on('orchestration_event', () => {
      this._onDidChangeTreeData.fire();
    });

    client.on('disconnected', () => {
      this.snapshot = null;
      this._onDidChangeTreeData.fire();
    });
  }

  getTreeItem(node: CycleNode): vscode.TreeItem {
    return node;
  }

  getChildren(node?: CycleNode): CycleNode[] {
    if (node instanceof WorkerNode) {
      return node.worker.files_modified.map((f) => new FileNode(f));
    }

    if (this.snapshot === null) {
      return [new PlaceholderNode()];
    }

    return [
      new PhaseNode(this.snapshot.phase),
      ...this.snapshot.workers.map((w) => new WorkerNode(w)),
    ];
  }
}

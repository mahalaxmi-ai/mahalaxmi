// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';
import { MahalaxmiClient } from './client';

interface WorkerFileWrittenEvent {
  type: 'WorkerFileWritten';
  file: string;
  original_content: string;
}

function isWorkerFileWrittenEvent(payload: unknown): payload is WorkerFileWrittenEvent {
  if (typeof payload !== 'object' || payload === null) {
    return false;
  }
  const p = payload as Record<string, unknown>;
  return (
    p['type'] === 'WorkerFileWritten' &&
    typeof p['file'] === 'string' &&
    typeof p['original_content'] === 'string'
  );
}

/** Manages original-vs-current file diffs for worker-modified files. */
export class DiffManager {
  private readonly originalContents = new Map<string, string>();

  constructor(private readonly client: MahalaxmiClient) {
    client.on('orchestration_event', (payload: unknown) => {
      if (isWorkerFileWrittenEvent(payload)) {
        this.originalContents.set(payload.file, payload.original_content);
      }
    });
  }

  /** Open a diff editor comparing the original content to the current on-disk file. */
  async showDiff(filePath: string): Promise<void> {
    const original = this.originalContents.get(filePath) ?? '';
    const scheme = 'mahalaxmi-original';

    const provider = vscode.workspace.registerTextDocumentContentProvider(scheme, {
      provideTextDocumentContent(): string {
        return original;
      },
    });

    const originalUri = vscode.Uri.from({ scheme, path: filePath });
    const currentUri = vscode.Uri.file(filePath);

    await vscode.commands.executeCommand(
      'vscode.diff',
      originalUri,
      currentUri,
      `Original ↔ Current: ${filePath}`,
    );

    setTimeout(() => provider.dispose(), 5000);
  }

  /** Accept a worker-modified file and notify the backend. */
  accept(cycleId: string, file: string): void {
    this.client.send('accept_file', { cycle_id: cycleId, file });
    vscode.window.showInformationMessage(`Accepted: ${file}`);
  }

  /** Reject a worker-modified file and notify the backend. */
  reject(cycleId: string, file: string): void {
    this.client.send('reject_file', { cycle_id: cycleId, file });
    vscode.window.showWarningMessage(`Rejected: ${file}`);
  }
}

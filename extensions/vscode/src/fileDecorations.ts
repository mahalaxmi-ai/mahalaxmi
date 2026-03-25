// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';
import type { IMahalaxmiClient, OrchestrationSnapshotDto } from './client-interface';

/**
 * File decoration provider that shows ✓/✗ badges on files that have been
 * accepted or rejected in the current Mahalaxmi cycle.
 */
export class MahalaxmiFileDecorationProvider implements vscode.FileDecorationProvider, vscode.Disposable {
  private readonly _onDidChangeFileDecorations = new vscode.EventEmitter<vscode.Uri | vscode.Uri[]>();
  readonly onDidChangeFileDecorations: vscode.Event<vscode.Uri | vscode.Uri[]> =
    this._onDidChangeFileDecorations.event;

  private acceptances: Record<string, boolean> = {};

  constructor(client: IMahalaxmiClient) {
    this.attachListeners(client);
  }

  /** Hot-swap the underlying transport; existing decorations are preserved. */
  setClient(client: IMahalaxmiClient): void {
    this.attachListeners(client);
  }

  private attachListeners(client: IMahalaxmiClient): void {
    client.on('snapshot', (s: OrchestrationSnapshotDto) => {
      this.acceptances = s.file_acceptances ?? {};
      this._onDidChangeFileDecorations.fire(
        Object.keys(this.acceptances).map((p) => vscode.Uri.file(p)),
      );
    });

    client.on('disconnected', () => {
      this.acceptances = {};
      this._onDidChangeFileDecorations.fire([]);
    });
  }

  provideFileDecoration(uri: vscode.Uri): vscode.FileDecoration | undefined {
    const accepted = this.acceptances[uri.fsPath];
    if (accepted === true) {
      return {
        badge: '✓',
        color: new vscode.ThemeColor('charts.green'),
        tooltip: 'Mahalaxmi: file accepted',
      };
    }
    if (accepted === false) {
      return {
        badge: '✗',
        color: new vscode.ThemeColor('charts.red'),
        tooltip: 'Mahalaxmi: file rejected',
      };
    }
    return undefined;
  }

  dispose(): void {
    this._onDidChangeFileDecorations.dispose();
  }
}

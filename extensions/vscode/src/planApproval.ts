// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';
import * as os from 'os';
import * as path from 'path';
import * as fs from 'fs';
import type { IMahalaxmiClient } from './client-interface';

/**
 * Listens for plan_ready events and shows the execution plan as a diff editor.
 * Prompts the user to Approve or Reject the plan.
 */
export class PlanApprovalManager implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private currentClient: IMahalaxmiClient;

  constructor(client: IMahalaxmiClient) {
    this.currentClient = client;
    this.attachListeners(client);
  }

  /** Hot-swap the underlying transport. */
  setClient(client: IMahalaxmiClient): void {
    this.currentClient = client;
    this.attachListeners(client);
  }

  private attachListeners(client: IMahalaxmiClient): void {
    client.on('plan_ready', async (cycleId: string, plan: unknown) => {
      await this.showPlanDiff(this.currentClient, cycleId, plan);
    });
  }

  private async showPlanDiff(client: IMahalaxmiClient, cycleId: string, plan: unknown): Promise<void> {
    const planJson = JSON.stringify(plan, null, 2);
    const tmpPath = path.join(os.tmpdir(), `mahalaxmi-plan-${cycleId}.json`);
    fs.writeFileSync(tmpPath, planJson, 'utf8');

    const emptyUri = vscode.Uri.from({ scheme: 'mahalaxmi-empty', path: '/empty' });
    const planUri = vscode.Uri.file(tmpPath);

    const emptyProvider = vscode.workspace.registerTextDocumentContentProvider('mahalaxmi-empty', {
      provideTextDocumentContent: () => '',
    });

    await vscode.commands.executeCommand(
      'vscode.diff',
      emptyUri,
      planUri,
      vscode.l10n.t('Mahalaxmi Execution Plan \u2014 {0}', cycleId),
    );

    const response = await vscode.window.showInformationMessage(
      vscode.l10n.t('Approve the Mahalaxmi execution plan?'),
      { modal: true },
      vscode.l10n.t('Approve'),
      vscode.l10n.t('Reject'),
    );

    if (response === vscode.l10n.t('Approve')) {
      client.approvePlan(cycleId);
      vscode.window.showInformationMessage(vscode.l10n.t('Mahalaxmi: Plan approved — workers dispatching.'));
    } else {
      vscode.window.showWarningMessage(vscode.l10n.t('Mahalaxmi: Plan rejected.'));
    }

    // Cleanup
    setTimeout(() => {
      emptyProvider.dispose();
      try { fs.unlinkSync(tmpPath); } catch { /* ignore */ }
    }, 5000);
  }

  dispose(): void {
    this.disposables.forEach((d) => d.dispose());
  }
}

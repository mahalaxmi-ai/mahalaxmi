// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';
import type { IMahalaxmiClient, CostUpdatedEvent, OrchestrationSnapshotDto } from './client-interface';

/** Cloud VM lifecycle states for cloud-hosted orchestration workers. */
export type CloudVmState =
  | { kind: 'Offline' }
  | { kind: 'Starting' }
  | { kind: 'Connected' }
  | { kind: 'Running'; workerCount: number; costUsd: number }
  | { kind: 'Idle'; countdownSeconds: number }
  | { kind: 'Stopping' }
  | { kind: 'Expired' };

/** Displays Mahalaxmi cycle status and live cost in the VS Code status bar. */
export class MahalaxmiStatusBar implements vscode.Disposable {
  private readonly item: vscode.StatusBarItem;
  private workerCount = 0;
  private costUsd = 0;
  private workerTaskNames: string[] = [];

  constructor(client: IMahalaxmiClient, context: vscode.ExtensionContext) {
    this.item = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    this.item.command = 'workbench.view.extension.mahalaxmi';
    this.setDisconnected();
    this.item.show();

    this.attachListeners(client);

    context.subscriptions.push(this);
  }

  /** Hot-swap the underlying transport without recreating the status bar item. */
  setClient(client: IMahalaxmiClient): void {
    this.attachListeners(client);
  }

  private attachListeners(client: IMahalaxmiClient): void {
    client.on('connected', () => this.setPhase('Idle'));
    client.on('disconnected', () => this.setDisconnected());
    client.on('snapshot', (s: OrchestrationSnapshotDto) => {
      this.workerCount = s.workers.filter((w) => w.status === 'Running').length;
      this.costUsd = s.running_cost_usd;
      this.render(s.phase);
    });
    client.on('cost_updated', (e: CostUpdatedEvent) => {
      this.workerCount = e.active_worker_count;
      this.costUsd = e.total_cost_usd;
      this.workerTaskNames = e.worker_task_names;
      this.renderCost();
    });
  }

  private getMode(): string {
    return vscode.workspace
      .getConfiguration('mahalaxmi')
      .get<string>('connectionMode', 'local')
      .toUpperCase();
  }

  setPhase(phase: string): void {
    const mode = this.getMode();
    const modeLabel = mode === 'LOCAL'
      ? vscode.l10n.t('Mahalaxmi [LOCAL]')
      : vscode.l10n.t('Mahalaxmi [REMOTE]');
    this.item.text = `$(robot) ${modeLabel}  ${phase}`;
    this.item.tooltip = `${modeLabel} — current phase: ${phase}`;
    this.item.backgroundColor = undefined;
  }

  setDisconnected(): void {
    const mode = this.getMode();
    const modeLabel = mode === 'LOCAL'
      ? vscode.l10n.t('Mahalaxmi [LOCAL]')
      : vscode.l10n.t('Mahalaxmi [REMOTE]');
    this.item.text = `$(robot) ${modeLabel}: Offline`;
    this.item.tooltip = `${modeLabel} — ${vscode.l10n.t('not connected')}`;
    this.item.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
  }

  private render(phase: string): void {
    if (this.workerCount > 0) {
      this.renderCost();
    } else {
      this.setPhase(phase);
    }
  }

  private renderCost(): void {
    const mode = this.getMode();
    const modeLabel = mode === 'LOCAL'
      ? vscode.l10n.t('Mahalaxmi [LOCAL]')
      : vscode.l10n.t('Mahalaxmi [REMOTE]');
    const cost = this.costUsd.toFixed(2);
    this.item.text = `$(robot) ${modeLabel} ⚙ ${this.workerCount}W $${cost}`;
    const taskList =
      this.workerTaskNames.length > 0
        ? this.workerTaskNames.map((t) => `• ${t}`).join('\n')
        : vscode.l10n.t('No active workers');
    this.item.tooltip = `${modeLabel} — ${this.workerCount} active worker(s)\n${taskList}\n${vscode.l10n.t('Total cost: ${0}', cost)}`;
    this.item.backgroundColor = undefined;
  }

  /** Update the status bar to reflect a cloud VM lifecycle state. */
  setCloudVmState(state: CloudVmState): void {
    const label = 'Mahalaxmi [CLOUD]';
    switch (state.kind) {
      case 'Offline':
        this.item.text = `$(cloud) ${label}: Offline`;
        this.item.tooltip = `${label} — not connected`;
        this.item.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
        break;
      case 'Starting':
        this.item.text = `$(sync~spin) ${label}: Starting VM...`;
        this.item.tooltip = `${label} — VM is starting`;
        this.item.backgroundColor = undefined;
        break;
      case 'Connected':
        this.item.text = `$(cloud) ${label}: Idle`;
        this.item.tooltip = `${label} — connected`;
        this.item.backgroundColor = undefined;
        break;
      case 'Running': {
        const cost = state.costUsd.toFixed(2);
        this.item.text = `$(robot) ${label} ⚙ ${state.workerCount}W $${cost}`;
        this.item.tooltip = `${label} — ${state.workerCount} active worker(s)\nTotal cost: $${cost}`;
        this.item.backgroundColor = undefined;
        break;
      }
      case 'Idle': {
        const totalSeconds = state.countdownSeconds;
        const minutes = Math.floor(totalSeconds / 60);
        const seconds = totalSeconds % 60;
        const mm = String(minutes).padStart(2, '0');
        const ss = String(seconds).padStart(2, '0');
        this.item.text = `$(clock) ${label}: Idle ${mm}:${ss}`;
        this.item.tooltip = `${label} — idle, shutting down in ${mm}:${ss}`;
        this.item.backgroundColor = totalSeconds < 60
          ? new vscode.ThemeColor('statusBarItem.warningBackground')
          : undefined;
        break;
      }
      case 'Stopping':
        this.item.text = `$(loading~spin) ${label}: Stopping...`;
        this.item.tooltip = `${label} — VM is stopping`;
        this.item.backgroundColor = undefined;
        break;
      case 'Expired':
        this.item.text = `$(error) ${label}: Subscription Expired`;
        this.item.tooltip = `${label} — subscription expired. Renew at https://mahalaxmi.ai`;
        this.item.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
        break;
    }
  }

  dispose(): void {
    this.item.dispose();
  }
}

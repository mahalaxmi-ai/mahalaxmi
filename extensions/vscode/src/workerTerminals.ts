// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';
import type { IMahalaxmiClient, OutputReceivedEvent, WorkerStartedEvent } from './client-interface';

/**
 * Manages VS Code terminal tabs for each active Mahalaxmi worker.
 * Creates a terminal on worker_started, streams PTY output, marks done on completion.
 */
export class WorkerTerminalManager implements vscode.Disposable {
  private readonly terminals = new Map<string, vscode.Terminal>();
  private readonly writeEmitters = new Map<string, vscode.EventEmitter<string>>();

  constructor(client: IMahalaxmiClient) {
    this.attachListeners(client);
  }

  /** Hot-swap the underlying transport without losing open terminal tabs. */
  setClient(client: IMahalaxmiClient): void {
    this.attachListeners(client);
  }

  private attachListeners(client: IMahalaxmiClient): void {
    client.on('worker_started', (event: WorkerStartedEvent) => {
      this.openTerminal(event.worker_id, event.worker_number, event.task_title);
    });

    client.on('output_received', (event: OutputReceivedEvent) => {
      this.writeEmitters.get(event.worker_id)?.fire(event.data);
    });

    client.on('worker_finished', (workerId: string, passed: boolean) => {
      const emitter = this.writeEmitters.get(workerId);
      if (emitter) {
        const icon = passed ? '✓' : '✗';
        const label = passed ? 'PASSED' : 'FAILED';
        emitter.fire(`\r\n\x1b[1m[Mahalaxmi] Worker ${icon} ${label}\x1b[0m\r\n`);
      }
    });

    client.on('disconnected', () => {
      this.disposeAll();
    });
  }

  private openTerminal(workerId: string, workerNumber: number, taskTitle: string): void {
    if (this.terminals.has(workerId)) return;

    const writeEmitter = new vscode.EventEmitter<string>();
    this.writeEmitters.set(workerId, writeEmitter);

    const pty: vscode.Pseudoterminal = {
      onDidWrite: writeEmitter.event,
      open: () => {
        writeEmitter.fire(`\x1b[1m[Mahalaxmi] Worker-${workerNumber}: ${taskTitle}\x1b[0m\r\n`);
      },
      close: () => {
        writeEmitter.dispose();
        this.writeEmitters.delete(workerId);
        this.terminals.delete(workerId);
      },
    };

    const terminal = vscode.window.createTerminal({
      name: `Mahalaxmi: Worker-${workerNumber} — ${taskTitle}`,
      pty,
      isTransient: true,
    });

    this.terminals.set(workerId, terminal);
  }

  private disposeAll(): void {
    for (const [, terminal] of this.terminals) {
      terminal.dispose();
    }
    this.terminals.clear();
    for (const [, emitter] of this.writeEmitters) {
      emitter.dispose();
    }
    this.writeEmitters.clear();
  }

  dispose(): void {
    this.disposeAll();
  }
}

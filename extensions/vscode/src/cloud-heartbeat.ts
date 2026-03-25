// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';
import type { CloudProvisioningClient } from './heartbeat';

// ---------------------------------------------------------------------------
// Module-level functional API (singleton heartbeat)
// ---------------------------------------------------------------------------

let _timer: ReturnType<typeof setInterval> | undefined;
let _paused = false;
let _endpoint = '';
let _apiKey = '';
let _windowDisposable: vscode.Disposable | undefined;

function _fireHeartbeat(): void {
  fetch(`${_endpoint}/v1/heartbeat`, {
    method: 'POST',
    headers: { Authorization: `Bearer ${_apiKey}` },
  }).catch(() => {});
}

/**
 * Start a singleton heartbeat loop that POSTs to `{endpoint}/v1/heartbeat`
 * every 120 seconds. Pauses ticks while the VS Code window is not focused.
 * Calling `start` while already running stops the previous loop first.
 */
export function start(endpoint: string, apiKey: string): void {
  stop();

  _endpoint = endpoint;
  _apiKey = apiKey;
  _paused = false;

  _timer = setInterval(() => {
    if (!_paused) {
      _fireHeartbeat();
    }
  }, 120_000);

  _windowDisposable = vscode.window.onDidChangeWindowState((state) => {
    const wasPaused = _paused;
    _paused = !state.focused;
    if (wasPaused && !_paused) {
      _fireHeartbeat();
    }
  });
}

/** Stop the singleton heartbeat loop. Safe to call when already stopped. */
export function stop(): void {
  if (_timer !== undefined) {
    clearInterval(_timer);
    _timer = undefined;
  }
  if (_windowDisposable !== undefined) {
    _windowDisposable.dispose();
    _windowDisposable = undefined;
  }
  _paused = false;
}

/**
 * Manages a repeating heartbeat to the ThriveTech Provisioning Service while
 * a cloud server is connected and the VS Code window is open.
 *
 * The manager is a singleton in the sense that calling {@link start} while
 * already running first calls {@link stop}, ensuring only one interval is
 * ever active at a time.
 *
 * Window focus is respected: heartbeat ticks are skipped while the VS Code
 * window is minimized or hidden, and an immediate tick fires when focus
 * returns.
 *
 * A {@link vscode.workspace.onDidChangeConfiguration} listener registered in
 * the constructor automatically stops the heartbeat when
 * `mahalaxmi.connectionMode` changes back to `'local'`.
 */
export class CloudHeartbeatManager {
  private readonly intervalMs: number;

  /** The active setInterval handle, or undefined when stopped. */
  private timer: ReturnType<typeof setInterval> | undefined;

  /** Disposable for the window-state listener created in {@link start}. */
  private windowStateListener: vscode.Disposable | undefined;

  /** Disposable for the configuration watcher created in the constructor. */
  private readonly configListener: vscode.Disposable;

  /** True while heartbeat ticks should be skipped (window not focused). */
  private paused = false;

  private projectId = '';
  private apiKey = '';
  private client: CloudProvisioningClient | undefined;

  /**
   * @param intervalMs Heartbeat interval in milliseconds.
   *   Defaults to 120 000 ms (2 minutes). Pass a smaller value in tests.
   */
  constructor(intervalMs = 120_000) {
    this.intervalMs = intervalMs;

    this.configListener = vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('mahalaxmi.connectionMode')) {
        const mode = vscode.workspace
          .getConfiguration('mahalaxmi')
          .get<string>('connectionMode', 'local');
        if (mode === 'local') {
          this.stop();
        }
      }
    });
  }

  /**
   * Starts the heartbeat loop.
   *
   * If a heartbeat is already running it is stopped first so only one interval
   * is ever live. An immediate heartbeat fires before the first timed tick.
   *
   * @param projectId - UUID of the cloud project to ping.
   * @param apiKey    - API key authorizing the heartbeat.
   * @param client    - Provisioning client that performs the network call.
   */
  start(projectId: string, apiKey: string, client: CloudProvisioningClient): void {
    if (this.isRunning()) {
      this.stop();
    }

    this.projectId = projectId;
    this.apiKey = apiKey;
    this.client = client;

    // Fire the immediate heartbeat unconditionally, then track focus state
    // for subsequent interval ticks.
    this.fireHeartbeat();
    this.paused = !vscode.window.state.focused;

    this.timer = setInterval(() => {
      if (!this.paused) {
        this.fireHeartbeat();
      }
    }, this.intervalMs);

    this.windowStateListener = vscode.window.onDidChangeWindowState((windowState) => {
      const wasPaused = this.paused;
      this.paused = !windowState.focused;

      if (wasPaused && !this.paused) {
        // Focus regained — fire immediately so callers see a timely ping.
        this.fireHeartbeat();
      }
    });
  }

  /**
   * Stops the heartbeat interval and disposes the window-state listener.
   *
   * Safe to call when already stopped.
   */
  stop(): void {
    if (this.timer !== undefined) {
      clearInterval(this.timer);
      this.timer = undefined;
    }

    if (this.windowStateListener !== undefined) {
      this.windowStateListener.dispose();
      this.windowStateListener = undefined;
    }

    this.client = undefined;
    this.paused = false;
  }

  /** Returns `true` when the heartbeat interval is active. */
  isRunning(): boolean {
    return this.timer !== undefined;
  }

  /**
   * Stops the heartbeat and disposes all listeners, including the
   * configuration watcher created in the constructor.
   *
   * After calling this method the instance should not be reused.
   */
  dispose(): void {
    this.stop();
    this.configListener.dispose();
  }

  private fireHeartbeat(): void {
    const client = this.client;
    if (client === undefined) {
      return;
    }

    const projectId = this.projectId;
    const apiKey = this.apiKey;

    client.sendHeartbeat(projectId, apiKey).catch((err: unknown) => {
      process.stderr.write(`[mahalaxmi] cloud heartbeat warning: ${String(err)}\n`);
    });
  }
}

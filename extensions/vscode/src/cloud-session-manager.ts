// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';

import type { CloudConfig } from './cloud-config';
import { RemoteApiClient } from './remote-client';
import type { MahalaxmiStatusBar } from './statusbar';

const TEN_MINUTES_MS = 10 * 60 * 1000;

/** Shape returned by the provisioning service for a running VM. */
export interface VmEndpoint {
  /** HTTPS endpoint of the mahalaxmi-service on the VM, e.g. `https://proj-x.mahalaxmi.ai:17421`. */
  endpoint: string;
  /** Short-lived Bearer token for this session. */
  sessionToken: string;
  /** UTC date at which `sessionToken` expires. */
  expiresAt: Date;
}

/**
 * Client that manages VM provisioning lifecycle operations against the
 * Mahalaxmi cloud control plane.
 */
export interface ProvisioningClient {
  /**
   * Start the cloud VM. Calls `onProgress` with human-readable status messages
   * while the VM boots. Resolves with the VM endpoint on success.
   */
  startVm(onProgress: (msg: string) => void): Promise<VmEndpoint>;
  /** Stop the running VM and release its resources. */
  stopVm(): Promise<void>;
  /** Retrieve the current VM endpoint along with a fresh session token. */
  getEndpoint(): Promise<VmEndpoint>;
}

/** Thrown when the provisioning service returns an error. */
export class ProvisioningError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ProvisioningError';
  }
}

/** Internal: RemoteApiClient augmented with proactive token-rotation support. */
interface TokenRotatableClient {
  updateToken(token: string): void;
}

/** Internal: RemoteApiClient augmented with a 401 callback hook. */
interface InterceptableClient {
  set401Handler(handler: () => Promise<string | null>): void;
}

/**
 * Orchestrates the full VM cloud session lifecycle for the VS Code extension.
 *
 * Responsibilities:
 * - Start / stop the cloud VM via {@link ProvisioningClient}
 * - Show progress and error notifications in VS Code
 * - Proactively refresh session tokens 10 minutes before expiry
 * - Handle server-stopped events with a user-facing reconnect prompt
 */
export class CloudSessionManager implements vscode.Disposable {
  private remoteClient: RemoteApiClient | null = null;
  private tokenRefreshTimer: ReturnType<typeof setTimeout> | null = null;
  private isConnected = false;

  constructor(
    private readonly config: CloudConfig,
    private readonly provisioning: ProvisioningClient,
    private readonly statusBar: MahalaxmiStatusBar,
    private readonly context: vscode.ExtensionContext,
  ) {}

  /**
   * Start the VM with a progress notification, then return a
   * {@link RemoteApiClient} pointed at the VM endpoint.
   *
   * Status bar transitions: `Starting` → `Connected` on success,
   * or `Offline` on failure.
   *
   * @throws The original error from {@link ProvisioningClient.startVm} after
   *   transitioning the status bar to `Offline` and showing an error toast.
   */
  async connect(): Promise<RemoteApiClient> {
    this.statusBar.setCloudVmState({ kind: 'Starting' });

    let vmEndpoint: VmEndpoint;
    try {
      vmEndpoint = await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: 'Mahalaxmi: Starting Cloud VM',
          cancellable: false,
        },
        (progress) =>
          this.provisioning.startVm((msg) => {
            this.statusBar.setCloudVmState({ kind: 'Starting' });
            progress.report({ message: msg });
          }),
      );
    } catch (err) {
      this.statusBar.setCloudVmState({ kind: 'Offline' });
      const message = err instanceof Error ? err.message : 'Unknown provisioning error';
      void vscode.window.showErrorMessage(
        `Mahalaxmi: Failed to start cloud VM — ${message}`,
      );
      throw err;
    }

    const client = new RemoteApiClient(vmEndpoint.endpoint, vmEndpoint.sessionToken);
    this.remoteClient = client;
    this.isConnected = true;
    this.statusBar.setCloudVmState({ kind: 'Connected' });

    this.scheduleTokenRefresh(vmEndpoint.expiresAt);
    this.register401Interceptor(client);

    return client;
  }

  /** Stop the VM and update the status bar: `Stopping` → `Offline`. */
  async disconnect(): Promise<void> {
    if (!this.isConnected) {
      return;
    }
    this.statusBar.setCloudVmState({ kind: 'Stopping' });
    try {
      await this.provisioning.stopVm();
    } finally {
      this.isConnected = false;
      this.remoteClient = null;
      this.statusBar.setCloudVmState({ kind: 'Offline' });
    }
  }

  /**
   * Called by the extension on a `server_stopped` WebSocket event.
   *
   * Sets the status bar to `Offline` and prompts the user to reconnect.
   * If the user clicks **Reconnect**, `connect()` is called automatically.
   */
  onServerStopped(): void {
    this.isConnected = false;
    this.statusBar.setCloudVmState({ kind: 'Offline' });
    void vscode.window
      .showInformationMessage(
        'Mahalaxmi: Cloud server stopped. Click Reconnect to start a new session.',
        'Reconnect',
      )
      .then((choice) => {
        if (choice === 'Reconnect') {
          void this.connect();
        }
      });
  }

  /** Clear the refresh timer and disconnect if currently connected. */
  dispose(): void {
    this.clearTokenRefreshTimer();
    if (this.isConnected) {
      void this.disconnect();
    }
  }

  // ---------------------------------------------------------------------------
  // Token refresh
  // ---------------------------------------------------------------------------

  private clearTokenRefreshTimer(): void {
    if (this.tokenRefreshTimer !== null) {
      clearTimeout(this.tokenRefreshTimer);
      this.tokenRefreshTimer = null;
    }
  }

  private scheduleTokenRefresh(expiresAt: Date): void {
    this.clearTokenRefreshTimer();
    const delay = expiresAt.getTime() - Date.now() - TEN_MINUTES_MS;
    if (delay <= 0) {
      void this.refreshToken();
      return;
    }
    this.tokenRefreshTimer = setTimeout(() => void this.refreshToken(), delay);
  }

  private async refreshToken(): Promise<void> {
    if (!this.remoteClient) {
      return;
    }
    let vmEndpoint: VmEndpoint;
    try {
      vmEndpoint = await this.provisioning.getEndpoint();
    } catch {
      return;
    }
    this.applyNewToken(this.remoteClient, vmEndpoint.sessionToken);
    this.scheduleTokenRefresh(vmEndpoint.expiresAt);
  }

  // ---------------------------------------------------------------------------
  // 401 interceptor
  // ---------------------------------------------------------------------------

  private register401Interceptor(client: RemoteApiClient): void {
    const interceptable = client as unknown as Partial<InterceptableClient>;
    if (typeof interceptable.set401Handler === 'function') {
      interceptable.set401Handler(async () => {
        try {
          const ep = await this.provisioning.getEndpoint();
          this.applyNewToken(client, ep.sessionToken);
          return ep.sessionToken;
        } catch {
          return null;
        }
      });
    }
  }

  private applyNewToken(client: RemoteApiClient, token: string): void {
    const rotatable = client as unknown as Partial<TokenRotatableClient>;
    if (typeof rotatable.updateToken === 'function') {
      rotatable.updateToken(token);
    }
  }
}

// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';
import type { IMahalaxmiClient } from './client-interface';
import { MahalaxmiClient } from './client';
import { RemoteApiClient } from './remote-client';
import { CyclePanel } from './panels/CyclePanel';
import { registerCommands, setCloudSessionManager } from './commands';
import type { CommandManager, ICloudSessionManager } from './commands';
import { MahalaxmiStatusBar } from './statusbar';
import { MahalaxmiFileDecorationProvider } from './fileDecorations';
import { WorkerTerminalManager } from './workerTerminals';
import { PlanApprovalManager } from './planApproval';
import { readCloudConfig, writeCloudConfig } from './cloud-config';

// ---------------------------------------------------------------------------
// Cloud infrastructure
// ---------------------------------------------------------------------------

type CloudVmState = 'disconnected' | 'connecting' | 'connected' | 'expired' | 'offline';

/**
 * Thin client for the ThriveTech provisioning API.  Handles VM lifecycle
 * (start / stop) and session token refresh.
 */
class CloudProvisioningClient {
  private disposed = false;
  private currentApiKey: string;

  constructor(
    private readonly endpoint: string,
    apiKey: string,
  ) {
    this.currentApiKey = apiKey;
  }

  getEndpoint(): string {
    return this.endpoint;
  }

  getApiKey(): string {
    return this.currentApiKey;
  }

  async startServer(): Promise<{ serviceEndpoint: string; apiKey: string; expiresAt: Date } | null> {
    if (this.disposed) return null;
    try {
      const resp = await fetch(`${this.endpoint}/v1/vm/start`, {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${this.currentApiKey}`,
          'Content-Type': 'application/json',
        },
      });
      if (!resp.ok) return null;
      const data = (await resp.json()) as {
        endpoint?: string;
        api_key?: string;
        expires_at?: string;
      };
      return {
        serviceEndpoint:
          typeof data.endpoint === 'string' ? data.endpoint : this.endpoint,
        apiKey:
          typeof data.api_key === 'string' ? data.api_key : this.currentApiKey,
        expiresAt:
          typeof data.expires_at === 'string'
            ? new Date(data.expires_at)
            : new Date(Date.now() + 3_600_000),
      };
    } catch {
      return null;
    }
  }

  async stopServer(): Promise<boolean> {
    if (this.disposed) return false;
    try {
      const resp = await fetch(`${this.endpoint}/v1/vm/stop`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${this.currentApiKey}` },
      });
      return resp.ok;
    } catch {
      return false;
    }
  }

  async refreshToken(): Promise<{ apiKey: string; expiresAt: Date } | null> {
    if (this.disposed) return null;
    try {
      const resp = await fetch(`${this.endpoint}/v1/auth/refresh`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${this.currentApiKey}` },
      });
      if (!resp.ok) return null;
      const data = (await resp.json()) as { api_key?: string; expires_at?: string };
      const apiKey =
        typeof data.api_key === 'string' ? data.api_key : this.currentApiKey;
      this.currentApiKey = apiKey;
      return {
        apiKey,
        expiresAt:
          typeof data.expires_at === 'string'
            ? new Date(data.expires_at)
            : new Date(Date.now() + 3_600_000),
      };
    } catch {
      return null;
    }
  }

  dispose(): void {
    this.disposed = true;
  }
}

/**
 * Manages a recurring heartbeat against the cloud endpoint and schedules
 * proactive token refresh 10 minutes before the session token expires.
 * A 401 on the heartbeat triggers one refresh attempt; a second consecutive
 * 401 marks the session as expired and shows an error toast.
 */
class VsCodeHeartbeatService {
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;
  private refreshInProgress = false;
  private disposed = false;
  private onExpired: (() => void) | null = null;

  constructor(private readonly provClient: CloudProvisioningClient) {}

  start(expiresAt: Date, onExpired: () => void): void {
    if (this.disposed) return;
    this.stop();
    this.onExpired = onExpired;
    this.scheduleProactiveRefresh(expiresAt);
    this.heartbeatTimer = setInterval(() => void this.sendHeartbeat(), 30_000);
  }

  stop(): void {
    if (this.heartbeatTimer !== null) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
    if (this.refreshTimer !== null) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
    this.onExpired = null;
  }

  dispose(): void {
    this.disposed = true;
    this.stop();
  }

  private scheduleProactiveRefresh(expiresAt: Date): void {
    if (this.refreshTimer !== null) {
      clearTimeout(this.refreshTimer);
    }
    const msUntilRefresh = expiresAt.getTime() - Date.now() - 10 * 60 * 1000;
    this.refreshTimer = setTimeout(
      () => void this.doRefresh(),
      Math.max(0, msUntilRefresh),
    );
  }

  private async doRefresh(): Promise<void> {
    if (this.disposed) return;
    const result = await this.provClient.refreshToken();
    if (result !== null) {
      this.scheduleProactiveRefresh(result.expiresAt);
    } else {
      void vscode.window.showErrorMessage(
        vscode.l10n.t('Mahalaxmi: Session expired \u2014 please reconnect.'),
      );
      this.onExpired?.();
      this.stop();
    }
  }

  private async sendHeartbeat(): Promise<void> {
    if (this.disposed) return;
    try {
      const resp = await fetch(`${this.provClient.getEndpoint()}/v1/health`, {
        headers: { Authorization: `Bearer ${this.provClient.getApiKey()}` },
      });
      if (resp.status === 401) {
        await this.handleUnauthorized();
      }
    } catch {
      // Network error — wait for the next heartbeat cycle
    }
  }

  private async handleUnauthorized(): Promise<void> {
    if (this.refreshInProgress || this.disposed) return;
    this.refreshInProgress = true;
    try {
      const refreshResult = await this.provClient.refreshToken();
      if (refreshResult === null) {
        void vscode.window.showErrorMessage(
          vscode.l10n.t('Mahalaxmi: Session expired \u2014 please reconnect.'),
        );
        this.onExpired?.();
        this.stop();
        return;
      }
      try {
        const retryResp = await fetch(`${this.provClient.getEndpoint()}/v1/health`, {
          headers: { Authorization: `Bearer ${refreshResult.apiKey}` },
        });
        if (retryResp.status === 401) {
          void vscode.window.showErrorMessage(
            vscode.l10n.t('Mahalaxmi: Session expired \u2014 please reconnect.'),
          );
          this.onExpired?.();
          this.stop();
        } else {
          this.scheduleProactiveRefresh(refreshResult.expiresAt);
        }
      } catch {
        // Network error after refresh — do not expire the session prematurely
      }
    } finally {
      this.refreshInProgress = false;
    }
  }
}

/**
 * Manages the cloud VM session lifecycle.  Satisfies the {@link ICloudSessionManager}
 * interface exported from commands.ts so commands can call connect/disconnect
 * without a circular import.
 */
class CloudSessionManager implements ICloudSessionManager {
  private state: CloudVmState = 'disconnected';

  constructor(
    private readonly provClient: CloudProvisioningClient,
    private readonly heartbeatSvc: VsCodeHeartbeatService,
  ) {}

  async connect(): Promise<IMahalaxmiClient | null> {
    this.state = 'connecting';
    const result = await this.provClient.startServer();
    if (result === null) {
      this.state = 'offline';
      return null;
    }
    this.state = 'connected';
    this.heartbeatSvc.start(result.expiresAt, () => {
      this.state = 'expired';
    });
    return new RemoteApiClient(result.serviceEndpoint, result.apiKey);
  }

  async disconnect(): Promise<void> {
    this.heartbeatSvc.stop();
    await this.provClient.stopServer();
    this.state = 'disconnected';
  }

  getState(): string {
    return this.state;
  }
}

// ---------------------------------------------------------------------------
// Module-level component references held for hot-swap
// ---------------------------------------------------------------------------

let panel: CyclePanel | undefined;
let statusBar: MahalaxmiStatusBar | undefined;
let fileDecorations: MahalaxmiFileDecorationProvider | undefined;
let workerTerminals: WorkerTerminalManager | undefined;
let planApproval: PlanApprovalManager | undefined;
let commandManager: CommandManager | undefined;
let activeClient: IMahalaxmiClient | undefined;

// Cloud resources — created lazily when connectionMode === 'cloud'
let cloudProvisioningClient: CloudProvisioningClient | undefined;
let heartbeatService: VsCodeHeartbeatService | undefined;

// ---------------------------------------------------------------------------
// Client factory
// ---------------------------------------------------------------------------

async function createClient(context: vscode.ExtensionContext): Promise<IMahalaxmiClient> {
  const cfg = vscode.workspace.getConfiguration('mahalaxmi');
  const mode = cfg.get<string>('connectionMode', 'local');

  if (mode === 'cloud') {
    const config = await readCloudConfig();
    if (config === null) {
      void vscode.window.showErrorMessage(
        vscode.l10n.t(
          'Mahalaxmi: Cloud not configured \u2014 run setup at https://mahalaxmi.ai/setup',
        ),
      );
      setCloudSessionManager(undefined);
      return new MahalaxmiClient('ws://127.0.0.1:17420');
    }

    // Dispose stale cloud resources before recreating them
    heartbeatService?.dispose();
    cloudProvisioningClient?.dispose();

    cloudProvisioningClient = new CloudProvisioningClient(config.endpoint, config.api_key);
    heartbeatService = new VsCodeHeartbeatService(cloudProvisioningClient);

    const sessionMgr = new CloudSessionManager(cloudProvisioningClient, heartbeatService);
    setCloudSessionManager(sessionMgr);

    return new RemoteApiClient(config.endpoint, config.api_key);
  }

  // Clear cloud session manager when switching away from cloud mode
  setCloudSessionManager(undefined);

  if (mode === 'remote') {
    const raw = cfg.get<string>('remoteUrl', 'https://localhost:17421');
    // Always enforce HTTPS — downgrade to plain HTTP is never permitted.
    const secureUrl = raw.startsWith('http://') ? raw.replace('http://', 'https://') : raw;
    if (raw !== secureUrl) {
      void vscode.window.showWarningMessage(
        `Mahalaxmi: Remote URL was configured as HTTP. Upgraded to HTTPS: ${secureUrl}`,
      );
    }
    // API token stored in globalState — not in settings — to avoid git exposure.
    const token = context.globalState.get<string>('mahalaxmi.apiToken', '');
    return new RemoteApiClient(secureUrl, token ?? '');
  }

  return new MahalaxmiClient('ws://127.0.0.1:17420');
}

// ---------------------------------------------------------------------------
// Hot-swap
// ---------------------------------------------------------------------------

async function hotSwap(context: vscode.ExtensionContext): Promise<void> {
  // Dispose the old client first — removes all its listeners and closes the
  // WS / SSE connection, so no stale events fire during the transition.
  activeClient?.dispose();

  const newClient = await createClient(context);
  activeClient = newClient;

  panel?.setClient(newClient);
  statusBar?.setClient(newClient);
  fileDecorations?.setClient(newClient);
  workerTerminals?.setClient(newClient);
  planApproval?.setClient(newClient);
  commandManager?.setClient(newClient);

  newClient.connect();
}

// ---------------------------------------------------------------------------
// Activate / deactivate
// ---------------------------------------------------------------------------

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const initialClient = await createClient(context);
  activeClient = initialClient;

  panel = new CyclePanel(initialClient);
  statusBar = new MahalaxmiStatusBar(initialClient, context);
  fileDecorations = new MahalaxmiFileDecorationProvider(initialClient);
  workerTerminals = new WorkerTerminalManager(initialClient);
  planApproval = new PlanApprovalManager(initialClient);
  commandManager = registerCommands(initialClient, context);

  const configWatcher = vscode.workspace.onDidChangeConfiguration((e) => {
    if (
      e.affectsConfiguration('mahalaxmi.connectionMode') ||
      e.affectsConfiguration('mahalaxmi.remoteUrl')
    ) {
      void hotSwap(context);
    }
  });

  // Internal command allowing commands.ts to trigger a hot-swap after credential rotation.
  const reconnectCmd = vscode.commands.registerCommand('mahalaxmi.reconnect', () => {
    void hotSwap(context);
  });

  // URI handler:
  //   vscode://thrivetech.mahalaxmi/connect?url=https://proj-x.mahalaxmi.ai&token=mah_tok_...
  //     Triggered by the website "Open in VS Code" button after provisioning completes.
  //   vscode://thrivetech.mahalaxmi/configure?endpoint=...&api_key=...&project_id=...
  //     Writes cloud.toml and switches to cloud mode.
  const uriHandler = vscode.window.registerUriHandler({
    handleUri(uri: vscode.Uri): void {
      if (uri.path === '/connect') {
        const params = new URLSearchParams(uri.query);
        const url = params.get('url');
        const token = params.get('token');

        if (!url || !token) {
          void vscode.window.showErrorMessage(
            vscode.l10n.t(
              'Mahalaxmi: Invalid connection link \u2014 missing url or token parameter.',
            ),
          );
          return;
        }

        // Enforce HTTPS — matching the same rule applied in createClient().
        const secureUrl = url.startsWith('http://')
          ? url.replace('http://', 'https://')
          : url;

        // Sequence matters: token must be stored before hotSwap reads it from globalState.
        void context.globalState.update('mahalaxmi.apiToken', token).then(async () => {
          const cfg = vscode.workspace.getConfiguration('mahalaxmi');
          await cfg.update('remoteUrl', secureUrl, vscode.ConfigurationTarget.Global);
          // Setting connectionMode last — this fires onDidChangeConfiguration → hotSwap().
          await cfg.update('connectionMode', 'remote', vscode.ConfigurationTarget.Global);
          void vscode.window.showInformationMessage(
            vscode.l10n.t('Mahalaxmi: Connected to {0}', secureUrl),
          );
        });
        return;
      }

      if (uri.path === '/configure') {
        const params = new URLSearchParams(uri.query);
        const endpoint = params.get('endpoint');
        const apiKey = params.get('api_key') ?? params.get('apiKey');
        const projectId = params.get('project_id') ?? params.get('projectId');

        if (!endpoint || !apiKey || !projectId) {
          void vscode.window.showErrorMessage(
            vscode.l10n.t(
              'Mahalaxmi: Invalid cloud config link \u2014 missing endpoint, api_key, or project_id.',
            ),
          );
          return;
        }

        if (!endpoint.startsWith('https://')) {
          void vscode.window.showErrorMessage(
            vscode.l10n.t('Mahalaxmi: Cloud endpoint must start with https://.'),
          );
          return;
        }

        void writeCloudConfig({ endpoint, api_key: apiKey, project_id: projectId })
          .then(async () => {
            const cfg = vscode.workspace.getConfiguration('mahalaxmi');
            // Setting connectionMode fires onDidChangeConfiguration → hotSwap().
            await cfg.update('connectionMode', 'cloud', vscode.ConfigurationTarget.Global);
            void vscode.window.showInformationMessage(
              vscode.l10n.t('Mahalaxmi: Cloud connection configured. Connecting\u2026'),
            );
          })
          .catch(() => {
            void vscode.window.showErrorMessage(
              vscode.l10n.t('Mahalaxmi: Failed to write cloud configuration.'),
            );
          });
        return;
      }
    },
  });

  // Worker stall notifications — shown once per stall episode.
  // Gives operators a Kill / Inspect action without opening the full sidebar.
  initialClient.on('orchestration_event', (raw: unknown) => {
    const event = raw as { type?: string; worker_id?: string; task_name?: string; idle_seconds?: number; auto_killed?: boolean };
    if (event?.type !== 'worker_stalled') return;
    const workerId = event.worker_id ?? 'unknown';
    const taskName = event.task_name ?? workerId;
    const mins = Math.round((event.idle_seconds ?? 0) / 60);
    const suffix = event.auto_killed ? ' (auto-killed)' : '';
    void vscode.window.showWarningMessage(
      `Mahalaxmi: Worker ${workerId} (${taskName}) has produced no output for ${mins}m${suffix}.`,
      'Kill Worker',
      'Inspect Terminal',
    ).then((action) => {
      if (action === 'Kill Worker') {
        void activeClient?.killWorker(workerId);
      }
      // 'Inspect Terminal' — open the sidebar; user can click the worker node there.
      if (action === 'Inspect Terminal') {
        void vscode.commands.executeCommand('mahalaxmi.sidebar.focus');
      }
    });
  });

  context.subscriptions.push(
    vscode.window.registerTreeDataProvider('mahalaxmi.sidebar', panel),
    vscode.window.registerFileDecorationProvider(fileDecorations),
    ...commandManager.disposables,
    statusBar,
    fileDecorations,
    workerTerminals,
    planApproval,
    configWatcher,
    reconnectCmd,
    uriHandler,
    { dispose: () => activeClient?.dispose() },
    // Cloud resource cleanup — heartbeatService and cloudProvisioningClient are
    // created lazily by createClient() when cloud mode is active.
    {
      dispose: (): void => {
        heartbeatService?.dispose();
        cloudProvisioningClient?.dispose();
      },
    },
  );

  initialClient.connect();
}

export function deactivate(): void {
  heartbeatService?.dispose();
  cloudProvisioningClient?.dispose();
  activeClient?.dispose();
}

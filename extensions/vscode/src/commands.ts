// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as vscode from 'vscode';
import type { IMahalaxmiClient, TemplateDto } from './client-interface';
import { readCloudConfig } from './cloud-config';

let cachedTemplates: TemplateDto[] = [];

/**
 * Minimal interface for the cloud session manager, injected from extension.ts
 * to avoid circular imports.
 */
export interface ICloudSessionManager {
  connect(): Promise<IMahalaxmiClient | null>;
  disconnect(): Promise<void>;
  getState(): string;
}

let cloudSessionManager: ICloudSessionManager | undefined;

/**
 * Inject or clear the active cloud session manager.  Called by extension.ts
 * whenever the client is created in cloud mode, and cleared when switching away.
 */
export function setCloudSessionManager(mgr: ICloudSessionManager | undefined): void {
  cloudSessionManager = mgr;
}

/**
 * Returned by {@link registerCommands}.  Holds the VS Code command disposables
 * and a `setClient()` method for hot-swapping the transport (local ↔ remote).
 */
export interface CommandManager {
  disposables: vscode.Disposable[];
  setClient(client: IMahalaxmiClient): void;
}

/**
 * Register all Mahalaxmi command palette commands.
 *
 * Returns a {@link CommandManager} whose `disposables` should be pushed to the
 * extension context and whose `setClient()` is called by the hot-swap handler
 * in `extension.ts` when the connection mode changes.
 */
export function registerCommands(
  initialClient: IMahalaxmiClient,
  context: vscode.ExtensionContext,
): CommandManager {
  // `client` is a mutable closure variable — all command closures read it at
  // invocation time, so setClient() updates take effect immediately.
  let client = initialClient;

  function onTemplates(templates: TemplateDto[]): void {
    cachedTemplates = templates;
  }
  client.on('templates', onTemplates);

  const startCycle = vscode.commands.registerCommand('mahalaxmi.startCycle', async () => {
    await runStartCycle(client);
  });

  const startCycleOnFolder = vscode.commands.registerCommand(
    'mahalaxmi.startCycleOnFolder',
    async (uri?: vscode.Uri) => {
      const folderPath = uri?.fsPath ?? vscode.workspace.rootPath;
      await runStartCycle(client, folderPath);
    },
  );

  const stopCycle = vscode.commands.registerCommand('mahalaxmi.stopCycle', async () => {
    const response = await vscode.window.showWarningMessage(
      vscode.l10n.t('Stop the current Mahalaxmi cycle?'),
      { modal: true },
      'Stop',
    );
    if (response === 'Stop') {
      client.stopCycle('__current__');
    }
  });

  const approvePlan = vscode.commands.registerCommand('mahalaxmi.approvePlan', async () => {
    const response = await vscode.window.showInformationMessage(
      vscode.l10n.t('Approve the Mahalaxmi execution plan?'),
      { modal: true },
      vscode.l10n.t('Approve'),
      vscode.l10n.t('Reject'),
    );
    if (response === vscode.l10n.t('Approve')) {
      client.approvePlan('__current__');
      vscode.window.showInformationMessage(vscode.l10n.t('Mahalaxmi: Plan approved — workers dispatching.'));
    }
  });

  const rejectPlan = vscode.commands.registerCommand('mahalaxmi.rejectPlan', async () => {
    vscode.window.showWarningMessage(vscode.l10n.t('Mahalaxmi: Plan rejected.'));
  });

  const stopChain = vscode.commands.registerCommand('mahalaxmi.stopChain', async () => {
    const response = await vscode.window.showWarningMessage(
      vscode.l10n.t('Stop Mahalaxmi chain mode after the current cycle?'),
      { modal: true },
      'Stop Chain',
    );
    if (response === 'Stop Chain') {
      client.stopChain();
    }
  });

  const acceptFile = vscode.commands.registerCommand(
    'mahalaxmi.acceptFile',
    async (uri?: vscode.Uri) => {
      const filePath = uri?.fsPath ?? vscode.window.activeTextEditor?.document.uri.fsPath;
      if (filePath) {
        client.setFileAcceptance(filePath, true);
        vscode.window.showInformationMessage(vscode.l10n.t('Mahalaxmi: Accepted {0}', filePath));
      }
    },
  );

  const rejectFile = vscode.commands.registerCommand(
    'mahalaxmi.rejectFile',
    async (uri?: vscode.Uri) => {
      const filePath = uri?.fsPath ?? vscode.window.activeTextEditor?.document.uri.fsPath;
      if (filePath) {
        client.setFileAcceptance(filePath, false);
        vscode.window.showWarningMessage(vscode.l10n.t('Mahalaxmi: Rejected {0}', filePath));
      }
    },
  );

  // -------------------------------------------------------------------------
  // Remote mode commands
  // -------------------------------------------------------------------------

  const configureRemote = vscode.commands.registerCommand(
    'mahalaxmi.configureRemote',
    async () => {
      const url = await vscode.window.showInputBox({
        prompt: vscode.l10n.t('Remote mahalaxmi-service URL'),
        placeHolder: vscode.l10n.t('https://my-server.com'),
        value: vscode.workspace
          .getConfiguration('mahalaxmi')
          .get<string>('remoteUrl', 'http://localhost:17421'),
        validateInput(v) {
          return v.startsWith('http://') || v.startsWith('https://')
            ? undefined
            : vscode.l10n.t('URL must start with https://, http://, or git@');
        },
      });
      if (url === undefined) return; // cancelled

      const token = await vscode.window.showInputBox({
        prompt: vscode.l10n.t('API token (leave empty for no authentication)'),
        password: true,
        placeHolder: vscode.l10n.t('Bearer token or leave blank'),
      });
      if (token === undefined) return; // cancelled

      await context.globalState.update('mahalaxmi.apiToken', token);
      const cfg = vscode.workspace.getConfiguration('mahalaxmi');
      await cfg.update('remoteUrl', url, vscode.ConfigurationTarget.Global);
      await cfg.update('connectionMode', 'remote', vscode.ConfigurationTarget.Global);
      // onDidChangeConfiguration in extension.ts fires automatically and
      // performs the client hot-swap.
      vscode.window.showInformationMessage(vscode.l10n.t('Mahalaxmi: Remote mode configured.'));
    },
  );

  const switchToLocal = vscode.commands.registerCommand('mahalaxmi.switchToLocal', async () => {
    await vscode.workspace
      .getConfiguration('mahalaxmi')
      .update('connectionMode', 'local', vscode.ConfigurationTarget.Global);
    // onDidChangeConfiguration fires → hot-swap to local WebSocket client.
    vscode.window.showInformationMessage(vscode.l10n.t('Mahalaxmi: Switched to local mode.'));
  });

  // -------------------------------------------------------------------------
  // Credential management (remote mode only)
  // -------------------------------------------------------------------------

  const CREDENTIAL_OPTIONS = [
    { label: 'Anthropic API Key (Claude Code)', key: 'ANTHROPIC_API_KEY' },
    { label: 'OpenAI API Key',                  key: 'OPENAI_API_KEY' },
    { label: 'GitHub Token (for workers)',       key: 'GITHUB_TOKEN' },
    { label: 'Rotate Service Auth Token',        key: 'MAHALAXMI_API_TOKEN' },
    { label: 'Encryption Key (for config)',      key: 'MAHALAXMI_ENCRYPTION_KEY' },
  ] as const;

  const configureCredentials = vscode.commands.registerCommand(
    'mahalaxmi.configureServerCredentials',
    async () => {
      const cfg = vscode.workspace.getConfiguration('mahalaxmi');
      if (cfg.get<string>('connectionMode') !== 'remote') {
        vscode.window.showWarningMessage(
          vscode.l10n.t('Mahalaxmi: Connect to a remote server first using "Mahalaxmi: Configure Remote Server".'),
        );
        return;
      }
      const baseUrl = cfg.get<string>('remoteUrl', 'http://localhost:17421');
      const currentToken = context.globalState.get<string>('mahalaxmi.apiToken', '') ?? '';

      const picked = await vscode.window.showQuickPick(
        CREDENTIAL_OPTIONS.map((o) => ({ label: o.label, description: o.key })),
        {
          placeHolder: vscode.l10n.t('Which credential do you want to update?'),
          title: vscode.l10n.t('Mahalaxmi: Configure Server Credentials'),
        },
      );
      if (!picked) return;

      const credKey = CREDENTIAL_OPTIONS.find((o) => o.label === picked.label)?.key;
      if (!credKey) return;

      const value = await vscode.window.showInputBox({
        prompt: vscode.l10n.t('New value for: {0}', picked.label),
        password: true,
      });
      if (value === undefined || value.trim() === '') return;

      let response: Response;
      try {
        response = await fetch(`${baseUrl}/v1/admin/credentials`, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${currentToken}`,
          },
          body: JSON.stringify({ credentials: { [credKey]: value.trim() } }),
        });
      } catch {
        vscode.window.showErrorMessage(vscode.l10n.t('Mahalaxmi: Could not reach the server. Is it running?'));
        return;
      }

      if (!response.ok) {
        const text = await response.text().catch(() => '');
        vscode.window.showErrorMessage(vscode.l10n.t('Mahalaxmi: Server error {0}: {1}', String(response.status), text));
        return;
      }

      // Token rotation: update local storage before reconnecting so the new
      // client uses the rotated token when the service comes back up.
      if (credKey === 'MAHALAXMI_API_TOKEN') {
        await context.globalState.update('mahalaxmi.apiToken', value.trim());
      }

      // Poll health until service restarts, then reconnect (up to 60 s).
      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: vscode.l10n.t('Credential saved. Waiting for service restart\u2026'),
          cancellable: false,
        },
        async (progress) => {
          for (let i = 0; i < 20; i++) {
            await new Promise<void>((r) => setTimeout(r, 3000));
            try {
              const h = await fetch(`${baseUrl}/v1/health`);
              if (h.ok) {
                await vscode.commands.executeCommand('mahalaxmi.reconnect');
                vscode.window.showInformationMessage(
                  vscode.l10n.t('{0} updated. Service reconnected.', picked.label),
                );
                return;
              }
            } catch {
              // still restarting — keep polling
            }
            progress.report({ increment: 5, message: vscode.l10n.t('{0}s elapsed\u2026', String((i + 1) * 3)) });
          }
          vscode.window.showWarningMessage(
            vscode.l10n.t('Mahalaxmi: Service did not come back in 60s. Check server logs.'),
          );
        },
      );
    },
  );

  // -------------------------------------------------------------------------
  // Cloud mode commands
  // -------------------------------------------------------------------------

  const startCloudVm = vscode.commands.registerCommand('mahalaxmi.startCloudVm', async () => {
    const config = await readCloudConfig();
    if (config === null) {
      void vscode.window.showErrorMessage(
        vscode.l10n.t(
          'Cloud not configured \u2014 run setup wizard at https://mahalaxmi.ai/setup',
        ),
      );
      return;
    }

    await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: vscode.l10n.t('Mahalaxmi: Starting cloud VM\u2026'),
        cancellable: false,
      },
      async () => {
        const result = await cloudSessionManager?.connect() ?? null;
        if (result === null) {
          void vscode.window.showErrorMessage(
            vscode.l10n.t('Mahalaxmi: Failed to start cloud VM.'),
          );
        } else {
          void vscode.window.showInformationMessage(
            vscode.l10n.t('Mahalaxmi: Cloud VM started.'),
          );
          void vscode.commands.executeCommand('mahalaxmi.reconnect');
        }
      },
    );
  });

  const stopCloudVm = vscode.commands.registerCommand('mahalaxmi.stopCloudVm', async () => {
    const response = await vscode.window.showWarningMessage(
      vscode.l10n.t('Stop cloud VM? Running cycles will be interrupted.'),
      { modal: true },
      'Stop',
      'Cancel',
    );
    if (response !== 'Stop') return;

    if (!cloudSessionManager) {
      void vscode.window.showErrorMessage(
        vscode.l10n.t('Mahalaxmi: No active cloud session.'),
      );
      return;
    }

    await cloudSessionManager.disconnect();
    void vscode.window.showInformationMessage(vscode.l10n.t('Mahalaxmi: Cloud VM stopped.'));
  });

  const cloudStatus = vscode.commands.registerCommand('mahalaxmi.cloudStatus', () => {
    const state = cloudSessionManager?.getState() ?? 'disconnected';
    void vscode.window.showInformationMessage(
      vscode.l10n.t('Mahalaxmi: Cloud VM state: {0}', state),
    );
  });

  return {
    disposables: [
      startCycle,
      startCycleOnFolder,
      stopCycle,
      approvePlan,
      rejectPlan,
      stopChain,
      acceptFile,
      rejectFile,
      configureRemote,
      switchToLocal,
      configureCredentials,
      startCloudVm,
      stopCloudVm,
      cloudStatus,
    ],
    setClient(newClient: IMahalaxmiClient): void {
      client.off('templates', onTemplates);
      cachedTemplates = []; // clear stale cache from previous transport
      client = newClient;
      client.on('templates', onTemplates);
    },
  };
}

async function runStartCycle(client: IMahalaxmiClient, prefillRoot?: string): Promise<void> {
  // Fetch templates if not yet cached
  if (cachedTemplates.length === 0) {
    client.queryTemplates();
    // Give a brief moment for templates to arrive; fall through if none
    await new Promise((r) => setTimeout(r, 300));
  }

  let templateId: string | undefined;
  if (cachedTemplates.length > 0) {
    const items = cachedTemplates.map((t) => ({
      label: t.name,
      description: t.category,
      detail: t.id,
    }));
    const chosen = await vscode.window.showQuickPick(items, {
      placeHolder: vscode.l10n.t('Select a requirements template (Escape to skip)'),
      title: vscode.l10n.t('Mahalaxmi: Choose Template'),
    });
    templateId = chosen?.detail;
  }

  const requirements = await vscode.window.showInputBox({
    prompt: vscode.l10n.t('Describe what the AI agents should build or fix'),
    placeHolder: vscode.l10n.t('Add user authentication, fix the login bug, implement dark mode\u2026'),
  });
  if (!requirements?.trim()) return;

  const chainAnswer = await vscode.window.showQuickPick(
    [vscode.l10n.t('No'), vscode.l10n.t('Yes')],
    {
      placeHolder: vscode.l10n.t('Enable Chain Mode? (auto-run subsequent phases)'),
      title: vscode.l10n.t('Mahalaxmi: Chain Mode'),
    },
  );
  const chainMode = chainAnswer === vscode.l10n.t('Yes');
  let chainRequirementsPath: string | undefined;
  if (chainMode) {
    const picked = await vscode.window.showOpenDialog({
      canSelectMany: false,
      openLabel: vscode.l10n.t('Select requirements file'),
      filters: { Markdown: ['md'] },
    });
    chainRequirementsPath = picked?.[0]?.fsPath;
  }

  if (client.isRemote) {
    // Remote mode: collect git repo URL + branch instead of a filesystem path.
    // `prefillRoot` is silently ignored in remote mode.
    const repoUrl = await vscode.window.showInputBox({
      prompt: vscode.l10n.t('Repository URL (cloned on the server)'),
      placeHolder: vscode.l10n.t('https://github.com/org/repo.git'),
      validateInput(v) {
        return v.startsWith('https://') || v.startsWith('http://') || v.startsWith('git@')
          ? undefined
          : vscode.l10n.t('URL must start with https://, http://, or git@');
      },
    });
    if (repoUrl === undefined) return; // cancelled

    const branch = await vscode.window.showInputBox({
      prompt: vscode.l10n.t('Branch to use'),
      value: 'main',
    });
    if (branch === undefined) return; // cancelled

    const status = await client.checkRepoStatus(repoUrl);

    let repoAction: string | undefined;
    if (status.exists) {
      const statusHint =
        status.status === 'has_worktrees'
          ? vscode.l10n.t('has active worktrees')
          : status.status;
      const picked = await vscode.window.showQuickPick(
        [
          {
            label: vscode.l10n.t('Continue'),
            description: vscode.l10n.t('Use existing repo as-is'),
            value: 'continue',
          },
          {
            label: vscode.l10n.t('Pull & Start'),
            description: vscode.l10n.t('Fetch and fast-forward, then start'),
            value: 'pull_start',
          },
          {
            label: vscode.l10n.t('Fresh Start'),
            description: vscode.l10n.t('Reset hard to origin and clean untracked files'),
            value: 'fresh_start',
          },
        ],
        {
          placeHolder: vscode.l10n.t('Repo exists ({0}). How should the server handle it?', statusHint),
          title: vscode.l10n.t('Mahalaxmi: Existing Repository'),
        },
      );
      if (!picked) return; // cancelled
      repoAction = picked.value;
    }

    client.startCycle(
      requirements.trim(),
      undefined,
      chainMode,
      chainRequirementsPath,
      repoUrl,
      branch,
      repoAction,
    );
  } else {
    // Local mode (unchanged): collect a filesystem project root path.
    const projectRoot = await vscode.window.showInputBox({
      prompt: vscode.l10n.t('Project root directory'),
      placeHolder: prefillRoot ?? vscode.workspace.rootPath ?? '/path/to/project',
      value: prefillRoot ?? vscode.workspace.rootPath ?? '',
    });
    if (projectRoot === undefined) return; // cancelled

    client.startCycle(
      requirements.trim(),
      projectRoot.trim() || undefined,
      chainMode,
      chainRequirementsPath,
    );
  }

  if (templateId) {
    vscode.window.showInformationMessage(vscode.l10n.t('Mahalaxmi: Cycle started ({0})', templateId));
  } else {
    vscode.window.showInformationMessage(vscode.l10n.t('Mahalaxmi: Cycle started'));
  }
}

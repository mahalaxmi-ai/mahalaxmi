// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { EventEmitter } from 'events';
import { registerCommands, setCloudSessionManager } from './commands';

// ---------------------------------------------------------------------------
// Hoist vi.fn() mocks so mock factories can reference them safely
// ---------------------------------------------------------------------------

const mocks = vi.hoisted(() => ({
  registerCommand: vi.fn(),
  showInputBox: vi.fn<() => Promise<string | undefined>>(),
  showWarningMessage: vi.fn<() => Promise<string | undefined>>(),
  showErrorMessage: vi.fn<() => Promise<string | undefined>>(),
  showInformationMessage: vi.fn<() => Promise<string | undefined>>(),
  showQuickPick: vi.fn<() => Promise<unknown>>(),
  showOpenDialog: vi.fn<() => Promise<unknown[]| undefined>>(),
  withProgress: vi.fn(),
  getConfiguration: vi.fn(),
  readCloudConfig: vi.fn(),
}));

// ---------------------------------------------------------------------------
// Mock vscode — only references hoisted mocks (no module-level imports)
// ---------------------------------------------------------------------------

vi.mock('vscode', () => ({
  commands: {
    registerCommand: mocks.registerCommand,
    executeCommand: vi.fn().mockResolvedValue(undefined),
  },
  window: {
    showInputBox: mocks.showInputBox,
    showWarningMessage: mocks.showWarningMessage,
    showErrorMessage: mocks.showErrorMessage,
    showInformationMessage: mocks.showInformationMessage,
    showQuickPick: mocks.showQuickPick,
    showOpenDialog: mocks.showOpenDialog,
    withProgress: mocks.withProgress,
  },
  workspace: {
    rootPath: '/workspace',
    getConfiguration: mocks.getConfiguration,
  },
  ConfigurationTarget: { Global: 1 },
  ProgressLocation: { Notification: 15 },
  l10n: {
    t: (str: string, ...args: unknown[]): string =>
      args.reduce<string>((s, arg, i) => s.replace(`{${i}}`, String(arg)), str),
  },
}));

vi.mock('./cloud-config', () => ({
  readCloudConfig: () => mocks.readCloudConfig(),
  writeCloudConfig: vi.fn().mockResolvedValue(undefined),
  isCloudConfigured: vi.fn().mockReturnValue(false),
}));

// ---------------------------------------------------------------------------
// Mock ws — no imported base class inside the factory
// ---------------------------------------------------------------------------

vi.mock('ws', () => {
  class MockWS {
    static OPEN = 1;
    readyState = 1;
    private readonly _listeners: Map<string, Array<(...args: unknown[]) => void>> = new Map();
    on(event: string, listener: (...args: unknown[]) => void): this {
      const list = this._listeners.get(event) ?? [];
      list.push(listener);
      this._listeners.set(event, list);
      return this;
    }
    emit(event: string, ...args: unknown[]): void {
      for (const l of (this._listeners.get(event) ?? [])) l(...args);
    }
    send(): void { /* noop */ }
    close(): void { /* noop */ }
  }
  return { default: MockWS };
});

// ---------------------------------------------------------------------------
// Stub IMahalaxmiClient
// ---------------------------------------------------------------------------

class StubClient extends EventEmitter {
  readonly isRemote: boolean;
  startCycleCalls: Array<{
    requirements: string;
    projectRoot?: string;
    repoUrl?: string;
    branch?: string;
    repoAction?: string;
  }> = [];
  stopCycleCalls: string[] = [];
  approvePlanCalls: string[] = [];
  stopChainCalls = 0;
  setFileAcceptanceCalls: Array<{ filePath: string; accepted: boolean }> = [];
  checkRepoStatus = vi.fn<(url: string) => Promise<{ exists: boolean; path: string; status: string }>>()
    .mockResolvedValue({ exists: false, path: '', status: 'not_cloned' });

  constructor(isRemote = false) {
    super();
    this.isRemote = isRemote;
  }

  startCycle(
    requirements: string,
    projectRoot?: string,
    _chainMode?: boolean,
    _chainRequirementsPath?: string,
    repoUrl?: string,
    branch?: string,
    repoAction?: string,
  ): void {
    this.startCycleCalls.push({ requirements, projectRoot, repoUrl, branch, repoAction });
  }
  stopCycle(cycleId: string): void {
    this.stopCycleCalls.push(cycleId);
  }
  approvePlan(cycleId: string): void {
    this.approvePlanCalls.push(cycleId);
  }
  stopChain(): void {
    this.stopChainCalls++;
  }
  setFileAcceptance(filePath: string, accepted: boolean): void {
    this.setFileAcceptanceCalls.push({ filePath, accepted });
  }
  queryTemplates(): void { /* noop */ }
  connect(): void { /* noop */ }
  dispose(): void { /* noop */ }
}

// Minimal ExtensionContext stub
const mockContext = {
  globalState: {
    get: vi.fn().mockReturnValue(''),
    update: vi.fn().mockResolvedValue(undefined),
  },
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('registerCommands', () => {
  let client: StubClient;
  const registeredCommands = new Map<string, (...args: unknown[]) => Promise<void>>();

  beforeEach(() => {
    vi.clearAllMocks();
    registeredCommands.clear();
    client = new StubClient();

    // Reset cloud session manager between tests
    setCloudSessionManager(undefined);

    // Default getConfiguration stub returns a config object with no-op update
    mocks.getConfiguration.mockReturnValue({
      get: vi.fn().mockReturnValue('local'),
      update: vi.fn().mockResolvedValue(undefined),
    });

    // Default readCloudConfig returns null (not configured)
    mocks.readCloudConfig.mockResolvedValue(null);

    // Default withProgress calls the task immediately
    mocks.withProgress.mockImplementation(
      (_opts: unknown, task: (progress: unknown) => Promise<unknown>) =>
        task({ report: vi.fn() }),
    );

    mocks.registerCommand.mockImplementation(
      (id: string, handler: (...args: unknown[]) => Promise<void>): { dispose: () => void } => {
        registeredCommands.set(id, handler);
        return { dispose: vi.fn() };
      },
    );
  });

  it('returns a CommandManager with 14 disposables', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const mgr = registerCommands(client as any, mockContext as any);
    expect(mgr.disposables).toHaveLength(14);
  });

  it('exposes a setClient() method', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const mgr = registerCommands(client as any, mockContext as any);
    expect(typeof mgr.setClient).toBe('function');
  });

  it('registers all expected command IDs', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    expect([...registeredCommands.keys()]).toEqual(expect.arrayContaining([
      'mahalaxmi.startCycle',
      'mahalaxmi.startCycleOnFolder',
      'mahalaxmi.stopCycle',
      'mahalaxmi.approvePlan',
      'mahalaxmi.rejectPlan',
      'mahalaxmi.stopChain',
      'mahalaxmi.acceptFile',
      'mahalaxmi.rejectFile',
      'mahalaxmi.configureRemote',
      'mahalaxmi.switchToLocal',
      'mahalaxmi.startCloudVm',
      'mahalaxmi.stopCloudVm',
      'mahalaxmi.cloudStatus',
    ]));
  });

  it('startCycle calls startCycle() with requirements and projectRoot', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    // New flow: requirements first, then local project root
    mocks.showInputBox
      .mockResolvedValueOnce('build a REST API')  // requirements
      .mockResolvedValueOnce('/my/project');        // projectRoot
    mocks.showQuickPick.mockResolvedValue('No');  // chain mode

    const handler = registeredCommands.get('mahalaxmi.startCycle');
    expect(handler).toBeDefined();
    await handler!();

    expect(client.startCycleCalls).toHaveLength(1);
    expect(client.startCycleCalls[0].requirements).toBe('build a REST API');
    expect(client.startCycleCalls[0].projectRoot).toBe('/my/project');
  });

  it('startCycle does NOT call startCycle() when requirements is empty', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    mocks.showInputBox.mockResolvedValueOnce('');  // empty requirements (now first)

    await registeredCommands.get('mahalaxmi.startCycle')!();
    expect(client.startCycleCalls).toHaveLength(0);
  });

  it('startCycle does NOT call startCycle() when projectRoot is cancelled', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    mocks.showInputBox
      .mockResolvedValueOnce('some requirements')  // requirements
      .mockResolvedValueOnce(undefined);            // cancelled projectRoot
    mocks.showQuickPick.mockResolvedValue('No');  // chain mode

    await registeredCommands.get('mahalaxmi.startCycle')!();
    expect(client.startCycleCalls).toHaveLength(0);
  });

  it('stopCycle command calls client.stopCycle("__current__") when user confirms', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    mocks.showWarningMessage.mockResolvedValue('Stop');

    await registeredCommands.get('mahalaxmi.stopCycle')!();
    expect(client.stopCycleCalls).toEqual(['__current__']);
  });

  it('stopCycle command does NOT call client.stopCycle() when user cancels', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    mocks.showWarningMessage.mockResolvedValue(undefined);

    await registeredCommands.get('mahalaxmi.stopCycle')!();
    expect(client.stopCycleCalls).toHaveLength(0);
  });

  it('approvePlan sends approve when user clicks Approve', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    mocks.showInformationMessage.mockResolvedValue('Approve');

    await registeredCommands.get('mahalaxmi.approvePlan')!();
    expect(client.approvePlanCalls).toEqual(['__current__']);
  });

  it('approvePlan does not send approve when user clicks Reject', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    mocks.showInformationMessage.mockResolvedValue('Reject');

    await registeredCommands.get('mahalaxmi.approvePlan')!();
    expect(client.approvePlanCalls).toHaveLength(0);
  });

  it('stopChain calls client.stopChain() when user confirms', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    mocks.showWarningMessage.mockResolvedValue('Stop Chain');

    await registeredCommands.get('mahalaxmi.stopChain')!();
    expect(client.stopChainCalls).toBe(1);
  });

  it('setClient updates the client used by command closures', () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const mgr = registerCommands(client as any, mockContext as any);
    const newClient = new StubClient();
    // Should not throw
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    expect(() => mgr.setClient(newClient as any)).not.toThrow();
  });

  // -------------------------------------------------------------------------
  // Remote vs local mode branching
  // -------------------------------------------------------------------------

  it('local_mode_shows_path_input: isRemote=false shows project root prompt, not URL prompt', async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any); // client has isRemote=false
    mocks.showInputBox
      .mockResolvedValueOnce('do something')  // requirements
      .mockResolvedValueOnce('/workspace');   // project root
    mocks.showQuickPick.mockResolvedValue('No');

    await registeredCommands.get('mahalaxmi.startCycle')!();

    expect(client.startCycleCalls).toHaveLength(1);
    expect(client.startCycleCalls[0].projectRoot).toBe('/workspace');
    expect(client.startCycleCalls[0].repoUrl).toBeUndefined();
    expect(client.checkRepoStatus).not.toHaveBeenCalled();
  });

  it('remote_mode_shows_url_input: isRemote=true shows URL prompt, not path prompt', async () => {
    const remoteClient = new StubClient(true);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(remoteClient as any, mockContext as any);
    mocks.showInputBox
      .mockResolvedValueOnce('add auth')                          // requirements
      .mockResolvedValueOnce('https://github.com/org/repo.git')  // repoUrl
      .mockResolvedValueOnce('main');                            // branch
    mocks.showQuickPick.mockResolvedValue('No'); // chain mode

    await registeredCommands.get('mahalaxmi.startCycle')!();

    expect(remoteClient.startCycleCalls).toHaveLength(1);
    expect(remoteClient.startCycleCalls[0].repoUrl).toBe('https://github.com/org/repo.git');
    expect(remoteClient.startCycleCalls[0].projectRoot).toBeUndefined();
  });

  it('remote_mode_shows_action_picker_when_exists: checkRepoStatus exists=true shows QuickPick', async () => {
    const remoteClient = new StubClient(true);
    remoteClient.checkRepoStatus.mockResolvedValue({ exists: true, path: '/srv/repo', status: 'clean' });
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(remoteClient as any, mockContext as any);
    mocks.showInputBox
      .mockResolvedValueOnce('fix bug')                           // requirements
      .mockResolvedValueOnce('https://github.com/org/repo.git')  // repoUrl
      .mockResolvedValueOnce('main');                            // branch
    // showQuickPick: chain=No, then action picker
    mocks.showQuickPick
      .mockResolvedValueOnce('No')                                               // chain mode
      .mockResolvedValueOnce({ label: 'Continue', value: 'continue' });         // action picker

    await registeredCommands.get('mahalaxmi.startCycle')!();

    expect(remoteClient.checkRepoStatus).toHaveBeenCalledWith('https://github.com/org/repo.git');
    expect(remoteClient.startCycleCalls[0].repoAction).toBe('continue');
  });

  it('remote_mode_skips_action_picker_when_not_cloned: exists=false shows no action QuickPick', async () => {
    const remoteClient = new StubClient(true);
    remoteClient.checkRepoStatus.mockResolvedValue({ exists: false, path: '', status: 'not_cloned' });
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(remoteClient as any, mockContext as any);
    mocks.showInputBox
      .mockResolvedValueOnce('add tests')                         // requirements
      .mockResolvedValueOnce('https://github.com/org/repo.git')  // repoUrl
      .mockResolvedValueOnce('main');                            // branch
    mocks.showQuickPick.mockResolvedValue('No'); // only chain mode picker

    await registeredCommands.get('mahalaxmi.startCycle')!();

    // showQuickPick called exactly once (chain mode only, no action picker)
    expect(mocks.showQuickPick).toHaveBeenCalledTimes(1);
    expect(remoteClient.startCycleCalls[0].repoAction).toBeUndefined();
  });

  it('cancelled_at_repo_url_aborts: showInputBox undefined for URL stops before startCycle', async () => {
    const remoteClient = new StubClient(true);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(remoteClient as any, mockContext as any);
    mocks.showInputBox
      .mockResolvedValueOnce('some work')  // requirements
      .mockResolvedValueOnce(undefined);   // cancelled at repo URL prompt
    mocks.showQuickPick.mockResolvedValue('No'); // chain mode

    await registeredCommands.get('mahalaxmi.startCycle')!();

    expect(remoteClient.startCycleCalls).toHaveLength(0);
  });

  // -------------------------------------------------------------------------
  // Cloud mode commands
  // -------------------------------------------------------------------------

  it('startCloudVm_shows_error_when_cloud_not_configured: readCloudConfig null → showErrorMessage, connect not called', async () => {
    mocks.readCloudConfig.mockResolvedValue(null);
    const mockConnect = vi.fn();
    setCloudSessionManager({
      connect: mockConnect,
      disconnect: vi.fn().mockResolvedValue(undefined),
      getState: vi.fn().mockReturnValue('disconnected'),
    });

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    const handler = registeredCommands.get('mahalaxmi.startCloudVm');
    expect(handler).toBeDefined();
    await handler!();

    expect(mocks.showErrorMessage).toHaveBeenCalledWith(
      expect.stringContaining('Cloud not configured'),
    );
    expect(mockConnect).not.toHaveBeenCalled();
  });

  it('stopCloudVm_shows_confirmation_dialog_before_disconnect: confirmed → disconnect called', async () => {
    const mockDisconnect = vi.fn().mockResolvedValue(undefined);
    setCloudSessionManager({
      connect: vi.fn().mockResolvedValue(null),
      disconnect: mockDisconnect,
      getState: vi.fn().mockReturnValue('connected'),
    });

    mocks.showWarningMessage.mockResolvedValue('Stop');

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    registerCommands(client as any, mockContext as any);
    const handler = registeredCommands.get('mahalaxmi.stopCloudVm');
    expect(handler).toBeDefined();
    await handler!();

    expect(mocks.showWarningMessage).toHaveBeenCalledWith(
      expect.stringContaining('Stop cloud VM'),
      expect.objectContaining({ modal: true }),
      'Stop',
      'Cancel',
    );
    expect(mockDisconnect).toHaveBeenCalled();
  });
});

// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

import { CloudSessionManager, ProvisioningError } from './cloud-session-manager';
import type { ProvisioningClient, VmEndpoint } from './cloud-session-manager';
import type { CloudConfig } from './cloud-config';
import type { MahalaxmiStatusBar } from './statusbar';

// ---------------------------------------------------------------------------
// Hoist shared mock state — must be declared before any vi.mock() call
// ---------------------------------------------------------------------------

const shared = vi.hoisted(() => {
  // Tracks every MockRemoteApiClient constructed during a test
  const remoteInstances: {
    baseUrl: string;
    apiToken: string;
    updateToken: ReturnType<typeof vi.fn>;
    set401Handler: ReturnType<typeof vi.fn>;
    on: ReturnType<typeof vi.fn>;
  }[] = [];

  return {
    withProgress: vi.fn(),
    showErrorMessage: vi.fn(),
    showInformationMessage: vi.fn(),
    remoteInstances,
  };
});

// ---------------------------------------------------------------------------
// Mock vscode
// ---------------------------------------------------------------------------

vi.mock('vscode', () => ({
  window: {
    withProgress: shared.withProgress,
    showErrorMessage: shared.showErrorMessage,
    showInformationMessage: shared.showInformationMessage,
  },
  ProgressLocation: { Notification: 15, Window: 10, SourceControl: 1 },
}));

// ---------------------------------------------------------------------------
// Mock RemoteApiClient — captures constructor args and exposes stubs
// ---------------------------------------------------------------------------

vi.mock('./remote-client', () => {
  class MockRemoteApiClient {
    updateToken = vi.fn();
    set401Handler = vi.fn();
    on = vi.fn();

    constructor(
      public readonly baseUrl: string,
      public readonly apiToken: string,
    ) {
      shared.remoteInstances.push(this);
    }
  }
  return { RemoteApiClient: MockRemoteApiClient };
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeVmEndpoint(overrides: Partial<VmEndpoint> = {}): VmEndpoint {
  return {
    endpoint: 'https://proj-x.mahalaxmi.ai:17421',
    sessionToken: 'tok-abc123',
    expiresAt: new Date(Date.now() + 3600 * 1000), // 1 hour from now
    ...overrides,
  };
}

function makeConfig(): CloudConfig {
  return {
    endpoint: 'https://proj-x.mahalaxmi.ai:17421',
    api_key: 'mhx_key_test',
    project_id: 'proj-uuid-test',
  };
}

function makeStatusBar(): { setCloudVmState: ReturnType<typeof vi.fn>; dispose: ReturnType<typeof vi.fn> } {
  return {
    setCloudVmState: vi.fn(),
    dispose: vi.fn(),
  };
}

function makeContext(): { subscriptions: { dispose(): void }[] } {
  return { subscriptions: [] };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('CloudSessionManager', () => {
  let provisioning: { [K in keyof ProvisioningClient]: ReturnType<typeof vi.fn> };
  let statusBar: ReturnType<typeof makeStatusBar>;
  let manager: CloudSessionManager;

  beforeEach(() => {
    vi.clearAllMocks();
    shared.remoteInstances.length = 0;

    provisioning = {
      startVm: vi.fn(),
      stopVm: vi.fn(),
      getEndpoint: vi.fn(),
    };

    statusBar = makeStatusBar();

    manager = new CloudSessionManager(
      makeConfig(),
      provisioning as unknown as ProvisioningClient,
      statusBar as unknown as MahalaxmiStatusBar,
      makeContext() as unknown as import('vscode').ExtensionContext,
    );

    // Default: withProgress executes the task and returns its result
    shared.withProgress.mockImplementation(
      async (
        _opts: unknown,
        task: (progress: { report(v: { message?: string }): void }, token: unknown) => Promise<unknown>,
      ) => task({ report: vi.fn() }, undefined),
    );
  });

  afterEach(() => {
    vi.useRealTimers();
    manager.dispose();
  });

  // -------------------------------------------------------------------------
  // connect() — happy path
  // -------------------------------------------------------------------------

  it('connect() calls startVm and returns a RemoteApiClient', async () => {
    const endpoint = makeVmEndpoint();
    provisioning.startVm.mockResolvedValueOnce(endpoint);

    const client = await manager.connect();

    expect(provisioning.startVm).toHaveBeenCalledOnce();
    expect(shared.remoteInstances).toHaveLength(1);

    const instance = shared.remoteInstances[0];
    expect(instance.baseUrl).toBe(endpoint.endpoint);
    expect(instance.apiToken).toBe(endpoint.sessionToken);
    expect(client).toBe(instance);
  });

  it('connect() transitions statusBar through Starting then Connected', async () => {
    provisioning.startVm.mockResolvedValueOnce(makeVmEndpoint());

    await manager.connect();

    const kinds = (statusBar.setCloudVmState.mock.calls as { kind: string }[][]).map(
      ([s]) => s.kind,
    );
    expect(kinds).toContain('Starting');
    expect(kinds).toContain('Connected');

    const startIdx = kinds.indexOf('Starting');
    const connIdx = kinds.lastIndexOf('Connected');
    expect(startIdx).toBeLessThan(connIdx);
  });

  it('connect() wraps startVm in a withProgress notification', async () => {
    provisioning.startVm.mockResolvedValueOnce(makeVmEndpoint());

    await manager.connect();

    expect(shared.withProgress).toHaveBeenCalledOnce();
    const [opts] = shared.withProgress.mock.calls[0] as [{ title: string; location: number }];
    expect(opts.title).toContain('Starting Cloud VM');
    expect(opts.location).toBe(15); // ProgressLocation.Notification
  });

  it('connect() passes an onProgress callback that reports to the progress notification', async () => {
    const reportFn = vi.fn();
    shared.withProgress.mockImplementation(
      async (
        _opts: unknown,
        task: (progress: { report(v: { message?: string }): void }, token: unknown) => Promise<unknown>,
      ) => task({ report: reportFn }, undefined),
    );

    provisioning.startVm.mockImplementationOnce(
      async (onProgress: (msg: string) => void) => {
        onProgress('Allocating resources');
        return makeVmEndpoint();
      },
    );

    await manager.connect();

    expect(reportFn).toHaveBeenCalledWith(
      expect.objectContaining({ message: 'Allocating resources' }),
    );
  });

  // -------------------------------------------------------------------------
  // connect() — ProvisioningError path
  // -------------------------------------------------------------------------

  it('connect() sets statusBar to Offline on ProvisioningError', async () => {
    provisioning.startVm.mockRejectedValueOnce(new ProvisioningError('VM quota exceeded'));

    await expect(manager.connect()).rejects.toThrow('VM quota exceeded');

    const kinds = (statusBar.setCloudVmState.mock.calls as { kind: string }[][]).map(
      ([s]) => s.kind,
    );
    expect(kinds).toContain('Offline');
  });

  it('connect() shows an error toast that includes the error message on ProvisioningError', async () => {
    provisioning.startVm.mockRejectedValueOnce(
      new ProvisioningError('Insufficient quota'),
    );
    shared.showErrorMessage.mockResolvedValueOnce(undefined);

    await expect(manager.connect()).rejects.toThrow();

    expect(shared.showErrorMessage).toHaveBeenCalledOnce();
    const [msg] = shared.showErrorMessage.mock.calls[0] as [string];
    expect(msg).toContain('Insufficient quota');
  });

  it('connect() sets statusBar Offline before showing the error toast', async () => {
    provisioning.startVm.mockRejectedValueOnce(new ProvisioningError('Network failure'));

    let offlineAlreadySet = false;
    shared.showErrorMessage.mockImplementationOnce(() => {
      const kinds = (statusBar.setCloudVmState.mock.calls as { kind: string }[][]).map(
        ([s]) => s.kind,
      );
      offlineAlreadySet = kinds.includes('Offline');
      return Promise.resolve(undefined);
    });

    await expect(manager.connect()).rejects.toThrow();

    expect(offlineAlreadySet).toBe(true);
  });

  it('connect() also shows an error toast for generic (non-ProvisioningError) failures', async () => {
    provisioning.startVm.mockRejectedValueOnce(new Error('Unexpected error'));
    shared.showErrorMessage.mockResolvedValueOnce(undefined);

    await expect(manager.connect()).rejects.toThrow('Unexpected error');

    expect(shared.showErrorMessage).toHaveBeenCalledOnce();
    const [msg] = shared.showErrorMessage.mock.calls[0] as [string];
    expect(msg).toContain('Unexpected error');
  });

  // -------------------------------------------------------------------------
  // Token refresh scheduling
  // -------------------------------------------------------------------------

  it('connect() schedules token refresh at expiresAt minus 10 minutes', async () => {
    vi.useFakeTimers();

    // expiresAt = fake-now + 30 min → timer fires after 20 min of fake time
    const expiresAt = new Date(Date.now() + 30 * 60 * 1000);
    provisioning.startVm.mockResolvedValueOnce(makeVmEndpoint({ expiresAt }));

    // Provide a far-future expiresAt on the refresh response so the
    // re-scheduled timer does not fire again during this test
    const freshEndpoint = makeVmEndpoint({
      sessionToken: 'tok-refreshed',
      expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 24 h away
    });
    provisioning.getEndpoint.mockResolvedValue(freshEndpoint);

    await manager.connect();

    const instance = shared.remoteInstances[0];
    expect(instance.updateToken).not.toHaveBeenCalled();

    // Advance to just before the 20-minute mark — refresh must not fire yet
    await vi.advanceTimersByTimeAsync(20 * 60 * 1000 - 1);
    expect(provisioning.getEndpoint).not.toHaveBeenCalled();

    // Advance past the 20-minute mark — first refresh fires
    await vi.advanceTimersByTimeAsync(2);
    // Flush the async chain inside refreshToken
    await Promise.resolve();
    await Promise.resolve();
    await Promise.resolve();

    expect(provisioning.getEndpoint).toHaveBeenCalledOnce();
    expect(instance.updateToken).toHaveBeenCalledWith('tok-refreshed');
  });

  it('connect() applies token immediately when expiresAt is already within 10 minutes', async () => {
    const expiresAt = new Date(Date.now() + 5 * 60 * 1000); // 5 min — already < 10 min
    provisioning.startVm.mockResolvedValueOnce(makeVmEndpoint({ expiresAt }));

    const freshEndpoint = makeVmEndpoint({ sessionToken: 'tok-immediate' });
    provisioning.getEndpoint.mockResolvedValueOnce(freshEndpoint);

    await manager.connect();

    // Allow microtask queue to flush
    await new Promise((r) => setTimeout(r, 0));

    expect(provisioning.getEndpoint).toHaveBeenCalledOnce();
    const instance = shared.remoteInstances[0];
    expect(instance.updateToken).toHaveBeenCalledWith('tok-immediate');
  });

  it('dispose() clears the token refresh timer so getEndpoint is never called', async () => {
    vi.useFakeTimers();

    const expiresAt = new Date(Date.now() + 30 * 60 * 1000);
    provisioning.startVm.mockResolvedValueOnce(makeVmEndpoint({ expiresAt }));
    provisioning.stopVm.mockResolvedValueOnce(undefined);

    await manager.connect();
    manager.dispose();

    await vi.runAllTimersAsync();

    expect(provisioning.getEndpoint).not.toHaveBeenCalled();
  });

  // -------------------------------------------------------------------------
  // onServerStopped()
  // -------------------------------------------------------------------------

  it('onServerStopped() shows an information message with a Reconnect action', () => {
    shared.showInformationMessage.mockResolvedValueOnce(undefined);

    manager.onServerStopped();

    expect(shared.showInformationMessage).toHaveBeenCalledOnce();
    const [msg, action] = shared.showInformationMessage.mock.calls[0] as [string, string];
    expect(msg).toContain('Cloud server stopped');
    expect(msg).toContain('Reconnect');
    expect(action).toBe('Reconnect');
  });

  it('onServerStopped() sets statusBar to Offline', () => {
    shared.showInformationMessage.mockResolvedValueOnce(undefined);

    manager.onServerStopped();

    expect(statusBar.setCloudVmState).toHaveBeenCalledWith({ kind: 'Offline' });
  });

  it('onServerStopped() calls connect() when user clicks Reconnect', async () => {
    shared.showInformationMessage.mockResolvedValueOnce('Reconnect');
    provisioning.startVm.mockResolvedValue(makeVmEndpoint());

    manager.onServerStopped();

    // Let the promise chain resolve
    await new Promise((r) => setTimeout(r, 0));

    expect(provisioning.startVm).toHaveBeenCalledOnce();
  });

  it('onServerStopped() does not call connect() when user dismisses the notification', async () => {
    shared.showInformationMessage.mockResolvedValueOnce(undefined);

    manager.onServerStopped();

    await new Promise((r) => setTimeout(r, 0));

    expect(provisioning.startVm).not.toHaveBeenCalled();
  });

  // -------------------------------------------------------------------------
  // disconnect()
  // -------------------------------------------------------------------------

  it('disconnect() transitions statusBar through Stopping then Offline', async () => {
    provisioning.startVm.mockResolvedValueOnce(makeVmEndpoint());
    provisioning.stopVm.mockResolvedValueOnce(undefined);

    await manager.connect();
    statusBar.setCloudVmState.mockClear();

    await manager.disconnect();

    expect(provisioning.stopVm).toHaveBeenCalledOnce();
    const kinds = (statusBar.setCloudVmState.mock.calls as { kind: string }[][]).map(
      ([s]) => s.kind,
    );
    expect(kinds).toContain('Stopping');
    expect(kinds).toContain('Offline');
    expect(kinds.indexOf('Stopping')).toBeLessThan(kinds.indexOf('Offline'));
  });

  it('disconnect() is a no-op when not connected', async () => {
    await manager.disconnect();

    expect(provisioning.stopVm).not.toHaveBeenCalled();
  });

  it('disconnect() sets statusBar to Offline even when stopVm throws', async () => {
    provisioning.startVm.mockResolvedValueOnce(makeVmEndpoint());
    provisioning.stopVm.mockRejectedValueOnce(new Error('stop failed'));

    await manager.connect();

    await expect(manager.disconnect()).rejects.toThrow('stop failed');

    const kinds = (statusBar.setCloudVmState.mock.calls as { kind: string }[][]).map(
      ([s]) => s.kind,
    );
    expect(kinds).toContain('Offline');
  });
});

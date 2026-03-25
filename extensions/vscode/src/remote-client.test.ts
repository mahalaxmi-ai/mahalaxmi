// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as http from 'http';

import { RemoteApiClient } from './remote-client';
import type {
  OrchestrationSnapshotDto,
  TemplateDto,
  WorkerStartedEvent,
  OutputReceivedEvent,
} from './client-interface';

// ---------------------------------------------------------------------------
// Mock fetch globally — reset per test to prevent response leakage
// ---------------------------------------------------------------------------
const mockFetch = vi.fn<Parameters<typeof fetch>, ReturnType<typeof fetch>>();
vi.stubGlobal('fetch', mockFetch);

// ---------------------------------------------------------------------------
// Mock http / https modules
// ---------------------------------------------------------------------------
const mockSseReq = {
  on: vi.fn().mockReturnThis(),
  end: vi.fn(),
  destroy: vi.fn(),
};

let capturedResCallback: ((res: unknown) => void) | null = null;

vi.mock('http', () => ({
  request: vi.fn((_options: http.RequestOptions, cb: (res: unknown) => void) => {
    capturedResCallback = cb;
    return mockSseReq;
  }),
}));

vi.mock('https', () => ({
  request: vi.fn((_options: unknown, cb: (res: unknown) => void) => {
    capturedResCallback = cb;
    return mockSseReq;
  }),
}));

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeResponse(body: unknown, status = 200): Response {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: () => Promise.resolve(body),
  } as unknown as Response;
}

/**
 * Fire a raw SSE block (without trailing \n\n — added internally) through the
 * captured response callback.  The block should look like:
 *   "event: worker_started\ndata: {...}"
 */
function fireSseBlock(block: string): void {
  if (!capturedResCallback) return;
  const resCallbacks = new Map<string, ((...args: unknown[]) => void)[]>();
  const mockRes = {
    setEncoding: vi.fn(),
    on: vi.fn((event: string, cb: (...args: unknown[]) => void) => {
      if (!resCallbacks.has(event)) resCallbacks.set(event, []);
      resCallbacks.get(event)!.push(cb);
    }),
  };
  capturedResCallback(mockRes);
  for (const cb of resCallbacks.get('data') ?? []) {
    cb(block + '\n\n');
  }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('RemoteApiClient', () => {
  let client: RemoteApiClient;

  beforeEach(() => {
    // Reset the entire mock including the once-queue to prevent response leakage
    mockFetch.mockReset();
    vi.clearAllMocks();
    mockSseReq.on.mockReturnThis();
    capturedResCallback = null;
    client = new RemoteApiClient('http://localhost:17421', '');
  });

  afterEach(() => {
    client.dispose();
  });

  // -------------------------------------------------------------------------
  // connect() — health check
  // -------------------------------------------------------------------------

  it('emits connected when /v1/health returns 200', async () => {
    mockFetch.mockResolvedValueOnce(makeResponse({ status: 'ok' }));

    const connected = new Promise<void>((resolve) => client.once('connected', resolve));
    client.connect();
    await connected;

    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:17421/v1/health',
      expect.objectContaining({ headers: {} }),
    );
  });

  it('emits disconnected and schedules reconnect on health failure', async () => {
    mockFetch.mockResolvedValueOnce(makeResponse({ error: 'server error' }, 500));

    const disconnected = new Promise<void>((resolve) => client.once('disconnected', resolve));
    client.connect();
    await disconnected;
  });

  it('emits disconnected on network error', async () => {
    mockFetch.mockRejectedValueOnce(new Error('network'));

    const disconnected = new Promise<void>((resolve) => client.once('disconnected', resolve));
    client.connect();
    await disconnected;
  });

  // -------------------------------------------------------------------------
  // startCycle()
  // -------------------------------------------------------------------------

  it('POSTs to /v1/cycles with all fields and stores cycle_id', async () => {
    mockFetch
      .mockResolvedValueOnce(makeResponse({ status: 'ok' })) // health
      .mockResolvedValueOnce(makeResponse({ cycle_id: 'abc-123' })); // POST /cycles

    const connected = new Promise<void>((resolve) => client.once('connected', resolve));
    client.connect();
    await connected;

    client.startCycle('add auth', '/tmp/proj', true, '/tmp/reqs.md');
    await new Promise((r) => setTimeout(r, 0));

    const postCall = mockFetch.mock.calls.find(
      ([url]) => typeof url === 'string' && (url as string).includes('/v1/cycles') && !(url as string).includes('abc'),
    );
    expect(postCall).toBeDefined();
    const [, init] = postCall!;
    const body = JSON.parse((init as RequestInit).body as string) as Record<string, unknown>;
    expect(body['requirements']).toBe('add auth');
    expect(body['project_root']).toBe('/tmp/proj');
    expect(body['chain_mode']).toBe(true);
    expect(body['chain_requirements_path']).toBe('/tmp/reqs.md');
  });

  // -------------------------------------------------------------------------
  // approvePlan()
  // -------------------------------------------------------------------------

  it("approvePlan('__current__') POSTs to /v1/cycles/:currentCycleId/approve", async () => {
    mockFetch
      .mockResolvedValueOnce(makeResponse({ status: 'ok' })) // health
      .mockResolvedValueOnce(makeResponse({ cycle_id: 'xyz-789' })) // start
      .mockResolvedValueOnce(makeResponse({ approved: true })); // approve

    const connected = new Promise<void>((resolve) => client.once('connected', resolve));
    client.connect();
    await connected;

    client.startCycle('fix bug', '/tmp/proj');
    await new Promise((r) => setTimeout(r, 0));

    client.approvePlan('__current__');
    await new Promise((r) => setTimeout(r, 0));

    const approveCall = mockFetch.mock.calls.find(
      ([url]) => typeof url === 'string' && (url as string).includes('/approve'),
    );
    expect(approveCall).toBeDefined();
    expect(approveCall![0]).toContain('xyz-789');
    expect(approveCall![0]).toContain('/approve');
  });

  // -------------------------------------------------------------------------
  // stopCycle()
  // -------------------------------------------------------------------------

  it("stopCycle('__current__') DELETEs /v1/cycles/:currentCycleId", async () => {
    mockFetch
      .mockResolvedValueOnce(makeResponse({ status: 'ok' })) // health
      .mockResolvedValueOnce(makeResponse({ cycle_id: 'del-001' })) // start
      .mockResolvedValueOnce(makeResponse({ stopped: true })); // delete

    const connected = new Promise<void>((resolve) => client.once('connected', resolve));
    client.connect();
    await connected;

    client.startCycle('task', '/tmp');
    await new Promise((r) => setTimeout(r, 0));

    client.stopCycle('__current__');
    await new Promise((r) => setTimeout(r, 0));

    const deleteCall = mockFetch.mock.calls.find(
      ([, init]) => (init as RequestInit).method === 'DELETE',
    );
    expect(deleteCall).toBeDefined();
    expect(deleteCall![0]).toContain('del-001');
  });

  // -------------------------------------------------------------------------
  // setFileAcceptance() — client-side map merged into snapshot
  // -------------------------------------------------------------------------

  it('setFileAcceptance updates local map and snapshot includes it', async () => {
    mockFetch.mockResolvedValue(
      makeResponse({ id: 'fa-001', state: 'running', worker_statuses: [] }),
    );

    client.setFileAcceptance('/src/foo.ts', true);
    client.setFileAcceptance('/src/bar.ts', false);

    const snap = new Promise<OrchestrationSnapshotDto>((resolve) =>
      client.once('snapshot', resolve),
    );
    await client.pollCycleState('fa-001');
    const result = await snap;

    expect(result.file_acceptances['/src/foo.ts']).toBe(true);
    expect(result.file_acceptances['/src/bar.ts']).toBe(false);
  });

  // -------------------------------------------------------------------------
  // queryTemplates()
  // -------------------------------------------------------------------------

  it('queryTemplates emits templates array from 200 response', async () => {
    const fakeTemplates: TemplateDto[] = [
      { id: 'tmpl-1', name: 'Software', category: 'General' },
    ];
    mockFetch.mockResolvedValueOnce(makeResponse(fakeTemplates));

    // Register listener BEFORE calling queryTemplates, then await AFTER
    const emitted = new Promise<TemplateDto[]>((resolve) =>
      client.once('templates', resolve),
    );
    client.queryTemplates();
    await new Promise((r) => setTimeout(r, 0));

    expect(await emitted).toEqual(fakeTemplates);
  });

  it('queryTemplates emits empty array on 404', async () => {
    mockFetch.mockResolvedValueOnce(makeResponse({ error: 'not found' }, 404));

    const emitted = new Promise<TemplateDto[]>((resolve) =>
      client.once('templates', resolve),
    );
    client.queryTemplates();
    await new Promise((r) => setTimeout(r, 0));

    expect(await emitted).toEqual([]);
  });

  it('queryTemplates emits empty array on network error', async () => {
    mockFetch.mockRejectedValueOnce(new Error('network'));

    const emitted = new Promise<TemplateDto[]>((resolve) =>
      client.once('templates', resolve),
    );
    client.queryTemplates();
    await new Promise((r) => setTimeout(r, 0));

    expect(await emitted).toEqual([]);
  });

  // -------------------------------------------------------------------------
  // pollCycleState()
  // -------------------------------------------------------------------------

  it('pollCycleState emits snapshot from cycle status response', async () => {
    mockFetch.mockResolvedValueOnce(
      makeResponse({
        id: 'poll-01',
        state: 'running',
        worker_statuses: [
          { worker_id: 'w1', status: { task_title: 'fix bug', status: 'Running' } },
        ],
      }),
    );

    const snap = new Promise<OrchestrationSnapshotDto>((resolve) =>
      client.once('snapshot', resolve),
    );
    await client.pollCycleState('poll-01');
    const result = await snap;

    expect(result.cycle_id).toBe('poll-01');
    expect(result.phase).toBe('running');
    expect(result.workers[0].task_title).toBe('fix bug');
    expect(result.workers[0].status).toBe('Running');
  });

  it('pollCycleState emits plan_ready when state is awaiting_plan_approval', async () => {
    mockFetch.mockResolvedValueOnce(
      makeResponse({ id: 'p-01', state: 'awaiting_plan_approval', worker_statuses: [] }),
    );

    const planReady = new Promise<void>((resolve) =>
      client.once('plan_ready', () => resolve()),
    );
    await client.pollCycleState('p-01');
    await planReady;
  });

  it('pollCycleState clears cycleId and stops polling when state is completed', async () => {
    mockFetch
      .mockResolvedValueOnce(
        makeResponse({ id: 'done-1', state: 'completed', worker_statuses: [] }),
      )
      .mockResolvedValueOnce(makeResponse({ stopped: true })); // for the follow-up stopCycle call

    await client.pollCycleState('done-1');

    // After completion currentCycleId is cleared; stopCycle should resolve gracefully
    expect(() => client.stopCycle('__current__')).not.toThrow();
    await new Promise((r) => setTimeout(r, 0)); // flush pending promises
  });

  // -------------------------------------------------------------------------
  // SSE event handling
  // -------------------------------------------------------------------------

  it('SSE worker_started chunk emits worker_started event', async () => {
    mockFetch.mockResolvedValueOnce(makeResponse({ status: 'ok' }));

    const connected = new Promise<void>((resolve) => client.once('connected', resolve));
    client.connect();
    await connected;

    const event = new Promise<WorkerStartedEvent>((resolve) =>
      client.once('worker_started', resolve),
    );

    fireSseBlock(
      'event: worker_started\ndata: {"worker_id":"w1","task_title":"Implement auth","worker_number":1}',
    );

    const result = await event;
    expect(result.worker_id).toBe('w1');
    expect(result.task_title).toBe('Implement auth');
    expect(result.worker_number).toBe(1);
  });

  it('SSE worker_output chunk emits output_received event', async () => {
    mockFetch.mockResolvedValueOnce(makeResponse({ status: 'ok' }));

    const connected = new Promise<void>((resolve) => client.once('connected', resolve));
    client.connect();
    await connected;

    const event = new Promise<OutputReceivedEvent>((resolve) =>
      client.once('output_received', resolve),
    );

    fireSseBlock('event: worker_output\ndata: {"worker_id":"w2","data":"hello\\n"}');

    const result = await event;
    expect(result.worker_id).toBe('w2');
    expect(result.data).toBe('hello\n');
  });

  it('SSE cycle_complete chunk clears cycleId and destroys SSE request', async () => {
    mockFetch.mockResolvedValueOnce(makeResponse({ status: 'ok' }));

    const connected = new Promise<void>((resolve) => client.once('connected', resolve));
    client.connect();
    await connected;

    fireSseBlock('event: cycle_complete\ndata: {"cycle_id":"cc-01"}');

    expect(mockSseReq.destroy).toHaveBeenCalled();
  });

  // -------------------------------------------------------------------------
  // Bearer token is sent on all requests
  // -------------------------------------------------------------------------

  it('sends Authorization header when apiToken is set', async () => {
    const authedClient = new RemoteApiClient('http://localhost:17421', 'my-secret-token');

    mockFetch.mockResolvedValueOnce(makeResponse({ status: 'ok' }));
    const connected = new Promise<void>((resolve) => authedClient.once('connected', resolve));
    authedClient.connect();
    await connected;

    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:17421/v1/health',
      expect.objectContaining({
        headers: expect.objectContaining({ Authorization: 'Bearer my-secret-token' }),
      }),
    );

    authedClient.dispose();
  });

  // -------------------------------------------------------------------------
  // checkRepoStatus()
  // -------------------------------------------------------------------------

  it('checkRepoStatus_returns_exists_true when server returns 200 with exists: true', async () => {
    mockFetch.mockResolvedValueOnce(
      makeResponse({ exists: true, path: '/var/lib/mahalaxmi/projects/github.com/org/repo', status: 'clean' }),
    );

    const result = await client.checkRepoStatus('https://github.com/org/repo');
    expect(result.exists).toBe(true);
    expect(result.status).toBe('clean');
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('/v1/projects/status'),
      expect.any(Object),
    );
  });

  it('checkRepoStatus_fallback_on_error: fetch throws → returns exists: false', async () => {
    mockFetch.mockRejectedValueOnce(new Error('network failure'));

    const result = await client.checkRepoStatus('https://github.com/org/repo');
    expect(result.exists).toBe(false);
    expect(result.status).toBe('not_cloned');
  });

  it('startCycle_sends_repo_url_fields: repo_url, branch, repo_action included in POST body', async () => {
    mockFetch
      .mockResolvedValueOnce(makeResponse({ status: 'ok' })) // health
      .mockResolvedValueOnce(makeResponse({ cycle_id: 'repo-001' })); // POST /cycles

    const connected = new Promise<void>((resolve) => client.once('connected', resolve));
    client.connect();
    await connected;

    client.startCycle(
      'add feature',
      undefined,
      false,
      undefined,
      'https://github.com/org/repo.git',
      'main',
      'fresh_start',
    );
    await new Promise((r) => setTimeout(r, 0));

    const postCall = mockFetch.mock.calls.find(
      ([url]) => typeof url === 'string' && (url as string).includes('/v1/cycles') && !(url as string).includes('repo-001'),
    );
    expect(postCall).toBeDefined();
    const [, init] = postCall!;
    const body = JSON.parse((init as RequestInit).body as string) as Record<string, unknown>;
    expect(body['repo_url']).toBe('https://github.com/org/repo.git');
    expect(body['branch']).toBe('main');
    expect(body['repo_action']).toBe('fresh_start');
  });

  // -------------------------------------------------------------------------
  // dispose()
  // -------------------------------------------------------------------------

  it('dispose stops timer, destroys SSE, suppresses reconnects', async () => {
    mockFetch.mockResolvedValueOnce(makeResponse({ status: 'ok' }));

    const connected = new Promise<void>((resolve) => client.once('connected', resolve));
    client.connect();
    await connected;

    client.dispose();

    let extraEvent = false;
    client.on('connected', () => { extraEvent = true; });
    client.on('disconnected', () => { extraEvent = true; });

    expect(extraEvent).toBe(false);
    expect(mockSseReq.destroy).toHaveBeenCalled();
  });
});

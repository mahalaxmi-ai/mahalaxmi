// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ProvisioningClient, ProvisioningError } from './provisioning-client';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeOkResponse(body: unknown): Response {
  return {
    ok: true,
    status: 200,
    text: async () => JSON.stringify(body),
    json: async () => body,
  } as unknown as Response;
}

function makeErrorResponse(status: number, text = ''): Response {
  return {
    ok: false,
    status,
    text: async () => text,
    json: async () => ({}),
  } as unknown as Response;
}

const VALID_ENDPOINT = {
  endpoint: 'https://vm-1.mahalaxmi.ai:17421',
  sessionToken: 'sess_abc123',
  expiresAt: '2026-12-31T00:00:00Z',
};

// ---------------------------------------------------------------------------
// startVm
// ---------------------------------------------------------------------------

describe('ProvisioningClient.startVm', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it('returns VmEndpoint when endpoint becomes available on the 2nd poll', async () => {
    let endpointCallCount = 0;

    const fetchMock = vi.fn().mockImplementation(async (url: string) => {
      if ((url as string).includes('/start')) {
        return makeOkResponse({});
      }
      if ((url as string).includes('/endpoint')) {
        endpointCallCount++;
        if (endpointCallCount === 1) {
          // Not ready on the first poll — return incomplete payload
          return makeOkResponse({});
        }
        // Ready on the second poll
        return makeOkResponse(VALID_ENDPOINT);
      }
      return makeOkResponse({});
    });

    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    const progressMsgs: string[] = [];

    const vmPromise = client.startVm((msg) => progressMsgs.push(msg));

    // Advance 5 seconds to trigger the poll interval after the first failed check
    await vi.advanceTimersByTimeAsync(5_000);

    const result = await vmPromise;

    expect(result.endpoint).toBe(VALID_ENDPOINT.endpoint);
    expect(result.sessionToken).toBe(VALID_ENDPOINT.sessionToken);
    expect(result.expiresAt).toBe(VALID_ENDPOINT.expiresAt);
    expect(progressMsgs.length).toBeGreaterThanOrEqual(1);
    expect(progressMsgs[0]).toMatch(/Starting VM\.\.\. \(\d+s elapsed\)/);
    expect(endpointCallCount).toBe(2);
  });

  it('calls onProgress on each poll tick with elapsed time in the message', async () => {
    let endpointCallCount = 0;

    const fetchMock = vi.fn().mockImplementation(async (url: string) => {
      if ((url as string).includes('/start')) {
        return makeOkResponse({});
      }
      if ((url as string).includes('/endpoint')) {
        endpointCallCount++;
        if (endpointCallCount < 3) {
          return makeOkResponse({});
        }
        return makeOkResponse(VALID_ENDPOINT);
      }
      return makeOkResponse({});
    });

    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    const progressMsgs: string[] = [];

    const vmPromise = client.startVm((msg) => progressMsgs.push(msg));

    // Advance 10 seconds — enough for 2 poll intervals
    await vi.advanceTimersByTimeAsync(10_000);

    await vmPromise;

    expect(progressMsgs.length).toBeGreaterThanOrEqual(2);
    for (const msg of progressMsgs) {
      expect(msg).toMatch(/^Starting VM\.\.\. \(\d+s elapsed\)$/);
    }
  });

  it('throws ProvisioningError with statusCode 408 after 120s without a valid endpoint', async () => {
    const fetchMock = vi.fn().mockImplementation(async (url: string) => {
      if ((url as string).includes('/start')) {
        return makeOkResponse({});
      }
      // /endpoint always returns incomplete payload — VM never becomes ready
      return makeOkResponse({});
    });

    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');

    // Capture the error via .catch before advancing timers to avoid an
    // unhandled-rejection warning during the fake-timer advancement.
    let caughtError: ProvisioningError | undefined;
    const vmPromise = client.startVm(() => {}).catch((err: unknown) => {
      if (err instanceof ProvisioningError) {
        caughtError = err;
      } else {
        throw err;
      }
    });

    // Advance past the 120-second timeout
    await vi.advanceTimersByTimeAsync(121_000);
    await vmPromise;

    expect(caughtError).toBeInstanceOf(ProvisioningError);
    expect(caughtError?.statusCode).toBe(408);
    expect(caughtError?.message).toBe('VM start timed out after 120s');
  });

  it('propagates ProvisioningError from the start POST', async () => {
    const fetchMock = vi.fn().mockImplementation(async (url: string) => {
      if ((url as string).includes('/start')) {
        return makeErrorResponse(503, 'Service Unavailable');
      }
      return makeOkResponse({});
    });

    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    await expect(client.startVm(() => {})).rejects.toBeInstanceOf(ProvisioningError);
    await expect(client.startVm(() => {})).rejects.toMatchObject({ statusCode: 503 });
  });
});

// ---------------------------------------------------------------------------
// stopVm
// ---------------------------------------------------------------------------

describe('ProvisioningClient.stopVm', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('resolves when the stop endpoint returns 200', async () => {
    const fetchMock = vi.fn().mockResolvedValue(makeOkResponse({}));
    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    await expect(client.stopVm()).resolves.toBeUndefined();
    expect(fetchMock).toHaveBeenCalledOnce();
    const [url] = fetchMock.mock.calls[0] as [string];
    expect(url).toContain('/stop');
  });

  it('throws ProvisioningError when the stop endpoint returns 500', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(makeErrorResponse(500, 'Internal Server Error'));
    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    await expect(client.stopVm()).rejects.toBeInstanceOf(ProvisioningError);
    await expect(client.stopVm()).rejects.toMatchObject({ statusCode: 500 });
  });
});

// ---------------------------------------------------------------------------
// getEndpoint
// ---------------------------------------------------------------------------

describe('ProvisioningClient.getEndpoint', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('returns VmEndpoint when all required fields are present', async () => {
    const fetchMock = vi.fn().mockResolvedValue(makeOkResponse(VALID_ENDPOINT));
    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    const result = await client.getEndpoint();

    expect(result).toEqual(VALID_ENDPOINT);
  });

  it('returns null when the response is missing required fields', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(makeOkResponse({ endpoint: 'https://x.example.com' }));
    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    const result = await client.getEndpoint();

    expect(result).toBeNull();
  });

  it('returns null on a 404 response', async () => {
    const fetchMock = vi.fn().mockResolvedValue(makeErrorResponse(404));
    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    const result = await client.getEndpoint();

    expect(result).toBeNull();
  });

  it('throws ProvisioningError on non-404 HTTP errors', async () => {
    const fetchMock = vi.fn().mockResolvedValue(makeErrorResponse(503));
    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    await expect(client.getEndpoint()).rejects.toBeInstanceOf(ProvisioningError);
    await expect(client.getEndpoint()).rejects.toMatchObject({ statusCode: 503 });
  });
});

// ---------------------------------------------------------------------------
// sendHeartbeat
// ---------------------------------------------------------------------------

describe('ProvisioningClient.sendHeartbeat', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('resolves without throwing when the heartbeat endpoint returns 200', async () => {
    const fetchMock = vi.fn().mockResolvedValue(makeOkResponse({}));
    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    await expect(client.sendHeartbeat()).resolves.toBeUndefined();
  });

  it('does not throw when fetch rejects with a network error', async () => {
    const fetchMock = vi.fn().mockRejectedValue(new Error('Network failure'));
    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    await expect(client.sendHeartbeat()).resolves.toBeUndefined();
  });

  it('does not throw when the server returns a non-2xx status', async () => {
    const fetchMock = vi.fn().mockResolvedValue(makeErrorResponse(503, 'Unavailable'));
    vi.stubGlobal('fetch', fetchMock);

    const client = new ProvisioningClient('api-key-test', 'proj-abc');
    await expect(client.sendHeartbeat()).resolves.toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// ProvisioningError
// ---------------------------------------------------------------------------

describe('ProvisioningError', () => {
  it('exposes statusCode and message', () => {
    const err = new ProvisioningError(422, 'Unprocessable entity');
    expect(err.statusCode).toBe(422);
    expect(err.message).toBe('Unprocessable entity');
    expect(err).toBeInstanceOf(Error);
    expect(err.name).toBe('ProvisioningError');
  });

  it('includes statusCode in instanceof checks', () => {
    const err = new ProvisioningError(408, 'Timeout');
    expect(err instanceof ProvisioningError).toBe(true);
    expect(err.statusCode).toBe(408);
  });
});

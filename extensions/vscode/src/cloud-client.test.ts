// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { CloudTimeoutError, startServer, refreshTokenIfNeeded, handleUnauthorized } from './cloud-client';

// ---------------------------------------------------------------------------
// Mock global fetch — reset per test to prevent response leakage
// ---------------------------------------------------------------------------

const mockFetch = vi.fn<Parameters<typeof fetch>, ReturnType<typeof fetch>>();
vi.stubGlobal('fetch', mockFetch);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeJsonResponse(body: unknown, status = 200): Response {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: () => Promise.resolve(body),
  } as unknown as Response;
}

// ---------------------------------------------------------------------------
// CloudTimeoutError
// ---------------------------------------------------------------------------

describe('CloudTimeoutError', () => {
  it('is an instance of Error', () => {
    const err = new CloudTimeoutError('timed out');
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(CloudTimeoutError);
  });

  it('carries the provided message', () => {
    const err = new CloudTimeoutError('server did not start');
    expect(err.message).toBe('server did not start');
  });
});

// ---------------------------------------------------------------------------
// startServer
// ---------------------------------------------------------------------------

describe('startServer', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    mockFetch.mockReset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('resolves when the endpoint returns ACTIVE on the first poll', async () => {
    mockFetch.mockResolvedValue(makeJsonResponse({ status: 'ACTIVE' }));

    await expect(
      startServer('https://proj-550e8400.mahalaxmi.ai:17421', 'mhx_cloud_solo_abcdef'),
    ).resolves.toBeUndefined();
  });

  it('continues polling until ACTIVE status is received', async () => {
    let callCount = 0;
    mockFetch.mockImplementation(() => {
      callCount++;
      return Promise.resolve(
        makeJsonResponse({ status: callCount < 3 ? 'PROVISIONING' : 'ACTIVE' }),
      );
    });

    const promise = startServer('https://proj-550e8400.mahalaxmi.ai:17421', 'mhx_cloud_solo_abcdef');
    await vi.advanceTimersByTimeAsync(20_000);

    await expect(promise).resolves.toBeUndefined();
    expect(callCount).toBeGreaterThanOrEqual(3);
  });

  it('includes the API key in the Authorization header', async () => {
    mockFetch.mockResolvedValue(makeJsonResponse({ status: 'ACTIVE' }));

    await startServer('https://proj-550e8400.mahalaxmi.ai:17421', 'mhx_cloud_solo_abcdef');

    expect(mockFetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: 'Bearer mhx_cloud_solo_abcdef',
        }),
      }),
    );
  });

  it('throws CloudTimeoutError after 90 seconds without an ACTIVE response', async () => {
    mockFetch.mockResolvedValue(makeJsonResponse({ status: 'PROVISIONING' }));

    const promise = startServer('https://proj-550e8400.mahalaxmi.ai:17421', 'mhx_cloud_solo_abcdef');
    // Suppress unhandled rejection — the promise will be checked below after timers advance.
    promise.catch(() => {});

    await vi.advanceTimersByTimeAsync(90_001);

    await expect(promise).rejects.toBeInstanceOf(CloudTimeoutError);
  });
});

// ---------------------------------------------------------------------------
// refreshTokenIfNeeded
// ---------------------------------------------------------------------------

describe('refreshTokenIfNeeded', () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  it('returns null when more than 10 minutes remain before expiry', async () => {
    const expiresAt = Date.now() + 15 * 60 * 1000; // 15 minutes from now

    const result = await refreshTokenIfNeeded(
      'https://proj-550e8400.mahalaxmi.ai:17421',
      'mhx_cloud_solo_abcdef',
      expiresAt,
    );

    expect(result).toBeNull();
    expect(mockFetch).not.toHaveBeenCalled();
  });

  it('returns a new token string when 10 minutes or less remain before expiry', async () => {
    mockFetch.mockResolvedValue(
      makeJsonResponse({ token: 'refreshed-token-xyz', expires_at: Date.now() + 3_600_000 }),
    );

    const expiresAt = Date.now() + 5 * 60 * 1000; // 5 minutes from now

    const result = await refreshTokenIfNeeded(
      'https://proj-550e8400.mahalaxmi.ai:17421',
      'mhx_cloud_solo_abcdef',
      expiresAt,
    );

    expect(result).toBe('refreshed-token-xyz');
  });

  it('triggers a refresh when exactly 10 minutes remain before expiry', async () => {
    mockFetch.mockResolvedValue(
      makeJsonResponse({ token: 'boundary-token', expires_at: Date.now() + 3_600_000 }),
    );

    const expiresAt = Date.now() + 10 * 60 * 1000; // exactly 10 minutes from now

    const result = await refreshTokenIfNeeded(
      'https://proj-550e8400.mahalaxmi.ai:17421',
      'mhx_cloud_solo_abcdef',
      expiresAt,
    );

    expect(result).toBe('boundary-token');
  });
});

// ---------------------------------------------------------------------------
// handleUnauthorized
// ---------------------------------------------------------------------------

describe('handleUnauthorized', () => {
  it('returns the response when fn succeeds on the first call', async () => {
    const fn = vi.fn().mockResolvedValue(makeJsonResponse({ ok: true }, 200));

    const result = await handleUnauthorized(fn);

    expect(result.status).toBe(200);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it('retries once on 401 and returns the successful retry response', async () => {
    const fn = vi.fn()
      .mockResolvedValueOnce(makeJsonResponse({}, 401))
      .mockResolvedValueOnce(makeJsonResponse({ ok: true }, 200));

    const result = await handleUnauthorized(fn);

    expect(result.status).toBe(200);
    expect(fn).toHaveBeenCalledTimes(2);
  });

  it('throws after the second consecutive 401 response', async () => {
    const fn = vi.fn().mockResolvedValue(makeJsonResponse({}, 401));

    await expect(handleUnauthorized(fn)).rejects.toThrow();
    expect(fn).toHaveBeenCalledTimes(2);
  });
});

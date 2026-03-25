// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// ---------------------------------------------------------------------------
// Capture the vscode window-state listener so tests can fire focus/blur events
// ---------------------------------------------------------------------------

const vscodeWindowState = vi.hoisted(() => ({
  callback: null as ((state: { focused: boolean }) => void) | null,
}));

vi.mock('vscode', () => ({
  window: {
    onDidChangeWindowState: vi.fn((cb: (state: { focused: boolean }) => void) => {
      vscodeWindowState.callback = cb;
      return { dispose: vi.fn() };
    }),
  },
}));

import { start, stop } from './cloud-heartbeat';

// ---------------------------------------------------------------------------
// Mock global fetch — reset per test to prevent state leakage
// ---------------------------------------------------------------------------

const mockFetch = vi.fn<Parameters<typeof fetch>, ReturnType<typeof fetch>>();
vi.stubGlobal('fetch', mockFetch);

// ---------------------------------------------------------------------------
// Test setup
// ---------------------------------------------------------------------------

const TEST_ENDPOINT = 'https://proj-550e8400.mahalaxmi.ai:17421';
const TEST_API_KEY = 'mhx_cloud_solo_abcdef';

beforeEach(() => {
  vi.useFakeTimers();
  mockFetch.mockReset();
  vscodeWindowState.callback = null;
  mockFetch.mockResolvedValue({ ok: true, status: 200 } as Response);
});

afterEach(() => {
  stop();
  vi.useRealTimers();
});

// ---------------------------------------------------------------------------
// start
// ---------------------------------------------------------------------------

describe('start', () => {
  it('fires a POST request after 120 seconds', async () => {
    start(TEST_ENDPOINT, TEST_API_KEY);

    await vi.advanceTimersByTimeAsync(120_000);

    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('/v1/heartbeat'),
      expect.objectContaining({ method: 'POST' }),
    );
  });

  it('fires a POST request on each 120-second interval', async () => {
    start(TEST_ENDPOINT, TEST_API_KEY);

    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockFetch).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockFetch).toHaveBeenCalledTimes(2);

    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockFetch).toHaveBeenCalledTimes(3);
  });

  it('sends the API key as a Bearer token', async () => {
    start(TEST_ENDPOINT, TEST_API_KEY);
    await vi.advanceTimersByTimeAsync(120_000);

    expect(mockFetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: `Bearer ${TEST_API_KEY}`,
        }),
      }),
    );
  });

  it('registers a vscode window state listener', () => {
    start(TEST_ENDPOINT, TEST_API_KEY);
    expect(vscodeWindowState.callback).not.toBeNull();
  });
});

// ---------------------------------------------------------------------------
// stop
// ---------------------------------------------------------------------------

describe('stop', () => {
  it('clears the interval so no more POSTs are sent after stop()', async () => {
    start(TEST_ENDPOINT, TEST_API_KEY);
    stop();

    await vi.advanceTimersByTimeAsync(360_000);

    expect(mockFetch).not.toHaveBeenCalled();
  });

  it('does not throw when called before start()', () => {
    expect(() => stop()).not.toThrow();
  });
});

// ---------------------------------------------------------------------------
// Error handling — heartbeat errors must be swallowed
// ---------------------------------------------------------------------------

describe('error handling', () => {
  it('swallows a network error without propagating it', async () => {
    mockFetch.mockRejectedValue(new Error('network error'));

    start(TEST_ENDPOINT, TEST_API_KEY);

    // Should complete without throwing even when fetch rejects
    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it('swallows a non-2xx response without propagating it', async () => {
    mockFetch.mockResolvedValue({ ok: false, status: 503 } as Response);

    start(TEST_ENDPOINT, TEST_API_KEY);

    // Should complete without throwing even when fetch returns non-2xx
    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it('continues sending heartbeats after a failed attempt', async () => {
    let callCount = 0;
    mockFetch.mockImplementation(() => {
      callCount++;
      if (callCount === 1) {
        return Promise.reject(new Error('transient error'));
      }
      return Promise.resolve({ ok: true, status: 200 } as Response);
    });

    start(TEST_ENDPOINT, TEST_API_KEY);

    await vi.advanceTimersByTimeAsync(120_000);
    expect(callCount).toBe(1);

    await vi.advanceTimersByTimeAsync(120_000);
    expect(callCount).toBe(2);
  });
});

// ---------------------------------------------------------------------------
// Window focus / blur
// ---------------------------------------------------------------------------

describe('window focus/blur', () => {
  it('pauses heartbeat when the VS Code window loses focus', async () => {
    start(TEST_ENDPOINT, TEST_API_KEY);
    expect(vscodeWindowState.callback).not.toBeNull();

    vscodeWindowState.callback!({ focused: false });

    await vi.advanceTimersByTimeAsync(360_000);

    expect(mockFetch).not.toHaveBeenCalled();
  });

  it('resumes heartbeat when the VS Code window regains focus', async () => {
    start(TEST_ENDPOINT, TEST_API_KEY);

    vscodeWindowState.callback!({ focused: false });
    await vi.advanceTimersByTimeAsync(120_000);
    expect(mockFetch).not.toHaveBeenCalled();

    vscodeWindowState.callback!({ focused: true });
    await vi.advanceTimersByTimeAsync(120_000);

    expect(mockFetch).toHaveBeenCalled();
  });

  it('does not pause heartbeat when the window remains focused', async () => {
    start(TEST_ENDPOINT, TEST_API_KEY);

    vscodeWindowState.callback!({ focused: true });

    await vi.advanceTimersByTimeAsync(120_000);

    expect(mockFetch).toHaveBeenCalledTimes(1);
  });
});

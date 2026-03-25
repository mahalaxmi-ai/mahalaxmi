// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
const REQUEST_TIMEOUT_MS = 10_000;
const POLL_INTERVAL_MS = 5_000;
const SERVER_START_TIMEOUT_MS = 120_000;

/** Thrown when an API request returns HTTP 401 and token refresh is required. */
export class AuthExpiredError extends Error {
  readonly code = 'AUTH_EXPIRED' as const;

  constructor(message: string) {
    super(message);
    this.name = 'AuthExpiredError';
  }
}

/** Thrown when a 401 persists after one retry, indicating permanent auth failure. */
export class CloudAuthError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CloudAuthError';
  }
}

/** Thrown when a polling operation exceeds its configured timeout. */
export class CloudTimeoutError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CloudTimeoutError';
  }
}

/** A cloud server endpoint returned after a successful start. */
export interface ServerEndpoint {
  endpoint: string;
  sessionToken: string;
  /** ISO-8601 expiry timestamp. */
  expiresAt: string;
}

/** Raw shape of the /endpoint JSON response. */
interface EndpointPayload {
  endpoint?: string;
  sessionToken?: string;
  expiresAt?: string;
}

/** Raw shape of the /auth/refresh JSON response. */
interface RefreshPayload {
  token?: string;
}

/**
 * General-purpose cloud API client with authentication handling,
 * token lifecycle management, and server provisioning control.
 *
 * All individual HTTP requests time out after 10 seconds.
 */
export class CloudClient {
  private consecutiveUnauthorized = 0;

  constructor(
    private readonly baseUrl: string,
    private readonly apiKey: string,
    private readonly projectId: string,
  ) {}

  private async fetchWithTimeout(
    url: string,
    options: RequestInit = {},
  ): Promise<Response> {
    const controller = new AbortController();
    const timerId = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
        headers: {
          Authorization: `Bearer ${this.apiKey}`,
          'Content-Type': 'application/json',
          ...((options.headers as Record<string, string>) ?? {}),
        },
      });

      if (response.status === 401) {
        const text = await response.text().catch(() => '');
        throw new AuthExpiredError(
          text.trim() !== '' ? text.trim() : `Unauthorized: ${url}`,
        );
      }

      if (!response.ok) {
        const text = await response.text().catch(() => '');
        throw new Error(
          text.trim() !== ''
            ? text.trim()
            : `HTTP ${response.status} from ${url}`,
        );
      }

      return response;
    } catch (err) {
      if (err instanceof AuthExpiredError) {
        throw err;
      }
      if (err instanceof Error && err.name === 'AbortError') {
        throw new CloudTimeoutError(`Request timed out: ${url}`);
      }
      if (err instanceof Error) {
        throw new Error(`Network error accessing ${url}: ${err.message}`);
      }
      throw new Error(`Network error accessing ${url}: ${String(err)}`);
    } finally {
      clearTimeout(timerId);
    }
  }

  /**
   * Wrap an operation so that a single HTTP 401 triggers one automatic retry.
   * A second consecutive 401 throws {@link CloudAuthError}.
   *
   * @param operation Async function to execute (and possibly retry).
   */
  async handleUnauthorized<T>(operation: () => Promise<T>): Promise<T> {
    try {
      const result = await operation();
      this.consecutiveUnauthorized = 0;
      return result;
    } catch (err) {
      if (err instanceof AuthExpiredError) {
        if (this.consecutiveUnauthorized >= 1) {
          this.consecutiveUnauthorized = 0;
          throw new CloudAuthError(
            'Authentication failed after retry: ' + err.message,
          );
        }
        this.consecutiveUnauthorized++;
        try {
          const result = await operation();
          this.consecutiveUnauthorized = 0;
          return result;
        } catch (retryErr) {
          if (retryErr instanceof AuthExpiredError) {
            this.consecutiveUnauthorized = 0;
            throw new CloudAuthError(
              'Authentication failed after retry: ' + retryErr.message,
            );
          }
          throw retryErr;
        }
      }
      throw err;
    }
  }

  /**
   * Refresh the API token if it expires within 10 minutes.
   *
   * @param expiresAt ISO-8601 expiry timestamp of the current token.
   * @returns The new token string when a refresh was performed, or `null`
   *   when more than 10 minutes remain.
   * @throws {AuthExpiredError} When the refresh call returns 401.
   * @throws {Error} On other HTTP or network failures.
   */
  async refreshTokenIfNeeded(expiresAt: string): Promise<string | null> {
    const expiryMs = new Date(expiresAt).getTime();
    const nowMs = Date.now();
    const tenMinMs = 10 * 60 * 1_000;

    if (expiryMs - nowMs > tenMinMs) {
      return null;
    }

    const response = await this.fetchWithTimeout(`${this.baseUrl}/auth/refresh`, {
      method: 'POST',
      body: JSON.stringify({ projectId: this.projectId }),
    });

    const data = (await response.json()) as RefreshPayload;

    if (typeof data.token !== 'string' || data.token === '') {
      throw new Error('Token refresh response is missing token field');
    }

    return data.token;
  }

  /**
   * Start the cloud server and wait until its endpoint is available.
   *
   * POSTs to the start endpoint, then polls GET /endpoint every 5 seconds
   * for up to 120 seconds.
   *
   * @param onProgress Called on each poll tick with a status message.
   * @returns The server endpoint once it becomes active.
   * @throws {Error} With message `'Server start timed out after 120s'` when no
   *   endpoint is returned within 120 seconds.
   * @throws {CloudTimeoutError} When an individual HTTP request times out.
   * @throws {AuthExpiredError} On 401 responses.
   */
  async startServer(onProgress: (msg: string) => void): Promise<ServerEndpoint> {
    const startTime = Date.now();

    await this.fetchWithTimeout(
      `${this.baseUrl}/provisioning/projects/${this.projectId}/start`,
      { method: 'POST', body: '{}' },
    );

    for (;;) {
      const elapsedMs = Date.now() - startTime;

      if (elapsedMs >= SERVER_START_TIMEOUT_MS) {
        throw new Error('Server start timed out after 120s');
      }

      const elapsedS = Math.round(elapsedMs / 1000);
      onProgress(`Starting VM... (${elapsedS}s elapsed)`);

      const response = await this.fetchWithTimeout(
        `${this.baseUrl}/provisioning/projects/${this.projectId}/endpoint`,
      );

      const data = (await response.json()) as EndpointPayload;

      if (
        typeof data.endpoint === 'string' &&
        data.endpoint !== '' &&
        typeof data.sessionToken === 'string' &&
        data.sessionToken !== '' &&
        typeof data.expiresAt === 'string' &&
        data.expiresAt !== ''
      ) {
        return {
          endpoint: data.endpoint,
          sessionToken: data.sessionToken,
          expiresAt: data.expiresAt,
        };
      }

      await new Promise<void>((resolve) => setTimeout(resolve, POLL_INTERVAL_MS));
    }
  }

  /**
   * Stop the cloud server for this project.
   *
   * @throws {AuthExpiredError} On 401 responses.
   * @throws {Error} On other HTTP or network failures.
   */
  async stopServer(): Promise<void> {
    await this.fetchWithTimeout(
      `${this.baseUrl}/provisioning/projects/${this.projectId}/stop`,
      { method: 'POST', body: '{}' },
    );
  }

  /**
   * Send a heartbeat ping. Fire-and-forget: all errors are caught and logged
   * to stderr at WARN level; this method never throws.
   */
  async sendHeartbeat(): Promise<void> {
    try {
      await this.fetchWithTimeout(
        `${this.baseUrl}/provisioning/projects/${this.projectId}/vscode-heartbeat`,
        { method: 'POST', body: '{}' },
      );
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      process.stderr.write(`[mahalaxmi] WARN heartbeat failed: ${msg}\n`);
    }
  }
}

// ---------------------------------------------------------------------------
// Module-level functional API
// ---------------------------------------------------------------------------

const FUNCTIONAL_POLL_INTERVAL_MS = 5_000;
const FUNCTIONAL_START_TIMEOUT_MS = 90_000;
const FUNCTIONAL_REFRESH_THRESHOLD_MS = 10 * 60 * 1_000;

interface StatusPayload {
  status?: string;
}

interface TokenPayload {
  token?: string;
}

/**
 * Poll `{endpoint}` for `{ status: 'ACTIVE' }`.
 * Throws `CloudTimeoutError` after 90 seconds.
 */
export async function startServer(endpoint: string, apiKey: string): Promise<void> {
  const deadline = Date.now() + FUNCTIONAL_START_TIMEOUT_MS;

  for (;;) {
    const response = await fetch(endpoint, {
      headers: { Authorization: `Bearer ${apiKey}` },
    });
    const data = (await response.json()) as StatusPayload;
    if (data.status === 'ACTIVE') {
      return;
    }
    if (Date.now() >= deadline) {
      throw new CloudTimeoutError('Server start timed out');
    }
    await new Promise<void>((resolve) => setTimeout(resolve, FUNCTIONAL_POLL_INTERVAL_MS));
  }
}

/**
 * Refresh the API token if it expires within 10 minutes.
 *
 * @param expiresAt Unix timestamp in milliseconds.
 * @returns New token string when refresh was performed, or `null` when not needed.
 */
export async function refreshTokenIfNeeded(
  endpoint: string,
  apiKey: string,
  expiresAt: number,
): Promise<string | null> {
  if (expiresAt - Date.now() > FUNCTIONAL_REFRESH_THRESHOLD_MS) {
    return null;
  }

  const response = await fetch(`${endpoint}/auth/refresh`, {
    method: 'POST',
    headers: { Authorization: `Bearer ${apiKey}`, 'Content-Type': 'application/json' },
  });
  const data = (await response.json()) as TokenPayload;

  if (typeof data.token !== 'string' || data.token === '') {
    throw new Error('Token refresh response is missing token field');
  }

  return data.token;
}

/**
 * Execute `fn`, retrying once on HTTP 401. Throws after two consecutive 401s.
 */
export async function handleUnauthorized(fn: () => Promise<Response>): Promise<Response> {
  const first = await fn();
  if (first.status !== 401) {
    return first;
  }
  const second = await fn();
  if (second.status === 401) {
    throw new Error('Authentication failed after retry');
  }
  return second;
}

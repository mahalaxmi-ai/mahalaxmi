// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
const BASE_URL = 'https://tokenapi.thrivetechservice.com';
const REQUEST_TIMEOUT_MS = 30_000;
const POLL_INTERVAL_MS = 5_000;
const START_TIMEOUT_MS = 120_000;

/** Thrown when the provisioning API returns a non-2xx response. */
export class ProvisioningError extends Error {
  constructor(
    public readonly statusCode: number,
    message: string,
  ) {
    super(message);
    this.name = 'ProvisioningError';
  }
}

/** A running VM endpoint returned by the provisioning API. */
export interface VmEndpoint {
  endpoint: string;
  sessionToken: string;
  /** ISO-8601 expiry timestamp. */
  expiresAt: string;
}

/** Raw shape of a successful /endpoint JSON response. */
interface EndpointPayload {
  endpoint?: string;
  sessionToken?: string;
  expiresAt?: string;
}

/** Raw shape of a successful token-refresh response. */
interface RefreshPayload {
  token?: string;
}

/**
 * HTTP client for the ThriveTech provisioning API at
 * https://tokenapi.thrivetechservice.com.
 */
export class ProvisioningClient {
  constructor(
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

      if (!response.ok) {
        const text = await response.text().catch(() => '');
        throw new ProvisioningError(
          response.status,
          text.trim() !== '' ? text.trim() : `HTTP ${response.status}`,
        );
      }

      return response;
    } catch (err) {
      if (err instanceof ProvisioningError) {
        throw err;
      }
      if (err instanceof Error && err.name === 'AbortError') {
        throw new ProvisioningError(408, `Request timed out: ${url}`);
      }
      const msg = err instanceof Error ? err.message : String(err);
      throw new ProvisioningError(0, `Network error accessing ${url}: ${msg}`);
    } finally {
      clearTimeout(timerId);
    }
  }

  /**
   * Start the cloud VM and wait until it is active.
   *
   * POSTs to the start endpoint, then polls GET /endpoint every 5 seconds
   * until a valid endpoint is returned or 120 seconds elapse.
   *
   * @param onProgress Called on each poll tick with a human-readable status
   *   message including elapsed time.
   * @returns The VM endpoint once the server is active.
   * @throws {ProvisioningError} With statusCode 408 if the VM does not become
   *   active within 120 seconds.
   */
  async startVm(onProgress: (msg: string) => void): Promise<VmEndpoint> {
    const startTime = Date.now();

    await this.fetchWithTimeout(
      `${BASE_URL}/provisioning/projects/${this.projectId}/start`,
      { method: 'POST', body: '{}' },
    );

    for (;;) {
      const elapsedMs = Date.now() - startTime;

      if (elapsedMs >= START_TIMEOUT_MS) {
        throw new ProvisioningError(408, 'VM start timed out after 120s');
      }

      const elapsedS = Math.round(elapsedMs / 1000);
      onProgress(`Starting VM... (${elapsedS}s elapsed)`);

      const endpoint = await this.getEndpoint();
      if (endpoint !== null) {
        return endpoint;
      }

      await new Promise<void>((resolve) => setTimeout(resolve, POLL_INTERVAL_MS));
    }
  }

  /**
   * Stop the cloud VM for this project.
   *
   * @throws {ProvisioningError} On HTTP errors.
   */
  async stopVm(): Promise<void> {
    await this.fetchWithTimeout(
      `${BASE_URL}/provisioning/projects/${this.projectId}/stop`,
      { method: 'POST', body: '{}' },
    );
  }

  /**
   * Fetch the current VM endpoint for this project.
   *
   * @returns A {@link VmEndpoint} when the VM is active and all required
   *   fields are present, or `null` when the VM is not yet ready.
   * @throws {ProvisioningError} On HTTP errors other than expected not-ready states.
   */
  async getEndpoint(): Promise<VmEndpoint | null> {
    let response: Response;
    try {
      response = await this.fetchWithTimeout(
        `${BASE_URL}/provisioning/projects/${this.projectId}/endpoint`,
      );
    } catch (err) {
      if (err instanceof ProvisioningError && err.statusCode === 404) {
        return null;
      }
      throw err;
    }

    const data = (await response.json()) as EndpointPayload;

    if (
      typeof data.endpoint !== 'string' ||
      data.endpoint === '' ||
      typeof data.sessionToken !== 'string' ||
      data.sessionToken === '' ||
      typeof data.expiresAt !== 'string' ||
      data.expiresAt === ''
    ) {
      return null;
    }

    return {
      endpoint: data.endpoint,
      sessionToken: data.sessionToken,
      expiresAt: data.expiresAt,
    };
  }

  /**
   * Send a VS Code heartbeat for this project.
   *
   * Fire-and-forget: all errors are caught and logged to stderr; this method
   * never throws.
   */
  async sendHeartbeat(): Promise<void> {
    try {
      await this.fetchWithTimeout(
        `${BASE_URL}/provisioning/projects/${this.projectId}/vscode-heartbeat`,
        { method: 'POST', body: '{}' },
      );
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      process.stderr.write(`[mahalaxmi] WARN heartbeat failed: ${msg}\n`);
    }
  }

  /**
   * Refresh the API token if it expires within 10 minutes.
   *
   * @param expiresAt ISO-8601 expiry timestamp of the current token.
   * @returns The new token string when a refresh was performed, or `null`
   *   when more than 10 minutes remain and no refresh is needed.
   * @throws {ProvisioningError} On HTTP errors during the refresh call.
   */
  async refreshTokenIfNeeded(expiresAt: string): Promise<string | null> {
    const expiryMs = new Date(expiresAt).getTime();
    const nowMs = Date.now();
    const tenMinMs = 10 * 60 * 1_000;

    if (expiryMs - nowMs > tenMinMs) {
      return null;
    }

    const response = await this.fetchWithTimeout(
      `${BASE_URL}/auth/refresh`,
      {
        method: 'POST',
        body: JSON.stringify({ projectId: this.projectId }),
      },
    );

    const data = (await response.json()) as RefreshPayload;

    if (typeof data.token !== 'string' || data.token === '') {
      throw new ProvisioningError(500, 'Token refresh response is missing token field');
    }

    return data.token;
  }
}

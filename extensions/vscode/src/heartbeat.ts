// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
/**
 * Minimal interface for a cloud provisioning client that supports sending
 * heartbeat pings. Implemented by the full CloudProvisioningClient in the
 * cloud session manager; defined here to break circular dependencies.
 */
export interface CloudProvisioningClient {
  /**
   * Sends a heartbeat ping for the given project.
   *
   * @param projectId - UUID of the cloud project being pinged.
   * @param apiKey    - API key authorizing the heartbeat.
   * @returns A promise that resolves on success and rejects on network failure.
   */
  sendHeartbeat(projectId: string, apiKey: string): Promise<void>;
}

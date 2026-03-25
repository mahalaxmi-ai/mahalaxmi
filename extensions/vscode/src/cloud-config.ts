// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import * as fs from 'fs/promises';
import * as os from 'os';
import * as path from 'path';

export interface CloudConfig {
  endpoint: string;
  api_key: string;
  project_id: string;
}

function configDir(): string {
  return path.join(os.homedir(), '.mahalaxmi');
}

function configPath(): string {
  return path.join(configDir(), 'cloud.toml');
}

function parseToml(content: string): Record<string, string> {
  const result: Record<string, string> = {};
  for (const rawLine of content.split('\n')) {
    const line = rawLine.trim();
    if (line === '' || line.startsWith('#')) {
      continue;
    }
    const match = /^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"(.*)"$/.exec(line);
    if (match) {
      result[match[1]] = match[2];
    }
  }
  return result;
}

function serializeToml(config: CloudConfig): string {
  return (
    `endpoint = "${config.endpoint}"\n` +
    `api_key = "${config.api_key}"\n` +
    `project_id = "${config.project_id}"\n`
  );
}

export async function readCloudConfig(): Promise<CloudConfig | null> {
  let content: string;
  try {
    content = await fs.readFile(configPath(), 'utf8');
  } catch {
    return null;
  }

  let parsed: Record<string, string>;
  try {
    parsed = parseToml(content);
  } catch {
    return null;
  }

  const endpoint = parsed['endpoint'] ?? '';
  const api_key = parsed['api_key'] ?? '';
  const project_id = parsed['project_id'] ?? '';

  if (!endpoint || !api_key || !project_id) {
    return null;
  }

  if (!endpoint.startsWith('https://')) {
    process.stderr.write(
      `[mahalaxmi] cloud.toml endpoint must start with https:// — ignoring config\n`,
    );
    return null;
  }

  return { endpoint, api_key, project_id };
}

export async function writeCloudConfig(config: CloudConfig): Promise<void> {
  if (!config.endpoint.startsWith('https://')) {
    throw new Error(
      `Cloud endpoint must start with https:// — refusing to write insecure config`,
    );
  }

  const dir = configDir();
  await fs.mkdir(dir, { recursive: true, mode: 0o700 });

  const toml = serializeToml(config);
  const tmpPath = configPath() + '.tmp';

  await fs.writeFile(tmpPath, toml, { mode: 0o600, encoding: 'utf8' });
  await fs.rename(tmpPath, configPath());
}

export async function clearCloudConfig(): Promise<void> {
  try {
    await fs.unlink(configPath());
  } catch (err: unknown) {
    if ((err as NodeJS.ErrnoException).code !== 'ENOENT') {
      throw err;
    }
  }
}

export function isCloudConfigured(config: CloudConfig | null | undefined): boolean {
  if (!config) {
    return false;
  }
  return (
    config.endpoint.length > 0 &&
    config.api_key.length > 0 &&
    config.project_id.length > 0
  );
}

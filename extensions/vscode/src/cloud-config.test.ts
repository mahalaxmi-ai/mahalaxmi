// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as fs from 'fs/promises';
import * as path from 'path';
import * as os from 'os';

// ---------------------------------------------------------------------------
// Mock os.homedir so cloud-config.ts resolves ~/.mahalaxmi into a tmp dir.
// vi.hoisted runs before imports, so we cannot call os.homedir() there —
// initialize with empty string and populate in beforeEach.
// ---------------------------------------------------------------------------
const hoistedHome = vi.hoisted(() => ({ value: '' }));

vi.mock('os', async (importOriginal) => {
  const actual = await importOriginal<typeof import('os')>();
  return { ...actual, homedir: () => hoistedHome.value };
});

import { readCloudConfig, writeCloudConfig, isCloudConfigured, clearCloudConfig } from './cloud-config';

// ---------------------------------------------------------------------------
// Per-test tmp directory wired into the homedir mock
// ---------------------------------------------------------------------------

let tmpHome: string;

beforeEach(async () => {
  tmpHome = await fs.mkdtemp(path.join(os.tmpdir(), 'mahalaxmi-cloud-config-test-'));
  hoistedHome.value = tmpHome;
});

afterEach(async () => {
  await fs.rm(tmpHome, { recursive: true, force: true });
});

// ---------------------------------------------------------------------------
// readCloudConfig
// ---------------------------------------------------------------------------

describe('readCloudConfig', () => {
  it('returns null when ~/.mahalaxmi/cloud.toml does not exist', async () => {
    const result = await readCloudConfig();
    expect(result).toBeNull();
  });

  it('returns null when endpoint does not start with https://', async () => {
    const dir = path.join(tmpHome, '.mahalaxmi');
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(
      path.join(dir, 'cloud.toml'),
      `endpoint = "http://proj-550e8400.mahalaxmi.ai:17421"\napi_key = "mhx_cloud_solo_abcdef"\nproject_id = "550e8400-e29b-41d4-a716-446655440000"\n`,
      'utf8',
    );

    const result = await readCloudConfig();
    expect(result).toBeNull();
  });

  it('parses a valid cloud.toml and returns the correct fields', async () => {
    const dir = path.join(tmpHome, '.mahalaxmi');
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(
      path.join(dir, 'cloud.toml'),
      `endpoint = "https://proj-550e8400.mahalaxmi.ai:17421"\napi_key = "mhx_cloud_solo_abcdef"\nproject_id = "550e8400-e29b-41d4-a716-446655440000"\n`,
      'utf8',
    );

    const result = await readCloudConfig();
    expect(result).not.toBeNull();
    expect(result!.endpoint).toBe('https://proj-550e8400.mahalaxmi.ai:17421');
    expect(result!.api_key).toBe('mhx_cloud_solo_abcdef');
    expect(result!.project_id).toBe('550e8400-e29b-41d4-a716-446655440000');
  });

  it('returns null when a required field is missing', async () => {
    const dir = path.join(tmpHome, '.mahalaxmi');
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(
      path.join(dir, 'cloud.toml'),
      `endpoint = "https://proj-550e8400.mahalaxmi.ai:17421"\napi_key = "mhx_cloud_solo_abcdef"\n`,
      'utf8',
    );

    const result = await readCloudConfig();
    expect(result).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// writeCloudConfig
// ---------------------------------------------------------------------------

describe('writeCloudConfig', () => {
  it('creates ~/.mahalaxmi/ directory with mode 0o700 and file with mode 0o600', async () => {
    const config = {
      endpoint: 'https://proj-550e8400.mahalaxmi.ai:17421',
      api_key: 'mhx_cloud_solo_abcdef',
      project_id: '550e8400-e29b-41d4-a716-446655440000',
    };

    await writeCloudConfig(config);

    const dir = path.join(tmpHome, '.mahalaxmi');
    const dirStat = await fs.stat(dir);
    expect(dirStat.isDirectory()).toBe(true);

    if (process.platform !== 'win32') {
      expect(dirStat.mode & 0o777).toBe(0o700);
    }

    const filePath = path.join(dir, 'cloud.toml');
    const fileStat = await fs.stat(filePath);
    if (process.platform !== 'win32') {
      expect(fileStat.mode & 0o777).toBe(0o600);
    }

    const written = await fs.readFile(filePath, 'utf8');
    expect(written).toContain('endpoint = "https://proj-550e8400.mahalaxmi.ai:17421"');
    expect(written).toContain('api_key = "mhx_cloud_solo_abcdef"');
    expect(written).toContain('project_id = "550e8400-e29b-41d4-a716-446655440000"');
  });

  it('throws when endpoint does not start with https://', async () => {
    const config = {
      endpoint: 'http://proj-550e8400.mahalaxmi.ai:17421',
      api_key: 'mhx_cloud_solo_abcdef',
      project_id: '550e8400-e29b-41d4-a716-446655440000',
    };

    await expect(writeCloudConfig(config)).rejects.toThrow(/https:\/\//);
  });
});

// ---------------------------------------------------------------------------
// Round-trip
// ---------------------------------------------------------------------------

describe('round-trip', () => {
  it('writeCloudConfig followed by readCloudConfig returns an equivalent object', async () => {
    const config = {
      endpoint: 'https://proj-deadbeef.mahalaxmi.ai:17421',
      api_key: 'mhx_cloud_solo_testkey123',
      project_id: 'deadbeef-dead-beef-dead-beef00000001',
    };

    await writeCloudConfig(config);
    const result = await readCloudConfig();

    expect(result).toEqual(config);
  });
});

// ---------------------------------------------------------------------------
// clearCloudConfig
// ---------------------------------------------------------------------------

describe('clearCloudConfig', () => {
  it('removes ~/.mahalaxmi/cloud.toml when the file exists', async () => {
    const config = {
      endpoint: 'https://proj-550e8400.mahalaxmi.ai:17421',
      api_key: 'mhx_cloud_solo_abcdef',
      project_id: '550e8400-e29b-41d4-a716-446655440000',
    };

    await writeCloudConfig(config);
    const filePath = path.join(tmpHome, '.mahalaxmi', 'cloud.toml');

    const statBefore = await fs.stat(filePath);
    expect(statBefore.isFile()).toBe(true);

    await clearCloudConfig();

    await expect(fs.stat(filePath)).rejects.toThrow();
  });

  it('does not throw when the file does not exist', async () => {
    await expect(clearCloudConfig()).resolves.toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// isCloudConfigured
// ---------------------------------------------------------------------------

describe('isCloudConfigured', () => {
  it('returns false for null', () => {
    expect(isCloudConfigured(null)).toBe(false);
  });

  it('returns false when endpoint is empty', () => {
    expect(isCloudConfigured({ endpoint: '', api_key: 'key', project_id: 'id' })).toBe(false);
  });

  it('returns false when api_key is empty', () => {
    expect(isCloudConfigured({ endpoint: 'https://x', api_key: '', project_id: 'id' })).toBe(false);
  });

  it('returns false when project_id is empty', () => {
    expect(isCloudConfigured({ endpoint: 'https://x', api_key: 'key', project_id: '' })).toBe(false);
  });

  it('returns true when all fields are non-empty', () => {
    expect(
      isCloudConfigured({ endpoint: 'https://x', api_key: 'key', project_id: 'id' }),
    ).toBe(true);
  });
});

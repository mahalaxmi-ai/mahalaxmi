// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
/**
 * Minimal VS Code API stub for Vite module resolution during tests.
 *
 * Each test file that exercises extension code overrides this entirely with
 * `vi.mock('vscode', () => { ... })`. This stub's sole purpose is to let
 * Vite's import-analysis pass resolve `import * as vscode from 'vscode'` in
 * the extension source files without failing — the mock replaces it at runtime.
 *
 * Properties match the subset of @types/vscode actually used by the extension.
 */
import { vi } from 'vitest';

export const TreeItemCollapsibleState = {
  None: 0,
  Collapsed: 1,
  Expanded: 2,
} as const;

export const StatusBarAlignment = {
  Left: 1,
  Right: 2,
} as const;

export class ThemeIcon {
  constructor(public readonly id: string) {}
}

export class ThemeColor {
  constructor(public readonly id: string) {}
}

export class TreeItem {
  label?: string;
  collapsibleState?: number;
  description?: string;
  iconPath?: ThemeIcon | string;
  resourceUri?: Uri;
  command?: { command: string; title: string; arguments?: unknown[] };

  constructor(
    label?: string,
    collapsibleState: number = TreeItemCollapsibleState.None,
  ) {
    this.label = label;
    this.collapsibleState = collapsibleState;
  }
}

export class EventEmitter<T = void> {
  private readonly _listeners: Array<(e: T) => void> = [];

  readonly event = (listener: (e: T) => void): { dispose: () => void } => {
    this._listeners.push(listener);
    return {
      dispose: () => {
        const idx = this._listeners.indexOf(listener);
        if (idx >= 0) this._listeners.splice(idx, 1);
      },
    };
  };

  fire(data?: T): void {
    for (const l of this._listeners) l(data as T);
  }

  dispose(): void {
    this._listeners.length = 0;
  }
}

export class Uri {
  private constructor(
    public readonly scheme: string,
    public readonly path: string,
  ) {}

  static file(p: string): Uri {
    return new Uri('file', p);
  }

  static from(components: { scheme: string; path: string }): Uri {
    return new Uri(components.scheme, components.path);
  }

  toString(): string {
    return `${this.scheme}:${this.path}`;
  }
}

export const window = {
  createStatusBarItem: vi.fn(() => ({
    text: '',
    tooltip: '',
    backgroundColor: undefined as ThemeColor | undefined,
    command: undefined as string | undefined,
    show: vi.fn(),
    dispose: vi.fn(),
  })),
  showInputBox: vi.fn((): Promise<string | undefined> => Promise.resolve(undefined)),
  showWarningMessage: vi.fn((): Promise<string | undefined> => Promise.resolve(undefined)),
  showInformationMessage: vi.fn((): Promise<string | undefined> => Promise.resolve(undefined)),
  registerTreeDataProvider: vi.fn(() => ({ dispose: vi.fn() })),
};

export const commands = {
  registerCommand: vi.fn((_id: string, _handler: (...args: unknown[]) => unknown) => ({
    dispose: vi.fn(),
  })),
  executeCommand: vi.fn((): Promise<void> => Promise.resolve()),
};

export const workspace = {
  registerTextDocumentContentProvider: vi.fn(() => ({ dispose: vi.fn() })),
};

export const l10n = {
  t: vi.fn((str: string, ...args: unknown[]): string =>
    args.reduce<string>((s, arg, i) => s.replace(`{${i}}`, String(arg)), str),
  ),
};

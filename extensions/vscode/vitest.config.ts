// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: {
    environment: 'node',
    globals: true,
    exclude: ['out/**', 'node_modules/**'],
  },
});

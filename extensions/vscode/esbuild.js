// SPDX-License-Identifier: MIT
// Copyright 2026 ThriveTech Services LLC
const esbuild = require('esbuild');

esbuild.build({
  entryPoints: ['src/extension.ts'],
  bundle: true,
  outfile: 'out/extension.js',
  external: ['vscode'],
  format: 'cjs',
  platform: 'node',
  target: 'node18',
  sourcemap: process.argv.includes('--sourcemap'),
  minify: process.argv.includes('--minify'),
}).catch(() => process.exit(1));

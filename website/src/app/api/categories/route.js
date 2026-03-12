import { NextResponse } from 'next/server';

const CATEGORIES = [
  {
    id: 'cat-terminal',
    name: 'Terminal Orchestration',
    slug: 'terminal-orchestration',
    description: 'Local AI terminal orchestration — runs on your machine',
    icon: 'psychology',
    color: '#00C8C8',
    product_count: 1,
  },
  {
    id: 'cat-cloud',
    name: 'Cloud Orchestration',
    slug: 'cloud-orchestration',
    description: 'Fully managed cloud servers — connect VS Code in one click',
    icon: 'cloud',
    color: '#C8A040',
    product_count: 1,
  },
  {
    id: 'cat-vscode',
    name: 'VS Code Extension',
    slug: 'vscode-extension',
    description: 'Orchestration inside your editor',
    icon: 'code',
    color: '#00C8C8',
    product_count: 1,
  },
];

export async function GET() {
  return NextResponse.json({ success: true, data: { data: { categories: CATEGORIES } } });
}

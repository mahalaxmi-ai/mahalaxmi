import { NextResponse } from 'next/server';

const PAK_MAP = {
  'mahalaxmi-ai-terminal-orchestration': {
    key: process.env.MAHALAXMI_TERMINAL_PAK_KEY,
    image: '/mahalaxmi_logo.png',
    is_featured: true,
  },
  'mahalaxmi-headless-orchestration': {
    key: process.env.MAHALAXMI_CLOUD_PAK_KEY,
    image: '/mahalaxmi_logo.png',
    is_featured: true,
  },
  'mahalaxmi-vscode-extension': {
    key: process.env.MAHALAXMI_VSCODE_PAK_KEY,
    image: '/mahalaxmi_logo.png',
    is_featured: false,
  },
};

const CATEGORY_SLUGS = {
  'cat-terminal':         ['mahalaxmi-ai-terminal-orchestration'],
  'terminal-orchestration': ['mahalaxmi-ai-terminal-orchestration'],
  'cat-cloud':            ['mahalaxmi-headless-orchestration'],
  'cloud-orchestration':  ['mahalaxmi-headless-orchestration'],
  'cat-vscode':           ['mahalaxmi-vscode-extension'],
  'vscode-extension':     ['mahalaxmi-vscode-extension'],
};

const PRODUCT_NAMES = {
  'mahalaxmi-ai-terminal-orchestration': 'Mahalaxmi AI Terminal Orchestration',
  'mahalaxmi-headless-orchestration': 'Mahalaxmi Headless Orchestration',
  'mahalaxmi-vscode-extension': 'Mahalaxmi VS Code Extension',
};

async function fetchPlatformProduct(slug, meta) {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  try {
    const res = await fetch(`${platformUrl}/api/v1/public/product`, {
      headers: { 'X-Channel-API-Key': meta.key },
      next: { revalidate: 30 },
    });
    if (!res.ok) throw new Error('Platform error');
    const data = await res.json();
    return { ...data, slug, image: meta.image, is_featured: meta.is_featured, is_platform_connected: true, data_source: 'platform' };
  } catch {
    return {
      slug,
      image: meta.image,
      is_featured: meta.is_featured,
      pricing_options: [],
      pricing_type: 'unavailable',
      name: PRODUCT_NAMES[slug] || slug,
      is_platform_connected: false,
      data_source: 'placeholder',
      platform_status_message: 'Pricing temporarily unavailable. Contact support@mahalaxmi.ai',
    };
  }
}

export async function GET(request) {
  const { searchParams } = new URL(request.url);
  const category = searchParams.get('category');
  const slugs = category && CATEGORY_SLUGS[category]
    ? CATEGORY_SLUGS[category]
    : Object.keys(PAK_MAP);
  const products = await Promise.all(slugs.map((slug) => fetchPlatformProduct(slug, PAK_MAP[slug])));
  return NextResponse.json({ data: { products } });
}

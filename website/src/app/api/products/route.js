import { NextResponse } from 'next/server';

const PAK_MAP = {
  'mahalaxmi-ai-terminal-orchestration': {
    key: process.env.MAHALAXMI_TERMINAL_PAK_KEY,
    category_id: 'cat-terminal',
    category_name: 'Terminal Orchestration',
    image: '/mahalaxmi_logo.png',
    is_featured: true,
  },
  'mahalaxmi-headless-orchestration': {
    key: process.env.MAHALAXMI_CLOUD_PAK_KEY,
    category_id: 'cat-cloud',
    category_name: 'Cloud Orchestration',
    image: '/mahalaxmi_logo.png',
    is_featured: true,
  },
  'mahalaxmi-vscode-extension': {
    key: process.env.MAHALAXMI_VSCODE_PAK_KEY,
    category_id: 'cat-vscode',
    category_name: 'VS Code Extension',
    image: '/mahalaxmi_logo.png',
    is_featured: false,
  },
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
    return { ...data, slug, ...meta, is_platform_connected: true, data_source: 'platform' };
  } catch {
    return {
      slug,
      ...meta,
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
  const categorySlug = searchParams.get('category');

  const slugs = Object.keys(PAK_MAP);
  const products = await Promise.all(slugs.map((slug) => fetchPlatformProduct(slug, PAK_MAP[slug])));

  const filtered = categorySlug
    ? products.filter((p) => {
        const meta = PAK_MAP[p.slug];
        return meta && meta.category_name.toLowerCase().replace(/\s+/g, '-') === categorySlug;
      })
    : products;

  return NextResponse.json({ success: true, data: { data: { products: filtered } } });
}

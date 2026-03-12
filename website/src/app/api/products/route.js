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

async function fetchPlatformProduct(slug, meta) {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const placeholder = {
    slug,
    ...meta,
    is_platform_connected: false,
    platform_status_message:
      'Pricing temporarily unavailable. Contact support@mahalaxmi.ai',
  };

  if (!platformUrl || !meta.key) {
    return placeholder;
  }

  try {
    const res = await fetch(`${platformUrl}/api/v1/public/product`, {
      headers: { 'X-Channel-API-Key': meta.key },
      next: { revalidate: 60 },
    });

    if (!res.ok) {
      return placeholder;
    }

    const data = await res.json();
    return {
      ...data,
      slug,
      category_id: meta.category_id,
      category_name: meta.category_name,
      image: meta.image,
      is_featured: meta.is_featured,
      is_platform_connected: true,
    };
  } catch {
    return placeholder;
  }
}

export async function GET(request) {
  const { searchParams } = new URL(request.url);
  const categorySlug = searchParams.get('category');

  const entries = Object.entries(PAK_MAP);

  const products = await Promise.all(
    entries.map(([slug, meta]) => fetchPlatformProduct(slug, meta))
  );

  const filtered =
    categorySlug
      ? products.filter((p) => {
          const meta = PAK_MAP[p.slug];
          if (!meta) return false;
          const derivedSlug = meta.category_name
            .toLowerCase()
            .replace(/\s+/g, '-');
          return derivedSlug === categorySlug;
        })
      : products;

  return NextResponse.json({
    success: true,
    data: { data: { products: filtered } },
  });
}

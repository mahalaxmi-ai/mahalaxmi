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

export async function GET(_request, { params }) {
  const { slug } = await params;
  const meta = PAK_MAP[slug];

  if (!meta) {
    return NextResponse.json({ error: 'Product not found' }, { status: 404 });
  }

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const placeholder = {
    slug,
    ...meta,
    key: undefined,
    is_platform_connected: false,
    platform_status_message:
      'Pricing temporarily unavailable. Contact support@mahalaxmi.ai',
  };

  if (!platformUrl || !meta.key) {
    return NextResponse.json({ success: true, data: { product: placeholder } });
  }

  try {
    const res = await fetch(`${platformUrl}/api/v1/public/product`, {
      headers: { 'X-Channel-API-Key': meta.key },
      next: { revalidate: 60 },
    });

    if (!res.ok) {
      return NextResponse.json({ success: true, data: { product: placeholder } });
    }

    const platformData = await res.json();
    const product = {
      ...platformData,
      slug,
      category_id: meta.category_id,
      category_name: meta.category_name,
      image: meta.image,
      is_featured: meta.is_featured,
      is_platform_connected: true,
    };

    return NextResponse.json({ success: true, data: { product } });
  } catch {
    return NextResponse.json({ success: true, data: { product: placeholder } });
  }
}

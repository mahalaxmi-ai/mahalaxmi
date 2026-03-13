// Server-side API helpers — fetch from platform directly using PAK keys
// Used in Server Components and generateMetadata

const PAK_MAP = {
  'mahalaxmi-ai-terminal-orchestration': process.env.MAHALAXMI_TERMINAL_PAK_KEY,
  'mahalaxmi-headless-orchestration': process.env.MAHALAXMI_CLOUD_PAK_KEY,
  'mahalaxmi-vscode-extension': process.env.MAHALAXMI_VSCODE_PAK_KEY,
};

const META_MAP = {
  'mahalaxmi-ai-terminal-orchestration': {
    category_id: 'cat-terminal',
    category_name: 'Terminal Orchestration',
    image: '/mahalaxmi_logo.png',
    is_featured: true,
  },
  'mahalaxmi-headless-orchestration': {
    category_id: 'cat-cloud',
    category_name: 'Cloud Orchestration',
    image: '/mahalaxmi_logo.png',
    is_featured: true,
  },
  'mahalaxmi-vscode-extension': {
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

export async function fetchProductBySlug(slug) {
  const pak = PAK_MAP[slug];
  const meta = META_MAP[slug];
  if (!pak || !meta) return null;

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  try {
    const res = await fetch(`${platformUrl}/api/v1/public/product`, {
      headers: { 'X-Channel-API-Key': pak },
      next: { revalidate: 30 },
    });
    if (!res.ok) throw new Error();
    const data = await res.json();
    const product = data.product;
    const pricing_options = (product.pricingTiers || []).map((tier) => ({
      ...tier,
      price: tier.pricing?.primaryPrice ?? tier.pricing?.monthly ?? 0,
      price_period: 'month',
      features: tier.features || [],
      is_popular: !!tier.isPopular,
      trial_enabled: false,
      displayAnnualPrice: tier.pricing?.yearly
        ? `${tier.pricing.yearly.toFixed(2)}/yr`
        : null,
    }));
    return { ...product, pricing_options, slug, ...meta, is_platform_connected: true, data_source: 'platform' };
  } catch {
    return {
      slug,
      ...meta,
      pricing_options: [],
      pricing_type: 'unavailable',
      name: PRODUCT_NAMES[slug] || slug,
      short_description: 'AI orchestration platform by ThriveTech Services LLC.',
      is_platform_connected: false,
      data_source: 'placeholder',
      platform_status_message: 'Pricing temporarily unavailable. Contact support@mahalaxmi.ai',
    };
  }
}

export async function fetchProducts(params = {}) {
  const slugs = Object.keys(PAK_MAP);
  const products = await Promise.all(slugs.map((s) => fetchProductBySlug(s)));
  const valid = products.filter(Boolean);

  if (params.category) {
    return { data: { products: valid.filter((p) => p.category_name?.toLowerCase().replace(/\s+/g, '-') === params.category) } };
  }

  return { data: { products: valid } };
}

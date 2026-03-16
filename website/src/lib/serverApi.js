// Server-side API helpers — fetch from platform directly using PAK keys.
// Used in Server Components and generateMetadata.

import { getProductMeta } from '@/lib/productApi';

const PAK_MAP = {
  'mahalaxmi-ai-terminal-orchestration': process.env.MAHALAXMI_TERMINAL_PAK_KEY,
  'mahalaxmi-headless-orchestration':    process.env.MAHALAXMI_CLOUD_PAK_KEY,
  'mahalaxmi-vscode-extension':          process.env.MAHALAXMI_VSCODE_PAK_KEY,
};

export async function getProductMetadata(slug) {
  const meta = await getProductMeta();
  return {
    name:        meta.name,
    description: meta.description,
    logo_url:    meta.logo_url,
    slug:        meta.slug,
  };
}

export async function fetchProductBySlug(slug) {
  const pak = PAK_MAP[slug];
  if (!pak) return null;

  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  try {
    const res = await fetch(`${platformUrl}/api/v1/public/product`, {
      headers: { 'X-Channel-API-Key': pak },
      cache: 'no-store',
    });
    if (!res.ok) throw new Error();
    const data = await res.json();
    const product = data.product;
    const pricing_options = (product.pricingTiers || []).map((tier) => ({
      ...tier,
      price: tier.pricing?.primaryPrice ?? tier.pricing?.monthly ?? 0,
      price_period: tier.billing_cycle ?? tier.price_period ?? 'month',
      features: tier.features || [],
      is_popular: !!tier.isPopular,
      trial_enabled: false,
      displayAnnualPrice: tier.pricing?.yearly
        ? `${tier.pricing.yearly.toFixed(2)}/yr`
        : null,
    }));
    return {
      ...product,
      pricing_options,
      slug,
      image:      product.logo_url ?? '/mahalaxmi_logo.png',
      is_featured: product.is_featured ?? true,
      is_platform_connected: true,
      data_source: 'platform',
    };
  } catch {
    return {
      slug,
      pricing_options: [],
      pricing_type: 'unavailable',
      name: slug,
      image: '/mahalaxmi_logo.png',
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

  return { data: { products: valid } };
}

// Central product data fetcher — all product details come from Platform API via PAK key.
// Used by server components only (async). Never imported by client components.

const PLATFORM_API_URL = process.env.MAHALAXMI_PLATFORM_API_URL;
const PAK_KEY = process.env.MAHALAXMI_CLOUD_PAK_KEY;

let cachedOffering = null;
let cacheExpiry = 0;
const CACHE_TTL_MS = 5 * 60 * 1000; // 5 minutes

export async function getProductOffering() {
  if (cachedOffering && Date.now() < cacheExpiry) {
    return cachedOffering;
  }

  const res = await fetch(
    `${PLATFORM_API_URL}/api/v1/products/offering`,
    {
      headers: { 'X-Channel-API-Key': PAK_KEY },
      next: { revalidate: 300 }, // Next.js ISR — 5 min
    }
  );

  if (!res.ok) throw new Error(`Product offering fetch failed: ${res.status}`);

  cachedOffering = await res.json();
  cacheExpiry = Date.now() + CACHE_TTL_MS;
  return cachedOffering;
}

export async function getPricingTiers() {
  const offering = await getProductOffering();
  return offering.pricing_tiers ?? [];
}

export async function getDownloads() {
  const offering = await getProductOffering();
  return offering.downloads ?? {};
}

export async function getProviderCatalog() {
  const offering = await getProductOffering();
  return offering.provider_catalog ?? [];
}

export async function getProductMeta() {
  const offering = await getProductOffering();
  return offering.product ?? {};
}

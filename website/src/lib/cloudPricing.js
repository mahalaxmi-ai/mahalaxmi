// Main server tier size codes — excludes add-ons (Keep Warm, etc.)
const MAIN_TIER_SIZES = new Set(['ccx13', 'ccx23', 'ccx33', 'ccx43']);

/**
 * Fetches cloud pricing from the Product Platform.
 * Returns { pricingTiers: [...] } in the format PricingTiersDisplay expects,
 * or null if the platform is unreachable.
 * The PAK key is never sent to the browser — this runs server-side only.
 */
export async function fetchCloudPricing() {
  const activationUrl = process.env.MAHALAXMI_ACTIVATION_API_URL;
  const pakKey = process.env.MAHALAXMI_CLOUD_PAK_KEY;

  if (!activationUrl || !pakKey) {
    console.error('[cloudPricing] MAHALAXMI_ACTIVATION_API_URL or MAHALAXMI_CLOUD_PAK_KEY not set');
    return null;
  }

  try {
    const res = await fetch(`${activationUrl}/api/v1/public/pricing`, {
      headers: { 'X-Channel-API-Key': pakKey },
      next: { revalidate: 60 }, // cache for 60 seconds
    });

    if (!res.ok) {
      console.error(`[cloudPricing] Platform returned ${res.status}`);
      return null;
    }

    const data = await res.json();
    if (!data.success || !Array.isArray(data.pricingTiers)) {
      console.error('[cloudPricing] Unexpected response:', data.error || data);
      return null;
    }

    // Filter to main server tiers only (exclude add-ons like Keep Warm)
    const tiers = data.pricingTiers.filter(
      (t) => MAIN_TIER_SIZES.has(t.featureLimits?.serverSize?.toLowerCase())
    );

    return tiers.length > 0 ? { pricingTiers: tiers } : null;
  } catch (err) {
    console.error('[cloudPricing] Fetch error:', err);
    return null;
  }
}

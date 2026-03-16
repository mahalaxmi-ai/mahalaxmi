/**
 * Cloud provider and tier display maps — fetched from Platform API via PAK key.
 * Used by server components only. Client components (ServerCard) receive these as props.
 */

import { getProviderCatalog, getPricingTiers } from '@/lib/productApi';

export async function getTierLabels() {
  const tiers = await getPricingTiers();
  return Object.fromEntries(
    tiers.map((t) => [t.slug, t.name])
  );
}

export async function getProviderLabels() {
  const providers = await getProviderCatalog();
  return Object.fromEntries(
    providers.map((p) => [p.slug, { name: p.display_name, shortName: p.short_name ?? p.slug.toUpperCase().slice(0, 3), color: p.hex_color }])
  );
}

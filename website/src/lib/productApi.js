// Central product data fetcher — all product details come from Platform API via PAK key.
// Used by server components only (async). Never imported by client components.

const PLATFORM_API_URL  = process.env.MAHALAXMI_PLATFORM_API_URL;
const CLOUD_PAK_KEY     = process.env.MAHALAXMI_CLOUD_PAK_KEY;
const DESKTOP_PAK_KEY   = process.env.MAHALAXMI_DESKTOP_PAK_KEY;
const TERMINAL_PAK_KEY  = process.env.MAHALAXMI_TERMINAL_PAK_KEY;

async function fetchOffering(pakKey) {
  const res = await fetch(
    `${PLATFORM_API_URL}/api/v1/products/offering`,
    {
      headers: { 'X-Channel-API-Key': pakKey },
      next: { revalidate: 300 }, // Next.js ISR — 5 min
    }
  );
  if (!res.ok) throw new Error(`Offering fetch failed: ${res.status}`);
  return res.json();
}

export async function getDesktopProductOffering() {
  return fetchOffering(DESKTOP_PAK_KEY);
}

export async function getCloudProductOffering() {
  return fetchOffering(CLOUD_PAK_KEY);
}

// Desktop-scoped helpers
export async function getPricingTiers() {
  const offering = await getDesktopProductOffering();
  return offering.pricing_tiers ?? [];
}

async function fetchLatestRelease(platform) {
  try {
    const url = `${PLATFORM_API_URL}/api/v1/public/releases/latest?platform=${platform}`;
    const res = await fetch(url, {
      headers: { 'X-Channel-API-Key': DESKTOP_PAK_KEY },
      cache: 'no-store',
    });
    if (!res.ok) return null;
    const data = await res.json();
    if (!data.success || !data.release) return null;
    return {
      url: `/api/releases/download?id=${data.release.id}`,
      filename: data.release.fileName,
      version: data.release.version,
      format: data.release.installerFormat,
    };
  } catch {
    return null;
  }
}

export async function getDownloads() {
  const [linux, windows, macos] = await Promise.all([
    fetchLatestRelease('linux'),
    fetchLatestRelease('windows'),
    fetchLatestRelease('macos'),
  ]);

  const desktop_app = {};
  if (linux)   desktop_app.linux_deb   = linux;
  if (windows) desktop_app.windows_exe = windows;
  if (macos)   desktop_app.macos_dmg   = macos;

  // Also get latest_version from any successful release
  const anyRelease = linux || windows || macos;

  return {
    desktop_app,
    latest_version: anyRelease?.version ?? null,
  };
}

// Cloud-scoped helpers
export async function getProviderCatalog() {
  const offering = await getCloudProductOffering();
  return offering.provider_catalog ?? [];
}

export async function getProductMeta() {
  const offering = await getDesktopProductOffering();
  return offering.product ?? {};
}

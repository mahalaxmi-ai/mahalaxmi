/**
 * Cloud provider and tier display maps.
 * Contract-locked as of 8d34065 on integration.
 *
 * Adding a new provider: one line in PROVIDER_LABELS.
 * Adding a new tier:     one line in TIER_LABELS.
 * No switch statements anywhere in the codebase.
 */

export const PROVIDER_LABELS = {
  hetzner: { name: 'Hetzner', shortName: 'HZ',  color: '#d50000' },
  aws:     { name: 'AWS',     shortName: 'AWS',  color: '#FF9900' },
  gcp:     { name: 'GCP',     shortName: 'GCP',  color: '#4285F4' },
};

export const TIER_LABELS = {
  'cloud-solo':    'Cloud Solo',
  'cloud-builder': 'Cloud Builder',
  'cloud-power':   'Cloud Power',
  'cloud-team':    'Cloud Team',
  'desktop':       'Desktop',
};

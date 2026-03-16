'use client';
import { useState } from 'react';

export default function CloudProviderTabs({ onProviderChange, providerLabels = {} }) {
  const PROVIDERS = [
    { key: 'hetzner', ...(providerLabels.hetzner ?? { name: 'Hetzner', color: '#d50000' }), available: true },
    { key: 'aws',     ...(providerLabels.aws     ?? { name: 'AWS',     color: '#FF9900' }), available: false },
    { key: 'gcp',     ...(providerLabels.gcp     ?? { name: 'GCP',     color: '#4285F4' }), available: false },
  ];

  const [active, setActive] = useState('hetzner');

  const handleSelect = (key) => {
    if (!PROVIDERS.find(p => p.key === key).available) return;
    setActive(key);
    onProviderChange(key);
  };

  return (
    <div className="cloud-provider-tabs">
      {PROVIDERS.map(p => (
        <button
          key={p.key}
          onClick={() => handleSelect(p.key)}
          disabled={!p.available}
          data-active={active === p.key}
          style={{ '--provider-color': p.color }}
        >
          {p.name}
          {!p.available && (
            <span className="coming-soon-badge">Coming Soon</span>
          )}
        </button>
      ))}
    </div>
  );
}

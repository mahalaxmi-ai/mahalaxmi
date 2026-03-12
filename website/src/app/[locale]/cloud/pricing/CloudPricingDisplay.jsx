'use client';

import { useState, useEffect } from 'react';
import { useSearchParams } from 'next/navigation';
import { Alert, Box, CircularProgress } from '@mui/material';
import PricingTiersDisplay from '@/components/Docs/PricingTiersDisplay';
import BuyNowButton from './BuyNowButton';

export default function CloudPricingDisplay({ pricingData: initialPricingData }) {
  const searchParams = useSearchParams();
  const billingCycleParam = searchParams.get('billing_cycle');
  const initialInterval = billingCycleParam === 'monthly' ? 'monthly'
    : billingCycleParam === 'annual' ? 'yearly'
    : undefined;

  const [pricingData, setPricingData] = useState(initialPricingData ?? null);
  const [loading, setLoading] = useState(!initialPricingData);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (initialPricingData) return;

    let cancelled = false;

    async function loadPricing() {
      try {
        const res = await fetch('/api/checkout', { method: 'GET' });
        if (!res.ok) {
          if (!cancelled) {
            setError('Pricing is temporarily unavailable. Please contact support@mahalaxmi.ai');
            setLoading(false);
          }
          return;
        }
        const data = await res.json();
        if (!cancelled) {
          if (data && Array.isArray(data.pricingTiers) && data.pricingTiers.length > 0) {
            setPricingData(data);
          } else {
            setError('Pricing is temporarily unavailable. Please contact support@mahalaxmi.ai');
          }
          setLoading(false);
        }
      } catch {
        if (!cancelled) {
          setError('Pricing is temporarily unavailable. Please contact support@mahalaxmi.ai');
          setLoading(false);
        }
      }
    }

    loadPricing();
    return () => { cancelled = true; };
  }, [initialPricingData]);

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 6 }}>
        <CircularProgress />
      </Box>
    );
  }

  if (error) {
    return (
      <Alert severity="error" sx={{ my: 4 }}>
        {error}
      </Alert>
    );
  }

  if (!pricingData) {
    return (
      <Alert severity="warning" sx={{ my: 4 }}>
        Pricing information is currently unavailable. Please contact{' '}
        <a href="mailto:support@mahalaxmi.ai">support@mahalaxmi.ai</a> for assistance.
      </Alert>
    );
  }

  return (
    <PricingTiersDisplay
      pricingData={pricingData}
      initialInterval={initialInterval}
      renderAction={(tier, billingInterval) => (
        <BuyNowButton
          tier={tier.slug}
          billingCycle={billingInterval === 'yearly' ? 'annual' : 'monthly'}
          label={`Start ${tier.name}`}
          variant={tier.isRecommended ? 'contained' : 'outlined'}
        />
      )}
    />
  );
}

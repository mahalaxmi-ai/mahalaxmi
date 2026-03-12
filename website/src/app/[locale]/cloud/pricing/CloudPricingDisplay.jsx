'use client';

import { useSearchParams } from 'next/navigation';
import PricingTiersDisplay from '@/components/Docs/PricingTiersDisplay';
import BuyNowButton from './BuyNowButton';

export default function CloudPricingDisplay({ pricingData }) {
  const searchParams = useSearchParams();
  const billingCycleParam = searchParams.get('billing_cycle');
  const initialInterval = billingCycleParam === 'monthly' ? 'monthly'
    : billingCycleParam === 'annual' ? 'yearly'
    : undefined;

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

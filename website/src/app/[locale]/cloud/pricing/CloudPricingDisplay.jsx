'use client';

import { useSearchParams } from 'next/navigation';
import { Button } from '@mui/material';
import PricingTiersDisplay from '@/components/Docs/PricingTiersDisplay';
import BuyNowButton from './BuyNowButton';
import { useAuth } from '@/contexts/AuthContext';

function StudentPricingButton({ variant }) {
  const { user } = useAuth();
  const email = user?.email ?? '';
  const subject = encodeURIComponent('Student Pricing Application');
  const body = encodeURIComponent(
    email
      ? `Hi,\n\nI'd like to apply for student pricing.\n\nMy account email: ${email}\n`
      : 'Hi,\n\nI\'d like to apply for student pricing.\n'
  );
  const href = `mailto:support@mahalaxmi.ai?subject=${subject}&body=${body}`;

  return (
    <Button
      component="a"
      href={href}
      variant={variant}
      fullWidth
      sx={{ textTransform: 'none' }}
    >
      Apply for Student Pricing
    </Button>
  );
}

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
      renderAction={(tier, billingInterval) => {
        if (tier.verification_required) {
          return (
            <StudentPricingButton
              variant={tier.isRecommended ? 'contained' : 'outlined'}
            />
          );
        }
        return (
          <BuyNowButton
            tier={tier.slug}
            billingCycle={billingInterval === 'yearly' ? 'annual' : 'monthly'}
            label={`Start ${tier.name}`}
            variant={tier.isRecommended ? 'contained' : 'outlined'}
          />
        );
      }}
    />
  );
}

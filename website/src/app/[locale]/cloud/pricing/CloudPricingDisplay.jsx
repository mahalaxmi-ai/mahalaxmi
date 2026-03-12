'use client';

import { useState, useEffect } from 'react';
import {
  Box,
  Button,
  ButtonGroup,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Container,
  Grid,
  Typography,
} from '@mui/material';
import { CheckCircle } from '@mui/icons-material';
import BuyNowButton from './BuyNowButton';

const checkoutAPI = {
  async getPricing() {
    const res = await fetch('/api/checkout', { cache: 'no-store' });
    if (!res.ok) throw new Error('Pricing unavailable');
    return res.json();
  },
};

const FALLBACK_TIERS = [
  {
    slug: 'solo',
    name: 'Cloud Solo',
    description: 'One dedicated VM for solo developers.',
    monthlyPrice: 49,
    annualPrice: 39,
    features: ['1 dedicated VM', '4 vCPU / 8 GB RAM', 'Unlimited workers', 'VS Code deep-link', 'Community support'],
    isRecommended: false,
  },
  {
    slug: 'builder',
    name: 'Cloud Builder',
    description: 'More power for demanding projects.',
    monthlyPrice: 99,
    annualPrice: 79,
    features: ['1 dedicated VM', '8 vCPU / 16 GB RAM', 'Unlimited workers', 'VS Code deep-link', 'Email support'],
    isRecommended: true,
  },
  {
    slug: 'power',
    name: 'Cloud Power',
    description: 'High-performance for intensive workloads.',
    monthlyPrice: 199,
    annualPrice: 159,
    features: ['1 dedicated VM', '16 vCPU / 32 GB RAM', 'Unlimited workers', 'VS Code deep-link', 'Priority support'],
    isRecommended: false,
  },
  {
    slug: 'team',
    name: 'Cloud Team',
    description: 'Multiple servers for growing teams.',
    monthlyPrice: null,
    annualPrice: null,
    features: ['Multiple VMs', 'Custom resources', 'Centralized billing', 'VS Code deep-link', 'Dedicated support'],
    isRecommended: false,
    isEnterprise: true,
  },
];

function normalizeTiers(data) {
  if (!data || !Array.isArray(data.tiers)) return null;
  return data.tiers;
}

function TierCard({ tier, billingCycle }) {
  const price = billingCycle === 'annual' ? tier.annualPrice : tier.monthlyPrice;

  return (
    <Card
      elevation={tier.isRecommended ? 6 : 1}
      sx={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        border: tier.isRecommended ? '2px solid' : '1px solid',
        borderColor: tier.isRecommended ? 'primary.main' : 'divider',
        borderRadius: 3,
        position: 'relative',
      }}
    >
      {tier.isRecommended && (
        <Chip
          label="Most Popular"
          color="primary"
          size="small"
          sx={{ position: 'absolute', top: 16, right: 16, fontWeight: 700 }}
        />
      )}
      <CardContent sx={{ flexGrow: 1, p: 3 }}>
        <Typography variant="h5" sx={{ fontWeight: 700, mb: 0.5 }}>
          {tier.name}
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          {tier.description}
        </Typography>

        {tier.isEnterprise ? (
          <Box sx={{ mb: 3 }}>
            <Typography variant="h4" sx={{ fontWeight: 800 }}>
              Custom
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Contact us for team pricing
            </Typography>
          </Box>
        ) : (
          <Box sx={{ mb: 3 }}>
            <Typography variant="h4" component="span" sx={{ fontWeight: 800 }}>
              ${price}
            </Typography>
            <Typography variant="body2" component="span" color="text.secondary">
              /month
            </Typography>
            {billingCycle === 'annual' && (
              <Typography variant="caption" display="block" color="success.main" sx={{ fontWeight: 600 }}>
                Billed annually — save ~20%
              </Typography>
            )}
          </Box>
        )}

        <Box sx={{ mb: 3 }}>
          {(tier.features || []).map((feature) => (
            <Box key={feature} sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
              <CheckCircle sx={{ fontSize: 18, color: 'success.main' }} />
              <Typography variant="body2">{feature}</Typography>
            </Box>
          ))}
        </Box>

        {tier.isEnterprise ? (
          <Button
            variant="outlined"
            color="primary"
            fullWidth
            component="a"
            href="mailto:sales@mahalaxmi.ai"
            sx={{ mb: 2 }}
          >
            Contact Sales
          </Button>
        ) : (
          <BuyNowButton tier={tier.slug} billingCycle={billingCycle} />
        )}
      </CardContent>
    </Card>
  );
}

export default function CloudPricingDisplay() {
  const [billingCycle, setBillingCycle] = useState('monthly');
  const [tiers, setTiers] = useState(null);
  const [loading, setLoading] = useState(true);
  const [pricingError, setPricingError] = useState(false);

  useEffect(() => {
    checkoutAPI.getPricing()
      .then((data) => {
        const normalized = normalizeTiers(data);
        setTiers(normalized || FALLBACK_TIERS);
      })
      .catch(() => {
        setPricingError(true);
      })
      .finally(() => {
        setLoading(false);
      });
  }, []);

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 8 }}>
        <CircularProgress />
      </Box>
    );
  }

  if (pricingError) {
    return (
      <Box sx={{ textAlign: 'center', py: 6 }}>
        <Typography variant="h6" color="error" sx={{ mb: 1 }}>
          Pricing temporarily unavailable. Contact support@mahalaxmi.ai
        </Typography>
        <Button
          component="a"
          href="mailto:support@mahalaxmi.ai"
          variant="outlined"
          color="primary"
        >
          support@mahalaxmi.ai
        </Button>
      </Box>
    );
  }

  const displayTiers = tiers || FALLBACK_TIERS;

  return (
    <Container maxWidth="lg" disableGutters>
      <Box sx={{ display: 'flex', justifyContent: 'center', mb: 5 }}>
        <ButtonGroup variant="outlined" size="small">
          <Button
            variant={billingCycle === 'monthly' ? 'contained' : 'outlined'}
            onClick={() => setBillingCycle('monthly')}
          >
            Monthly
          </Button>
          <Button
            variant={billingCycle === 'annual' ? 'contained' : 'outlined'}
            onClick={() => setBillingCycle('annual')}
          >
            Annual
            <Chip
              label="Save 20%"
              size="small"
              color="success"
              sx={{ ml: 1, height: 18, fontSize: '0.65rem' }}
            />
          </Button>
        </ButtonGroup>
      </Box>

      <Grid container spacing={3} alignItems="stretch">
        {displayTiers.map((tier) => (
          <Grid item xs={12} sm={6} lg={3} key={tier.slug}>
            <TierCard tier={tier} billingCycle={billingCycle} />
          </Grid>
        ))}
      </Grid>
    </Container>
  );
}

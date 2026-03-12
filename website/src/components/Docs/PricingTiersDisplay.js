'use client';

import React, { useState, useMemo } from 'react';
import {
  Box,
  Card,
  CardContent,
  Chip,
  Typography,
  ToggleButton,
  ToggleButtonGroup,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Grid,
} from '@mui/material';
import { Check, Close, Star } from '@mui/icons-material';

function formatPrice(amount, currency = 'USD') {
  if (amount === 0) return 'Free';
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(amount);
}

function intervalLabel(interval) {
  switch (interval) {
    case 'monthly': return '/mo';
    case 'yearly': return '/yr';
    case 'lifetime': return ' one-time';
    default: return '';
  }
}

export default function PricingTiersDisplay({ pricingData, renderAction, initialInterval }) {
  const tiers = useMemo(() => {
    if (!pricingData?.pricingTiers?.length) return [];
    return [...pricingData.pricingTiers].sort((a, b) => (a.displayOrder ?? 0) - (b.displayOrder ?? 0));
  }, [pricingData]);

  const availableIntervals = useMemo(() => {
    const intervals = new Set();
    for (const tier of tiers) {
      if (tier.pricing?.monthly > 0) intervals.add('monthly');
      if (tier.pricing?.yearly > 0) intervals.add('yearly');
      if (tier.pricing?.lifetime > 0) intervals.add('lifetime');
    }
    const order = ['monthly', 'yearly', 'lifetime'];
    return order.filter((i) => intervals.has(i));
  }, [tiers]);

  const [billingInterval, setBillingInterval] = useState(() => {
    if (initialInterval && availableIntervals.includes(initialInterval)) return initialInterval;
    return availableIntervals.includes('yearly') ? 'yearly' : availableIntervals[0] || 'yearly';
  });

  const allFeatures = useMemo(() => {
    const featureMap = new Map();
    for (const tier of tiers) {
      for (const feat of tier.features || []) {
        const key = typeof feat === 'string' ? feat : feat.name || feat;
        if (!featureMap.has(key)) featureMap.set(key, key);
      }
    }
    return [...featureMap.values()];
  }, [tiers]);

  if (!tiers.length) return null;

  const currency = tiers[0]?.pricing?.currency || 'USD';

  return (
    <Box sx={{ mt: 4 }}>
      {/* Billing interval toggle */}
      {availableIntervals.length > 1 && (
        <Box sx={{ display: 'flex', justifyContent: 'center', mb: 4 }}>
          <ToggleButtonGroup
            value={billingInterval}
            exclusive
            onChange={(_, val) => val && setBillingInterval(val)}
            size="small"
          >
            {availableIntervals.map((interval) => (
              <ToggleButton key={interval} value={interval} sx={{ px: 3, textTransform: 'capitalize' }}>
                {interval === 'monthly' ? 'Monthly' : interval === 'yearly' ? 'Yearly' : 'Lifetime'}
              </ToggleButton>
            ))}
          </ToggleButtonGroup>
        </Box>
      )}

      {/* Pricing cards */}
      <Grid container spacing={3} sx={{ mb: 6 }}>
        {tiers.map((tier) => {
          const price = tier.pricing?.[billingInterval];
          const isRecommended = tier.isRecommended;

          return (
            <Grid item xs={12} sm={6} md={4} key={tier.id || tier.slug} sx={{ pt: isRecommended ? '24px !important' : undefined }}>
              <Card
                variant="outlined"
                sx={{
                  height: '100%',
                  display: 'flex',
                  flexDirection: 'column',
                  borderColor: isRecommended ? 'primary.main' : 'divider',
                  borderWidth: isRecommended ? 2 : 1,
                  position: 'relative',
                  overflow: 'visible',
                }}
              >
                {isRecommended && (
                  <Chip
                    icon={<Star sx={{ fontSize: 16 }} />}
                    label="Recommended"
                    color="primary"
                    size="small"
                    sx={{
                      position: 'absolute',
                      top: -12,
                      left: '50%',
                      transform: 'translateX(-50%)',
                    }}
                  />
                )}
                <CardContent sx={{ flexGrow: 1, pt: isRecommended ? 3 : 2 }}>
                  <Typography variant="h6" sx={{ fontWeight: 600, mb: 0.5 }}>
                    {tier.name}
                  </Typography>

                  {tier.description && (
                    <Typography variant="body2" color="text.secondary" sx={{ mb: 2, minHeight: 40 }}>
                      {tier.description}
                    </Typography>
                  )}

                  <Box sx={{ mb: 2 }}>
                    {price != null ? (
                      <Typography variant="h4" sx={{ fontWeight: 700, color: 'primary.main' }}>
                        {formatPrice(price, currency)}
                        <Typography component="span" variant="body2" color="text.secondary">
                          {intervalLabel(billingInterval)}
                        </Typography>
                      </Typography>
                    ) : (
                      <Typography variant="body1" color="text.secondary">
                        Contact us
                      </Typography>
                    )}
                  </Box>

                  {tier.trial?.enabled && (
                    <Chip
                      label={`${tier.trial.durationDays}-day free trial`}
                      size="small"
                      variant="outlined"
                      color="success"
                      sx={{ mb: 2 }}
                    />
                  )}

                  {renderAction && (
                    <Box sx={{ mt: 2, mb: 1 }}>
                      {renderAction(tier, billingInterval)}
                    </Box>
                  )}

                  {tier.seats && (
                    <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 2 }}>
                      {tier.seats.included} seat{tier.seats.included !== 1 ? 's' : ''} included
                      {tier.seats.max ? ` (up to ${tier.seats.max})` : ''}
                    </Typography>
                  )}

                  {tier.features?.length > 0 && (
                    <Box sx={{ mt: 2 }}>
                      {tier.features.map((feat, idx) => {
                        const name = typeof feat === 'string' ? feat : feat.name || feat;
                        return (
                          <Box key={idx} sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.5 }}>
                            <Check sx={{ fontSize: 16, color: 'success.main' }} />
                            <Typography variant="body2">{name}</Typography>
                          </Box>
                        );
                      })}
                    </Box>
                  )}
                </CardContent>
              </Card>
            </Grid>
          );
        })}
      </Grid>

      {/* Feature comparison table */}
      {allFeatures.length > 0 && tiers.length > 1 && (
        <Box sx={{ mt: 4 }}>
          <Typography variant="h5" sx={{ fontWeight: 600, mb: 2 }}>
            Feature Comparison
          </Typography>
          <TableContainer component={Paper} variant="outlined">
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell sx={{ fontWeight: 600 }}>Feature</TableCell>
                  {tiers.map((tier) => (
                    <TableCell key={tier.id || tier.slug} align="center" sx={{ fontWeight: 600 }}>
                      {tier.name}
                    </TableCell>
                  ))}
                </TableRow>
              </TableHead>
              <TableBody>
                {allFeatures.map((featureName) => (
                  <TableRow key={featureName}>
                    <TableCell>{featureName}</TableCell>
                    {tiers.map((tier) => {
                      const tierFeatures = (tier.features || []).map((f) =>
                        typeof f === 'string' ? f : f.name || f
                      );
                      const has = tierFeatures.includes(featureName);
                      return (
                        <TableCell key={tier.id || tier.slug} align="center">
                          {has ? (
                            <Check sx={{ fontSize: 18, color: 'success.main' }} />
                          ) : (
                            <Close sx={{ fontSize: 18, color: 'text.disabled' }} />
                          )}
                        </TableCell>
                      );
                    })}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </Box>
      )}
    </Box>
  );
}

'use client';

import { useState, useEffect } from 'react';
import { Button, CircularProgress } from '@mui/material';
import { Cloud } from '@mui/icons-material';
import { useAuth } from '@/contexts/AuthContext';

const SITE_ORIGIN = 'https://mahalaxmi.ai';

export default function BuyNowButton({ tier, billingCycle = 'monthly', cloudProvider = 'hetzner', label, variant = 'contained' }) {
  const [loading, setLoading] = useState(false);
  const [pendingCheckout, setPendingCheckout] = useState(false);
  const { isAuthenticated, isLoading: authLoading } = useAuth();

  // If auth was still hydrating when the user clicked, proceed once resolved
  useEffect(() => {
    if (!pendingCheckout || authLoading) return;
    setPendingCheckout(false);
    if (isAuthenticated) {
      startCheckout();
    } else {
      redirectToRegister();
      setLoading(false);
    }
  }, [pendingCheckout, authLoading, isAuthenticated]); // eslint-disable-line react-hooks/exhaustive-deps

  function redirectToRegister() {
    window.location.href = `${SITE_ORIGIN}/register?redirect=/cloud/pricing`;
  }

  async function startCheckout() {
    const successUrl = `${SITE_ORIGIN}/checkout/success?session_id={CHECKOUT_SESSION_ID}`;
    const cancelUrl = `${SITE_ORIGIN}/cloud/pricing`;

    try {
      const res = await fetch('/api/checkout', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          tier,
          billing_cycle: billingCycle,
          cloud_provider: cloudProvider,
          success_url: successUrl,
          cancel_url: cancelUrl,
        }),
      });

      const data = await res.json();

      if (res.status === 401) {
        redirectToRegister();
        return;
      }

      if (res.status === 403 && data.error === 'verification_required') {
        alert('Student verification is required before purchasing this tier. Please apply for student pricing first.');
        setLoading(false);
        return;
      }

      if (!res.ok || !data.checkout_url) {
        console.error('[BuyNowButton] Checkout error:', data.error);
        alert('Checkout is temporarily unavailable. Please contact support@mahalaxmi.ai');
        setLoading(false);
        return;
      }

      window.location.href = data.checkout_url;
    } catch (err) {
      console.error('[BuyNowButton] Network error:', err);
      alert('Checkout is temporarily unavailable. Please contact support@mahalaxmi.ai');
      setLoading(false);
    }
  }

  function handleClick() {
    if (loading) return;
    setLoading(true);

    if (authLoading) {
      // Auth still hydrating — wait for it, then proceed
      setPendingCheckout(true);
      return;
    }

    if (!isAuthenticated) {
      redirectToRegister();
      setLoading(false);
      return;
    }

    startCheckout();
  }

  return (
    <Button
      variant={variant}
      fullWidth
      startIcon={loading ? <CircularProgress size={18} color="inherit" /> : <Cloud />}
      onClick={handleClick}
      disabled={loading}
      sx={{ mb: variant === 'contained' ? 3 : 2.5 }}
    >
      {loading ? 'Redirecting…' : label}
    </Button>
  );
}

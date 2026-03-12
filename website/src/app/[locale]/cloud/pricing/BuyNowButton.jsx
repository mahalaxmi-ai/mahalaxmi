'use client';

import { useState, useEffect } from 'react';
import { Button, CircularProgress } from '@mui/material';
import { Cloud } from '@mui/icons-material';
import { useAuth } from '@/contexts/AuthContext';

export default function BuyNowButton({ tier, billingCycle = 'monthly', label, variant = 'contained' }) {
  const [loading, setLoading] = useState(false);
  const [pendingCheckout, setPendingCheckout] = useState(false);
  const { isAuthenticated, isLoading: authLoading, token } = useAuth();

  // If auth was still hydrating when the user clicked, proceed once resolved
  useEffect(() => {
    if (!pendingCheckout || authLoading) return;
    setPendingCheckout(false);
    if (isAuthenticated) {
      startCheckout();
    } else {
      redirectToLogin();
      setLoading(false);
    }
  }, [pendingCheckout, authLoading, isAuthenticated]); // eslint-disable-line react-hooks/exhaustive-deps

  function redirectToLogin() {
    const params = new URLSearchParams({ redirect: '/cloud/pricing', tier, billing_cycle: billingCycle });
    window.location.href = `/login?${params}`;
  }

  async function startCheckout() {
    const origin = window.location.origin;
    const successUrl = `${origin}/checkout/success?session_id={CHECKOUT_SESSION_ID}`;
    const cancelUrl = `${origin}/cloud/pricing`;

    try {
      const res = await fetch('/api/checkout', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify({ tier, billing_cycle: billingCycle, success_url: successUrl, cancel_url: cancelUrl }),
      });

      const data = await res.json();

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
      redirectToLogin();
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

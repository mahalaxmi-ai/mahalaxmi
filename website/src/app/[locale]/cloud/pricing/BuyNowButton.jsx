'use client';

import { useState, useEffect } from 'react';
import { Button, CircularProgress } from '@mui/material';
import { Cloud } from '@mui/icons-material';
import { useAuth } from '@/contexts/AuthContext';

const checkoutAPI = {
  async createSession({ tier, billing_cycle, success_url, cancel_url }) {
    const res = await fetch('/api/checkout', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ tier, billing_cycle, success_url, cancel_url }),
    });
    const data = await res.json();
    if (!res.ok || !data.checkout_url) {
      throw new Error(data.error || 'Checkout unavailable');
    }
    return data;
  },
};

export default function BuyNowButton({ tier, billingCycle = 'monthly' }) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [pendingCheckout, setPendingCheckout] = useState(false);
  const { isAuthenticated, isLoading: authLoading } = useAuth();

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
    const params = new URLSearchParams({ redirect: '/cloud/pricing', tier });
    window.location.href = `/login?${params}`;
  }

  async function startCheckout() {
    setError(null);
    try {
      const data = await checkoutAPI.createSession({
        tier,
        billing_cycle: billingCycle,
        success_url: window.location.origin + '/checkout/success',
        cancel_url: window.location.href,
      });
      window.location.href = data.checkout_url;
    } catch {
      setError('Checkout is temporarily unavailable. Please contact support@mahalaxmi.ai');
      setLoading(false);
    }
  }

  function handleClick() {
    if (loading) return;
    setLoading(true);
    setError(null);

    if (authLoading) {
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
    <>
      <Button
        variant="contained"
        color="primary"
        fullWidth
        startIcon={loading ? <CircularProgress size={18} color="inherit" /> : <Cloud />}
        onClick={handleClick}
        disabled={loading}
        sx={{ mb: 2 }}
      >
        {loading ? 'Redirecting…' : 'Get Started'}
      </Button>
      {error && (
        <p style={{ color: '#d32f2f', fontSize: '0.8rem', marginTop: 4, textAlign: 'center' }}>
          {error}
        </p>
      )}
    </>
  );
}

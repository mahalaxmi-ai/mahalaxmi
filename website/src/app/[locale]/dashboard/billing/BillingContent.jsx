'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { useQuery } from '@tanstack/react-query';
import {
  Alert,
  Box,
  Button,
  CircularProgress,
  Container,
  Divider,
  Typography,
} from '@mui/material';
import { CreditCard, OpenInNew } from '@mui/icons-material';
import { useAuth } from '@/contexts/AuthContext';
import { Link } from '@/i18n/navigation';
export default function BillingContent({ tierLabels = {} }) {
  const { isAuthenticated, isLoading: authLoading, user } = useAuth();
  const router = useRouter();
  const [portalLoading, setPortalLoading] = useState(false);
  const [portalError, setPortalError] = useState(null);

  const userHeaders = {
    ...(user?.id    ? { 'x-user-id':    String(user.id)    } : {}),
    ...(user?.email ? { 'x-user-email': user.email } : {}),
  };

  // Fetch servers to derive tier (bare array per spec)
  const { data: servers = [] } = useQuery({
    queryKey: ['mahalaxmi-servers', user?.id],
    queryFn: async () => {
      const res = await fetch('/api/mahalaxmi/servers', {
        cache: 'no-store',
        headers: userHeaders,
      });
      if (!res.ok) return [];
      const data = await res.json();
      return Array.isArray(data) ? data : (data.servers ?? []);
    },
    enabled: !authLoading && isAuthenticated,
    staleTime: 10_000,
  });

  if (!authLoading && !isAuthenticated) {
    router.replace('/login?redirect=/dashboard/billing');
    return null;
  }

  if (authLoading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 10 }}>
        <CircularProgress />
      </Box>
    );
  }

  // Derive tier from user object or first active/provisioning server
  const tierKey = user?.tier
    ?? servers.find((s) => s.status === 'active' || s.status === 'provisioning')?.tier
    ?? null;
  const tierLabel = tierKey ? (tierLabels[tierKey] ?? tierKey) : null;

  async function handleManageBilling() {
    setPortalLoading(true);
    setPortalError(null);
    try {
      const res = await fetch('/api/mahalaxmi/billing/portal-url', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...userHeaders },
      });
      const data = await res.json();

      if (res.status === 404 && data.error === 'no_active_subscription') {
        setPortalError('no_subscription');
        return;
      }
      if (!res.ok || !data.url) {
        setPortalError('unavailable');
        return;
      }
      window.location.href = data.url;
    } catch {
      setPortalError('unavailable');
    } finally {
      setPortalLoading(false);
    }
  }

  return (
    <Container maxWidth="sm" sx={{ py: { xs: 4, md: 6 } }}>
      <Typography variant="h4" component="h1" sx={{ fontWeight: 700, mb: 1 }}>
        Billing
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 4 }}>
        Manage your Mahalaxmi Cloud subscription and payment method.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      {/* Current tier */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="overline" color="text.secondary">
          Current plan
        </Typography>
        <Typography variant="h6" sx={{ mt: 0.5, fontWeight: 700 }}>
          {tierLabel ?? '—'}
        </Typography>
        {!tierLabel && (
          <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
            No active subscription detected.
          </Typography>
        )}
      </Box>

      {/* Error states */}
      {portalError === 'no_subscription' && (
        <Alert severity="info" sx={{ mb: 3 }}>
          No active subscription found.{' '}
          <Link href="/cloud/pricing" style={{ color: 'inherit', fontWeight: 600 }}>
            Start at /cloud/pricing
          </Link>
        </Alert>
      )}
      {portalError === 'unavailable' && (
        <Alert severity="error" sx={{ mb: 3 }}>
          Billing portal is temporarily unavailable. Please try again or contact{' '}
          <a href="mailto:billing@mahalaxmi.ai">billing@mahalaxmi.ai</a>
        </Alert>
      )}

      {/* Manage billing button */}
      <Button
        variant="contained"
        size="large"
        startIcon={portalLoading ? <CircularProgress size={18} color="inherit" /> : <CreditCard />}
        endIcon={!portalLoading ? <OpenInNew fontSize="small" /> : null}
        onClick={handleManageBilling}
        disabled={portalLoading}
      >
        {portalLoading ? 'Loading…' : 'Manage Billing'}
      </Button>

      <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 2 }}>
        You will be redirected to the Stripe billing portal.
      </Typography>
    </Container>
  );
}

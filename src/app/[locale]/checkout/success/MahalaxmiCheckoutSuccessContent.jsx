'use client';

import { useEffect, useRef, useState } from 'react';
import { useSearchParams } from 'next/navigation';
import {
  Alert,
  Box,
  Button,
  CircularProgress,
  Container,
  LinearProgress,
  Paper,
  Typography,
} from '@mui/material';
import { CheckCircleOutline, ErrorOutline } from '@mui/icons-material';

const POLL_INTERVAL_MS = 5_000;
const TIMEOUT_MS = 10 * 60 * 1_000;

export default function MahalaxmiCheckoutSuccessContent() {
  const searchParams = useSearchParams();
  const sessionId = searchParams.get('session_id');

  const [uiStatus, setUiStatus] = useState('loading');
  const intervalRef = useRef(null);
  const timeoutRef = useRef(null);

  function stopPolling() {
    if (intervalRef.current) clearInterval(intervalRef.current);
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
  }

  async function poll(id) {
    try {
      const res = await fetch(`/api/checkout/session/${id}`, { cache: 'no-store' });
      if (!res.ok) {
        return;
      }
      const data = await res.json();

      if (data.status === 'active' || data.status === 'provisioning') {
        stopPolling();
        setUiStatus('paid');
      } else if (data.status === 'failed') {
        setUiStatus('failed');
        stopPolling();
      } else if (data.status === 'pending_payment') {
        setUiStatus('pending_payment');
      } else {
        setUiStatus('provisioning');
      }
    } catch {
      // Network errors are transient; keep polling
    }
  }

  useEffect(() => {
    if (!sessionId) {
      setUiStatus('failed');
      return;
    }

    poll(sessionId);
    intervalRef.current = setInterval(() => poll(sessionId), POLL_INTERVAL_MS);

    timeoutRef.current = setTimeout(() => {
      stopPolling();
      setUiStatus((prev) => (prev !== 'paid' && prev !== 'failed' ? 'timeout' : prev));
    }, TIMEOUT_MS);

    return stopPolling;
  }, [sessionId]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <Container maxWidth="sm" sx={{ py: { xs: 6, md: 10 } }}>
      <Paper elevation={3} sx={{ p: { xs: 3, md: 5 } }}>

        {/* LOADING */}
        {uiStatus === 'loading' && (
          <Box sx={{ textAlign: 'center', py: 4 }}>
            <CircularProgress sx={{ mb: 3 }} />
            <Typography variant="h6">Connecting to your server…</Typography>
          </Box>
        )}

        {/* PAID / SUCCESS */}
        {uiStatus === 'paid' && (
          <Box sx={{ textAlign: 'center', py: 2 }}>
            <CheckCircleOutline color="success" sx={{ fontSize: 48, mb: 2 }} />
            <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>
              Payment confirmed!
            </Typography>
            <Alert severity="success" sx={{ mb: 3, textAlign: 'left' }}>
              Your Mahalaxmi Cloud server is being provisioned. Head to your dashboard to track
              its status — it will be ready in 1–3 minutes.
            </Alert>
            <Button variant="contained" href="/dashboard/servers" size="large">
              Go to My Servers
            </Button>
          </Box>
        )}

        {/* PROVISIONING */}
        {uiStatus === 'provisioning' && (
          <Box>
            <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>
              Payment confirmed — provisioning your server
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 4 }}>
              Your dedicated Mahalaxmi Cloud server is being configured. This usually takes under 3 minutes.
            </Typography>
            <LinearProgress sx={{ mb: 2, borderRadius: 1 }} />
            <Typography variant="caption" color="text.secondary">
              Your server is being configured… this page will update automatically.
            </Typography>
          </Box>
        )}

        {/* PENDING PAYMENT */}
        {uiStatus === 'pending_payment' && (
          <Box sx={{ textAlign: 'center', py: 2 }}>
            <CircularProgress sx={{ mb: 3 }} />
            <Typography variant="h6" sx={{ mb: 1 }}>Waiting for payment confirmation</Typography>
            <Typography variant="body2" color="text.secondary">
              Your payment is being processed. This page will update automatically once confirmed.
            </Typography>
            <LinearProgress sx={{ mt: 3, borderRadius: 1 }} />
          </Box>
        )}

        {/* FAILED */}
        {uiStatus === 'failed' && (
          <Box sx={{ textAlign: 'center', py: 2 }}>
            <ErrorOutline color="error" sx={{ fontSize: 48, mb: 2 }} />
            <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>
              Provisioning failed
            </Typography>
            <Alert severity="error" sx={{ mb: 3, textAlign: 'left' }}>
              Something went wrong while setting up your server. Your payment has not been charged.
              Please contact support and we&apos;ll get you sorted immediately.
            </Alert>
            <Button
              variant="contained"
              href="mailto:support@mahalaxmi.ai"
              component="a"
            >
              support@mahalaxmi.ai
            </Button>
          </Box>
        )}

        {/* TIMEOUT */}
        {uiStatus === 'timeout' && (
          <Box sx={{ textAlign: 'center', py: 2 }}>
            <ErrorOutline color="warning" sx={{ fontSize: 48, mb: 2 }} />
            <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>
              Taking longer than expected
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
              Your server is taking longer than usual to provision. Please check your dashboard
              or contact support if this persists.
            </Typography>
            <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center' }}>
              <Button variant="contained" href="/dashboard/servers">
                Go to My Servers
              </Button>
              <Button
                variant="outlined"
                href="mailto:support@mahalaxmi.ai"
                component="a"
              >
                support@mahalaxmi.ai
              </Button>
            </Box>
          </Box>
        )}

      </Paper>
    </Container>
  );
}

'use client';

import { useEffect, useRef, useState } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import {
  Box,
  Button,
  Chip,
  CircularProgress,
  Container,
  LinearProgress,
  Paper,
  Typography,
} from '@mui/material';
import { CheckCircleOutline, ErrorOutline, OpenInNew } from '@mui/icons-material';
import { useAuth } from '@/contexts/AuthContext';
import { PROVIDER_LABELS, TIER_LABELS } from '@/lib/cloudConstants';

const POLL_INTERVAL_MS = 3_000;
const POLL_TIMEOUT_MS  = 10 * 60 * 1_000; // 10 minutes
const REDIRECT_DELAY_S = 5;
const DASHBOARD_URL    = '/dashboard/servers';

// ProviderBadge — reads from the shared contract-locked PROVIDER_LABELS map
function ProviderBadge({ provider }) {
  if (!provider) return null;
  const cfg = PROVIDER_LABELS[provider] ?? { name: provider, color: 'grey' };
  return (
    <Chip
      label={cfg.name}
      size="small"
      sx={{ bgcolor: cfg.color, color: 'white', fontWeight: 700, fontSize: '0.7rem', height: 22 }}
    />
  );
}

export default function MahalaxmiCheckoutSuccessContent() {
  const searchParams = useSearchParams();
  const router       = useRouter();
  const sessionId    = searchParams.get('session_id');

  const { isAuthenticated, isLoading: authLoading, user } = useAuth();

  // 'init' | 'provisioning' | 'active' | 'failed' | 'timeout'
  const [uiStatus, setUiStatus] = useState('init');
  const [sessionData, setSessionData] = useState(null);   // active/failed response body
  const [progressPct, setProgressPct] = useState(0);
  const [countdown, setCountdown] = useState(REDIRECT_DELAY_S);

  const pollRef          = useRef(null);
  const timeoutRef       = useRef(null);
  const progressRef      = useRef(null);
  const countdownRef     = useRef(null);
  const provisionStartMs = useRef(null);
  const estimatedSecRef  = useRef(300); // default 5 min until first response updates it
  const isMounted        = useRef(true);

  function stopAll() {
    [pollRef, timeoutRef, progressRef, countdownRef].forEach((r) => {
      if (r.current) { clearInterval(r.current); clearTimeout(r.current); r.current = null; }
    });
  }

  function startProgressTimer() {
    if (progressRef.current) return; // already running
    if (provisionStartMs.current === null) provisionStartMs.current = Date.now();

    progressRef.current = setInterval(() => {
      if (!isMounted.current) return;
      const elapsed = (Date.now() - provisionStartMs.current) / 1000;
      const pct = Math.min(95, (elapsed / estimatedSecRef.current) * 100);
      setProgressPct(pct);
    }, 500);
  }

  function startCountdown() {
    let remaining = REDIRECT_DELAY_S;
    countdownRef.current = setInterval(() => {
      remaining -= 1;
      if (isMounted.current) setCountdown(remaining);
      if (remaining <= 0) {
        clearInterval(countdownRef.current);
        countdownRef.current = null;
        router.replace(DASHBOARD_URL);
      }
    }, 1_000);
  }

  async function poll() {
    if (!isMounted.current) return;
    try {
      const res = await fetch(
        `/api/mahalaxmi/checkout/session/${sessionId}`,
        {
          cache: 'no-store',
          headers: {
            ...(user?.id    ? { 'x-user-id':    user.id    } : {}),
            ...(user?.email ? { 'x-user-email': user.email } : {}),
          },
        }
      );

      if (!isMounted.current) return;

      // 401 — lost auth mid-poll
      if (res.status === 401) {
        stopAll();
        router.replace(`/login?redirect=${encodeURIComponent(`/checkout/success?session_id=${sessionId}`)}`);
        return;
      }

      const data = await res.json();

      // 202 — still provisioning, keep polling
      if (res.status === 202) {
        if (data.estimated_ready_seconds) {
          estimatedSecRef.current = data.estimated_ready_seconds;
        }
        setUiStatus('provisioning');
        startProgressTimer();
        return;
      }

      // 200 — terminal state
      if (res.status === 200) {
        stopAll();
        if (data.status === 'active') {
          setProgressPct(100);
          setSessionData(data);
          setUiStatus('active');
          startCountdown();
        } else {
          // failed or unknown terminal
          setSessionData(data);
          setUiStatus('failed');
        }
        return;
      }

      // Non-202/200 — transient error, keep polling
    } catch {
      // Network error — keep polling
    }
  }

  // Auth gate + no session_id gate
  useEffect(() => {
    if (authLoading) return;

    if (!isAuthenticated) {
      const dest = sessionId
        ? `/login?redirect=${encodeURIComponent(`/checkout/success?session_id=${sessionId}`)}`
        : `/login?redirect=${encodeURIComponent(DASHBOARD_URL)}`;
      router.replace(dest);
      return;
    }

    if (!sessionId) {
      router.replace(DASHBOARD_URL);
      return;
    }

    // Start polling
    poll();
    pollRef.current = setInterval(poll, POLL_INTERVAL_MS);

    timeoutRef.current = setTimeout(() => {
      if (!isMounted.current) return;
      stopAll();
      setUiStatus((prev) => (prev === 'provisioning' || prev === 'init' ? 'timeout' : prev));
    }, POLL_TIMEOUT_MS);

    return () => {
      isMounted.current = false;
      stopAll();
    };
  }, [authLoading, isAuthenticated]); // eslint-disable-line react-hooks/exhaustive-deps

  const tierLabel     = sessionData?.tier     ? (TIER_LABELS[sessionData.tier]     ?? sessionData.tier)     : null;
  const providerLabel = sessionData?.cloud_provider
    ? (PROVIDER_LABELS[sessionData.cloud_provider]?.name ?? sessionData.cloud_provider)
    : null;

  return (
    <Container maxWidth="sm" sx={{ py: { xs: 6, md: 10 } }}>
      <Paper elevation={3} sx={{ p: { xs: 3, md: 5 } }}>

        {/* AUTH LOADING */}
        {(authLoading || uiStatus === 'init') && (
          <Box sx={{ textAlign: 'center', py: 4 }}>
            <CircularProgress sx={{ mb: 3 }} />
            <Typography variant="h6" color="text.secondary">Loading…</Typography>
          </Box>
        )}

        {/* PROVISIONING — 202 responses */}
        {uiStatus === 'provisioning' && (
          <Box>
            <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>
              Setting up your {tierLabel ?? 'cloud'} server…
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 4 }}>
              Your dedicated Mahalaxmi Cloud server is being configured. This usually takes
              under 3 minutes. This page updates automatically.
            </Typography>
            <LinearProgress
              variant="determinate"
              value={progressPct}
              sx={{ mb: 1, borderRadius: 1, height: 8 }}
            />
            <Typography variant="caption" color="text.secondary">
              {Math.round(progressPct)}% — please keep this tab open
            </Typography>
          </Box>
        )}

        {/* ACTIVE — server ready */}
        {uiStatus === 'active' && sessionData && (
          <Box sx={{ textAlign: 'center', py: 2 }}>
            <CheckCircleOutline color="success" sx={{ fontSize: 64, mb: 2 }} />
            <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>
              Your {tierLabel ?? 'cloud'} server is ready
            </Typography>

            {/* Provider badge */}
            {sessionData.cloud_provider && (
              <Box sx={{ mb: 2 }}>
                <ProviderBadge provider={sessionData.cloud_provider} />
              </Box>
            )}

            <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
              {sessionData.endpoint && (
                <Box component="span" sx={{ fontFamily: 'monospace', display: 'block', mb: 1 }}>
                  {sessionData.endpoint}
                </Box>
              )}
              Redirecting to dashboard in {countdown}s…
            </Typography>

            <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', mt: 3, flexWrap: 'wrap' }}>
              {/* deep_link used as href — not constructed, not window.open */}
              {sessionData.deep_link && (
                <Button
                  variant="contained"
                  size="large"
                  startIcon={<OpenInNew />}
                  href={sessionData.deep_link}
                >
                  Open in VS Code
                </Button>
              )}
              <Button variant="outlined" size="large" href={DASHBOARD_URL}>
                Go to Dashboard
              </Button>
            </Box>
          </Box>
        )}

        {/* FAILED */}
        {uiStatus === 'failed' && (
          <Box sx={{ textAlign: 'center', py: 2 }}>
            <ErrorOutline color="error" sx={{ fontSize: 48, mb: 2 }} />
            <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>
              Something went wrong setting up your server
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
              Your card has not been charged.
            </Typography>
            {sessionData?.project_id && (
              <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                Contact{' '}
                <a href="mailto:support@mahalaxmi.ai" style={{ color: 'inherit' }}>
                  support@mahalaxmi.ai
                </a>
                {' '}with your order ID:{' '}
                <Box component="span" sx={{ fontFamily: 'monospace', fontWeight: 600 }}>
                  {sessionData.project_id}
                </Box>
              </Typography>
            )}
            {!sessionData?.project_id && (
              <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                Contact{' '}
                <a href="mailto:support@mahalaxmi.ai" style={{ color: 'inherit' }}>
                  support@mahalaxmi.ai
                </a>{' '}
                and we&apos;ll get you sorted immediately.
              </Typography>
            )}
          </Box>
        )}

        {/* TIMEOUT — still pending after 10 min */}
        {uiStatus === 'timeout' && (
          <Box sx={{ textAlign: 'center', py: 2 }}>
            <ErrorOutline color="warning" sx={{ fontSize: 48, mb: 2 }} />
            <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>
              Taking longer than expected
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
              Your payment may still be processing. Check your dashboard in a few minutes or contact{' '}
              <a href="mailto:support@mahalaxmi.ai" style={{ color: 'inherit' }}>
                support@mahalaxmi.ai
              </a>{' '}
              if this persists.
            </Typography>
            <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
              <Button variant="contained" href={DASHBOARD_URL}>Check Dashboard</Button>
              <Button variant="outlined" href="mailto:support@mahalaxmi.ai">Contact Support</Button>
            </Box>
          </Box>
        )}

      </Paper>
    </Container>
  );
}

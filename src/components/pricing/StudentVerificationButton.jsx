'use client';

import { useState, useEffect } from 'react';
import { Button, Alert, CircularProgress } from '@mui/material';
import { School, CheckCircle, HourglassEmpty } from '@mui/icons-material';
import Link from 'next/link';
import { useAuth } from '@/contexts/AuthContext';

export default function StudentVerificationButton({
  tierId = 'student',
  verifiedCtaHref = '/cloud/pricing',
  verifiedCtaLabel = 'Buy Now — Student Pricing',
}) {
  const { user, loading: authLoading } = useAuth();
  const [loading, setLoading] = useState(false);
  const [verificationState, setVerificationState] = useState('idle'); // idle | approved | pending
  const [statusChecked, setStatusChecked] = useState(false);

  // Check verification status on mount — skip button entirely if already verified
  useEffect(() => {
    if (authLoading) return;
    fetch('/api/mahalaxmi/verification/status', { credentials: 'include' })
      .then(res => (res.ok ? res.json() : null))
      .then(data => { if (data?.verified) setVerificationState('approved'); })
      .catch(() => {})
      .finally(() => setStatusChecked(true));
  }, [authLoading]);

  async function handleApply() {
    setLoading(true);
    try {
      // Check current status first
      const statusRes = await fetch('/api/mahalaxmi/verification/status', { credentials: 'include' });
      const status = await statusRes.json();
      if (status.verified) {
        setVerificationState('approved');
        setLoading(false);
        return;
      }

      // Submit verification application
      const applyRes = await fetch('/api/mahalaxmi/verification/apply', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ tier_id: tierId }),
      });
      const result = await applyRes.json();

      if (result.status === 'approved') {
        setVerificationState('approved');
      } else if (result.status === 'pending') {
        setVerificationState('pending');
      }
    } catch {
      // network error — stay in idle so user can retry
    }
    setLoading(false);
  }

  // Spinner while auth is hydrating or initial status check is in flight
  if (authLoading || !statusChecked) {
    return <CircularProgress size={20} sx={{ display: 'block', mx: 'auto', my: 1.5 }} />;
  }

  if (verificationState === 'approved') {
    return (
      <>
        <Alert severity="success" icon={<CheckCircle />} sx={{ mb: 2, fontSize: '0.8rem' }}>
          Your student status has been verified. You can now purchase at the student price.
        </Alert>
        <Button component={Link} href={verifiedCtaHref} variant="outlined" fullWidth sx={{ mb: 3 }}>
          {verifiedCtaLabel}
        </Button>
      </>
    );
  }

  if (verificationState === 'pending') {
    return (
      <>
        <Alert severity="info" icon={<HourglassEmpty />} sx={{ mb: 2, fontSize: '0.8rem' }}>
          We have received your request. Please email a photo of your student ID to{' '}
          <strong>support@mahalaxmi.ai</strong> with subject{' '}
          <strong>Student Verification — {user?.email ?? 'your account email'}</strong>. We will
          review within 48 hours.
        </Alert>
        <Button variant="outlined" fullWidth disabled sx={{ mb: 3 }}>
          Verification pending
        </Button>
      </>
    );
  }

  return (
    <Button
      variant="outlined"
      fullWidth
      startIcon={loading ? <CircularProgress size={18} color="inherit" /> : <School />}
      onClick={handleApply}
      disabled={loading}
      sx={{ mb: 3 }}
    >
      {loading ? 'Submitting…' : 'Apply for Student Pricing'}
    </Button>
  );
}

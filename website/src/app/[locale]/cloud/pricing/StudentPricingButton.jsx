'use client';

import { useState } from 'react';
import { Alert, Box, Button, CircularProgress } from '@mui/material';
import { School } from '@mui/icons-material';
import { useAuth } from '@/contexts/AuthContext';

export default function StudentPricingButton({ tierId, variant = 'outlined', onVerified }) {
  const { user } = useAuth();
  const [loading, setLoading] = useState(false);
  const [verificationState, setVerificationState] = useState(null); // null | 'pending'

  async function handleApply() {
    setLoading(true);

    try {
      const statusRes = await fetch('/api/mahalaxmi/verification/status');
      const status = await statusRes.json();

      if (status.verified) {
        onVerified();
        return;
      }

      const applyRes = await fetch('/api/mahalaxmi/verification/apply', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ tier_id: tierId }),
      });
      const result = await applyRes.json();

      if (result.status === 'approved') {
        setVerificationState('approved');
        onVerified();
      } else if (result.status === 'pending') {
        setVerificationState('pending');
      }
    } catch {
      // Network error — leave in default state so user can retry
    } finally {
      setLoading(false);
    }
  }

  if (verificationState === 'pending') {
    const email = user?.email ?? '';
    const subject = `Student Verification — ${email}`;
    return (
      <Box sx={{ mb: 2 }}>
        <Alert severity="info" sx={{ mb: 1.5, fontSize: '0.8rem' }}>
          We have received your request. Please email a photo of your student ID to{' '}
          <strong>support@mahalaxmi.ai</strong> with subject{' '}
          <strong>{subject}</strong>. We will review within 48 hours.
        </Alert>
        <Button variant={variant} fullWidth disabled startIcon={<School />} sx={{ textTransform: 'none' }}>
          Verification pending
        </Button>
      </Box>
    );
  }

  return (
    <Button
      variant={variant}
      fullWidth
      startIcon={loading ? <CircularProgress size={16} color="inherit" /> : <School />}
      onClick={handleApply}
      disabled={loading}
      sx={{ mb: variant === 'contained' ? 3 : 2.5, textTransform: 'none' }}
    >
      {loading ? 'Checking…' : 'Apply for Student Pricing'}
    </Button>
  );
}

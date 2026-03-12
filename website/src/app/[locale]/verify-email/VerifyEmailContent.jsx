'use client';

import { useState, useEffect } from 'react';
import {
  Container, Paper, Typography, Box, Button, CircularProgress, Alert,
} from '@mui/material';
import { CheckCircleOutline, ErrorOutline } from '@mui/icons-material';
import { Link } from '@/i18n/navigation';
import { useSearchParams } from 'next/navigation';

export default function VerifyEmailContent() {
  const searchParams = useSearchParams();
  const [status, setStatus] = useState('verifying');
  const [error, setError] = useState('');

  const token = searchParams.get('token');

  useEffect(() => {
    const verify = async () => {
      if (!token) {
        setStatus('error');
        setError('Invalid verification link. No token provided.');
        return;
      }

      try {
        const res = await fetch(`/api/auth/verify-email?token=${encodeURIComponent(token)}`);
        const data = await res.json();
        if (data.success) {
          setStatus('success');
        } else {
          setStatus('error');
          setError(data.message || 'Failed to verify email. The link may have expired.');
        }
      } catch {
        setStatus('error');
        setError('Failed to verify email. Please try again.');
      }
    };

    verify();
  }, [token]);

  if (status === 'verifying') {
    return (
      <Container maxWidth="sm" sx={{ py: 8 }}>
        <Paper elevation={3} sx={{ p: 4, textAlign: 'center' }}>
          <CircularProgress size={60} sx={{ mb: 3 }} />
          <Typography variant="h5" gutterBottom>Verifying Your Email</Typography>
          <Typography variant="body2" color="text.secondary">
            Please wait while we verify your email address…
          </Typography>
        </Paper>
      </Container>
    );
  }

  if (status === 'success') {
    return (
      <Container maxWidth="sm" sx={{ py: 8 }}>
        <Paper elevation={3} sx={{ p: 4, textAlign: 'center' }}>
          <CheckCircleOutline sx={{ fontSize: 80, color: 'success.main', mb: 2 }} />
          <Typography variant="h4" component="h1" gutterBottom>Email Verified!</Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
            Your email has been verified. You can now sign in to your account.
          </Typography>
          <Button component={Link} href="/login" variant="contained" size="large" fullWidth>
            Sign In
          </Button>
        </Paper>
      </Container>
    );
  }

  return (
    <Container maxWidth="sm" sx={{ py: 8 }}>
      <Paper elevation={3} sx={{ p: 4, textAlign: 'center' }}>
        <ErrorOutline sx={{ fontSize: 80, color: 'error.main', mb: 2 }} />
        <Typography variant="h4" component="h1" gutterBottom>Verification Failed</Typography>
        <Alert severity="error" sx={{ mb: 3, textAlign: 'left' }}>{error}</Alert>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 4 }}>
          The verification link may have expired or already been used. You can request a new one.
        </Typography>
        <Box sx={{ display: 'flex', gap: 2, flexDirection: 'column' }}>
          <Button component={Link} href="/resend-verification" variant="contained" size="large" fullWidth>
            Resend Verification Email
          </Button>
          <Button component={Link} href="/login" variant="outlined" size="large" fullWidth>
            Go to Login
          </Button>
        </Box>
      </Paper>
    </Container>
  );
}

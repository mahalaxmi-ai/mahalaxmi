'use client';

import { useState } from 'react';
import {
  Container, Paper, TextField, Button, Typography, Box, Alert,
  Link as MuiLink,
} from '@mui/material';
import { MarkEmailRead } from '@mui/icons-material';
import { Link } from '@/i18n/navigation';

export default function ForgotPasswordContent() {
  const [email, setEmail] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);

    try {
      const res = await fetch('/api/auth/forgot-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email }),
      });
      const data = await res.json();
      if (data.success !== false) {
        // Always show success to prevent email enumeration
        setSubmitted(true);
      } else {
        setError(data.message || 'Request failed. Please try again.');
      }
    } catch {
      setError('An unexpected error occurred. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  if (submitted) {
    return (
      <Container maxWidth="sm" sx={{ py: 8 }}>
        <Paper elevation={3} sx={{ p: 4, textAlign: 'center' }}>
          <MarkEmailRead sx={{ fontSize: 80, color: 'primary.main', mb: 2 }} />
          <Typography variant="h4" component="h1" gutterBottom>Check Your Email</Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
            If an account exists for <strong>{email}</strong>, we&apos;ve sent a password reset link.
            The link expires in 1 hour.
          </Typography>
          <Button component={Link} href="/login" variant="contained" size="large" fullWidth>
            Back to Sign In
          </Button>
        </Paper>
      </Container>
    );
  }

  return (
    <Container maxWidth="sm" sx={{ py: 8 }}>
      <Paper elevation={3} sx={{ p: 4 }}>
        <Box sx={{ textAlign: 'center', mb: 4 }}>
          <Typography variant="h4" component="h1" gutterBottom>Forgot Password?</Typography>
          <Typography variant="body2" color="text.secondary">
            Enter your email and we&apos;ll send you a reset link.
          </Typography>
        </Box>

        {error && <Alert severity="error" sx={{ mb: 3 }}>{error}</Alert>}

        <form onSubmit={handleSubmit}>
          <TextField
            fullWidth
            label="Email Address"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
            autoComplete="email"
            sx={{ mb: 3 }}
          />
          <Button
            type="submit"
            fullWidth
            variant="contained"
            size="large"
            disabled={isLoading}
            sx={{ mb: 3 }}
          >
            {isLoading ? 'Sending…' : 'Send Reset Link'}
          </Button>
        </form>

        <Box sx={{ textAlign: 'center' }}>
          <MuiLink component={Link} href="/login" variant="body2" color="primary">
            Back to Sign In
          </MuiLink>
        </Box>
      </Paper>
    </Container>
  );
}

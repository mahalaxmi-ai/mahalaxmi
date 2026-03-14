'use client';

import { useState } from 'react';
import {
  Container, Paper, TextField, Button, Typography, Box, Alert,
  Link as MuiLink, InputAdornment, IconButton,
} from '@mui/material';
import { Visibility, VisibilityOff } from '@mui/icons-material';
import { Link } from '@/i18n/navigation';
import { useRouter, useSearchParams } from 'next/navigation';
import { useAuth } from '@/contexts/AuthContext';

export default function LoginContent() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');
  const [errorCode, setErrorCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const { login } = useAuth();
  const router = useRouter();
  const searchParams = useSearchParams();

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setErrorCode('');
    setIsLoading(true);

    try {
      const result = await login(email, password);

      if (result.success) {
        const redirectPath = searchParams.get('redirect') || '/dashboard/servers';
        const tier = searchParams.get('tier');
        const billingCycle = searchParams.get('billing_cycle');
        const forwardParams = new URLSearchParams();
        if (tier) forwardParams.set('tier', tier);
        if (billingCycle) forwardParams.set('billing_cycle', billingCycle);
        const qs = forwardParams.toString();
        router.replace(qs ? `${redirectPath}?${qs}` : redirectPath);
      } else {
        setError(result.message);
        setErrorCode(result.code || '');
      }
    } catch {
      setError('An unexpected error occurred. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Container maxWidth="sm" sx={{ py: 8 }}>
      <Paper elevation={3} sx={{ p: 4 }}>
        <Box sx={{ textAlign: 'center', mb: 4 }}>
          <Typography variant="h4" component="h1" gutterBottom>
            Welcome Back
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Sign in to your Mahalaxmi account
          </Typography>
        </Box>

        {error && (
          <Alert severity={errorCode === 'email_not_verified' ? 'warning' : 'error'} sx={{ mb: 3 }}>
            {error}
            {errorCode === 'email_not_verified' && (
              <Box sx={{ mt: 1 }}>
                <MuiLink component={Link} href="/resend-verification" color="inherit" sx={{ fontWeight: 'medium' }}>
                  Resend verification email
                </MuiLink>
              </Box>
            )}
          </Alert>
        )}

        <form onSubmit={handleSubmit}>
          <TextField
            fullWidth
            label="Email Address"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
            autoComplete="email"
            InputLabelProps={{ shrink: !!email }}
            sx={{ mb: 3 }}
          />

          <TextField
            fullWidth
            label="Password"
            type={showPassword ? 'text' : 'password'}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            autoComplete="current-password"
            InputLabelProps={{ shrink: !!password }}
            sx={{ mb: 1 }}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton onClick={() => setShowPassword(!showPassword)} edge="end">
                    {showPassword ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />

          <Box sx={{ textAlign: 'right', mb: 3 }}>
            <MuiLink component={Link} href="/forgot-password" variant="body2" color="primary">
              Forgot password?
            </MuiLink>
          </Box>

          <Button
            type="submit"
            fullWidth
            variant="contained"
            size="large"
            disabled={isLoading}
            sx={{ mb: 3 }}
          >
            {isLoading ? 'Signing In…' : 'Sign In'}
          </Button>
        </form>

        <Box sx={{ textAlign: 'center' }}>
          <Typography variant="body2" color="text.secondary">
            Don&apos;t have an account?{' '}
            <MuiLink component={Link} href="/register" color="primary">
              Sign up here
            </MuiLink>
          </Typography>
        </Box>
      </Paper>
    </Container>
  );
}

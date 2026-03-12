'use client';

import { useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import Head from 'next/head';
import Container from '@mui/material/Container';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Typography from '@mui/material/Typography';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Alert from '@mui/material/Alert';
import CircularProgress from '@mui/material/CircularProgress';
import Link from '@mui/material/Link';
import Box from '@mui/material/Box';
import NextLink from 'next/link';
import { useAuth } from '@/contexts/AuthContext';

export default function LoginPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const redirectParam = searchParams.get('redirect') || '/dashboard/servers';
  const tierParam = searchParams.get('tier');

  const { login } = useAuth();

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      await login(email, password);
      const destination = tierParam
        ? `${redirectParam}?tier=${encodeURIComponent(tierParam)}`
        : redirectParam;
      router.push(destination);
    } catch (err) {
      if (err?.response?.status === 401) {
        setError({ type: 'invalid', message: 'Invalid email or password' });
      } else if (err?.response?.status === 403) {
        setError({ type: 'verify' });
      } else if (err?.request) {
        setError({ type: 'network', message: 'Login failed. Please try again.' });
      } else {
        setError({ type: 'network', message: 'Login failed. Please try again.' });
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <Head>
        <title>Log In | Mahalaxmi AI</title>
      </Head>
      <Container
        maxWidth="sm"
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '100vh',
          py: 4,
        }}
      >
        <Card sx={{ width: '100%', maxWidth: 440 }}>
          <CardContent sx={{ p: 4 }}>
            <Typography variant="h5" component="h1" fontWeight={700} gutterBottom>
              Log In
            </Typography>

            {error && error.type === 'invalid' && (
              <Alert severity="error" sx={{ mb: 2 }}>
                {error.message}
              </Alert>
            )}

            {error && error.type === 'verify' && (
              <Alert severity="warning" sx={{ mb: 2 }}>
                Please verify your email before logging in.{' '}
                <Link component={NextLink} href="/verify-email" underline="always">
                  Resend verification email
                </Link>
              </Alert>
            )}

            {error && error.type === 'network' && (
              <Alert severity="error" sx={{ mb: 2 }}>
                {error.message}
              </Alert>
            )}

            <Box component="form" onSubmit={handleSubmit} noValidate>
              <TextField
                label="Email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                fullWidth
                required
                margin="normal"
                autoComplete="email"
                inputProps={{ 'aria-label': 'Email' }}
              />
              <TextField
                label="Password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                fullWidth
                required
                margin="normal"
                autoComplete="current-password"
                inputProps={{ 'aria-label': 'Password' }}
              />

              <Button
                type="submit"
                fullWidth
                variant="contained"
                disabled={loading}
                sx={{
                  mt: 3,
                  mb: 2,
                  backgroundColor: '#00C8C8',
                  color: '#0A2A2A',
                  fontWeight: 700,
                  '&:hover': {
                    backgroundColor: '#00b0b0',
                  },
                  '&:disabled': {
                    backgroundColor: '#00C8C8',
                    opacity: 0.7,
                  },
                }}
              >
                {loading ? (
                  <CircularProgress size={24} sx={{ color: '#0A2A2A' }} />
                ) : (
                  'Log In'
                )}
              </Button>
            </Box>

            <Box sx={{ display: 'flex', justifyContent: 'space-between', mt: 1 }}>
              <Link component={NextLink} href="/forgot-password" underline="hover" variant="body2">
                Forgot password?
              </Link>
              <Link component={NextLink} href="/register" underline="hover" variant="body2">
                Create account
              </Link>
            </Box>
          </CardContent>
        </Card>
      </Container>
    </>
  );
}

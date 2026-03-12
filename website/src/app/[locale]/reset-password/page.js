'use client';

import { useState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import axios from 'axios';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import Alert from '@mui/material/Alert';
import CircularProgress from '@mui/material/CircularProgress';
import Link from 'next/link';
import Image from 'next/image';

export default function ResetPasswordPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [token, setToken] = useState('');
  const [form, setForm] = useState({ password: '', confirmPassword: '' });
  const [error, setError] = useState('');
  const [success, setSuccess] = useState(false);
  const [loading, setLoading] = useState(false);
  const [tokenMissing, setTokenMissing] = useState(false);

  useEffect(() => {
    const t = searchParams.get('token');
    if (!t) {
      setTokenMissing(true);
    } else {
      setToken(t);
    }
  }, [searchParams]);

  function handleChange(e) {
    setForm((prev) => ({ ...prev, [e.target.name]: e.target.value }));
  }

  async function handleSubmit(e) {
    e.preventDefault();
    setError('');

    if (form.password !== form.confirmPassword) {
      setError('Passwords do not match.');
      return;
    }

    if (form.password.length < 8) {
      setError('Password must be at least 8 characters.');
      return;
    }

    setLoading(true);
    try {
      await axios.post('/api/auth/reset-password', {
        password: form.password,
        token,
      });
      setSuccess(true);
      setTimeout(() => {
        router.push('/login');
      }, 2000);
    } catch (err) {
      const status = err?.response?.status;
      if (status === 400 || status === 410) {
        setError('This reset link is invalid or has expired.');
      } else {
        const message =
          err?.response?.data?.message ||
          err?.response?.data?.error ||
          'Failed to reset password. Please try again.';
        setError(message);
      }
    } finally {
      setLoading(false);
    }
  }

  if (tokenMissing) {
    return (
      <Box
        sx={{
          minHeight: '100vh',
          backgroundColor: '#0A2A2A',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          px: 2,
        }}
      >
        <Card
          sx={{
            width: '100%',
            maxWidth: 440,
            backgroundColor: '#0D3333',
            border: '1px solid #1a4a4a',
            borderRadius: 2,
          }}
        >
          <CardContent sx={{ p: 4, textAlign: 'center' }}>
            <Box sx={{ display: 'flex', justifyContent: 'center', mb: 3 }}>
              <Image
                src="/mahalaxmi_logo.png"
                alt="Mahalaxmi"
                width={140}
                height={48}
                style={{ objectFit: 'contain' }}
                onError={(e) => { e.currentTarget.style.display = 'none'; }}
              />
            </Box>
            <Alert severity="error" sx={{ mb: 3 }}>
              This reset link is invalid or has expired.
            </Alert>
            <Link href="/forgot-password" style={{ color: '#00C8C8', textDecoration: 'none' }}>
              Request a new password reset link
            </Link>
          </CardContent>
        </Card>
      </Box>
    );
  }

  return (
    <Box
      sx={{
        minHeight: '100vh',
        backgroundColor: '#0A2A2A',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        px: 2,
      }}
    >
      <Card
        sx={{
          width: '100%',
          maxWidth: 440,
          backgroundColor: '#0D3333',
          border: '1px solid #1a4a4a',
          borderRadius: 2,
        }}
      >
        <CardContent sx={{ p: 4 }}>
          <Box sx={{ display: 'flex', justifyContent: 'center', mb: 3 }}>
            <Image
              src="/mahalaxmi_logo.png"
              alt="Mahalaxmi"
              width={140}
              height={48}
              style={{ objectFit: 'contain' }}
              onError={(e) => { e.currentTarget.style.display = 'none'; }}
            />
          </Box>

          <Typography variant="h5" fontWeight={700} color="#fff" textAlign="center" mb={1}>
            Set a new password
          </Typography>
          <Typography variant="body2" color="text.secondary" textAlign="center" mb={3}>
            Choose a strong password for your account.
          </Typography>

          {success && (
            <Alert severity="success" sx={{ mb: 2 }}>
              Password reset successful! Redirecting to sign in&hellip;
            </Alert>
          )}

          {error && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {error}
              {(error.includes('invalid') || error.includes('expired')) && (
                <>
                  {' '}
                  <Link href="/forgot-password" style={{ color: '#00C8C8' }}>
                    Request a new link.
                  </Link>
                </>
              )}
            </Alert>
          )}

          {!success && (
            <Box component="form" onSubmit={handleSubmit} noValidate>
              <TextField
                name="password"
                type="password"
                placeholder="New password"
                value={form.password}
                onChange={handleChange}
                required
                fullWidth
                size="small"
                sx={{ ...textFieldSx, mb: 2 }}
                inputProps={{ 'aria-label': 'New password' }}
              />
              <TextField
                name="confirmPassword"
                type="password"
                placeholder="Confirm new password"
                value={form.confirmPassword}
                onChange={handleChange}
                required
                fullWidth
                size="small"
                sx={{ ...textFieldSx, mb: 3 }}
                inputProps={{ 'aria-label': 'Confirm new password' }}
              />
              <Button
                type="submit"
                variant="contained"
                fullWidth
                disabled={loading}
                sx={{
                  backgroundColor: '#00C8C8',
                  color: '#0A2A2A',
                  fontWeight: 700,
                  py: 1.25,
                  '&:hover': { backgroundColor: '#00a8a8' },
                  '&:disabled': { backgroundColor: '#005555', color: '#aaa' },
                }}
              >
                {loading ? <CircularProgress size={20} sx={{ color: '#0A2A2A' }} /> : 'Reset password'}
              </Button>
            </Box>
          )}

          <Typography variant="body2" color="text.secondary" textAlign="center" mt={3}>
            Remember your password?{' '}
            <Link href="/login" style={{ color: '#00C8C8', textDecoration: 'none' }}>
              Sign in
            </Link>
          </Typography>
        </CardContent>
      </Card>
    </Box>
  );
}

const textFieldSx = {
  '& .MuiInputBase-root': {
    backgroundColor: '#0A2A2A',
    color: '#fff',
  },
  '& .MuiInputBase-input::placeholder': {
    color: '#6b9999',
    opacity: 1,
  },
  '& .MuiOutlinedInput-notchedOutline': {
    borderColor: '#1a4a4a',
  },
  '& .MuiOutlinedInput-root:hover .MuiOutlinedInput-notchedOutline': {
    borderColor: '#00C8C8',
  },
  '& .MuiOutlinedInput-root.Mui-focused .MuiOutlinedInput-notchedOutline': {
    borderColor: '#00C8C8',
  },
};

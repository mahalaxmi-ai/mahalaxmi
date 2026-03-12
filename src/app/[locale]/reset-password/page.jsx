'use client';
import { useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import {
  Box,
  Button,
  TextField,
  Typography,
  Alert,
  CircularProgress,
  Link as MuiLink,
} from '@mui/material';
import NextLink from 'next/link';
import axios from 'axios';

export default function ResetPasswordPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const token = searchParams.get('token') || '';

  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');

    if (password !== confirm) {
      setError('Passwords do not match.');
      return;
    }

    if (!token) {
      setError('Reset token is missing. Please use the link from your email.');
      return;
    }

    setLoading(true);
    try {
      await axios.post('/api/auth/reset-password', { token, password });
      router.push('/login');
    } catch (err) {
      const msg =
        err?.response?.data?.error ||
        err?.message ||
        'Password reset failed. Please try again.';
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

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
      <Box
        component="form"
        onSubmit={handleSubmit}
        sx={{
          width: '100%',
          maxWidth: 420,
          backgroundColor: 'rgba(255,255,255,0.04)',
          border: '1px solid rgba(0,200,200,0.2)',
          borderRadius: 2,
          p: 4,
        }}
      >
        <Typography variant="h5" sx={{ color: '#00C8C8', mb: 3, fontWeight: 700 }}>
          Reset Password
        </Typography>

        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        <TextField
          fullWidth
          label="New Password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
          sx={{ mb: 2 }}
          InputLabelProps={{ style: { color: '#aaa' } }}
          InputProps={{ style: { color: '#fff' } }}
        />

        <TextField
          fullWidth
          label="Confirm Password"
          type="password"
          value={confirm}
          onChange={(e) => setConfirm(e.target.value)}
          required
          sx={{ mb: 3 }}
          InputLabelProps={{ style: { color: '#aaa' } }}
          InputProps={{ style: { color: '#fff' } }}
        />

        <Button
          type="submit"
          fullWidth
          variant="contained"
          disabled={loading}
          sx={{
            backgroundColor: '#00C8C8',
            color: '#0A2A2A',
            fontWeight: 700,
            '&:hover': { backgroundColor: '#00b0b0' },
            mb: 2,
          }}
        >
          {loading ? <CircularProgress size={22} sx={{ color: '#0A2A2A' }} /> : 'Reset Password'}
        </Button>

        <Box sx={{ textAlign: 'center' }}>
          <MuiLink component={NextLink} href="/login" sx={{ color: '#00C8C8', fontSize: 14 }}>
            Back to Sign In
          </MuiLink>
        </Box>
      </Box>
    </Box>
  );
}

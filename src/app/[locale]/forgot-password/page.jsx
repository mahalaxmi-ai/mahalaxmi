'use client';
import { useState } from 'react';
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

export default function ForgotPasswordPage() {
  const [email, setEmail] = useState('');
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setSuccess('');
    setLoading(true);
    try {
      const res = await axios.post('/api/auth/forgot-password', { email });
      setSuccess(res.data.message || 'If that email exists, a reset link was sent.');
    } catch (err) {
      const msg =
        err?.response?.data?.error ||
        err?.message ||
        'Something went wrong. Please try again.';
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
        <Typography variant="h5" sx={{ color: '#00C8C8', mb: 1, fontWeight: 700 }}>
          Forgot Password
        </Typography>
        <Typography variant="body2" sx={{ color: '#aaa', mb: 3 }}>
          Enter your email and we&apos;ll send a reset link if the account exists.
        </Typography>

        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        {success && (
          <Alert severity="success" sx={{ mb: 2 }}>
            {success}
          </Alert>
        )}

        <TextField
          fullWidth
          label="Email"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
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
          {loading ? <CircularProgress size={22} sx={{ color: '#0A2A2A' }} /> : 'Send Reset Link'}
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

'use client';
import { useState } from 'react';
import { useRouter } from 'next/navigation';
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

export default function RegisterPage() {
  const router = useRouter();

  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setSuccess('');
    setLoading(true);
    try {
      const res = await axios.post('/api/auth/register', { name, email, password });
      setSuccess(res.data.message || 'Check your email to verify your account.');
    } catch (err) {
      const msg =
        err?.response?.data?.error ||
        err?.message ||
        'Registration failed. Please try again.';
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  if (success) {
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
        <Box sx={{ maxWidth: 420, width: '100%', textAlign: 'center' }}>
          <Alert severity="success" sx={{ mb: 3 }}>
            {success}
          </Alert>
          <MuiLink component={NextLink} href="/login" sx={{ color: '#00C8C8' }}>
            Go to Sign In
          </MuiLink>
        </Box>
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
          Create Account
        </Typography>

        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        <TextField
          fullWidth
          label="Name"
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
          sx={{ mb: 2 }}
          InputLabelProps={{ style: { color: '#aaa' } }}
          InputProps={{ style: { color: '#fff' } }}
        />

        <TextField
          fullWidth
          label="Email"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
          sx={{ mb: 2 }}
          InputLabelProps={{ style: { color: '#aaa' } }}
          InputProps={{ style: { color: '#fff' } }}
        />

        <TextField
          fullWidth
          label="Password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
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
          {loading ? <CircularProgress size={22} sx={{ color: '#0A2A2A' }} /> : 'Create Account'}
        </Button>

        <Box sx={{ textAlign: 'center' }}>
          <MuiLink component={NextLink} href="/login" sx={{ color: '#00C8C8', fontSize: 14 }}>
            Already have an account? Sign in
          </MuiLink>
        </Box>
      </Box>
    </Box>
  );
}

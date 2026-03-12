'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
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

export default function RegisterPage() {
  const router = useRouter();
  const [form, setForm] = useState({ firstName: '', lastName: '', email: '', password: '' });
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  function handleChange(e) {
    setForm((prev) => ({ ...prev, [e.target.name]: e.target.value }));
  }

  async function handleSubmit(e) {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      await axios.post('/api/auth/register', {
        firstName: form.firstName,
        lastName: form.lastName,
        email: form.email,
        password: form.password,
      });
      router.push(`/verify-email?email=${encodeURIComponent(form.email)}`);
    } catch (err) {
      const message =
        err?.response?.data?.message ||
        err?.response?.data?.error ||
        'Registration failed. Please try again.';
      setError(message);
    } finally {
      setLoading(false);
    }
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
            Create your account
          </Typography>
          <Typography variant="body2" color="text.secondary" textAlign="center" mb={3}>
            Join Mahalaxmi and start orchestrating AI workers.
          </Typography>

          {error && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {error}
            </Alert>
          )}

          <Box component="form" onSubmit={handleSubmit} noValidate>
            <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
              <TextField
                name="firstName"
                placeholder="First name"
                value={form.firstName}
                onChange={handleChange}
                required
                fullWidth
                size="small"
                inputProps={{ 'aria-label': 'First name' }}
                sx={textFieldSx}
              />
              <TextField
                name="lastName"
                placeholder="Last name"
                value={form.lastName}
                onChange={handleChange}
                required
                fullWidth
                size="small"
                inputProps={{ 'aria-label': 'Last name' }}
                sx={textFieldSx}
              />
            </Box>
            <TextField
              name="email"
              type="email"
              placeholder="Email address"
              value={form.email}
              onChange={handleChange}
              required
              fullWidth
              size="small"
              sx={{ ...textFieldSx, mb: 2 }}
              inputProps={{ 'aria-label': 'Email address' }}
            />
            <TextField
              name="password"
              type="password"
              placeholder="Password"
              value={form.password}
              onChange={handleChange}
              required
              fullWidth
              size="small"
              sx={{ ...textFieldSx, mb: 3 }}
              inputProps={{ 'aria-label': 'Password' }}
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
              {loading ? <CircularProgress size={20} sx={{ color: '#0A2A2A' }} /> : 'Create account'}
            </Button>
          </Box>

          <Typography variant="body2" color="text.secondary" textAlign="center" mt={3}>
            Already have an account?{' '}
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

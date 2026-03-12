'use client';

import { useState } from 'react';
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

export default function ForgotPasswordPage() {
  const [email, setEmail] = useState('');
  const [submitted, setSubmitted] = useState(false);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e) {
    e.preventDefault();
    setLoading(true);
    try {
      await axios.post('/api/auth/forgot-password', { email });
    } catch {
    } finally {
      setLoading(false);
      setSubmitted(true);
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
            Reset your password
          </Typography>
          <Typography variant="body2" color="text.secondary" textAlign="center" mb={3}>
            Enter your email and we&apos;ll send you a reset link.
          </Typography>

          {submitted ? (
            <Alert severity="success" sx={{ mb: 2 }}>
              Check your email for a reset link.
            </Alert>
          ) : (
            <Box component="form" onSubmit={handleSubmit} noValidate>
              <TextField
                type="email"
                placeholder="Email address"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                fullWidth
                size="small"
                sx={{ ...textFieldSx, mb: 3 }}
                inputProps={{ 'aria-label': 'Email address' }}
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
                {loading ? <CircularProgress size={20} sx={{ color: '#0A2A2A' }} /> : 'Send reset link'}
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

'use client';

import { useState } from 'react';
import { useSearchParams } from 'next/navigation';
import axios from 'axios';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import Alert from '@mui/material/Alert';
import CircularProgress from '@mui/material/CircularProgress';
import Image from 'next/image';

export default function VerifyEmailPage() {
  const searchParams = useSearchParams();
  const email = searchParams.get('email') || '';
  const [resendStatus, setResendStatus] = useState(null);
  const [loading, setLoading] = useState(false);

  async function handleResend() {
    if (!email) {
      setResendStatus({ type: 'error', message: 'No email address found. Please register again.' });
      return;
    }
    setResendStatus(null);
    setLoading(true);
    try {
      await axios.post('/api/auth/resend-verification', { email });
      setResendStatus({ type: 'success', message: 'Verification email sent! Please check your inbox.' });
    } catch (err) {
      const message =
        err?.response?.data?.message ||
        err?.response?.data?.error ||
        'Failed to resend verification email. Please try again.';
      setResendStatus({ type: 'error', message });
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

          <Typography variant="h5" fontWeight={700} color="#fff" mb={2}>
            Check your email
          </Typography>
          <Typography variant="body1" color="text.secondary" mb={1}>
            Check your email for a verification link.
          </Typography>
          {email && (
            <Typography variant="body2" color="#00C8C8" mb={3}>
              Sent to: {email}
            </Typography>
          )}

          {resendStatus && (
            <Alert severity={resendStatus.type} sx={{ mb: 2, textAlign: 'left' }}>
              {resendStatus.message}
            </Alert>
          )}

          <Typography variant="body2" color="text.secondary" mb={2}>
            Didn&apos;t receive the email?
          </Typography>
          <Button
            variant="outlined"
            onClick={handleResend}
            disabled={loading}
            sx={{
              borderColor: '#00C8C8',
              color: '#00C8C8',
              fontWeight: 600,
              '&:hover': { borderColor: '#00a8a8', color: '#00a8a8', backgroundColor: 'transparent' },
              '&:disabled': { borderColor: '#005555', color: '#005555' },
            }}
          >
            {loading ? <CircularProgress size={20} sx={{ color: '#00C8C8' }} /> : 'Resend verification email'}
          </Button>
        </CardContent>
      </Card>
    </Box>
  );
}

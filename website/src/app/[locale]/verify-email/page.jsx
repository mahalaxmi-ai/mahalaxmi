'use client';

import { useState, Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import Head from 'next/head';
import Container from '@mui/material/Container';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import EmailOutlinedIcon from '@mui/icons-material/EmailOutlined';
import Link from 'next/link';

function VerifyEmailContent() {
  const searchParams = useSearchParams();
  const emailFromParams = searchParams.get('email') || '';

  const [resendStatus, setResendStatus] = useState(null);
  const [loading, setLoading] = useState(false);

  async function handleResend() {
    setResendStatus(null);
    setLoading(true);
    try {
      const res = await fetch('/api/auth/resend-verification', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: emailFromParams }),
      });

      if (res.ok) {
        setResendStatus('success');
      } else if (res.status >= 500) {
        setResendStatus('server-error');
      } else {
        setResendStatus('error');
      }
    } catch {
      setResendStatus('network-error');
    } finally {
      setLoading(false);
    }
  }

  function getResendAlert() {
    if (resendStatus === 'success') {
      return <Alert severity="success" sx={{ mb: 3 }}>Verification email sent!</Alert>;
    }
    if (resendStatus === 'server-error') {
      return <Alert severity="error" sx={{ mb: 3 }}>A server error occurred. Please try again later.</Alert>;
    }
    if (resendStatus === 'error') {
      return <Alert severity="error" sx={{ mb: 3 }}>Failed to resend verification email. Please try again.</Alert>;
    }
    if (resendStatus === 'network-error') {
      return <Alert severity="error" sx={{ mb: 3 }}>Unable to connect. Please check your connection and try again.</Alert>;
    }
    return null;
  }

  return (
    <Container maxWidth="sm" sx={{ py: 10, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
      <Card sx={{ width: '100%', maxWidth: 480, bgcolor: '#111827', border: '1px solid #1F2937', textAlign: 'center' }}>
        <CardContent sx={{ p: 5 }}>
          <Box sx={{ mb: 3, display: 'flex', justifyContent: 'center' }}>
            <EmailOutlinedIcon sx={{ fontSize: 56, color: '#00C8C8' }} />
          </Box>

          <Typography variant="h5" component="h1" fontWeight={700} gutterBottom>
            Verify your email address
          </Typography>

          <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
            We sent a verification link to your email. Click the link to activate your account.
          </Typography>

          {getResendAlert()}

          <Button
            variant="outlined"
            onClick={handleResend}
            disabled={loading}
            sx={{
              mb: 3,
              borderColor: '#00C8C8',
              color: '#00C8C8',
              '&:hover': { borderColor: '#00AEAE', bgcolor: 'rgba(0,200,200,0.08)' },
              '&:disabled': { borderColor: '#006666', color: '#006666' },
            }}
          >
            {loading ? 'Sending…' : 'Resend verification email'}
          </Button>

          <Box>
            <Typography variant="body2" color="text.secondary">
              Already verified?{' '}
              <Box
                component={Link}
                href="/login"
                sx={{ color: '#00C8C8', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
              >
                Log in
              </Box>
            </Typography>
          </Box>
        </CardContent>
      </Card>
    </Container>
  );
}

export default function VerifyEmailPage() {
  return (
    <>
      <Head>
        <title>Verify Email | Mahalaxmi AI</title>
      </Head>
      <Suspense fallback={null}>
        <VerifyEmailContent />
      </Suspense>
    </>
  );
}

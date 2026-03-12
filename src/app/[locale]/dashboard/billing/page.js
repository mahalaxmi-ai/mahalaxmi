'use client';

import { useState } from 'react';
import {
  Box,
  Button,
  Card,
  CardContent,
  CircularProgress,
  Alert,
  Typography,
} from '@mui/material';
import CreditCardIcon from '@mui/icons-material/CreditCard';
import { useAuth } from '@/contexts/AuthContext';
import { billingAPI } from '@/lib/api';

export const metadata = {
  title: 'Billing — Mahalaxmi Dashboard',
};

export default function BillingPage() {
  const { user } = useAuth();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const tierName = (user && user.tier) ? user.tier : 'Cloud Builder';

  async function handleManageBilling() {
    setLoading(true);
    setError(null);
    try {
      const response = await billingAPI.getPortalUrl();
      const url = response?.data?.url ?? response?.data;
      if (!url || typeof url !== 'string') {
        throw new Error('Invalid portal URL received');
      }
      window.location.href = url;
    } catch {
      setError('Billing portal temporarily unavailable. Contact support@mahalaxmi.ai');
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
        alignItems: 'flex-start',
        justifyContent: 'center',
        pt: 6,
        px: 2,
      }}
    >
      <Card
        sx={{
          width: '100%',
          maxWidth: 600,
          backgroundColor: '#0F3333',
          border: '1px solid rgba(0, 200, 200, 0.2)',
          borderRadius: 2,
        }}
      >
        <CardContent sx={{ p: 4 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 3 }}>
            <CreditCardIcon sx={{ color: '#00C8C8', fontSize: 28 }} />
            <Typography
              variant="h5"
              component="h1"
              sx={{ color: '#FFFFFF', fontWeight: 600 }}
            >
              Billing
            </Typography>
          </Box>

          <Box
            sx={{
              backgroundColor: 'rgba(0, 200, 200, 0.08)',
              border: '1px solid rgba(0, 200, 200, 0.15)',
              borderRadius: 1,
              px: 3,
              py: 2,
              mb: 3,
            }}
          >
            <Typography variant="body2" sx={{ color: '#00C8C8', mb: 0.5 }}>
              Current Plan
            </Typography>
            <Typography variant="h6" sx={{ color: '#FFFFFF', fontWeight: 600 }}>
              {tierName}
            </Typography>
          </Box>

          <Typography
            variant="body1"
            sx={{ color: 'rgba(255, 255, 255, 0.7)', mb: 4 }}
          >
            Full usage dashboard coming soon when cycle metering is enabled.
          </Typography>

          {error && (
            <Alert
              severity="error"
              sx={{
                mb: 3,
                backgroundColor: 'rgba(211, 47, 47, 0.15)',
                color: '#FFFFFF',
                border: '1px solid rgba(211, 47, 47, 0.3)',
                '& .MuiAlert-icon': { color: '#f44336' },
              }}
            >
              Billing portal temporarily unavailable. Contact{' '}
              <Box
                component="a"
                href="mailto:support@mahalaxmi.ai"
                sx={{ color: '#00C8C8', textDecoration: 'underline' }}
              >
                support@mahalaxmi.ai
              </Box>
            </Alert>
          )}

          <Button
            variant="contained"
            color="primary"
            disabled={loading}
            onClick={handleManageBilling}
            startIcon={loading ? <CircularProgress size={18} color="inherit" /> : null}
            sx={{
              backgroundColor: '#00C8C8',
              color: '#0A2A2A',
              fontWeight: 600,
              px: 3,
              py: 1.25,
              '&:hover': {
                backgroundColor: '#00AAAA',
              },
              '&.Mui-disabled': {
                backgroundColor: 'rgba(0, 200, 200, 0.3)',
                color: 'rgba(10, 42, 42, 0.6)',
              },
            }}
          >
            {loading ? 'Loading…' : 'Manage Billing'}
          </Button>
        </CardContent>
      </Card>
    </Box>
  );
}

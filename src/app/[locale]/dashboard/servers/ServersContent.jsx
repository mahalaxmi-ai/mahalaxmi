'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  Alert,
  Box,
  Button,
  CircularProgress,
  Container,
  Grid,
  IconButton,
  Tooltip,
  Typography,
} from '@mui/material';
import { Cloud, Refresh } from '@mui/icons-material';
import { useAuth } from '@/contexts/AuthContext';
import ServerCard from './ServerCard';

const POLL_INTERVAL_MS = 5_000;

async function fetchServersApi() {
  const res = await fetch('/api/mahalaxmi/servers', { cache: 'no-store' });
  if (res.status === 401) {
    const err = new Error('Unauthorized');
    err.status = 401;
    throw err;
  }
  if (!res.ok) {
    throw new Error('Failed to load servers. Please refresh.');
  }
  const data = await res.json();
  return Array.isArray(data) ? data : [];
}

export default function ServersContent() {
  const { isAuthenticated, isLoading: authLoading } = useAuth();
  const router = useRouter();
  const queryClient = useQueryClient();

  const {
    data: servers = [],
    isLoading: fetchLoading,
    error: fetchError,
  } = useQuery({
    queryKey: ['mahalaxmi-servers'],
    queryFn: fetchServersApi,
    refetchInterval: POLL_INTERVAL_MS,
    enabled: !authLoading && !!isAuthenticated,
    retry: (failureCount, error) => error?.status === 401 ? false : failureCount < 3,
  });

  useEffect(() => {
    if (authLoading) return;
    if (!isAuthenticated) {
      router.replace('/login?redirect=/dashboard/servers');
    }
  }, [authLoading, isAuthenticated, router]);

  useEffect(() => {
    if (fetchError?.status === 401) {
      router.replace('/login?redirect=/dashboard/servers');
    }
  }, [fetchError, router]);

  function handleServerUpdated(updated) {
    queryClient.setQueryData(['mahalaxmi-servers'], (prev) =>
      (prev || []).map((s) => (s.id === updated.id ? updated : s))
    );
  }

  if (authLoading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 10 }}>
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 6 } }}>
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 4 }}>
        <Box>
          <Typography variant="h4" component="h1" sx={{ fontWeight: 700 }}>
            My cloud servers
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
            Each server is a dedicated Mahalaxmi orchestration VM, scoped to your account.
          </Typography>
        </Box>
        <Tooltip title="Refresh">
          <IconButton
            onClick={() => queryClient.invalidateQueries({ queryKey: ['mahalaxmi-servers'] })}
            disabled={fetchLoading}
          >
            <Refresh />
          </IconButton>
        </Tooltip>
      </Box>

      {fetchError && fetchError.status !== 401 && (
        <Alert severity="error" sx={{ mb: 4 }}>
          {fetchError.message}
        </Alert>
      )}

      {fetchLoading && (
        <Box sx={{ display: 'flex', justifyContent: 'center', py: 8 }}>
          <CircularProgress />
        </Box>
      )}

      {!fetchLoading && !fetchError && servers.length === 0 && (
        <Box
          sx={{
            textAlign: 'center',
            py: 10,
            border: '1px dashed',
            borderColor: 'divider',
            borderRadius: 3,
          }}
        >
          <Cloud sx={{ fontSize: 64, color: 'text.disabled', mb: 2 }} />
          <Typography variant="h6" color="text.secondary" sx={{ mb: 1 }}>
            No servers yet
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
            Purchase a subscription to provision your first cloud server.
          </Typography>
          <Button variant="contained" href="/cloud/pricing">
            View plans
          </Button>
        </Box>
      )}

      {!fetchLoading && servers.length > 0 && (
        <Grid container spacing={3}>
          {servers.map((server) => (
            <Grid item xs={12} sm={6} md={4} key={server.id}>
              <ServerCard
                server={server}
                onUpdated={handleServerUpdated}
              />
            </Grid>
          ))}
        </Grid>
      )}

      {!fetchLoading && servers.length > 0 && (
        <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 4, textAlign: 'center' }}>
          Server data refreshes every 5 seconds. For support, email{' '}
          <a href="mailto:support@mahalaxmi.ai" style={{ color: 'inherit' }}>
            support@mahalaxmi.ai
          </a>
        </Typography>
      )}
    </Container>
  );
}

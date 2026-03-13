'use client';

import { useCallback } from 'react';
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
  Skeleton,
  Tooltip,
  Typography,
} from '@mui/material';
import { Cloud, Refresh } from '@mui/icons-material';
import { useAuth } from '@/contexts/AuthContext';
import { Link } from '@/i18n/navigation';
import ServerCard from './ServerCard';

const POLL_INTERVAL_MS = 5_000;
const QUERY_KEY = 'mahalaxmi-servers';

export default function ServersContent() {
  const { isAuthenticated, isLoading: authLoading, user } = useAuth();
  const router = useRouter();
  const queryClient = useQueryClient();

  const userHeaders = {
    ...(user?.id    ? { 'x-user-id':    String(user.id)    } : {}),
    ...(user?.email ? { 'x-user-email': user.email } : {}),
  };

  const fetchServers = useCallback(async () => {
    const res = await fetch('/api/mahalaxmi/servers', {
      cache: 'no-store',
      headers: userHeaders,
    });
    if (res.status === 401) {
      router.replace('/login?redirect=/dashboard/servers');
      return [];
    }
    if (!res.ok) throw new Error('Failed to load servers');
    const data = await res.json();
    // Spec: bare array — no envelope. Handle both for safety.
    return Array.isArray(data) ? data : (data.servers ?? []);
  }, [user?.id, user?.email]); // eslint-disable-line react-hooks/exhaustive-deps

  const { data: servers = [], isLoading, error, refetch } = useQuery({
    queryKey: [QUERY_KEY, user?.id],
    queryFn: fetchServers,
    enabled: !authLoading && isAuthenticated,
    refetchInterval: POLL_INTERVAL_MS,
    staleTime: 0,
  });

  if (!authLoading && !isAuthenticated) {
    router.replace('/login?redirect=/dashboard/servers');
    return null;
  }

  function handleOptimisticUpdate(serverId, patch) {
    queryClient.setQueryData([QUERY_KEY, user?.id], (prev = []) =>
      prev.map((s) => (s.id === serverId ? { ...s, ...patch } : s))
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
          <IconButton onClick={() => refetch()} disabled={isLoading}>
            <Refresh />
          </IconButton>
        </Tooltip>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 4 }}>
          Failed to load servers. Please refresh.
        </Alert>
      )}

      {/* Loading skeleton on first load */}
      {isLoading && (
        <Grid container spacing={3}>
          {[1, 2, 3].map((n) => (
            <Grid item xs={12} sm={6} md={4} key={n}>
              <Skeleton variant="rounded" height={220} />
            </Grid>
          ))}
        </Grid>
      )}

      {/* Empty state */}
      {!isLoading && !error && servers.length === 0 && (
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
            Get started at{' '}
            <Link href="/cloud/pricing" style={{ color: 'inherit' }}>
              /cloud/pricing
            </Link>
          </Typography>
          <Button variant="contained" component={Link} href="/cloud/pricing">
            View plans
          </Button>
        </Box>
      )}

      {/* Server grid */}
      {!isLoading && servers.length > 0 && (
        <Grid container spacing={3}>
          {servers.map((server) => (
            <Grid item xs={12} sm={6} md={4} key={server.id}>
              <ServerCard
                server={server}
                onOptimisticUpdate={handleOptimisticUpdate}
                onRefresh={refetch}
                user={user}
              />
            </Grid>
          ))}
        </Grid>
      )}

      {!isLoading && servers.length > 0 && (
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

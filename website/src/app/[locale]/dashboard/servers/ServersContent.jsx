'use client';

import { useEffect, useRef, useState, useCallback } from 'react';
import { useRouter } from 'next/navigation';
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Container,
  Grid,
  IconButton,
  InputAdornment,
  LinearProgress,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material';
import {
  Cloud,
  ContentCopy,
  CreditCard,
  OpenInNew,
  Refresh,
  Settings,
} from '@mui/icons-material';
import { useAuth } from '@/contexts/AuthContext';
import ProjectNameModal from './ProjectNameModal';
import { PROVIDER_LABELS } from '@/lib/cloudConstants';

const POLL_INTERVAL_MS = 5_000;

function ProviderBadge({ provider }) {
  if (!provider) return null;
  const cfg = PROVIDER_LABELS[provider] ?? { name: provider, color: 'grey' };
  return (
    <Chip
      label={cfg.name}
      size="small"
      sx={{ bgcolor: cfg.color, color: 'white', fontSize: '0.65rem', height: 20, fontWeight: 600 }}
    />
  );
}

function CopyField({ label, value }) {
  const [copied, setCopied] = useState(false);

  function handleCopy() {
    navigator.clipboard.writeText(value).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  return (
    <TextField
      label={label}
      value={value}
      fullWidth
      size="small"
      slotProps={{
        input: {
          readOnly: true,
          endAdornment: (
            <InputAdornment position="end">
              <Tooltip title={copied ? 'Copied!' : 'Copy'}>
                <IconButton onClick={handleCopy} edge="end" size="small">
                  <ContentCopy fontSize="small" />
                </IconButton>
              </Tooltip>
            </InputAdornment>
          ),
        },
      }}
    />
  );
}

function StatusChip({ status }) {
  const map = {
    provisioning: { label: 'Provisioning', color: 'warning' },
    ready:        { label: 'Ready',        color: 'success' },
    failed:       { label: 'Failed',       color: 'error'   },
    stopped:      { label: 'Stopped',      color: 'default' },
  };
  const cfg = map[status] ?? { label: status, color: 'default' };
  return <Chip label={cfg.label} color={cfg.color} size="small" />;
}

function ServerCard({ server, onUpdated }) {
  const [configureOpen, setConfigureOpen] = useState(false);

  const isProvisioning = server.status === 'provisioning';
  const isReady = server.status === 'ready';
  const needsConfigure = isProvisioning && !server.project_name;

  const endpoint = server.fqdn ? `https://${server.fqdn}:17421` : null;
  const deepLink = endpoint && server.api_key
    ? `vscode://thrivetech.mahalaxmi/configure?endpoint=${encodeURIComponent(endpoint)}&api_key=${encodeURIComponent(server.api_key)}`
    : null;

  function handleConfigured({ project_name, fqdn }) {
    setConfigureOpen(false);
    onUpdated({ ...server, project_name, fqdn: fqdn || `${project_name}.mahalaxmi.ai` });
  }

  return (
    <>
      <Card variant="outlined" sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
        <CardContent sx={{ flexGrow: 1 }}>
          {/* Header row: name + status */}
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <Cloud sx={{ color: 'primary.main', fontSize: 20 }} />
              <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
                {server.project_name || 'Unnamed server'}
              </Typography>
            </Box>
            <StatusChip status={server.status} />
          </Box>

          {/* Provider badge */}
          <Box sx={{ mb: 1.5 }}>
            <ProviderBadge provider={server.cloud_provider} />
          </Box>

          {/* Tier + date */}
          <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 1.5 }}>
            {server.tier && <span style={{ textTransform: 'capitalize' }}>{server.tier}</span>}
            {server.tier && server.created_at && ' · '}
            {server.created_at && `Created ${new Date(server.created_at).toLocaleDateString()}`}
          </Typography>

          {/* FQDN */}
          {server.fqdn && (
            <Typography variant="body2" sx={{ fontFamily: 'monospace', mb: 2, color: 'text.secondary' }}>
              {server.fqdn}
            </Typography>
          )}

          {/* Provisioning progress */}
          {isProvisioning && (
            <Box sx={{ mb: 2 }}>
              <LinearProgress sx={{ borderRadius: 1, mb: 0.5 }} />
              <Typography variant="caption" color="text.secondary">
                Server is being provisioned — this takes 1–3 minutes.
              </Typography>
            </Box>
          )}

          {/* Set project name (provisioning, no project_name yet) */}
          {needsConfigure && (
            <Button
              variant="outlined"
              startIcon={<Settings />}
              onClick={() => setConfigureOpen(true)}
              fullWidth
              sx={{ mb: 1.5 }}
            >
              Set project name
            </Button>
          )}

          {/* Ready actions */}
          {isReady && (
            <>
              {deepLink && (
                <Button
                  variant="contained"
                  fullWidth
                  startIcon={<OpenInNew />}
                  href={deepLink}
                  sx={{ mb: 1.5 }}
                >
                  Open in VS Code
                </Button>
              )}
              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
                {endpoint && <CopyField label="Server endpoint" value={endpoint} />}
                {server.api_key && <CopyField label="API key" value={server.api_key} />}
              </Box>
            </>
          )}

          {/* Failed state */}
          {server.status === 'failed' && (
            <Alert severity="error" sx={{ mt: 1 }}>
              Provisioning failed. Contact{' '}
              <a href="mailto:support@mahalaxmi.ai">support@mahalaxmi.ai</a>
            </Alert>
          )}
        </CardContent>
      </Card>

      <ProjectNameModal
        open={configureOpen}
        serverId={server.id}
        onConfigured={handleConfigured}
        onClose={() => setConfigureOpen(false)}
      />
    </>
  );
}

export default function ServersContent() {
  const { isAuthenticated, isLoading: authLoading, user } = useAuth();
  const router = useRouter();

  const [servers, setServers] = useState([]);
  const [fetchLoading, setFetchLoading] = useState(true);
  const [fetchError, setFetchError] = useState(null);
  const [billingLoading, setBillingLoading] = useState(false);
  const intervalRef = useRef(null);

  const fetchServers = useCallback(async () => {
    try {
      const res = await fetch('/api/mahalaxmi/servers', {
        cache: 'no-store',
        headers: {
          ...(user?.id    ? { 'x-user-id':    user.id    } : {}),
          ...(user?.email ? { 'x-user-email': user.email } : {}),
        },
      });

      if (res.status === 401) {
        router.replace('/login?redirect=/dashboard/servers');
        return;
      }
      if (!res.ok) {
        setFetchError('Failed to load servers. Please refresh.');
        return;
      }

      const data = await res.json();
      setServers(data.servers || []);
      setFetchError(null);
    } catch {
      setFetchError('Network error. Please check your connection.');
    } finally {
      setFetchLoading(false);
    }
  }, [router, user]);

  // Start polling once auth resolves
  useEffect(() => {
    if (authLoading) return;
    if (!isAuthenticated) {
      router.replace('/login?redirect=/dashboard/servers');
      return;
    }

    fetchServers();
    intervalRef.current = setInterval(fetchServers, POLL_INTERVAL_MS);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [authLoading, isAuthenticated, fetchServers]); // eslint-disable-line react-hooks/exhaustive-deps

  function handleServerUpdated(updated) {
    setServers((prev) => prev.map((s) => (s.id === updated.id ? updated : s)));
  }

  async function handleManageBilling() {
    setBillingLoading(true);
    try {
      const res = await fetch('/api/billing/portal', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(user?.id    ? { 'x-user-id':    user.id    } : {}),
          ...(user?.email ? { 'x-user-email': user.email } : {}),
        },
      });
      const data = await res.json();
      if (data.portal_url) {
        window.location.href = data.portal_url;
      } else {
        window.location.href = 'mailto:billing@mahalaxmi.ai';
      }
    } catch {
      window.location.href = 'mailto:billing@mahalaxmi.ai';
    } finally {
      setBillingLoading(false);
    }
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
      {/* Page header */}
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 4 }}>
        <Box>
          <Typography variant="h4" component="h1" sx={{ fontWeight: 700 }}>
            My cloud servers
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
            Each server is a dedicated Mahalaxmi orchestration VM, scoped to your account.
          </Typography>
        </Box>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <Tooltip title="Manage billing">
            <Button
              variant="outlined"
              size="small"
              startIcon={<CreditCard />}
              onClick={handleManageBilling}
              disabled={billingLoading}
            >
              {billingLoading ? 'Loading…' : 'Billing'}
            </Button>
          </Tooltip>
          <Tooltip title="Refresh">
            <IconButton onClick={fetchServers} disabled={fetchLoading}>
              <Refresh />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {/* Error state */}
      {fetchError && (
        <Alert severity="error" sx={{ mb: 4 }}>{fetchError}</Alert>
      )}

      {/* Loading */}
      {fetchLoading && (
        <Box sx={{ display: 'flex', justifyContent: 'center', py: 8 }}>
          <CircularProgress />
        </Box>
      )}

      {/* No servers */}
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

      {/* Server grid */}
      {!fetchLoading && servers.length > 0 && (
        <Grid container spacing={3}>
          {servers.map((server) => (
            <Grid item xs={12} sm={6} md={4} key={server.id}>
              <ServerCard server={server} onUpdated={handleServerUpdated} />
            </Grid>
          ))}
        </Grid>
      )}

      {/* Footer */}
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

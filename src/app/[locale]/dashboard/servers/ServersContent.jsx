'use client';

import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import axios from 'axios';
import {
  Alert,
  Box,
  Button,
  Card,
  CardActions,
  CardContent,
  Chip,
  CircularProgress,
  Container,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Grid,
  LinearProgress,
  Link,
  Snackbar,
  Typography,
} from '@mui/material';
import {
  Cloud,
  Code,
  ContentCopy,
  Delete,
  Settings,
  Stop,
} from '@mui/icons-material';
import { serversAPI } from '@/lib/api';
import ProjectNameModal from './ProjectNameModal';

const STATUS_MAP = {
  pending_payment: {
    label: 'Awaiting Payment',
    chipColor: 'warning',
  },
  provisioning: {
    label: 'Provisioning',
    chipColor: 'info',
    showProgress: true,
  },
  active: {
    label: 'Active',
    chipColor: 'success',
  },
  degraded: {
    label: 'Degraded',
    chipSx: { bgcolor: 'rgba(253, 216, 53, 0.15)', color: '#fdd835', border: '1px solid #fdd835' },
  },
  stopping: {
    label: 'Stopping',
    chipColor: 'warning',
    showProgress: true,
  },
  stopped: {
    label: 'Stopped',
    chipColor: 'default',
  },
  deleting: {
    label: 'Deleting',
    chipColor: 'warning',
    showProgress: true,
    disableAll: true,
  },
  deleted: {
    label: 'Deleted',
    chipColor: 'default',
    noActions: true,
  },
  failed: {
    label: 'Failed',
    chipColor: 'error',
  },
};

function ConfirmDialog({ open, title, message, confirmLabel, onConfirm, onCancel, loading }) {
  return (
    <Dialog open={open} onClose={onCancel} maxWidth="xs" fullWidth>
      <DialogTitle>{title}</DialogTitle>
      <DialogContent>
        <DialogContentText>{message}</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={onCancel} disabled={loading}>
          Cancel
        </Button>
        <Button
          variant="contained"
          color="error"
          onClick={onConfirm}
          disabled={loading}
          startIcon={loading ? <CircularProgress size={16} color="inherit" /> : null}
        >
          {loading ? 'Processing…' : confirmLabel}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function ServerCard({ server, onRefetch, onAlreadyConfigured }) {
  const [configureOpen, setConfigureOpen] = useState(false);
  const [stopDialogOpen, setStopDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);
  const [actionError, setActionError] = useState(null);
  const [vsCodeLoading, setVsCodeLoading] = useState(false);

  const statusCfg = STATUS_MAP[server.status] || { label: server.status, chipColor: 'default' };
  const isDeleting = server.status === 'deleting';
  const isDeleted = server.status === 'deleted';

  const fqdn = server.fqdn || (server.project_name ? `${server.project_name}.mahalaxmi.ai` : null);

  async function handleOpenVsCode() {
    setVsCodeLoading(true);
    setActionError(null);
    try {
      const res = await axios.get(`/api/mahalaxmi/servers/${server.id}/vscode-config`);
      const deepLink = res.data?.deep_link;
      if (!deepLink) {
        setActionError('VS Code configuration is not available.');
        return;
      }
      window.location.href = deepLink;
    } catch {
      setActionError('Failed to retrieve VS Code configuration. Please try again.');
    } finally {
      setVsCodeLoading(false);
    }
  }

  async function handleCopyConfig() {
    setActionError(null);
    try {
      const res = await axios.get(`/api/mahalaxmi/servers/${server.id}/vscode-config`);
      const endpoint = res.data?.config_json?.endpoint;
      const apiKey = res.data?.config_json?.api_key;
      if (!endpoint && !apiKey) {
        setActionError('Configuration data is not available yet.');
        return;
      }
      const text = [endpoint && `Endpoint: ${endpoint}`, apiKey && `API Key: ${apiKey}`]
        .filter(Boolean)
        .join('\n');
      await navigator.clipboard.writeText(text);
    } catch {
      setActionError('Failed to copy configuration. Please try again.');
    }
  }

  async function handleStop() {
    setActionLoading(true);
    setActionError(null);
    try {
      await axios.post(`/api/mahalaxmi/servers/${server.id}/stop`);
      setStopDialogOpen(false);
      onRefetch();
    } catch {
      setActionError('Failed to stop server. Please try again.');
    } finally {
      setActionLoading(false);
    }
  }

  async function handleDelete() {
    setActionLoading(true);
    setActionError(null);
    try {
      await serversAPI.deleteProject(server.id);
      setDeleteDialogOpen(false);
      onRefetch();
    } catch {
      setActionError('Failed to delete server. Please try again.');
    } finally {
      setActionLoading(false);
    }
  }

  const canStop = (server.status === 'active' || server.status === 'degraded') && !isDeleting;
  const canDelete = !isDeleting && !isDeleted;
  const canConfigure = server.is_configured === false && !isDeleting && !isDeleted;
  const canOpenVsCode =
    server.status === 'active' && server.is_configured === true && !isDeleting;

  return (
    <>
      <Card
        variant="outlined"
        sx={{ height: '100%', display: 'flex', flexDirection: 'column', opacity: isDeleted ? 0.6 : 1 }}
      >
        <CardContent sx={{ flexGrow: 1 }}>
          <Box sx={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', mb: 1 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, minWidth: 0 }}>
              <Cloud sx={{ color: 'primary.main', fontSize: 20, flexShrink: 0 }} />
              <Typography variant="subtitle1" sx={{ fontWeight: 700, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {server.project_name || 'Unnamed'}
              </Typography>
            </Box>
            <Chip
              label={statusCfg.label}
              color={statusCfg.chipColor}
              size="small"
              sx={statusCfg.chipSx || {}}
            />
          </Box>

          {fqdn && (
            <Typography
              variant="body2"
              sx={{ fontFamily: 'monospace', color: 'text.secondary', mb: 1, wordBreak: 'break-all' }}
            >
              {fqdn}
            </Typography>
          )}

          {server.tier && (
            <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 1 }}>
              {server.tier}
            </Typography>
          )}

          {statusCfg.showProgress && (
            <LinearProgress sx={{ mt: 1, mb: 1, borderRadius: 1 }} />
          )}

          {server.status === 'failed' && (
            <Alert severity="error" sx={{ mt: 1 }}>
              Provisioning failed. Contact{' '}
              <Link href="mailto:support@mahalaxmi.ai" underline="hover">
                support@mahalaxmi.ai
              </Link>
            </Alert>
          )}

          {actionError && (
            <Alert severity="error" sx={{ mt: 1 }} onClose={() => setActionError(null)}>
              {actionError}
            </Alert>
          )}
        </CardContent>

        {!isDeleted && (
          <CardActions sx={{ flexWrap: 'wrap', gap: 1, px: 2, pb: 2 }}>
            {canConfigure && (
              <Button
                size="small"
                variant="outlined"
                startIcon={<Settings />}
                onClick={() => setConfigureOpen(true)}
                disabled={isDeleting}
              >
                Configure
              </Button>
            )}

            {canOpenVsCode && (
              <Button
                size="small"
                variant="contained"
                startIcon={vsCodeLoading ? <CircularProgress size={14} color="inherit" /> : <Code />}
                onClick={handleOpenVsCode}
                disabled={vsCodeLoading || isDeleting}
              >
                Open in VS Code
              </Button>
            )}

            {server.status === 'active' && server.is_configured === true && (
              <Button
                size="small"
                variant="outlined"
                startIcon={<ContentCopy />}
                onClick={handleCopyConfig}
                disabled={isDeleting}
              >
                Copy config
              </Button>
            )}

            {canStop && (
              <Button
                size="small"
                variant="outlined"
                color="warning"
                startIcon={<Stop />}
                onClick={() => setStopDialogOpen(true)}
                disabled={isDeleting}
              >
                Stop
              </Button>
            )}

            {canDelete && (
              <Button
                size="small"
                variant="outlined"
                color="error"
                startIcon={<Delete />}
                onClick={() => setDeleteDialogOpen(true)}
                disabled={isDeleting}
              >
                Delete
              </Button>
            )}
          </CardActions>
        )}
      </Card>

      <ProjectNameModal
        open={configureOpen}
        serverId={server.id}
        onConfigured={() => {
          setConfigureOpen(false);
          onRefetch();
        }}
        onClose={() => setConfigureOpen(false)}
        onAlreadyConfigured={(message) => {
          setConfigureOpen(false);
          onAlreadyConfigured(message);
          onRefetch();
        }}
      />

      <ConfirmDialog
        open={stopDialogOpen}
        title="Stop server?"
        message="The server will be stopped. You can restart it later."
        confirmLabel="Stop"
        onConfirm={handleStop}
        onCancel={() => setStopDialogOpen(false)}
        loading={actionLoading}
      />

      <ConfirmDialog
        open={deleteDialogOpen}
        title="Delete server?"
        message="This will permanently delete the server and all associated data. This action cannot be undone."
        confirmLabel="Delete"
        onConfirm={handleDelete}
        onCancel={() => setDeleteDialogOpen(false)}
        loading={actionLoading}
      />
    </>
  );
}

export default function ServersContent() {
  const [toastOpen, setToastOpen] = useState(false);
  const [toastMessage, setToastMessage] = useState('');

  const {
    data: servers,
    isLoading,
    isError,
    refetch,
  } = useQuery({
    queryKey: ['servers'],
    queryFn: async () => {
      const res = await axios.get('/api/mahalaxmi/servers');
      return Array.isArray(res.data) ? res.data : [];
    },
    refetchInterval: 5000,
  });

  function handleAlreadyConfigured(message) {
    setToastMessage(message || 'This server has already been configured.');
    setToastOpen(true);
  }

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 6 } }}>
      <Typography variant="h4" component="h1" sx={{ fontWeight: 700, mb: 4 }}>
        My Servers
      </Typography>

      {isLoading && (
        <Box sx={{ display: 'flex', justifyContent: 'center', py: 10 }}>
          <CircularProgress />
        </Box>
      )}

      {isError && (
        <Alert severity="error" sx={{ mb: 4 }}>
          Unable to load servers. Try refreshing.{' '}
          <Link href="mailto:support@mahalaxmi.ai" underline="hover">
            Contact support@mahalaxmi.ai
          </Link>
        </Alert>
      )}

      {!isLoading && !isError && Array.isArray(servers) && servers.length === 0 && (
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
            No servers yet. Get started with{' '}
            <Link href="/cloud/pricing" underline="hover">
              Cloud pricing
            </Link>
            .
          </Typography>
        </Box>
      )}

      {!isLoading && !isError && Array.isArray(servers) && servers.length > 0 && (
        <Grid container spacing={3}>
          {servers.map((server) => (
            <Grid item xs={12} sm={6} md={4} key={server.id}>
              <ServerCard
                server={server}
                onRefetch={refetch}
                onAlreadyConfigured={handleAlreadyConfigured}
              />
            </Grid>
          ))}
        </Grid>
      )}

      <Snackbar
        open={toastOpen}
        autoHideDuration={6000}
        onClose={() => setToastOpen(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert severity="error" onClose={() => setToastOpen(false)} sx={{ width: '100%' }}>
          {toastMessage}
        </Alert>
      </Snackbar>
    </Container>
  );
}

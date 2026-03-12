'use client';

import React, { useState } from 'react';
import {
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  LinearProgress,
  Link,
  Snackbar,
  Alert,
  Typography,
} from '@mui/material';
import ProjectNameModal from './ProjectNameModal';

interface Server {
  id: string;
  project_name: string | null;
  fqdn: string | null;
  status: string;
  tier: string;
  created_at: string;
  is_configured: boolean;
}

interface ServerCardProps {
  server: Server;
  onRefresh: () => void;
}

interface SnackbarState {
  open: boolean;
  message: string;
  severity: 'success' | 'error' | 'info' | 'warning';
}

type LoadingKey = 'vscode' | 'copy' | 'delete';

const STATUS_IN_PROGRESS = new Set(['pending_payment', 'provisioning', 'stopping', 'deleting']);

function StatusBadge({ status }: { status: string }): React.ReactElement {
  switch (status) {
    case 'pending_payment':
      return (
        <Box sx={{ width: '100%' }}>
          <LinearProgress />
          <Typography variant="caption" sx={{ mt: 0.5, display: 'block' }}>
            Awaiting payment
          </Typography>
        </Box>
      );
    case 'provisioning':
      return (
        <Box sx={{ width: '100%' }}>
          <LinearProgress />
          <Typography variant="caption" sx={{ mt: 0.5, display: 'block' }}>
            Provisioning...
          </Typography>
        </Box>
      );
    case 'active':
      return <Chip color="success" label="Active" size="small" />;
    case 'degraded':
      return (
        <Chip
          label="Degraded"
          size="small"
          sx={{ backgroundColor: '#C8A040', color: '#fff' }}
        />
      );
    case 'stopping':
      return (
        <Box sx={{ width: '100%' }}>
          <LinearProgress />
          <Typography variant="caption" sx={{ mt: 0.5, display: 'block' }}>
            Stopping...
          </Typography>
        </Box>
      );
    case 'stopped':
      return <Chip color="default" label="Stopped" size="small" />;
    case 'deleting':
      return (
        <Box sx={{ width: '100%' }}>
          <LinearProgress />
          <Typography variant="caption" sx={{ mt: 0.5, display: 'block' }}>
            Deleting...
          </Typography>
        </Box>
      );
    case 'deleted':
      return <Chip color="default" label="Deleted" size="small" />;
    case 'failed':
      return (
        <Box>
          <Chip color="error" label="Failed" size="small" />
          <Typography variant="caption" sx={{ ml: 1 }}>
            Contact{' '}
            <Link href="mailto:support@mahalaxmi.ai">support@mahalaxmi.ai</Link>
          </Typography>
        </Box>
      );
    default:
      return <Chip color="default" label={status} size="small" />;
  }
}

export default function ServerCard({ server, onRefresh }: ServerCardProps): React.ReactElement {
  const [configureModalOpen, setConfigureModalOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [loading, setLoading] = useState<Record<LoadingKey, boolean>>({
    vscode: false,
    copy: false,
    delete: false,
  });
  const [snackbar, setSnackbar] = useState<SnackbarState>({
    open: false,
    message: '',
    severity: 'error',
  });

  const isDeleting = server.status === 'deleting';
  const isDeleted = server.status === 'deleted';
  const anyLoading = Object.values(loading).some(Boolean);

  function showSnackbar(message: string, severity: SnackbarState['severity'] = 'error'): void {
    setSnackbar({ open: true, message, severity });
  }

  function closeSnackbar(_event?: React.SyntheticEvent | Event, reason?: string): void {
    if (reason === 'clickaway') return;
    setSnackbar((prev) => ({ ...prev, open: false }));
  }

  function setLoadingKey(key: LoadingKey, value: boolean): void {
    setLoading((prev) => ({ ...prev, [key]: value }));
  }

  async function fetchVscodeConfig(): Promise<{ deep_link: string; config_json: { endpoint: string; api_key: string } }> {
    const res = await fetch(`/api/mahalaxmi/servers/${server.id}/vscode-config`);
    if (!res.ok) {
      const text = await res.text().catch(() => 'Unknown error');
      throw new Error(`Failed to fetch VS Code config: ${res.status} ${text}`);
    }
    return res.json() as Promise<{ deep_link: string; config_json: { endpoint: string; api_key: string } }>;
  }

  async function handleOpenVSCode(): Promise<void> {
    setLoadingKey('vscode', true);
    try {
      const data = await fetchVscodeConfig();
      window.location.href = data.deep_link;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to open VS Code';
      showSnackbar(message);
    } finally {
      setLoadingKey('vscode', false);
    }
  }

  async function handleCopyConfig(): Promise<void> {
    setLoadingKey('copy', true);
    try {
      const data = await fetchVscodeConfig();
      const payload = JSON.stringify({
        endpoint: data.config_json.endpoint,
        api_key: data.config_json.api_key,
      });
      await navigator.clipboard.writeText(payload);
      showSnackbar('Config copied to clipboard', 'success');
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to copy config';
      showSnackbar(message);
    } finally {
      setLoadingKey('copy', false);
    }
  }

  async function handleDelete(): Promise<void> {
    setDeleteDialogOpen(false);
    setLoadingKey('delete', true);
    try {
      const res = await fetch(`/api/mahalaxmi/projects/${server.id}`, {
        method: 'DELETE',
      });
      if (res.status !== 202) {
        const text = await res.text().catch(() => 'Unknown error');
        throw new Error(`Delete failed: ${res.status} ${text}`);
      }
      onRefresh();
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to delete server';
      showSnackbar(message);
    } finally {
      setLoadingKey('delete', false);
    }
  }

  function handleConfigureSuccess(): void {
    setConfigureModalOpen(false);
    onRefresh();
  }

  function handleConfigureNameTaken(): void {
    showSnackbar('That name is already taken', 'error');
  }

  const showConfigure = !server.is_configured;
  const showVSCode = server.status === 'active' && server.is_configured;
  const showCopyConfig = !isDeleted;
  const showDelete = !['deleting', 'deleted'].includes(server.status);

  return (
    <>
      <Card variant="outlined" sx={{ mb: 2 }}>
        <CardContent>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 1 }}>
            <Box>
              <Typography variant="h6" component="div">
                {server.project_name ?? server.fqdn ?? server.id}
              </Typography>
              {server.fqdn && server.project_name && (
                <Typography variant="body2" color="text.secondary">
                  {server.fqdn}
                </Typography>
              )}
              <Typography variant="caption" color="text.secondary">
                Tier: {server.tier}
              </Typography>
            </Box>
            <Box sx={{ minWidth: 120, textAlign: 'right' }}>
              <StatusBadge status={server.status} />
            </Box>
          </Box>

          {!isDeleted && (
            <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap', mt: 2 }}>
              {showConfigure && (
                <Button
                  variant="outlined"
                  size="small"
                  disabled={isDeleting || anyLoading}
                  onClick={() => setConfigureModalOpen(true)}
                >
                  Configure
                </Button>
              )}

              {showVSCode && (
                <Button
                  variant="contained"
                  size="small"
                  disabled={isDeleting || anyLoading}
                  onClick={() => { void handleOpenVSCode(); }}
                  startIcon={loading.vscode ? <CircularProgress size={14} color="inherit" /> : undefined}
                >
                  Open in VS Code
                </Button>
              )}

              {showCopyConfig && (
                <Button
                  variant="outlined"
                  size="small"
                  disabled={isDeleting || anyLoading}
                  onClick={() => { void handleCopyConfig(); }}
                  startIcon={loading.copy ? <CircularProgress size={14} color="inherit" /> : undefined}
                >
                  Copy Config
                </Button>
              )}

              {showDelete && (
                <Button
                  variant="outlined"
                  color="error"
                  size="small"
                  disabled={isDeleting || anyLoading}
                  onClick={() => setDeleteDialogOpen(true)}
                  startIcon={loading.delete ? <CircularProgress size={14} color="inherit" /> : undefined}
                >
                  Delete
                </Button>
              )}
            </Box>
          )}
        </CardContent>
      </Card>

      {configureModalOpen && (
        <ProjectNameModal
          serverId={server.id}
          open={configureModalOpen}
          onClose={() => setConfigureModalOpen(false)}
          onSuccess={handleConfigureSuccess}
          onNameTaken={handleConfigureNameTaken}
        />
      )}

      <Dialog
        open={deleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        aria-labelledby="delete-dialog-title"
      >
        <DialogTitle id="delete-dialog-title">Delete Server</DialogTitle>
        <DialogContent>
          <DialogContentText>
            This will permanently delete this server and cancel the subscription. Continue?
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteDialogOpen(false)}>Cancel</Button>
          <Button color="error" onClick={() => { void handleDelete(); }} autoFocus>
            Delete
          </Button>
        </DialogActions>
      </Dialog>

      <Snackbar
        open={snackbar.open}
        autoHideDuration={6000}
        onClose={closeSnackbar}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert onClose={closeSnackbar} severity={snackbar.severity} sx={{ width: '100%' }}>
          {snackbar.message}
        </Alert>
      </Snackbar>
    </>
  );
}

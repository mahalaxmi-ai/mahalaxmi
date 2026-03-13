'use client';

import { useState } from 'react';
import {
  Alert,
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
  Typography,
} from '@mui/material';
import { Code, Delete, Pause, PlayArrow, Settings } from '@mui/icons-material';
import { PROVIDER_LABELS } from '@/lib/cloudConstants';
import ProjectNameModal from './ProjectNameModal';

// ── Status config — all 9 states ──────────────────────────────────────────────
const STATUS_CONFIG = {
  pending_payment: { label: 'Awaiting Payment', color: '#F59E0B' },
  provisioning:    { label: 'Provisioning',     color: '#3B82F6' },
  active:          { label: 'Active',            color: '#10B981' },
  degraded:        { label: 'Degraded',          color: '#F97316' },
  stopping:        { label: 'Stopping',          color: '#6B7280' },
  stopped:         { label: 'Stopped',           color: '#6B7280' },
  deleting:        { label: 'Deleting',          color: '#6B7280' },
  deleted:         { label: 'Deleted',           color: '#6B7280' },
  failed:          { label: 'Failed',            color: '#EF4444' },
};

function StatusBadge({ status }) {
  const cfg = STATUS_CONFIG[status] ?? { label: status, color: '#6B7280' };
  return (
    <Chip
      label={cfg.label}
      size="small"
      sx={{ bgcolor: cfg.color, color: 'white', fontWeight: 600, fontSize: '0.7rem' }}
    />
  );
}

function ProviderBadge({ provider }) {
  if (!provider) return null;
  const cfg = PROVIDER_LABELS[provider] ?? { name: provider, color: '#6B7280' };
  return (
    <Chip
      label={cfg.name}
      size="small"
      sx={{ bgcolor: cfg.color, color: 'white', fontSize: '0.65rem', height: 20, fontWeight: 600 }}
    />
  );
}

function ConfirmDialog({ open, title, message, confirmLabel = 'Confirm', danger = false, onConfirm, onClose }) {
  return (
    <Dialog open={open} onClose={onClose} maxWidth="xs" fullWidth>
      <DialogTitle sx={{ fontWeight: 700 }}>{title}</DialogTitle>
      <DialogContent>
        <DialogContentText>{message}</DialogContentText>
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 3 }}>
        <Button onClick={onClose}>Cancel</Button>
        <Button variant="contained" color={danger ? 'error' : 'primary'} onClick={onConfirm}>
          {confirmLabel}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

export default function ServerCard({ server, onOptimisticUpdate, onRefresh, user }) {
  const [configureOpen, setConfigureOpen] = useState(false);
  const [stopConfirmOpen, setStopConfirmOpen] = useState(false);
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [vscodeLoading, setVscodeLoading] = useState(false);
  const [actionLoading, setActionLoading] = useState(null); // 'stop' | 'restart' | 'delete'
  const [actionError, setActionError] = useState(null);

  const { status, is_configured, has_keep_warm } = server;

  const userHeaders = {
    ...(user?.id    ? { 'x-user-id':    String(user.id)    } : {}),
    ...(user?.email ? { 'x-user-email': user.email } : {}),
  };

  const isDeleted  = status === 'deleted';
  const isDeleting = status === 'deleting';
  const isBusy     = !!actionLoading || vscodeLoading;

  // ── VS Code — must call endpoint, never construct deep link ─────────────────
  async function handleOpenVSCode() {
    setVscodeLoading(true);
    setActionError(null);
    try {
      const res = await fetch(`/api/mahalaxmi/servers/${server.id}/vscode-config`, {
        headers: userHeaders,
        cache: 'no-store',
      });
      if (!res.ok) {
        setActionError('Could not load VS Code configuration. Try again shortly.');
        return;
      }
      const data = await res.json();
      if (!data.deep_link) {
        setActionError('VS Code deep link not available yet.');
        return;
      }
      window.location.href = data.deep_link;
    } catch {
      setActionError('Network error loading VS Code config.');
    } finally {
      setVscodeLoading(false);
    }
  }

  // ── Stop ────────────────────────────────────────────────────────────────────
  async function handleStop() {
    setStopConfirmOpen(false);
    setActionLoading('stop');
    setActionError(null);
    onOptimisticUpdate(server.id, { status: 'stopping' });
    try {
      const res = await fetch(`/api/mahalaxmi/servers/${server.id}/stop`, {
        method: 'POST',
        headers: userHeaders,
      });
      // 501 = stub, treat as accepted
      if (!res.ok && res.status !== 501) {
        setActionError('Stop failed. The server list will refresh shortly.');
        onRefresh();
      }
    } catch {
      setActionError('Network error. Please try again.');
      onRefresh();
    } finally {
      setActionLoading(null);
    }
  }

  // ── Restart ─────────────────────────────────────────────────────────────────
  async function handleRestart() {
    setActionLoading('restart');
    setActionError(null);
    onOptimisticUpdate(server.id, { status: 'provisioning' });
    try {
      const res = await fetch(`/api/mahalaxmi/servers/${server.id}/restart`, {
        method: 'POST',
        headers: userHeaders,
      });
      if (!res.ok && res.status !== 501) {
        setActionError('Restart failed. The server list will refresh shortly.');
        onRefresh();
      }
    } catch {
      setActionError('Network error. Please try again.');
      onRefresh();
    } finally {
      setActionLoading(null);
    }
  }

  // ── Delete — uses projects/:id per spec ─────────────────────────────────────
  async function handleDelete() {
    setDeleteConfirmOpen(false);
    setActionLoading('delete');
    setActionError(null);
    onOptimisticUpdate(server.id, { status: 'deleting' });
    try {
      const projectId = server.project_id || server.id;
      const res = await fetch(`/api/mahalaxmi/projects/${projectId}`, {
        method: 'DELETE',
        headers: userHeaders,
      });
      if (!res.ok) {
        setActionError('Delete failed. Please try again.');
        onRefresh();
      }
    } catch {
      setActionError('Network error. Please try again.');
      onRefresh();
    } finally {
      setActionLoading(null);
    }
  }

  function handleConfigured({ project_name, fqdn }) {
    setConfigureOpen(false);
    onOptimisticUpdate(server.id, { project_name, fqdn, is_configured: true });
  }

  return (
    <>
      <Card
        variant="outlined"
        sx={{
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          opacity: isDeleted ? 0.45 : 1,
          transition: 'opacity 0.2s',
          pointerEvents: isDeleted ? 'none' : 'auto',
        }}
      >
        <CardContent sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column' }}>
          {/* Header: name + status */}
          <Box sx={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', mb: 1, gap: 1 }}>
            <Typography
              variant="subtitle1"
              sx={{ fontWeight: 700, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', flex: 1 }}
            >
              {server.project_name || 'Unnamed server'}
            </Typography>
            <StatusBadge status={status} />
          </Box>

          {/* Provider badge + Keep Warm */}
          <Box sx={{ display: 'flex', gap: 1, mb: 1.5, flexWrap: 'wrap' }}>
            <ProviderBadge provider={server.cloud_provider} />
            {has_keep_warm && (
              <Chip
                label="Keep Warm"
                size="small"
                sx={{ bgcolor: '#0D9488', color: 'white', fontSize: '0.65rem', height: 20, fontWeight: 600 }}
              />
            )}
          </Box>

          {/* Tier + date */}
          <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 1.5 }}>
            {server.tier}
            {server.tier && server.created_at && ' · '}
            {server.created_at && `Created ${new Date(server.created_at).toLocaleDateString()}`}
          </Typography>

          {/* FQDN */}
          {server.fqdn && (
            <Typography variant="body2" sx={{ fontFamily: 'monospace', mb: 2, color: 'text.secondary', fontSize: '0.78rem' }}>
              {server.fqdn}
            </Typography>
          )}

          {/* Action error */}
          {actionError && (
            <Alert severity="error" sx={{ mb: 1.5 }} onClose={() => setActionError(null)}>
              {actionError}
            </Alert>
          )}

          {/* ── Status-specific content ── */}

          {status === 'pending_payment' && (
            <Alert icon={false} severity="warning" sx={{ mb: 1.5 }}>
              Awaiting payment confirmation. This usually takes a moment.
            </Alert>
          )}

          {status === 'provisioning' && (
            <Box sx={{ mb: 2 }}>
              <LinearProgress sx={{ borderRadius: 1, mb: 0.5 }} />
              <Typography variant="caption" color="text.secondary">
                Provisioning your server — this takes 1–3 minutes.
              </Typography>
            </Box>
          )}

          {status === 'stopping' && (
            <Box sx={{ mb: 2 }}>
              <LinearProgress color="inherit" sx={{ borderRadius: 1, mb: 0.5 }} />
              <Typography variant="caption" color="text.secondary">Stopping…</Typography>
            </Box>
          )}

          {status === 'deleting' && (
            <Box sx={{ mb: 2 }}>
              <LinearProgress color="inherit" sx={{ borderRadius: 1, mb: 0.5 }} />
              <Typography variant="caption" color="text.secondary">Deleting…</Typography>
            </Box>
          )}

          {status === 'failed' && (
            <Alert severity="error" sx={{ mb: 1.5 }}>
              Provisioning failed. Contact{' '}
              <a href="mailto:support@mahalaxmi.ai">support@mahalaxmi.ai</a>
            </Alert>
          )}

          {/* Configure button — shown when not yet configured (and not terminal/pending states) */}
          {!isDeleted && !isDeleting && !is_configured &&
           status !== 'pending_payment' && status !== 'failed' && status !== 'stopping' && (
            <Button
              variant="outlined"
              fullWidth
              startIcon={<Settings />}
              onClick={() => setConfigureOpen(true)}
              disabled={isBusy}
              sx={{ mb: 1.5 }}
            >
              Set project name
            </Button>
          )}

          {/* ── Action buttons ── */}
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1, mt: 'auto' }}>
            {/* Open in VS Code — active + degraded + is_configured */}
            {(status === 'active' || status === 'degraded') && is_configured && (
              <Button
                variant="contained"
                fullWidth
                startIcon={vscodeLoading ? <CircularProgress size={16} color="inherit" /> : <Code />}
                onClick={handleOpenVSCode}
                disabled={isBusy}
              >
                {vscodeLoading ? 'Opening…' : 'Open in VS Code'}
              </Button>
            )}

            {/* Stop — active + degraded */}
            {(status === 'active' || status === 'degraded') && (
              <Button
                variant="outlined"
                fullWidth
                color="warning"
                startIcon={actionLoading === 'stop' ? <CircularProgress size={16} color="inherit" /> : <Pause />}
                onClick={() => setStopConfirmOpen(true)}
                disabled={isBusy}
              >
                Stop
              </Button>
            )}

            {/* Restart — stopped only */}
            {status === 'stopped' && (
              <Button
                variant="outlined"
                fullWidth
                startIcon={actionLoading === 'restart' ? <CircularProgress size={16} color="inherit" /> : <PlayArrow />}
                onClick={handleRestart}
                disabled={isBusy}
              >
                Restart
              </Button>
            )}

            {/* Delete — active, degraded, stopped, failed */}
            {(status === 'active' || status === 'degraded' || status === 'stopped' || status === 'failed') && (
              <Button
                variant="outlined"
                fullWidth
                color="error"
                startIcon={actionLoading === 'delete' ? <CircularProgress size={16} color="inherit" /> : <Delete />}
                onClick={() => setDeleteConfirmOpen(true)}
                disabled={isBusy}
              >
                Delete
              </Button>
            )}
          </Box>
        </CardContent>
      </Card>

      <ProjectNameModal
        open={configureOpen}
        serverId={server.id}
        onConfigured={handleConfigured}
        onClose={() => setConfigureOpen(false)}
        onRefresh={onRefresh}
        user={user}
      />

      <ConfirmDialog
        open={stopConfirmOpen}
        title="Stop server?"
        message="The server will be stopped. You can restart it at any time."
        confirmLabel="Stop server"
        onConfirm={handleStop}
        onClose={() => setStopConfirmOpen(false)}
      />

      <ConfirmDialog
        open={deleteConfirmOpen}
        title="Delete server?"
        message="This will permanently delete the server and all associated data. This cannot be undone."
        confirmLabel="Delete permanently"
        danger
        onConfirm={handleDelete}
        onClose={() => setDeleteConfirmOpen(false)}
      />
    </>
  );
}

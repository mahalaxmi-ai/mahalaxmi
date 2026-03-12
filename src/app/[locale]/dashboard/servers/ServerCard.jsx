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
  LinearProgress,
  Typography,
} from '@mui/material';
import { Cloud, OpenInNew, Settings } from '@mui/icons-material';
import ProjectNameModal from './ProjectNameModal';

const STATUS_CONFIG = {
  pending_payment: { label: 'Pending Payment', color: 'warning' },
  provisioning:   { label: 'Provisioning',    color: 'info' },
  active:         { label: 'Active',           color: 'success' },
  degraded:       { label: 'Degraded',         color: 'warning' },
  stopping:       { label: 'Stopping',         color: 'warning' },
  stopped:        { label: 'Stopped',          color: 'default' },
  deleting:       { label: 'Deleting',         color: 'warning' },
  deleted:        { label: 'Deleted',          color: 'default' },
  failed:         { label: 'Failed',           color: 'error' },
};

function StatusChip({ status }) {
  const cfg = STATUS_CONFIG[status] || { label: status, color: 'default' };
  return <Chip label={cfg.label} color={cfg.color} size="small" />;
}

export default function ServerCard({ server, onUpdated }) {
  const [configureOpen, setConfigureOpen] = useState(false);
  const [vscodeLink, setVscodeLink] = useState(null);
  const [vscodeLoading, setVscodeLoading] = useState(false);
  const [vscodeError, setVscodeError] = useState(null);
  const [deleting, setDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState(null);

  const isProvisioning = server.status === 'provisioning';
  const isActive = server.status === 'active';
  const isTransient = ['provisioning', 'stopping', 'deleting', 'pending_payment'].includes(server.status);
  const needsConfigure = isProvisioning && !server.project_name;

  async function handleOpenVSCode() {
    if (vscodeLink) {
      window.location.href = vscodeLink;
      return;
    }
    setVscodeLoading(true);
    setVscodeError(null);
    try {
      const res = await fetch(`/api/mahalaxmi/servers/${server.id}/vscode-config`);
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        setVscodeError(data.error || 'Failed to generate VS Code link. Please try again.');
        return;
      }
      const data = await res.json();
      if (!data.deep_link) {
        setVscodeError('VS Code link unavailable. Please try again later.');
        return;
      }
      setVscodeLink(data.deep_link);
      window.location.href = data.deep_link;
    } catch {
      setVscodeError('Network error. Please check your connection.');
    } finally {
      setVscodeLoading(false);
    }
  }

  async function handleDelete() {
    setDeleting(true);
    setDeleteError(null);
    try {
      const res = await fetch(`/api/mahalaxmi/projects/${server.id}`, { method: 'DELETE' });
      if (res.status === 202 || res.ok) {
        onUpdated({ ...server, status: 'deleting' });
        return;
      }
      const data = await res.json().catch(() => ({}));
      setDeleteError(data.error || 'Failed to delete server. Please try again.');
    } catch {
      setDeleteError('Network error. Please check your connection.');
    } finally {
      setDeleting(false);
    }
  }

  function handleConfigured({ project_name, fqdn }) {
    setConfigureOpen(false);
    onUpdated({ ...server, project_name, fqdn: fqdn || `${project_name}.mahalaxmi.ai` });
  }

  return (
    <>
      <Card variant="outlined" sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
        <CardContent sx={{ flexGrow: 1 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1.5 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <Cloud sx={{ color: 'primary.main', fontSize: 20 }} />
              <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
                {server.project_name || 'Unnamed server'}
              </Typography>
            </Box>
            <StatusChip status={server.status} />
          </Box>

          <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 1.5 }}>
            {server.tier && <span style={{ textTransform: 'capitalize' }}>{server.tier}</span>}
            {server.tier && server.created_at && ' · '}
            {server.created_at && `Created ${new Date(server.created_at).toLocaleDateString()}`}
          </Typography>

          {server.fqdn && (
            <Typography variant="body2" sx={{ fontFamily: 'monospace', mb: 2, color: 'text.secondary' }}>
              {server.fqdn}
            </Typography>
          )}

          {isTransient && (
            <Box sx={{ mb: 2 }}>
              <LinearProgress sx={{ borderRadius: 1, mb: 0.5 }} />
              <Typography variant="caption" color="text.secondary">
                {isProvisioning && 'Server is being provisioned — this takes 1–3 minutes.'}
                {server.status === 'stopping' && 'Server is stopping…'}
                {server.status === 'deleting' && 'Server is being deleted…'}
                {server.status === 'pending_payment' && 'Awaiting payment confirmation…'}
              </Typography>
            </Box>
          )}

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

          {isActive && (
            <>
              <Button
                variant="contained"
                fullWidth
                startIcon={vscodeLoading ? <CircularProgress size={16} color="inherit" /> : <OpenInNew />}
                onClick={handleOpenVSCode}
                disabled={vscodeLoading}
                sx={{ mb: 1.5 }}
              >
                {vscodeLoading ? 'Opening…' : 'Open in VS Code'}
              </Button>
              {vscodeError && (
                <Alert severity="error" sx={{ mb: 1.5 }}>
                  {vscodeError}
                </Alert>
              )}
            </>
          )}

          {server.status === 'failed' && (
            <Alert severity="error" sx={{ mt: 1 }}>
              Provisioning failed. Contact{' '}
              <a href="mailto:support@mahalaxmi.ai">support@mahalaxmi.ai</a>
            </Alert>
          )}

          {deleteError && (
            <Alert severity="error" sx={{ mt: 1 }}>
              {deleteError}
            </Alert>
          )}

          {['stopped', 'failed'].includes(server.status) && (
            <Button
              variant="outlined"
              color="error"
              size="small"
              onClick={handleDelete}
              disabled={deleting}
              startIcon={deleting ? <CircularProgress size={14} color="inherit" /> : null}
              sx={{ mt: 1.5 }}
              fullWidth
            >
              {deleting ? 'Deleting…' : 'Delete server'}
            </Button>
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

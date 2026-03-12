'use client';

import { useState } from 'react';
import {
  Button,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  TextField,
  Typography,
} from '@mui/material';

const PROJECT_NAME_RE = /^[a-z0-9][a-z0-9-]{1,38}[a-z0-9]$/;

function validateProjectName(value) {
  if (!value) return 'Project name is required.';
  if (value.length < 3) return 'Must be at least 3 characters.';
  if (value.length > 40) return 'Must be 40 characters or fewer.';
  if (!PROJECT_NAME_RE.test(value)) {
    return 'Only lowercase letters, digits, and hyphens. Cannot start or end with a hyphen.';
  }
  return null;
}

export default function ProjectNameModal({ open, serverId, onConfigured, onClose }) {
  const [value, setValue] = useState('');
  const [validationError, setValidationError] = useState(null);
  const [saving, setSaving] = useState(false);
  const [apiError, setApiError] = useState(null);

  function handleChange(e) {
    const raw = e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, '');
    setValue(raw);
    setValidationError(validateProjectName(raw));
    setApiError(null);
  }

  async function handleSave() {
    const err = validateProjectName(value);
    if (err) {
      setValidationError(err);
      return;
    }

    setSaving(true);
    setApiError(null);

    try {
      const res = await fetch(`/api/mahalaxmi/servers/${serverId}/configure`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ project_name: value }),
      });

      if (res.status === 409) {
        const data = await res.json().catch(() => ({}));
        if (data.code === 'already_configured') {
          onClose();
          return;
        }
        if (data.code === 'name_taken') {
          setApiError('That project name is already taken. Please choose a different name.');
          setSaving(false);
          return;
        }
        setApiError(data.error || 'Conflict. Please try a different name.');
        setSaving(false);
        return;
      }

      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        setApiError(data.error || 'Failed to save project name. Please try again.');
        setSaving(false);
        return;
      }

      const data = await res.json().catch(() => ({}));
      onConfigured({ project_name: value, fqdn: data.fqdn });
    } catch {
      setApiError('Network error. Please check your connection and try again.');
      setSaving(false);
    }
  }

  function handleClose() {
    if (saving) return;
    setValue('');
    setValidationError(null);
    setApiError(null);
    onClose();
  }

  const isInvalid = !!validationError && value.length > 0;

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle sx={{ fontWeight: 700 }}>Name your server</DialogTitle>
      <DialogContent>
        <DialogContentText sx={{ mb: 2 }}>
          Choose a project name. This becomes your server&apos;s subdomain:{' '}
          <Typography component="span" variant="body2" sx={{ fontFamily: 'monospace', fontWeight: 600 }}>
            {value || 'your-name'}.mahalaxmi.ai
          </Typography>
          <br />
          Lowercase letters, digits, and hyphens only. 3–40 characters. Cannot start or end with a hyphen.
          Once set, this cannot be changed.
        </DialogContentText>

        <TextField
          autoFocus
          fullWidth
          label="Project name"
          value={value}
          onChange={handleChange}
          error={isInvalid || !!apiError}
          helperText={
            apiError ||
            (isInvalid ? validationError : value ? `→ ${value}.mahalaxmi.ai` : ' ')
          }
          placeholder="my-project"
          inputProps={{ maxLength: 40 }}
          disabled={saving}
        />
      </DialogContent>
      <DialogActions sx={{ px: 3, pb: 3 }}>
        <Button onClick={handleClose} disabled={saving}>
          Cancel
        </Button>
        <Button
          variant="contained"
          onClick={handleSave}
          disabled={saving || !value || !!validationError}
          startIcon={saving ? <CircularProgress size={16} color="inherit" /> : null}
        >
          {saving ? 'Saving…' : 'Save project name'}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

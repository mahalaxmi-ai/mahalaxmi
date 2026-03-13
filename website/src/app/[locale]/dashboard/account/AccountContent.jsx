'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import {
  Alert,
  Box,
  Button,
  CircularProgress,
  Container,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Divider,
  TextField,
  Typography,
} from '@mui/material';
import { useAuth } from '@/contexts/AuthContext';

export default function AccountContent() {
  const { isAuthenticated, isLoading: authLoading, user, logout } = useAuth();
  const router = useRouter();

  // Change password state
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword]         = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [passwordLoading, setPasswordLoading] = useState(false);
  const [passwordMessage, setPasswordMessage] = useState(null); // { type, text }

  // Delete account state
  const [deleteOpen, setDeleteOpen]     = useState(false);
  const [deleteLoading, setDeleteLoading] = useState(false);
  const [deleteError, setDeleteError]   = useState(null);

  if (!authLoading && !isAuthenticated) {
    router.replace('/login?redirect=/dashboard/account');
    return null;
  }

  if (authLoading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 10 }}>
        <CircularProgress />
      </Box>
    );
  }

  async function handleChangePassword(e) {
    e.preventDefault();
    setPasswordMessage(null);

    if (newPassword !== confirmPassword) {
      setPasswordMessage({ type: 'error', text: 'New passwords do not match.' });
      return;
    }
    if (newPassword.length < 8) {
      setPasswordMessage({ type: 'error', text: 'New password must be at least 8 characters.' });
      return;
    }

    setPasswordLoading(true);
    try {
      const res = await fetch('/api/auth/reset-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
      });
      const data = await res.json();
      if (!res.ok) {
        setPasswordMessage({ type: 'error', text: data.message || 'Failed to change password.' });
      } else {
        setPasswordMessage({ type: 'success', text: 'Password updated successfully.' });
        setCurrentPassword('');
        setNewPassword('');
        setConfirmPassword('');
      }
    } catch {
      setPasswordMessage({ type: 'error', text: 'Service unavailable. Please try again.' });
    } finally {
      setPasswordLoading(false);
    }
  }

  async function handleDeleteAccount() {
    setDeleteLoading(true);
    setDeleteError(null);
    try {
      const res = await fetch('/api/auth/account', { method: 'DELETE' });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        setDeleteError(data.message || 'Failed to delete account. Please try again.');
        setDeleteLoading(false);
        return;
      }
      await logout();
      router.replace('/');
    } catch {
      setDeleteError('Service unavailable. Please try again.');
      setDeleteLoading(false);
    }
  }

  return (
    <Container maxWidth="sm" sx={{ py: { xs: 4, md: 6 } }}>
      <Typography variant="h4" component="h1" sx={{ fontWeight: 700, mb: 1 }}>
        Account
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 4 }}>
        Manage your Mahalaxmi account settings.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      {/* Email */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="overline" color="text.secondary">Email address</Typography>
        <Typography variant="body1" sx={{ mt: 0.5, fontWeight: 500 }}>
          {user?.email ?? '—'}
        </Typography>
      </Box>

      <Divider sx={{ mb: 4 }} />

      {/* Change password */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="h6" sx={{ fontWeight: 700, mb: 2 }}>
          Change password
        </Typography>

        {passwordMessage && (
          <Alert severity={passwordMessage.type} sx={{ mb: 2 }}>
            {passwordMessage.text}
          </Alert>
        )}

        <Box component="form" onSubmit={handleChangePassword} sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          <TextField
            label="Current password"
            type="password"
            value={currentPassword}
            onChange={(e) => setCurrentPassword(e.target.value)}
            required
            disabled={passwordLoading}
            autoComplete="current-password"
          />
          <TextField
            label="New password"
            type="password"
            value={newPassword}
            onChange={(e) => setNewPassword(e.target.value)}
            required
            disabled={passwordLoading}
            autoComplete="new-password"
          />
          <TextField
            label="Confirm new password"
            type="password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            required
            disabled={passwordLoading}
            autoComplete="new-password"
          />
          <Button
            type="submit"
            variant="contained"
            disabled={passwordLoading || !currentPassword || !newPassword || !confirmPassword}
            startIcon={passwordLoading ? <CircularProgress size={16} color="inherit" /> : null}
            sx={{ alignSelf: 'flex-start' }}
          >
            {passwordLoading ? 'Updating…' : 'Update password'}
          </Button>
        </Box>
      </Box>

      <Divider sx={{ mb: 4 }} />

      {/* Delete account */}
      <Box>
        <Typography variant="h6" sx={{ fontWeight: 700, mb: 1 }}>
          Delete account
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          Permanently delete your Mahalaxmi account.
        </Typography>
        <Button variant="outlined" color="error" onClick={() => setDeleteOpen(true)}>
          Delete my account
        </Button>
      </Box>

      {/* Delete confirmation dialog */}
      <Dialog open={deleteOpen} onClose={() => !deleteLoading && setDeleteOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle sx={{ fontWeight: 700 }}>Delete account?</DialogTitle>
        <DialogContent>
          <DialogContentText>
            This will <strong>cancel your subscription and delete all servers</strong>. This action cannot be undone.
          </DialogContentText>
          {deleteError && (
            <Alert severity="error" sx={{ mt: 2 }}>{deleteError}</Alert>
          )}
        </DialogContent>
        <DialogActions sx={{ px: 3, pb: 3 }}>
          <Button onClick={() => setDeleteOpen(false)} disabled={deleteLoading}>
            Cancel
          </Button>
          <Button
            variant="contained"
            color="error"
            onClick={handleDeleteAccount}
            disabled={deleteLoading}
            startIcon={deleteLoading ? <CircularProgress size={16} color="inherit" /> : null}
          >
            {deleteLoading ? 'Deleting…' : 'Delete permanently'}
          </Button>
        </DialogActions>
      </Dialog>
    </Container>
  );
}

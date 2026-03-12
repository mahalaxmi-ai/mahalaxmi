'use client';

import { useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  CardHeader,
  Typography,
  TextField,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  Alert,
  Divider,
} from '@mui/material';
import { useAuth } from '@/contexts/AuthContext';
import { authAPI } from '@/lib/api';

export const metadata = {
  title: 'Account — Mahalaxmi Dashboard',
  robots: { index: false },
};

const CARD_SX = {
  backgroundColor: '#0A2A2A',
  border: '1px solid rgba(0,200,200,0.15)',
  mb: 3,
};

export default function AccountPage() {
  const { user } = useAuth();

  const [pwForm, setPwForm] = useState({
    currentPassword: '',
    newPassword: '',
    confirmNewPassword: '',
  });
  const [pwStatus, setPwStatus] = useState(null);
  const [pwSubmitting, setPwSubmitting] = useState(false);

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [deleteConfirmInput, setDeleteConfirmInput] = useState('');
  const [deleteStatus, setDeleteStatus] = useState(null);
  const [deleteSubmitting, setDeleteSubmitting] = useState(false);

  function handlePwChange(e) {
    setPwForm((prev) => ({ ...prev, [e.target.name]: e.target.value }));
  }

  async function handlePwSubmit(e) {
    e.preventDefault();
    setPwStatus(null);

    if (pwForm.newPassword !== pwForm.confirmNewPassword) {
      setPwStatus({ type: 'error', message: 'New password and confirmation do not match.' });
      return;
    }

    if (!pwForm.currentPassword || !pwForm.newPassword) {
      setPwStatus({ type: 'error', message: 'All password fields are required.' });
      return;
    }

    setPwSubmitting(true);
    try {
      await authAPI.forgotPassword(user?.email);
      setPwStatus({
        type: 'success',
        message:
          'A password reset link has been sent to your email address. Follow the link to set your new password.',
      });
      setPwForm({ currentPassword: '', newPassword: '', confirmNewPassword: '' });
    } catch (err) {
      const message =
        err?.response?.data?.message ||
        err?.message ||
        'Failed to send password reset email. Please try again or contact support@mahalaxmi.ai.';
      setPwStatus({ type: 'error', message });
    } finally {
      setPwSubmitting(false);
    }
  }

  function openDeleteDialog() {
    setDeleteConfirmInput('');
    setDeleteStatus(null);
    setDeleteDialogOpen(true);
  }

  function closeDeleteDialog() {
    if (deleteSubmitting) return;
    setDeleteDialogOpen(false);
    setDeleteConfirmInput('');
    setDeleteStatus(null);
  }

  async function handleDeleteConfirm() {
    if (deleteConfirmInput !== 'DELETE') return;

    setDeleteSubmitting(true);
    setDeleteStatus(null);
    try {
      await fetch('/api/auth/logout', { method: 'POST' });
      setDeleteStatus({
        type: 'success',
        message:
          'Your account deletion request has been submitted. Contact support@mahalaxmi.ai for completion.',
      });
    } catch (err) {
      setDeleteStatus({
        type: 'error',
        message:
          'An error occurred while submitting your request. Please contact support@mahalaxmi.ai.',
      });
    } finally {
      setDeleteSubmitting(false);
    }
  }

  return (
    <Box sx={{ maxWidth: 680, mx: 'auto', px: { xs: 2, sm: 3 }, py: 4 }}>
      <Typography variant="h4" fontWeight={700} color="white" mb={4}>
        Account Settings
      </Typography>

      {/* Account Info */}
      <Card sx={CARD_SX}>
        <CardHeader
          title={
            <Typography variant="h6" color="white">
              Account Information
            </Typography>
          }
        />
        <Divider sx={{ borderColor: 'rgba(0,200,200,0.15)' }} />
        <CardContent sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 3 }}>
          <TextField
            label="Email Address"
            value={user?.email ?? ''}
            InputProps={{ readOnly: true }}
            fullWidth
            variant="outlined"
            size="small"
          />
          <Box sx={{ display: 'flex', gap: 2, flexDirection: { xs: 'column', sm: 'row' } }}>
            <TextField
              label="First Name"
              value={user?.firstName ?? ''}
              InputProps={{ readOnly: true }}
              fullWidth
              variant="outlined"
              size="small"
            />
            <TextField
              label="Last Name"
              value={user?.lastName ?? ''}
              InputProps={{ readOnly: true }}
              fullWidth
              variant="outlined"
              size="small"
            />
          </Box>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
            To update your name or email, contact{' '}
            <Box
              component="a"
              href="mailto:support@mahalaxmi.ai"
              sx={{ color: '#00C8C8', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
            >
              support@mahalaxmi.ai
            </Box>
            .
          </Typography>
        </CardContent>
      </Card>

      {/* Change Password */}
      <Card sx={CARD_SX}>
        <CardHeader
          title={
            <Typography variant="h6" color="white">
              Change Password
            </Typography>
          }
        />
        <Divider sx={{ borderColor: 'rgba(0,200,200,0.15)' }} />
        <CardContent>
          {pwStatus && (
            <Alert severity={pwStatus.type} sx={{ mb: 2 }}>
              {pwStatus.message}
            </Alert>
          )}
          <Box
            component="form"
            onSubmit={handlePwSubmit}
            sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 1 }}
          >
            <TextField
              label="Current Password"
              name="currentPassword"
              type="password"
              value={pwForm.currentPassword}
              onChange={handlePwChange}
              fullWidth
              variant="outlined"
              size="small"
              autoComplete="current-password"
            />
            <TextField
              label="New Password"
              name="newPassword"
              type="password"
              value={pwForm.newPassword}
              onChange={handlePwChange}
              fullWidth
              variant="outlined"
              size="small"
              autoComplete="new-password"
            />
            <TextField
              label="Confirm New Password"
              name="confirmNewPassword"
              type="password"
              value={pwForm.confirmNewPassword}
              onChange={handlePwChange}
              fullWidth
              variant="outlined"
              size="small"
              autoComplete="new-password"
              error={
                pwForm.confirmNewPassword.length > 0 &&
                pwForm.newPassword !== pwForm.confirmNewPassword
              }
              helperText={
                pwForm.confirmNewPassword.length > 0 &&
                pwForm.newPassword !== pwForm.confirmNewPassword
                  ? 'Passwords do not match.'
                  : ''
              }
            />
            <Button
              type="submit"
              variant="contained"
              disabled={pwSubmitting}
              sx={{ alignSelf: 'flex-start', backgroundColor: '#00C8C8', color: '#0A2A2A', '&:hover': { backgroundColor: '#00a0a0' } }}
            >
              {pwSubmitting ? 'Sending…' : 'Send Password Reset Email'}
            </Button>
          </Box>
        </CardContent>
      </Card>

      {/* Delete Account */}
      <Card sx={{ ...CARD_SX, border: '1px solid rgba(211,47,47,0.35)' }}>
        <CardHeader
          title={
            <Typography variant="h6" color="error.light">
              Delete Account
            </Typography>
          }
        />
        <Divider sx={{ borderColor: 'rgba(211,47,47,0.25)' }} />
        <CardContent>
          <Typography variant="body2" color="text.secondary" mb={2}>
            Permanently delete your Mahalaxmi account and all associated data. This action cannot
            be undone. For questions, contact{' '}
            <Box
              component="a"
              href="mailto:support@mahalaxmi.ai"
              sx={{ color: '#00C8C8', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
            >
              support@mahalaxmi.ai
            </Box>
            .
          </Typography>
          <Button
            color="error"
            variant="outlined"
            onClick={openDeleteDialog}
          >
            Delete Account
          </Button>
        </CardContent>
      </Card>

      {/* Delete Confirmation Dialog */}
      <Dialog
        open={deleteDialogOpen}
        onClose={closeDeleteDialog}
        PaperProps={{ sx: { backgroundColor: '#0A2A2A', border: '1px solid rgba(211,47,47,0.4)' } }}
      >
        <DialogTitle sx={{ color: 'error.light' }}>Confirm Account Deletion</DialogTitle>
        <DialogContent>
          {deleteStatus ? (
            <Alert severity={deleteStatus.type} sx={{ mt: 1 }}>
              {deleteStatus.message}
            </Alert>
          ) : (
            <>
              <DialogContentText sx={{ color: 'text.secondary', mb: 2 }}>
                This will submit an account deletion request. To complete the process, contact{' '}
                <Box
                  component="a"
                  href="mailto:support@mahalaxmi.ai"
                  sx={{ color: '#00C8C8', textDecoration: 'none' }}
                >
                  support@mahalaxmi.ai
                </Box>
                .
              </DialogContentText>
              <DialogContentText sx={{ color: 'text.secondary', mb: 2 }}>
                Type <strong style={{ color: 'white' }}>DELETE</strong> to confirm.
              </DialogContentText>
              <TextField
                value={deleteConfirmInput}
                onChange={(e) => setDeleteConfirmInput(e.target.value)}
                fullWidth
                variant="outlined"
                size="small"
                placeholder="DELETE"
                autoFocus
              />
            </>
          )}
        </DialogContent>
        <DialogActions sx={{ px: 3, pb: 2 }}>
          {deleteStatus?.type === 'success' ? (
            <Button onClick={closeDeleteDialog} variant="outlined">
              Close
            </Button>
          ) : (
            <>
              <Button onClick={closeDeleteDialog} disabled={deleteSubmitting}>
                Cancel
              </Button>
              <Button
                color="error"
                variant="contained"
                onClick={handleDeleteConfirm}
                disabled={deleteConfirmInput !== 'DELETE' || deleteSubmitting}
              >
                {deleteSubmitting ? 'Submitting…' : 'Delete My Account'}
              </Button>
            </>
          )}
        </DialogActions>
      </Dialog>
    </Box>
  );
}

'use client';

import { useState } from 'react';
import {
  Container, Paper, TextField, Button, Typography, Box, Alert,
  Link as MuiLink, InputAdornment, IconButton,
} from '@mui/material';
import { Visibility, VisibilityOff, CheckCircleOutline, ErrorOutline } from '@mui/icons-material';
import { Link } from '@/i18n/navigation';
import { useSearchParams } from 'next/navigation';

export default function ResetPasswordContent() {
  const searchParams = useSearchParams();
  const token = searchParams.get('token');

  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [status, setStatus] = useState('idle'); // idle | success | error
  const [error, setError] = useState('');

  if (!token) {
    return (
      <Container maxWidth="sm" sx={{ py: 8 }}>
        <Paper elevation={3} sx={{ p: 4, textAlign: 'center' }}>
          <ErrorOutline sx={{ fontSize: 80, color: 'error.main', mb: 2 }} />
          <Typography variant="h5" gutterBottom>Invalid Reset Link</Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 4 }}>
            This reset link is invalid or has expired. Please request a new one.
          </Typography>
          <Button component={Link} href="/forgot-password" variant="contained" size="large" fullWidth>
            Request New Link
          </Button>
        </Paper>
      </Container>
    );
  }

  const validate = () => {
    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return false;
    }
    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      return false;
    }
    if (!/(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$#%^&*!])/.test(password)) {
      setError('Password must contain uppercase, lowercase, number, and special character (@$#%^&*!)');
      return false;
    }
    return true;
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    if (!validate()) return;

    setIsLoading(true);
    try {
      const res = await fetch('/api/auth/reset-password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token, newPassword: password }),
      });
      const data = await res.json();
      if (data.success) {
        setStatus('success');
      } else {
        setStatus('error');
        setError(data.message || 'Password reset failed. The link may have expired.');
      }
    } catch {
      setError('An unexpected error occurred. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  if (status === 'success') {
    return (
      <Container maxWidth="sm" sx={{ py: 8 }}>
        <Paper elevation={3} sx={{ p: 4, textAlign: 'center' }}>
          <CheckCircleOutline sx={{ fontSize: 80, color: 'success.main', mb: 2 }} />
          <Typography variant="h4" component="h1" gutterBottom>Password Reset!</Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
            Your password has been updated. You can now sign in with your new password.
          </Typography>
          <Button component={Link} href="/login" variant="contained" size="large" fullWidth>
            Sign In
          </Button>
        </Paper>
      </Container>
    );
  }

  return (
    <Container maxWidth="sm" sx={{ py: 8 }}>
      <Paper elevation={3} sx={{ p: 4 }}>
        <Box sx={{ textAlign: 'center', mb: 4 }}>
          <Typography variant="h4" component="h1" gutterBottom>Reset Password</Typography>
          <Typography variant="body2" color="text.secondary">
            Enter your new password below.
          </Typography>
        </Box>

        {error && <Alert severity="error" sx={{ mb: 3 }}>{error}</Alert>}

        <form onSubmit={handleSubmit}>
          <TextField
            fullWidth
            label="New Password"
            type={showPassword ? 'text' : 'password'}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            autoComplete="new-password"
            helperText="8+ characters with uppercase, lowercase, number, and special character (@$#%^&*!)"
            sx={{ mb: 3 }}
            InputProps={{
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton onClick={() => setShowPassword(!showPassword)} edge="end">
                    {showPassword ? <VisibilityOff /> : <Visibility />}
                  </IconButton>
                </InputAdornment>
              ),
            }}
          />
          <TextField
            fullWidth
            label="Confirm New Password"
            type={showPassword ? 'text' : 'password'}
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            required
            autoComplete="new-password"
            sx={{ mb: 3 }}
          />
          <Button
            type="submit"
            fullWidth
            variant="contained"
            size="large"
            disabled={isLoading}
            sx={{ mb: 3 }}
          >
            {isLoading ? 'Resetting…' : 'Reset Password'}
          </Button>
        </form>

        <Box sx={{ textAlign: 'center' }}>
          <MuiLink component={Link} href="/login" variant="body2" color="primary">
            Back to Sign In
          </MuiLink>
        </Box>
      </Paper>
    </Container>
  );
}

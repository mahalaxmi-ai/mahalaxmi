'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import Head from 'next/head';
import Container from '@mui/material/Container';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Typography from '@mui/material/Typography';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import CircularProgress from '@mui/material/CircularProgress';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import Link from 'next/link';

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function validate(fields) {
  const errors = {};
  if (!fields.firstName.trim()) errors.firstName = 'First name is required.';
  if (!fields.lastName.trim()) errors.lastName = 'Last name is required.';
  if (!EMAIL_REGEX.test(fields.email)) errors.email = 'Enter a valid email address.';
  if (fields.password.length < 8) errors.password = 'Password must be at least 8 characters.';
  if (fields.confirmPassword !== fields.password) errors.confirmPassword = 'Passwords do not match.';
  return errors;
}

export default function RegisterPage() {
  const router = useRouter();
  const [fields, setFields] = useState({
    firstName: '',
    lastName: '',
    email: '',
    password: '',
    confirmPassword: '',
  });
  const [fieldErrors, setFieldErrors] = useState({});
  const [serverError, setServerError] = useState('');
  const [loading, setLoading] = useState(false);

  function handleChange(e) {
    const { name, value } = e.target;
    setFields((prev) => ({ ...prev, [name]: value }));
    setFieldErrors((prev) => ({ ...prev, [name]: '' }));
    if (name === 'email') setServerError('');
  }

  async function handleSubmit(e) {
    e.preventDefault();
    setServerError('');

    const errors = validate(fields);
    if (Object.keys(errors).length > 0) {
      setFieldErrors(errors);
      return;
    }

    setLoading(true);
    try {
      const res = await fetch('/api/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          firstName: fields.firstName.trim(),
          lastName: fields.lastName.trim(),
          email: fields.email.trim(),
          password: fields.password,
        }),
      });

      if (res.status === 201) {
        router.push('/verify-email');
        return;
      }

      if (res.status === 409) {
        setFieldErrors((prev) => ({
          ...prev,
          email: 'An account with this email already exists.',
        }));
        return;
      }

      if (res.status >= 500) {
        setServerError('A server error occurred. Please try again later.');
        return;
      }

      setServerError('Registration failed. Please try again.');
    } catch {
      setServerError('Unable to connect. Please check your connection and try again.');
    } finally {
      setLoading(false);
    }
  }

  return (
    <>
      <Head>
        <title>Create Account | Mahalaxmi AI</title>
      </Head>
      <Container maxWidth="sm" sx={{ py: 8, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
        <Card sx={{ width: '100%', maxWidth: 480, bgcolor: '#111827', border: '1px solid #1F2937' }}>
          <CardContent sx={{ p: 4 }}>
            <Typography variant="h5" component="h1" fontWeight={700} gutterBottom>
              Create Account
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
              Join Mahalaxmi AI and start orchestrating your AI workers.
            </Typography>

            {serverError && (
              <Alert severity="error" sx={{ mb: 3 }}>
                {serverError}
              </Alert>
            )}

            <Box component="form" onSubmit={handleSubmit} noValidate>
              <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
                <TextField
                  label="First Name"
                  name="firstName"
                  value={fields.firstName}
                  onChange={handleChange}
                  error={Boolean(fieldErrors.firstName)}
                  helperText={fieldErrors.firstName}
                  fullWidth
                  required
                  autoComplete="given-name"
                  size="small"
                />
                <TextField
                  label="Last Name"
                  name="lastName"
                  value={fields.lastName}
                  onChange={handleChange}
                  error={Boolean(fieldErrors.lastName)}
                  helperText={fieldErrors.lastName}
                  fullWidth
                  required
                  autoComplete="family-name"
                  size="small"
                />
              </Box>

              <TextField
                label="Email"
                name="email"
                type="email"
                value={fields.email}
                onChange={handleChange}
                error={Boolean(fieldErrors.email)}
                helperText={fieldErrors.email}
                fullWidth
                required
                autoComplete="email"
                size="small"
                sx={{ mb: 2 }}
              />

              <TextField
                label="Password"
                name="password"
                type="password"
                value={fields.password}
                onChange={handleChange}
                error={Boolean(fieldErrors.password)}
                helperText={fieldErrors.password || 'Minimum 8 characters'}
                fullWidth
                required
                autoComplete="new-password"
                size="small"
                sx={{ mb: 2 }}
              />

              <TextField
                label="Confirm Password"
                name="confirmPassword"
                type="password"
                value={fields.confirmPassword}
                onChange={handleChange}
                error={Boolean(fieldErrors.confirmPassword)}
                helperText={fieldErrors.confirmPassword}
                fullWidth
                required
                autoComplete="new-password"
                size="small"
                sx={{ mb: 3 }}
              />

              <Button
                type="submit"
                variant="contained"
                fullWidth
                disabled={loading}
                sx={{
                  py: 1.25,
                  bgcolor: '#00C8C8',
                  color: '#000',
                  fontWeight: 700,
                  '&:hover': { bgcolor: '#00AEAE' },
                  '&:disabled': { bgcolor: '#006666', color: '#444' },
                }}
              >
                {loading ? <CircularProgress size={22} sx={{ color: '#000' }} /> : 'Create Account'}
              </Button>
            </Box>

            <Typography variant="body2" color="text.secondary" sx={{ mt: 3, textAlign: 'center' }}>
              Already have an account?{' '}
              <Box
                component={Link}
                href="/login"
                sx={{ color: '#00C8C8', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
              >
                Log in
              </Box>
            </Typography>
          </CardContent>
        </Card>
      </Container>
    </>
  );
}

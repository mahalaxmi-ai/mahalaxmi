'use client';

import React from 'react';
import { Box, Container, Typography, Button, Alert } from '@mui/material';
import { RefreshOutlined } from '@mui/icons-material';

class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true, error };
  }

  componentDidCatch(error, errorInfo) {
    // Error details captured in state via getDerivedStateFromError
    void error;
    void errorInfo;
  }

  handleReload = () => {
    if (typeof window !== 'undefined') {
      window.location.reload();
    }
  };

  render() {
    if (this.state.hasError) {
      return (
        <Box
          sx={{
            minHeight: '100vh',
            display: 'flex',
            alignItems: 'center',
            bgcolor: 'background.default',
          }}
        >
          <Container maxWidth="md">
            <Box sx={{ textAlign: 'center', py: 8 }}>
              <Typography variant="h3" component="h1" gutterBottom>
                Oops! Something went wrong
              </Typography>
              <Typography variant="h6" color="text.secondary" paragraph>
                We&apos;re sorry, but something unexpected happened.
              </Typography>
              <Alert severity="error" sx={{ my: 3, textAlign: 'left' }}>
                {this.state.error?.message || 'An unexpected error occurred'}
              </Alert>
              <Typography variant="body1" color="text.secondary" paragraph>
                Please try refreshing the page. If the problem persists, contact our support team at{' '}
                <a href="mailto:support@mahalaxmi.ai">support@mahalaxmi.ai</a>.
              </Typography>
              <Button
                variant="contained"
                size="large"
                startIcon={<RefreshOutlined />}
                onClick={this.handleReload}
                sx={{ mt: 2 }}
              >
                Refresh Page
              </Button>
            </Box>
          </Container>
        </Box>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;

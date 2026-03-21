'use client';

import { Box, Container, Typography, Button, Divider, Paper } from '@mui/material';
import { Article, Comment } from '@mui/icons-material';
import Link from 'next/link';

export default function MfopOverviewContent() {
  return (
    <Box sx={{ minHeight: '100vh', pt: { xs: 12, md: 16 }, pb: 12, backgroundColor: 'background.default' }}>
      <Container maxWidth="md">
        {/* Header */}
        <Typography
          variant="overline"
          sx={{ color: 'primary.main', letterSpacing: '0.15em', fontWeight: 600, fontSize: '0.75rem' }}
        >
          Open Specification
        </Typography>
        <Typography
          variant="h2"
          sx={{ fontWeight: 800, mt: 1, mb: 2, fontSize: { xs: '2rem', md: '2.75rem' }, lineHeight: 1.15 }}
        >
          Mahalaxmi Federation and Orchestration Protocol
        </Typography>
        <Typography
          variant="h6"
          component="p"
          sx={{ color: 'text.secondary', fontWeight: 400, mb: 5, maxWidth: 640, lineHeight: 1.6 }}
        >
          MFOP is an open protocol for federated distributed AI orchestration across heterogeneous
          compute nodes — with compliance-zone-aware routing, cryptographically signed billing
          receipts, and configurable economic settlement.
        </Typography>

        <Divider sx={{ mb: 5, borderColor: 'rgba(0,200,200,0.15)' }} />

        {/* Two action cards */}
        <Box sx={{ display: 'flex', flexDirection: { xs: 'column', sm: 'row' }, gap: 3 }}>
          <Paper
            variant="outlined"
            sx={{
              flex: 1,
              p: 4,
              borderColor: 'rgba(0,200,200,0.25)',
              backgroundColor: 'rgba(0,200,200,0.04)',
              display: 'flex',
              flexDirection: 'column',
              gap: 2,
            }}
          >
            <Article sx={{ fontSize: 36, color: 'primary.main' }} />
            <Typography variant="h6" sx={{ fontWeight: 700 }}>
              Read the specification
            </Typography>
            <Typography variant="body2" sx={{ color: 'text.secondary', flex: 1 }}>
              The full pre-publication draft — protocol overview, message schema, routing model,
              billing receipts, and compliance zones.
            </Typography>
            <Button
              component={Link}
              href="/mfop/draft"
              variant="contained"
              sx={{
                alignSelf: 'flex-start',
                backgroundColor: '#00C8C8',
                color: '#000',
                fontWeight: 700,
                '&:hover': { backgroundColor: '#00AAAA' },
              }}
            >
              Read the draft
            </Button>
          </Paper>

          <Paper
            variant="outlined"
            sx={{
              flex: 1,
              p: 4,
              borderColor: 'rgba(255,255,255,0.1)',
              backgroundColor: 'rgba(255,255,255,0.02)',
              display: 'flex',
              flexDirection: 'column',
              gap: 2,
            }}
          >
            <Comment sx={{ fontSize: 36, color: 'text.secondary' }} />
            <Typography variant="h6" sx={{ fontWeight: 700 }}>
              Comment on this draft
            </Typography>
            <Typography variant="body2" sx={{ color: 'text.secondary', flex: 1 }}>
              Leave structured feedback on any section. Comments are open to distributed systems
              engineers, security researchers, and enterprise architects.
            </Typography>
            <Button
              component={Link}
              href="/mfop/discuss"
              variant="outlined"
              sx={{
                alignSelf: 'flex-start',
                borderColor: 'rgba(0,200,200,0.4)',
                color: '#00C8C8',
                '&:hover': { borderColor: '#00C8C8', backgroundColor: 'rgba(0,200,200,0.08)' },
              }}
            >
              Leave a comment
            </Button>
          </Paper>
        </Box>
      </Container>
    </Box>
  );
}

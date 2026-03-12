'use client';

import { Box, Button, Card, CardActions, CardContent, Grid, Typography } from '@mui/material';
import Link from 'next/link';

const DOC_CARDS = [
  {
    title: 'Quick Start',
    description: 'Get running in 3 commands',
    href: '/docs/quickstart',
  },
  {
    title: 'VS Code',
    description: 'Connect to your cloud server',
    href: '/docs/vscode',
  },
  {
    title: 'Cloud',
    description: 'Set up and manage cloud servers',
    href: '/docs/cloud',
  },
  {
    title: 'API Reference',
    description: 'Headless API reference',
    href: '/docs/api',
  },
  {
    title: 'FAQ',
    description: 'Common questions answered',
    href: '/docs/faq',
  },
];

export default function DocsPage() {
  return (
    <Box
      sx={{
        minHeight: '100vh',
        bgcolor: 'background.default',
        color: 'text.primary',
        px: { xs: 2, sm: 4, md: 8 },
        py: { xs: 4, md: 8 },
      }}
    >
      <Typography variant="h1" sx={{ mb: 6, fontSize: { xs: '2rem', md: '3rem' }, fontWeight: 700 }}>
        Documentation
      </Typography>

      <Grid container spacing={3}>
        {DOC_CARDS.map((card) => (
          <Grid item xs={12} sm={6} md={4} key={card.href}>
            <Card
              sx={{
                height: '100%',
                display: 'flex',
                flexDirection: 'column',
                bgcolor: 'background.paper',
                border: '1px solid',
                borderColor: 'divider',
              }}
            >
              <CardContent sx={{ flexGrow: 1 }}>
                <Typography variant="h5" component="h2" sx={{ mb: 1, fontWeight: 600 }}>
                  {card.title}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {card.description}
                </Typography>
              </CardContent>
              <CardActions sx={{ px: 2, pb: 2 }}>
                <Button
                  component={Link}
                  href={card.href}
                  variant="contained"
                  size="small"
                >
                  Read More
                </Button>
              </CardActions>
            </Card>
          </Grid>
        ))}
      </Grid>
    </Box>
  );
}

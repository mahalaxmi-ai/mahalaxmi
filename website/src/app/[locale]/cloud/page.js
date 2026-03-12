import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Box,
  Button,
  Card,
  CardContent,
  Container,
  Grid,
  Typography,
} from '@mui/material';
import {
  Cloud,
  Code,
  Bolt,
  Security,
  ArrowForward,
} from '@mui/icons-material';
import Link from 'next/link';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Mahalaxmi Cloud — Dedicated AI Orchestration Servers',
    description:
      'Run Mahalaxmi on a dedicated cloud server. One-click VS Code integration, persistent workers, automatic provisioning. Flat monthly subscription.',
    alternates: {
      canonical: getCanonical(locale, '/cloud'),
      languages: getAlternateLanguages('/cloud'),
    },
    openGraph: {
      title: 'Mahalaxmi Cloud — Dedicated AI Orchestration Servers',
      description:
        'Run Mahalaxmi on a dedicated cloud server with VS Code deep-link integration.',
      url: '/cloud',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const features = [
  {
    icon: <Cloud sx={{ fontSize: 36 }} />,
    title: 'Dedicated VM',
    body: 'Your own server — no shared resources, no noisy neighbours. Persistent across sessions.',
  },
  {
    icon: <Code sx={{ fontSize: 36 }} />,
    title: 'VS Code Integration',
    body: 'One-click "Open in VS Code" button connects the extension to your cloud server automatically.',
  },
  {
    icon: <Bolt sx={{ fontSize: 36 }} />,
    title: 'Instant Provisioning',
    body: 'Your server is ready within minutes of completing checkout. No setup, no SSH keys.',
  },
  {
    icon: <Security sx={{ fontSize: 36 }} />,
    title: 'Encrypted API Keys',
    body: 'AI provider keys are encrypted with AES-256-GCM. They never leave your VM and are never visible to us.',
  },
];

export default async function CloudPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 8 } }}>
      <Box sx={{ textAlign: 'center', mb: 8 }}>
        <Typography variant="h2" component="h1" sx={{ fontWeight: 800, mb: 2 }}>
          Mahalaxmi Cloud
        </Typography>
        <Typography variant="h5" color="text.secondary" sx={{ maxWidth: 640, mx: 'auto', mb: 4 }}>
          Your dedicated AI orchestration server — always on, always ready. Connect from VS Code in one click.
        </Typography>
        <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
          <Button
            component={Link}
            href="/cloud/pricing"
            variant="contained"
            size="large"
            endIcon={<ArrowForward />}
            sx={{ px: 4, fontWeight: 700 }}
          >
            See Pricing
          </Button>
          <Button
            component={Link}
            href="/docs/cloud"
            variant="outlined"
            size="large"
            sx={{ px: 4 }}
          >
            View Docs
          </Button>
        </Box>
      </Box>

      <Grid container spacing={3} sx={{ mb: 8 }}>
        {features.map((f) => (
          <Grid item xs={12} sm={6} key={f.title}>
            <Card elevation={0} variant="outlined" sx={{ height: '100%', borderRadius: 3, p: 1 }}>
              <CardContent>
                <Box sx={{ color: 'primary.main', mb: 1.5 }}>{f.icon}</Box>
                <Typography variant="h6" sx={{ fontWeight: 700, mb: 1 }}>{f.title}</Typography>
                <Typography variant="body2" color="text.secondary">{f.body}</Typography>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      <Box
        sx={{
          textAlign: 'center',
          bgcolor: 'primary.main',
          color: 'primary.contrastText',
          borderRadius: 4,
          py: 6,
          px: 3,
        }}
      >
        <Typography variant="h4" sx={{ fontWeight: 800, mb: 2 }}>
          Ready to get started?
        </Typography>
        <Typography variant="h6" sx={{ mb: 4, opacity: 0.85 }}>
          Flat monthly subscription. Cancel anytime. No long-term commitment.
        </Typography>
        <Button
          component={Link}
          href="/cloud/pricing"
          variant="contained"
          size="large"
          sx={{
            bgcolor: 'white',
            color: 'primary.main',
            fontWeight: 700,
            px: 5,
            '&:hover': { bgcolor: 'grey.100' },
          }}
          endIcon={<ArrowForward />}
        >
          View Plans
        </Button>
      </Box>
    </Container>
  );
}

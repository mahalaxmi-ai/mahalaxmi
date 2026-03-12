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
import { ArrowForward, Bolt, Cloud, FolderOpen } from '@mui/icons-material';
import Link from 'next/link';
import Image from 'next/image';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Mahalaxmi — AI Terminal Orchestration',
    description:
      'Mahalaxmi orchestrates AI coding tools so you can run multiple parallel agents across your codebase. Available as a desktop app and a dedicated cloud server.',
    alternates: {
      canonical: getCanonical(locale, '/'),
      languages: getAlternateLanguages('/'),
    },
    openGraph: {
      title: 'Mahalaxmi — AI Terminal Orchestration',
      description:
        'Run multiple AI coding agents in parallel. Cloud or desktop, VS Code or terminal.',
      url: '/',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const highlights = [
  {
    icon: <Bolt sx={{ fontSize: 40 }} />,
    title: 'Parallel AI Workers',
    body: 'Spawn dozens of AI coding agents simultaneously across your codebase. Mahalaxmi manages the orchestration so you stay focused on results.',
  },
  {
    icon: <Cloud sx={{ fontSize: 40 }} />,
    title: 'Cloud Servers',
    body: 'Dedicated VM, always on. Connect from VS Code in one click and run workers 24/7 without leaving your laptop open.',
  },
  {
    icon: <FolderOpen sx={{ fontSize: 40 }} />,
    title: 'Open Source Core',
    body: 'The Mahalaxmi orchestration engine is open source. Inspect, extend, and contribute on GitHub.',
  },
];

export default async function HomePage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <>
      {/* Hero */}
      <Box
        sx={{
          background: 'linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%)',
          color: 'white',
          py: { xs: 10, md: 16 },
        }}
      >
        <Container maxWidth="lg">
          <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center' }}>
            <Box sx={{ mb: 4 }}>
              <Image
                src="/mahalaxmi_logo.png"
                alt="Mahalaxmi logo"
                width={96}
                height={96}
                style={{ borderRadius: 16 }}
                priority
              />
            </Box>
            <Typography
              variant="h1"
              sx={{ fontWeight: 900, fontSize: { xs: '2.5rem', md: '4rem' }, mb: 2, lineHeight: 1.1 }}
            >
              AI Terminal Orchestration
            </Typography>
            <Typography
              variant="h5"
              sx={{ maxWidth: 640, mb: 5, opacity: 0.8, fontWeight: 400 }}
            >
              Mahalaxmi runs multiple AI coding agents in parallel so you ship faster.
              Desktop app or dedicated cloud server — your choice.
            </Typography>
            <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap', justifyContent: 'center' }}>
              <Button
                component={Link}
                href="/pricing"
                variant="contained"
                size="large"
                endIcon={<ArrowForward />}
                sx={{ px: 5, py: 1.5, fontWeight: 700, fontSize: '1rem' }}
              >
                Get Started
              </Button>
              <Button
                component={Link}
                href="/open-source"
                variant="outlined"
                size="large"
                sx={{
                  px: 5,
                  py: 1.5,
                  fontWeight: 700,
                  fontSize: '1rem',
                  borderColor: 'rgba(255,255,255,0.5)',
                  color: 'white',
                  '&:hover': { borderColor: 'white', bgcolor: 'rgba(255,255,255,0.08)' },
                }}
              >
                Open Source
              </Button>
            </Box>
          </Box>
        </Container>
      </Box>

      {/* Highlights */}
      <Container maxWidth="lg" sx={{ py: { xs: 6, md: 10 } }}>
        <Grid container spacing={4}>
          {highlights.map((h) => (
            <Grid item xs={12} md={4} key={h.title}>
              <Card elevation={0} variant="outlined" sx={{ height: '100%', borderRadius: 3, p: 1 }}>
                <CardContent>
                  <Box sx={{ color: 'primary.main', mb: 2 }}>{h.icon}</Box>
                  <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>{h.title}</Typography>
                  <Typography variant="body1" color="text.secondary">{h.body}</Typography>
                </CardContent>
              </Card>
            </Grid>
          ))}
        </Grid>
      </Container>

      {/* CTA band */}
      <Box sx={{ bgcolor: 'primary.main', py: { xs: 6, md: 8 } }}>
        <Container maxWidth="md" sx={{ textAlign: 'center' }}>
          <Typography variant="h4" sx={{ fontWeight: 800, color: 'white', mb: 2 }}>
            Start orchestrating today
          </Typography>
          <Typography variant="h6" sx={{ color: 'rgba(255,255,255,0.8)', mb: 4 }}>
            Desktop license or cloud subscription — flat pricing, cancel anytime.
          </Typography>
          <Button
            component={Link}
            href="/pricing"
            variant="contained"
            size="large"
            endIcon={<ArrowForward />}
            sx={{
              bgcolor: 'white',
              color: 'primary.main',
              fontWeight: 700,
              px: 5,
              '&:hover': { bgcolor: 'grey.100' },
            }}
          >
            View Pricing
          </Button>
        </Container>
      </Box>
    </>
  );
}

import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import { fetchCloudPricing } from '@/lib/cloudPricing';
import {
  Container, Box, Typography, Button, Grid, Card, CardContent,
  Chip, Paper, Table, TableBody, TableCell, TableContainer, TableHead, TableRow,
} from '@mui/material';
import {
  Cloud, Extension, Lock, Speed, CheckCircle,
  ArrowForward, Storage, Code, Bolt, Security,
} from '@mui/icons-material';
import Link from 'next/link';
import CloudPricingDisplay from './pricing/CloudPricingDisplay';

export const dynamic = 'force-dynamic';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Mahalaxmi Cloud — Hosted AI Orchestration, No Local Setup',
    description: 'Run Mahalaxmi headless on a dedicated cloud server. Flat monthly subscription. Connect from VS Code in seconds. No Docker, no config, no local setup required.',
    alternates: {
      canonical: getCanonical(locale, '/cloud'),
      languages: getAlternateLanguages('/cloud'),
    },
    openGraph: {
      title: 'Mahalaxmi Cloud — Hosted AI Orchestration, No Local Setup',
      description: 'Your own Mahalaxmi server, provisioned in minutes. Flat monthly subscription. Connect from VS Code. Cancel anytime.',
      url: '/cloud',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const steps = [
  {
    number: '1',
    title: 'Pick a plan and launch',
    body: 'Choose your server size. We provision a dedicated Hetzner VM running mahalaxmi-service — your own isolated orchestration engine — in under 3 minutes.',
  },
  {
    number: '2',
    title: 'Connect from VS Code',
    body: 'Click "Open in VS Code" on your dashboard. The Mahalaxmi VS Code extension connects to your server automatically. No SSH, no Docker, no port forwarding.',
  },
  {
    number: '3',
    title: 'Run cycles. Stop when done.',
    body: 'Point Mahalaxmi at your codebase, add AI provider keys, and run cycles exactly as you would locally — from any machine, anywhere. Your project index and memory persist across sessions.',
  },
];

const localVsCloud = [
  ['Setup time', 'Install app, configure providers', '~3 minutes, one click'],
  ['Machine resources', 'Your CPU, RAM, and battery', 'Dedicated cloud VM, your machine stays fast'],
  ['Concurrent workers', 'Limited by local CPU/RAM', 'Scale to the VM size you need'],
  ['Access from multiple machines', 'Not supported', 'Any machine with VS Code + the extension'],
  ['Always-on automation', 'Requires your machine on', 'Server runs 24/7 if you want it to'],
  ['Cost model', 'One-time purchase or Pro license', 'Flat monthly server subscription + your AI costs'],
];

const features = [
  {
    icon: <Bolt />,
    title: 'Provisioned in minutes',
    body: 'Your dedicated VM is ready in under 3 minutes from the moment you complete checkout. No waiting for a support ticket.',
  },
  {
    icon: <Extension />,
    title: 'VS Code native',
    body: 'The Mahalaxmi VS Code extension handles the remote connection. One click on your dashboard opens a live, connected session.',
  },
  {
    icon: <Lock />,
    title: 'Your server, your keys',
    body: 'AI provider API keys are stored only on your VM — never sent to Mahalaxmi. Your code never leaves your machine or your server.',
  },
  {
    icon: <Storage />,
    title: 'Persistent project state',
    body: 'Your codebase index, shared memory, and cycle history persist across sessions. Stop the server, restart it — everything is still there.',
  },
  {
    icon: <Speed />,
    title: 'Cancel anytime',
    body: 'Flat monthly subscription — no long-term contract, no exit fees. Delete your server when a project ends; create a new one in minutes when the next one begins.',
  },
  {
    icon: <Code />,
    title: 'Full headless API',
    body: 'mahalaxmi-service exposes a REST + SSE API. Automate cycles from CI/CD pipelines without ever opening VS Code.',
  },
];


export default async function CloudPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  const pricingData = await fetchCloudPricing();
  const lowestPrice = pricingData?.pricingTiers?.[0]?.pricing?.monthly;

  return (
    <Box>
      {/* Hero */}
      <Box sx={{ bgcolor: 'primary.dark', color: 'white', py: { xs: 8, md: 14 }, textAlign: 'center' }}>
        <Container maxWidth="md">
          <Chip
            label="New — Mahalaxmi Cloud"
            sx={{ mb: 3, bgcolor: 'rgba(255,255,255,0.15)', color: 'white', fontWeight: 600 }}
          />
          <Typography variant="h2" component="h1" sx={{ fontWeight: 800, mb: 3, fontSize: { xs: '2rem', md: '3rem' } }}>
            Mahalaxmi. Hosted. No setup.
          </Typography>
          <Typography variant="h6" sx={{ mb: 5, opacity: 0.9, maxWidth: 640, mx: 'auto' }}>
            Your own dedicated orchestration server, provisioned in minutes. Connect from VS Code. Run a team of AI agents without touching Docker or config files. Stop the server when you&apos;re done.
          </Typography>
          <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
            <Button
              component={Link}
              href="/cloud/pricing"
              variant="contained"
              size="large"
              startIcon={<Cloud />}
              sx={{ bgcolor: 'white', color: 'primary.dark', '&:hover': { bgcolor: 'grey.100' }, fontWeight: 700 }}
            >
              See Plans &amp; Launch
            </Button>
            <Button
              component={Link}
              href="/cloud/pricing"
              variant="outlined"
              size="large"
              endIcon={<ArrowForward />}
              sx={{ borderColor: 'white', color: 'white', '&:hover': { borderColor: 'white', bgcolor: 'rgba(255,255,255,0.1)' } }}
            >
              See pricing
            </Button>
          </Box>
          <Typography variant="body2" sx={{ mt: 3, opacity: 0.7 }}>
            {lowestPrice != null
              ? `From $${lowestPrice}/month. You bring your own AI provider keys — we run the infrastructure.`
              : 'You bring your own AI provider keys — we run the infrastructure.'}
          </Typography>
        </Container>
      </Box>

      {/* How it works */}
      <Box sx={{ py: { xs: 6, md: 10 } }}>
        <Container maxWidth="md">
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 2, textAlign: 'center' }}>
            From zero to running in 3 minutes
          </Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 6, textAlign: 'center' }}>
            No Docker. No SSH. No configuration files. Just a working orchestration engine in your VS Code.
          </Typography>
          <Grid container spacing={4}>
            {steps.map(({ number, title, body }) => (
              <Grid item xs={12} md={4} key={number}>
                <Box sx={{ textAlign: 'center' }}>
                  <Box
                    sx={{
                      width: 56, height: 56, borderRadius: '50%',
                      bgcolor: 'primary.main', color: 'white',
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                      fontWeight: 800, fontSize: '1.5rem',
                      mx: 'auto', mb: 2,
                    }}
                  >
                    {number}
                  </Box>
                  <Typography variant="h6" sx={{ fontWeight: 700, mb: 1 }}>{title}</Typography>
                  <Typography variant="body2" color="text.secondary">{body}</Typography>
                </Box>
              </Grid>
            ))}
          </Grid>
        </Container>
      </Box>

      {/* Local vs Cloud comparison */}
      <Box sx={{ bgcolor: 'grey.50', py: { xs: 6, md: 10 } }}>
        <Container maxWidth="md">
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 4, textAlign: 'center' }}>
            Local desktop vs. Mahalaxmi Cloud
          </Typography>
          <TableContainer component={Paper} elevation={2}>
            <Table>
              <TableHead>
                <TableRow sx={{ bgcolor: 'grey.800' }}>
                  <TableCell sx={{ color: 'white', fontWeight: 600 }}></TableCell>
                  <TableCell sx={{ color: 'white', fontWeight: 600 }}>Mahalaxmi Desktop</TableCell>
                  <TableCell sx={{ color: 'primary.light', fontWeight: 700 }}>Mahalaxmi Cloud</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {localVsCloud.map(([aspect, local, cloud]) => (
                  <TableRow key={aspect} sx={{ '&:nth-of-type(odd)': { bgcolor: 'grey.50' } }}>
                    <TableCell sx={{ fontWeight: 600 }}>{aspect}</TableCell>
                    <TableCell color="text.secondary">{local}</TableCell>
                    <TableCell sx={{ color: 'success.main', fontWeight: 500 }}>
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
                        <CheckCircle fontSize="small" />
                        {cloud}
                      </Box>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </Container>
      </Box>

      {/* Features */}
      <Box sx={{ py: { xs: 6, md: 10 } }}>
        <Container maxWidth="lg">
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 6, textAlign: 'center' }}>
            Everything you need. Nothing you don&apos;t.
          </Typography>
          <Grid container spacing={3}>
            {features.map(({ icon, title, body }) => (
              <Grid item xs={12} sm={6} md={4} key={title}>
                <Card elevation={1} sx={{ height: '100%' }}>
                  <CardContent sx={{ p: 3 }}>
                    <Box sx={{ color: 'primary.main', mb: 1.5 }}>{icon}</Box>
                    <Typography variant="h6" sx={{ fontWeight: 600, mb: 1 }}>{title}</Typography>
                    <Typography variant="body2" color="text.secondary">{body}</Typography>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        </Container>
      </Box>

      {/* Server tiers */}
      <Box sx={{ bgcolor: 'grey.50', py: { xs: 6, md: 10 } }}>
        <Container maxWidth="lg">
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 2, textAlign: 'center' }}>
            Choose your server size
          </Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 2, textAlign: 'center' }}>
            All prices are for server infrastructure only. AI provider costs (Claude, OpenAI, etc.) are billed directly by your provider.
          </Typography>

          {pricingData ? (
            <CloudPricingDisplay pricingData={pricingData} />
          ) : (
            <Box
              sx={{
                textAlign: 'center',
                p: { xs: 4, md: 6 },
                borderRadius: 4,
                bgcolor: 'white',
                border: '1px solid',
                borderColor: 'warning.light',
                boxShadow: '0 4px 20px rgba(237, 108, 2, 0.1)',
              }}
            >
              <Box
                sx={{
                  width: 60, height: 60, borderRadius: '50%',
                  bgcolor: 'warning.light',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  mx: 'auto', mb: 3,
                }}
              >
                <Security sx={{ fontSize: 32, color: 'warning.dark' }} />
              </Box>
              <Typography variant="h5" sx={{ fontWeight: 700, mb: 2 }}>Contact Support for Pricing</Typography>
              <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
                <Button component={Link} href="/contact" variant="contained" endIcon={<ArrowForward />}>
                  Contact Support
                </Button>
                <Button component="a" href="mailto:support@mahalaxmi.ai" variant="outlined">
                  support@mahalaxmi.ai
                </Button>
              </Box>
            </Box>
          )}

          <Box sx={{ textAlign: 'center', mt: 3 }}>
            <Button component={Link} href="/cloud/pricing" endIcon={<ArrowForward />} variant="text">
              Full pricing breakdown
            </Button>
          </Box>
        </Container>
      </Box>

      {/* Privacy callout */}
      <Box sx={{ py: { xs: 6, md: 10 } }}>
        <Container maxWidth="sm">
          <Paper elevation={0} variant="outlined" sx={{ p: 4, borderLeft: '4px solid', borderColor: 'primary.main', borderRadius: '0 8px 8px 0' }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
              <Lock color="primary" />
              <Typography variant="h6" sx={{ fontWeight: 700 }}>Your code never touches our servers</Typography>
            </Box>
            <Typography variant="body2" color="text.secondary">
              Your source code stays on your machine. AI provider API calls go directly from your VM to your provider&apos;s endpoint — Mahalaxmi never proxies, receives, or stores your prompts, completions, or code. The only data Mahalaxmi receives is a license token and machine fingerprint for validation.
            </Typography>
          </Paper>
        </Container>
      </Box>

      {/* Bottom CTA */}
      <Box sx={{ bgcolor: 'primary.dark', color: 'white', py: { xs: 8, md: 12 }, textAlign: 'center' }}>
        <Container maxWidth="sm">
          <Typography variant="h4" sx={{ fontWeight: 700, mb: 2 }}>
            Ready to run a team of AI agents in the cloud?
          </Typography>
          <Typography variant="body1" sx={{ mb: 4, opacity: 0.9 }}>
            No setup. No local config. Just a working orchestration engine in your VS Code — provisioned in minutes.
          </Typography>
          <Button
            component={Link}
            href="/cloud/pricing"
            variant="contained"
            size="large"
            startIcon={<Cloud />}
            sx={{ bgcolor: 'white', color: 'primary.dark', '&:hover': { bgcolor: 'grey.100' }, fontWeight: 700 }}
          >
            {lowestPrice != null ? `Launch Your Server — From $${lowestPrice}/mo` : 'See Plans & Launch'}
          </Button>
          <Typography variant="body2" sx={{ mt: 2, opacity: 0.7 }}>
            Flat monthly subscription. Cancel anytime. Bring your own AI provider keys.
          </Typography>
        </Container>
      </Box>
    </Box>
  );
}

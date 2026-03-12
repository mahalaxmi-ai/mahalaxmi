import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Container,
  Box,
  Typography,
  Button,
  Paper,
  Breadcrumbs,
  Accordion,
  AccordionSummary,
  AccordionDetails,
} from '@mui/material';
import { ExpandMore, NavigateNext, ArrowForward } from '@mui/icons-material';
import Link from 'next/link';
import CloudPricingDisplay from './CloudPricingDisplay';

export const dynamic = 'force-dynamic';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Cloud Pricing — Mahalaxmi',
    description:
      'Mahalaxmi Cloud pricing: flat monthly subscription for your dedicated orchestration VM. AI provider costs are separate — you bring your own keys.',
    alternates: {
      canonical: getCanonical(locale, '/cloud/pricing'),
      languages: getAlternateLanguages('/cloud/pricing'),
    },
    openGraph: {
      title: 'Cloud Pricing — Mahalaxmi',
      description:
        'Flat monthly subscription for your dedicated orchestration VM. Cancel anytime, no long-term contract.',
      url: '/cloud/pricing',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const faqs = [
  {
    q: 'When does billing start and stop?',
    a: 'Billing starts the moment your Stripe subscription is confirmed at checkout. Your server is provisioned automatically and your subscription renews monthly via Stripe. To stop billing, cancel your subscription from your dashboard — your server will be deprovisioned at the end of the billing period.',
  },
  {
    q: 'Do I pay for AI tokens separately?',
    a: 'Yes. Mahalaxmi Cloud pricing covers your dedicated orchestration VM only. AI provider costs (Claude Code, OpenAI, AWS Bedrock, Google Gemini, etc.) are billed directly by your provider based on your usage and their pricing.',
  },
  {
    q: 'What happens to my data when I stop the server?',
    a: 'Stopping the server pauses the VM but preserves your disk. Your codebase index, project memory, cycle history, and configuration all persist. Restarting restores exactly where you left off. Deleting a server permanently removes all associated data.',
  },
  {
    q: 'Is there a minimum commitment or contract?',
    a: 'No long-term contract. Mahalaxmi Cloud is a month-to-month Stripe subscription. Cancel anytime with no cancellation fee. Your server remains active until the end of the current billing period.',
  },
  {
    q: 'How does VS Code connect to my server?',
    a: 'The Mahalaxmi VS Code extension handles the connection automatically. After provisioning you receive an "Open in VS Code" button that passes a secure connection URL via a deep link (vscode://thrivetech.mahalaxmi/configure). No manual SSH or config required.',
  },
  {
    q: 'Are my API keys safe?',
    a: 'Yes. Your AI provider keys are encrypted (AES-256-GCM) before being stored and are only decrypted on your VM during the boot sequence. They are never logged, never returned by any API, and never visible to ThriveTech staff.',
  },
  {
    q: 'Where are the servers located?',
    a: "All servers are provisioned in Hetzner's Helsinki (EU) data center. Your project gets a dedicated subdomain at proj-{id}.mahalaxmi.ai with a TLS certificate provisioned automatically during boot.",
  },
  {
    q: 'Do I need a Mahalaxmi license?',
    a: 'Mahalaxmi Cloud includes your license — you do not need a separate desktop license. The subscription covers both the orchestration software and the cloud server.',
  },
];

export default async function CloudPricingPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 8 } }}>
      <Breadcrumbs separator={<NavigateNext fontSize="small" />} sx={{ mb: 3 }}>
        <Link href="/cloud" style={{ textDecoration: 'none', color: 'inherit' }}>Cloud</Link>
        <Typography color="text.primary">Pricing</Typography>
      </Breadcrumbs>

      <Box sx={{ textAlign: 'center', mb: 6 }}>
        <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 2 }}>
          Simple monthly subscription
        </Typography>
        <Typography variant="h6" color="text.secondary" sx={{ maxWidth: 600, mx: 'auto' }}>
          One flat monthly price for your dedicated orchestration server. No hourly meters, no usage math.
          AI provider costs go directly to your provider — not through us.
        </Typography>
      </Box>

      <CloudPricingDisplay />

      <Paper
        elevation={0}
        variant="outlined"
        sx={{ p: 3, mt: 6, mb: 8, borderLeft: '4px solid', borderColor: 'info.main', borderRadius: '0 8px 8px 0' }}
      >
        <Typography variant="body2" color="text.secondary">
          <strong>AI provider costs not included.</strong> Mahalaxmi Cloud covers your dedicated VM only.
          AI provider costs (Claude Code, OpenAI, AWS Bedrock, Google Gemini, etc.) are billed directly by your provider.
          You add your own API keys to your server during setup — they are encrypted at rest and never leave your VM.
        </Typography>
      </Paper>

      <Box sx={{ mb: 8 }}>
        <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 4 }}>
          Frequently asked questions
        </Typography>
        {faqs.map(({ q, a }) => (
          <Accordion key={q} elevation={1} sx={{ mb: 1 }}>
            <AccordionSummary expandIcon={<ExpandMore />}>
              <Typography variant="body1" sx={{ fontWeight: 600 }}>{q}</Typography>
            </AccordionSummary>
            <AccordionDetails>
              <Typography variant="body2" color="text.secondary">{a}</Typography>
            </AccordionDetails>
          </Accordion>
        ))}
      </Box>

      <Box sx={{ textAlign: 'center', py: 4, borderTop: '1px solid', borderColor: 'divider' }}>
        <Button
          component={Link}
          href="/cloud"
          variant="outlined"
          size="large"
          endIcon={<ArrowForward />}
        >
          Learn how it works
        </Button>
      </Box>
    </Container>
  );
}

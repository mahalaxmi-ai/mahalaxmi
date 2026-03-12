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
  Bolt,
  Cloud,
  Code,
  FolderOpen,
  Security,
  SyncAlt,
  ArrowForward,
} from '@mui/icons-material';
import Link from 'next/link';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Features — Mahalaxmi',
    description:
      'Mahalaxmi features: parallel AI workers, VS Code integration, dedicated cloud servers, encrypted API key storage, and an open-source orchestration core.',
    alternates: {
      canonical: getCanonical(locale, '/features'),
      languages: getAlternateLanguages('/features'),
    },
    openGraph: {
      title: 'Features — Mahalaxmi',
      description: 'Parallel AI workers, VS Code integration, cloud servers, and more.',
      url: '/features',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const features = [
  {
    icon: <Bolt sx={{ fontSize: 44 }} />,
    title: 'Parallel AI Workers',
    body: 'Spawn dozens of AI coding agents simultaneously. Each worker gets its own context, tools, and task scope. Mahalaxmi coordinates them so changes don\'t conflict.',
  },
  {
    icon: <Code sx={{ fontSize: 44 }} />,
    title: 'VS Code Integration',
    body: 'The Mahalaxmi VS Code extension lets you review, accept, or reject every file change proposed by a worker. Full diff view, inline comments, and keyboard shortcuts.',
  },
  {
    icon: <Cloud sx={{ fontSize: 44 }} />,
    title: 'Cloud Servers',
    body: 'Dedicated VM in Hetzner\'s Helsinki data center. Workers run 24/7 without your laptop. One-click connection via VS Code deep link — no SSH, no manual config.',
  },
  {
    icon: <Security sx={{ fontSize: 44 }} />,
    title: 'Encrypted API Keys',
    body: 'Your AI provider keys are encrypted with AES-256-GCM at rest on your VM. They are decrypted only during the boot sequence and are never logged or transmitted.',
  },
  {
    icon: <SyncAlt sx={{ fontSize: 44 }} />,
    title: 'Multi-Provider Support',
    body: 'Claude Code, Codex CLI, Gemini CLI, Grok CLI, GitHub Copilot, and local models via Ollama. Switch providers per worker or per project.',
  },
  {
    icon: <FolderOpen sx={{ fontSize: 44 }} />,
    title: 'Open Source Core',
    body: 'The Mahalaxmi orchestration engine is MIT-licensed. Inspect the source, contribute improvements, or self-host the core on your own infrastructure.',
  },
];

export default async function FeaturesPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 8 } }}>
      <Box sx={{ textAlign: 'center', mb: 8 }}>
        <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 2 }}>
          Everything you need to orchestrate AI
        </Typography>
        <Typography variant="h6" color="text.secondary" sx={{ maxWidth: 580, mx: 'auto' }}>
          Mahalaxmi handles the complexity of running parallel AI coding agents so you can focus on the outcome.
        </Typography>
      </Box>

      <Grid container spacing={4} sx={{ mb: 10 }}>
        {features.map((f) => (
          <Grid item xs={12} sm={6} lg={4} key={f.title}>
            <Card
              elevation={0}
              variant="outlined"
              sx={{ height: '100%', borderRadius: 3, p: 1 }}
            >
              <CardContent>
                <Box sx={{ color: 'primary.main', mb: 2 }}>{f.icon}</Box>
                <Typography variant="h6" sx={{ fontWeight: 700, mb: 1 }}>
                  {f.title}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {f.body}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      <Box sx={{ textAlign: 'center', borderTop: '1px solid', borderColor: 'divider', pt: 6 }}>
        <Typography variant="h5" sx={{ fontWeight: 700, mb: 2 }}>
          Ready to see it in action?
        </Typography>
        <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
          <Button
            component={Link}
            href="/pricing"
            variant="contained"
            size="large"
            endIcon={<ArrowForward />}
          >
            Get Started
          </Button>
          <Button
            component={Link}
            href="/docs/quickstart"
            variant="outlined"
            size="large"
          >
            Read the Docs
          </Button>
        </Box>
      </Box>
    </Container>
  );
}

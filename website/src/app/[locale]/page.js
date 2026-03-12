import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Container, Box, Typography, Button, Grid, Card, CardContent,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow,
  Paper, Chip, Divider,
} from '@mui/material';
import {
  CheckCircle, Download, Speed, Security, Psychology, Terminal,
  Groups, ArrowForward, Cloud,
} from '@mui/icons-material';
import Link from 'next/link';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Mahalaxmi — Multi-Agent AI Terminal Orchestration',
    description: 'Run a whole engineering team of AI agents on your machine. Mahalaxmi orchestrates 8+ AI coding agents in parallel with Git worktree isolation, a consensus planning engine, and PTY-native terminal control.',
    alternates: {
      canonical: getCanonical(locale, '/'),
      languages: getAlternateLanguages('/'),
    },
    openGraph: {
      title: 'Mahalaxmi — Multi-Agent AI Terminal Orchestration',
      description: 'Run a whole engineering team of AI agents on your machine. Ship in hours what used to take days.',
      url: '/',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const features = [
  {
    icon: <Terminal />,
    title: 'PTY-Native Terminal Control',
    body: 'Not screen capture. Real pseudo-terminal control that works with any AI CLI tool — reliable in ways that OCR never is.',
  },
  {
    icon: <Psychology />,
    title: 'Consensus Engine',
    body: 'Four merge strategies (Union, Intersection, WeightedVoting, ComplexityWeighted) with semantic deduplication using Jaccard similarity and LLM arbitration.',
  },
  {
    icon: <Speed />,
    title: 'Intelligent Context Routing',
    body: 'Workers receive only the files relevant to their task — scored by keyword overlap, import-graph proximity, and historical co-occurrence.',
  },
  {
    icon: <Security />,
    title: 'Security Pipeline',
    body: 'Every worker diff is scanned for secrets, CVEs, SAST issues, and license violations before the PR is created.',
  },
  {
    icon: <Groups />,
    title: 'AI-Agnostic',
    body: 'Claude Code, OpenAI Foundry, AWS Bedrock, Google Gemini, Kiro, Goose, DeepSeek, Qwen Coder — mix providers in a single cycle.',
  },
  {
    icon: <CheckCircle />,
    title: 'Git Worktree Isolation',
    body: 'Every worker runs in a dedicated Git worktree. Workers cannot interfere with each other. Each output is a clean pull request.',
  },
];

const tiers = [
  {
    name: 'Trial',
    price: 'Free',
    sub: 'forever',
    highlight: false,
    cta: 'Download Free',
    ctaHref: '/products/mahalaxmi-ai-terminal-orchestration',
    features: ['2 AI providers', '4 concurrent workers', 'Basic codebase indexing', 'Session shared memory', 'Windows, macOS, Linux'],
  },
  {
    name: 'Professional',
    price: '$49',
    sub: '/ developer / month',
    highlight: true,
    cta: 'Start 30-Day Trial',
    ctaHref: '/products/mahalaxmi-ai-terminal-orchestration',
    features: ['All 8+ AI providers', 'Unlimited concurrent workers', 'Full GraphRAG knowledge graph', 'Project + Global memory', 'Post-cycle validation dashboard', 'PR review response loop', 'IDE extensions (VS Code, JetBrains, Neovim)', 'Email support'],
  },
  {
    name: 'Enterprise',
    price: 'Contact us',
    sub: '',
    highlight: false,
    cta: 'Contact Sales',
    ctaHref: '/contact',
    features: ['Everything in Professional', 'HIPAA & FedRAMP compliance profiles', 'Headless service mode (REST + SSE API)', 'Intake adapters (Jira, Slack, GitHub Issues)', 'Per-developer cost reporting', 'Dedicated Slack support + SLA'],
  },
];

const testimonials = [
  {
    quote: 'We went from 4-hour AI sessions with one agent to 20-minute parallel cycles with 8. The plan review step alone saves us from shipping garbage.',
    author: 'Senior Engineer, fintech startup',
  },
  {
    quote: 'The AI-agnostic part is what sold us. We had Claude and Bedrock contracts already. Mahalaxmi uses both in the same cycle — the right task goes to the right model.',
    author: 'CTO, Series A SaaS company',
  },
  {
    quote: 'The security pipeline running automatically on every worker PR means we stopped playing whack-a-mole with secrets in diffs.',
    author: 'DevSecOps Lead, healthcare tech',
  },
];

export default async function MahalaxmiLandingPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Box>
      {/* Hero */}
      <Box sx={{ bgcolor: 'primary.main', color: 'white', py: { xs: 8, md: 14 }, textAlign: 'center' }}>
        <Container maxWidth="md">
          <Chip label="Cross-platform · Windows · macOS · Linux" sx={{ mb: 3, bgcolor: 'rgba(255,255,255,0.15)', color: 'white' }} />
          <Typography variant="h2" component="h1" sx={{ fontWeight: 800, mb: 3, fontSize: { xs: '2rem', md: '3rem' } }}>
            Run a whole engineering team of AI agents. On your machine. Right now.
          </Typography>
          <Typography variant="h6" sx={{ mb: 5, opacity: 0.9, maxWidth: 680, mx: 'auto' }}>
            Mahalaxmi orchestrates dozens of AI coding agents working in parallel — each with its own isolated workspace, intelligently assigned tasks, and only the code context it needs. Ship in hours what used to take days.
          </Typography>
          <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
            <Button
              component={Link}
              href="/products/mahalaxmi-ai-terminal-orchestration"
              variant="contained"
              size="large"
              startIcon={<Download />}
              sx={{ bgcolor: 'white', color: 'primary.main', '&:hover': { bgcolor: 'grey.100' }, fontWeight: 700 }}
            >
              Download Free — macOS / Windows / Linux
            </Button>
            <Button
              component={Link}
              href="/whitepaper"
              variant="outlined"
              size="large"
              sx={{ borderColor: 'white', color: 'white', '&:hover': { borderColor: 'white', bgcolor: 'rgba(255,255,255,0.1)' } }}
            >
              Read the Whitepaper
            </Button>
          </Box>
          <Typography variant="body2" sx={{ mt: 3, opacity: 0.75 }}>
            Trusted by engineering teams using Claude Code, OpenAI, AWS Bedrock, Google Gemini, and more.
          </Typography>
        </Container>
      </Box>

      {/* Problem */}
      <Box sx={{ bgcolor: 'grey.50', py: { xs: 6, md: 10 } }}>
        <Container maxWidth="md" sx={{ textAlign: 'center' }}>
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 3 }}>
            You&apos;re paying for AI. You&apos;re using 5% of it.
          </Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 2, fontSize: '1.1rem' }}>
            Every AI coding assistant works the same way: one conversation. One stream of output. One task at a time.
          </Typography>
          <Typography variant="body1" color="text.secondary" sx={{ fontSize: '1.1rem' }}>
            Your codebase has 50 independent modules that could be built in parallel. Your AI is working on them sequentially. That&apos;s not a tool problem. That&apos;s an orchestration problem.
          </Typography>
        </Container>
      </Box>

      {/* Comparison table */}
      <Box sx={{ py: { xs: 6, md: 10 } }}>
        <Container maxWidth="md">
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 4, textAlign: 'center' }}>
            Mahalaxmi: Multi-Agent AI Orchestration for Real Codebases
          </Typography>
          <TableContainer component={Paper} elevation={2}>
            <Table>
              <TableHead>
                <TableRow sx={{ bgcolor: 'primary.main' }}>
                  <TableCell sx={{ color: 'white', fontWeight: 600 }}>What a single AI agent does</TableCell>
                  <TableCell sx={{ color: 'white', fontWeight: 600 }}>What Mahalaxmi does</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {[
                  ['One conversation, one task', '8+ agents, 8+ tasks simultaneously'],
                  ['Full codebase context (overwhelming)', 'Relevant files only, per task'],
                  ['One AI provider', 'Any combination of providers'],
                  ['Manual retries on failure', 'Self-verification + auto-retry with error context'],
                  ['No oversight structure', 'Plan review, budget gate, file accept/reject'],
                ].map(([left, right], i) => (
                  <TableRow key={i} sx={{ '&:nth-of-type(odd)': { bgcolor: 'grey.50' } }}>
                    <TableCell>{left}</TableCell>
                    <TableCell sx={{ color: 'success.main', fontWeight: 500 }}>{right}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </Container>
      </Box>

      {/* How it works */}
      <Box sx={{ bgcolor: 'grey.50', py: { xs: 6, md: 10 } }}>
        <Container maxWidth="md">
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 6, textAlign: 'center' }}>
            How a Mahalaxmi cycle works
          </Typography>
          <Grid container spacing={3}>
            {[
              { step: '1', title: 'Manager Phase — Build the plan', body: 'Manager AI agents analyze your codebase and requirements, then propose an execution plan. Multiple managers debate via a consensus engine that semantically deduplicates overlapping proposals.' },
              { step: '2', title: 'You review — before work begins', body: 'The execution plan surfaces as a visual list. Add tasks, remove tasks, adjust scope. Every modification is audit-logged. Only when you approve do workers start.' },
              { step: '3', title: 'Worker Phase — Execute in parallel', body: 'Worker agents claim tasks in dependency order. Each worker runs in a Git worktree — a fully isolated copy of the repo. Context is pre-filtered to only the relevant files.' },
              { step: '4', title: 'Results — Clean PRs, one per worker', body: 'Each worker produces a branch and pull request. Post-cycle validation checks acceptance criteria. You accept or reject individual file changes. The work is done.' },
            ].map(({ step, title, body }) => (
              <Grid item xs={12} sm={6} key={step}>
                <Card elevation={1} sx={{ height: '100%' }}>
                  <CardContent sx={{ p: 3 }}>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
                      <Box sx={{ width: 36, height: 36, borderRadius: '50%', bgcolor: 'primary.main', color: 'white', display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 700, flexShrink: 0 }}>
                        {step}
                      </Box>
                      <Typography variant="h6" sx={{ fontWeight: 600 }}>{title}</Typography>
                    </Box>
                    <Typography variant="body2" color="text.secondary">{body}</Typography>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        </Container>
      </Box>

      {/* Features */}
      <Box sx={{ py: { xs: 6, md: 10 } }}>
        <Container maxWidth="lg">
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 6, textAlign: 'center' }}>
            Built for developers who ship
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
          <Box sx={{ textAlign: 'center', mt: 4 }}>
            <Button component={Link} href="/features" endIcon={<ArrowForward />} variant="outlined">
              View all features
            </Button>
          </Box>
        </Container>
      </Box>

      {/* Testimonials */}
      <Box sx={{ bgcolor: 'grey.50', py: { xs: 6, md: 10 } }}>
        <Container maxWidth="lg">
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 6, textAlign: 'center' }}>
            What engineers say
          </Typography>
          <Grid container spacing={3}>
            {testimonials.map(({ quote, author }) => (
              <Grid item xs={12} md={4} key={author}>
                <Card elevation={1} sx={{ height: '100%' }}>
                  <CardContent sx={{ p: 3 }}>
                    <Typography variant="body1" sx={{ mb: 2, fontStyle: 'italic' }}>
                      &ldquo;{quote}&rdquo;
                    </Typography>
                    <Divider sx={{ mb: 1.5 }} />
                    <Typography variant="body2" color="text.secondary">— {author}</Typography>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        </Container>
      </Box>

      {/* Pricing */}
      <Box sx={{ py: { xs: 6, md: 10 } }}>
        <Container maxWidth="lg">
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 2, textAlign: 'center' }}>
            Start free. Scale as you grow.
          </Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 6, textAlign: 'center' }}>
            You pay for the orchestration software — not for AI tokens, not for compute, not for a cloud proxy.
          </Typography>
          <Grid container spacing={3} justifyContent="center">
            {tiers.map(({ name, price, sub, highlight, cta, ctaHref, features: tierFeatures }) => (
              <Grid item xs={12} sm={6} md={4} key={name}>
                <Card
                  elevation={highlight ? 6 : 1}
                  sx={{
                    height: '100%',
                    border: highlight ? '2px solid' : '1px solid',
                    borderColor: highlight ? 'primary.main' : 'divider',
                    position: 'relative',
                  }}
                >
                  {highlight && (
                    <Chip label="Most Popular" color="primary" size="small" sx={{ position: 'absolute', top: -12, left: '50%', transform: 'translateX(-50%)' }} />
                  )}
                  <CardContent sx={{ p: 3 }}>
                    <Typography variant="h5" sx={{ fontWeight: 700, mb: 0.5 }}>{name}</Typography>
                    <Typography variant="h4" sx={{ fontWeight: 800, color: highlight ? 'primary.main' : 'text.primary' }}>
                      {price}
                    </Typography>
                    {sub && <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>{sub}</Typography>}
                    <Button
                      component={Link}
                      href={ctaHref}
                      variant={highlight ? 'contained' : 'outlined'}
                      fullWidth
                      sx={{ mb: 3 }}
                    >
                      {cta}
                    </Button>
                    <Box component="ul" sx={{ pl: 2, m: 0 }}>
                      {tierFeatures.map((f) => (
                        <Box component="li" key={f} sx={{ mb: 0.75 }}>
                          <Typography variant="body2">{f}</Typography>
                        </Box>
                      ))}
                    </Box>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
          <Box sx={{ textAlign: 'center', mt: 4 }}>
            <Button component={Link} href="/pricing" endIcon={<ArrowForward />} variant="text">
              See full comparison table
            </Button>
          </Box>
        </Container>
      </Box>

      {/* Cloud callout */}
      <Box sx={{ py: { xs: 6, md: 10 } }}>
        <Container maxWidth="md">
          <Paper
            elevation={0}
            variant="outlined"
            sx={{
              p: { xs: 4, md: 6 },
              borderColor: 'primary.main',
              borderWidth: 2,
              borderRadius: 2,
              textAlign: 'center',
            }}
          >
            <Chip label="New" color="primary" size="small" sx={{ mb: 2 }} />
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 1, mb: 2 }}>
              <Cloud color="primary" sx={{ fontSize: 32 }} />
              <Typography variant="h4" sx={{ fontWeight: 800 }}>Mahalaxmi Cloud</Typography>
            </Box>
            <Typography variant="h6" color="text.secondary" sx={{ mb: 3, maxWidth: 480, mx: 'auto' }}>
              Don&apos;t want to install anything? Get a dedicated hosted orchestration server — provisioned in minutes, accessible from VS Code.
            </Typography>
            <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
              <Button
                component={Link}
                href="/cloud"
                variant="contained"
                size="large"
                startIcon={<Cloud />}
              >
                Learn about Mahalaxmi Cloud
              </Button>
              <Button
                component={Link}
                href="/cloud/pricing"
                variant="outlined"
                size="large"
                endIcon={<ArrowForward />}
              >
                Cloud pricing — from $0.06/hr
              </Button>
            </Box>
          </Paper>
        </Container>
      </Box>

      {/* Bottom CTA */}
      <Box sx={{ bgcolor: 'primary.main', color: 'white', py: { xs: 8, md: 12 }, textAlign: 'center' }}>
        <Container maxWidth="sm">
          <Typography variant="h4" sx={{ fontWeight: 700, mb: 2 }}>
            Stop running one AI agent. Start running a team.
          </Typography>
          <Typography variant="body1" sx={{ mb: 4, opacity: 0.9 }}>
            Your codebase is parallelizable. Your development shouldn&apos;t be serialized.
          </Typography>
          <Button
            component={Link}
            href="/products/mahalaxmi-ai-terminal-orchestration"
            variant="contained"
            size="large"
            startIcon={<Download />}
            sx={{ bgcolor: 'white', color: 'primary.main', '&:hover': { bgcolor: 'grey.100' }, fontWeight: 700 }}
          >
            Download Mahalaxmi — Free
          </Button>
          <Typography variant="body2" sx={{ mt: 2, opacity: 0.75 }}>
            Runs on Windows, macOS, and Linux. No account required for Trial.
          </Typography>
        </Container>
      </Box>
    </Box>
  );
}

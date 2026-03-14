'use client';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Container, Box, Typography, Grid, Card, CardContent, Button,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow,
  Paper, Chip, Accordion, AccordionSummary, AccordionDetails, Breadcrumbs,
} from '@mui/material';
import { ExpandMore, CheckCircle, Remove, NavigateNext } from '@mui/icons-material';
import Link from 'next/link';


const tiers = [
  {
    name: 'Trial',
    price: 'Free',
    sub: 'forever',
    note: 'No credit card. No account required.',
    highlight: false,
    cta: 'Download Free',
    ctaHref: '/products/mahalaxmi-ai-terminal-orchestration',
    features: [
      '2 AI providers (Claude Code + one other)',
      '4 concurrent workers',
      '2 template categories',
      'Basic codebase indexing',
      'Session-only shared memory',
      'Windows, macOS, Linux',
    ],
  },
  {
    name: 'Professional',
    price: '$49',
    sub: '/ developer / month',
    note: '$39/month billed annually',
    highlight: true,
    cta: 'Start 30-Day Trial — No Card Required',
    ctaHref: '/products/mahalaxmi-ai-terminal-orchestration',
    features: [
      'Everything in Trial, plus:',
      'All 8+ AI providers',
      'Unlimited concurrent workers',
      'All template categories',
      'Full GraphRAG knowledge graph',
      'Project + Global memory store',
      'Post-cycle validation dashboard',
      'PR review response loop',
      'Codebase Q&A and wiki generation',
      'Cost analytics (full history + export)',
      'VS Code, JetBrains, and Neovim extensions',
      'Email support',
    ],
  },
  {
    name: 'Enterprise',
    price: 'Contact us',
    sub: '',
    note: 'Negotiated based on team size and contract length.',
    highlight: false,
    cta: 'Contact Sales',
    ctaHref: '/contact',
    features: [
      'Everything in Professional, plus:',
      'Unlimited developer seats',
      'Per-developer cost reporting and chargeback',
      'HIPAA compliance profile',
      'FedRAMP compliance profile',
      'Security pipeline with enterprise policies',
      'Plan audit log',
      'Headless service mode (REST+SSE API)',
      'Intake adapters (Jira, Slack, GitHub Issues)',
      'Output adapters (Jira, Slack, GitHub, Webhooks)',
      'SSO / SAML integration (roadmap)',
      'Offline grace periods (90 days)',
      'Dedicated Slack channel support + SLA',
    ],
  },
];

const comparisonRows = [
  { feature: 'AI Providers', trial: '2', pro: '8+', enterprise: '8+' },
  { feature: 'Concurrent workers', trial: '4', pro: 'Unlimited', enterprise: 'Unlimited' },
  { feature: 'Template categories', trial: '2', pro: 'All', enterprise: 'All + custom' },
  { feature: 'Codebase indexing', trial: 'Basic', pro: 'Full', enterprise: 'Full' },
  { feature: 'GraphRAG knowledge graph', trial: false, pro: true, enterprise: true },
  { feature: 'Shared memory — Session', trial: true, pro: true, enterprise: true },
  { feature: 'Shared memory — Project/Global', trial: false, pro: true, enterprise: true },
  { feature: 'Memory team sync', trial: false, pro: false, enterprise: true },
  { feature: 'Security pipeline', trial: false, pro: true, enterprise: '+ enterprise profiles' },
  { feature: 'HIPAA/FedRAMP profiles', trial: false, pro: false, enterprise: true },
  { feature: 'Post-cycle validation', trial: 'Basic', pro: 'Full', enterprise: 'Full' },
  { feature: 'PR review loop', trial: false, pro: true, enterprise: true },
  { feature: 'Cost analytics', trial: 'Basic', pro: 'Full history', enterprise: 'Per-developer' },
  { feature: 'Plan audit log', trial: false, pro: false, enterprise: true },
  { feature: 'IDE extensions', trial: false, pro: true, enterprise: true },
  { feature: 'Headless service API', trial: false, pro: false, enterprise: true },
  { feature: 'Intake/output adapters', trial: false, pro: false, enterprise: true },
  { feature: 'Support', trial: 'Community', pro: 'Email', enterprise: 'Dedicated' },
];

const faqs = [
  {
    q: 'Do I pay for AI tokens separately?',
    a: 'Yes. Mahalaxmi orchestrates the AI tools — it does not proxy or re-sell AI capacity. You use your own API keys and subscriptions for Claude Code, OpenAI, Bedrock, Gemini, or any other provider. Provider costs depend on your own usage and provider pricing.',
  },
  {
    q: 'Can I use Mahalaxmi with free-tier AI tools?',
    a: 'Yes. If your AI provider offers a free or included tier (e.g., Claude Code with an Anthropic Max subscription), Mahalaxmi works with it. We have no visibility into your provider billing.',
  },
  {
    q: 'How does licensing work offline?',
    a: 'Professional and Enterprise licenses include offline grace periods (30 days Professional, 90 days Enterprise). The license is validated at startup against a cached status and heartbeated when connectivity is available.',
  },
  {
    q: 'What happens if I exceed my concurrent worker limit on Trial?',
    a: 'Additional tasks queue and start as running workers complete. Trial is not capped at 4 tasks total — just 4 concurrent workers.',
  },
  {
    q: 'Is there a team discount?',
    a: 'Enterprise pricing is negotiated based on team size and contract length. Contact sales for a custom quote.',
  },
  {
    q: 'Can I run Mahalaxmi in CI/CD without a GUI?',
    a: 'Headless service mode (mahalaxmi-service) is an Enterprise feature. It provides a full REST+SSE API that integrates with any CI/CD pipeline.',
  },
  {
    q: 'Is my code sent to ThriveTech?',
    a: 'No. All orchestration runs locally. Provider API calls go directly from your machine to your provider\'s endpoint. ThriveTech\'s licensing system only receives a machine fingerprint and license token for validation — no code, no prompts, no AI output.',
  },
];

function CellValue({ value }) {
  if (value === true) return <CheckCircle color="success" fontSize="small" />;
  if (value === false) return <Remove color="disabled" fontSize="small" />;
  return <Typography variant="body2">{value}</Typography>;
}

export default async function MahalaxmiPricingPage({ params }) {
  const { locale } = await params;

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 8 } }}>
      <Breadcrumbs separator={<NavigateNext fontSize="small" />} sx={{ mb: 3 }}>
        <Link href="/" style={{ textDecoration: 'none', color: 'inherit' }}>Mahalaxmi</Link>
        <Typography color="text.primary">Pricing</Typography>
      </Breadcrumbs>

      <Box sx={{ textAlign: 'center', mb: 6 }}>
        <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 2 }}>
          Pricing that scales with how you work
        </Typography>
        <Typography variant="h6" color="text.secondary" sx={{ maxWidth: 640, mx: 'auto' }}>
          Mahalaxmi runs on your machine with your AI subscriptions. You pay ThriveTech for the orchestration software — not for AI tokens, not for compute, not for a cloud proxy.
        </Typography>
      </Box>

      {/* Tier cards */}
      <Grid container spacing={3} sx={{ mb: 8 }} justifyContent="center">
        {tiers.map(({ name, price, sub, note, highlight, cta, ctaHref, features }) => (
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
              <CardContent sx={{ p: 3, display: 'flex', flexDirection: 'column', height: '100%' }}>
                <Typography variant="h5" sx={{ fontWeight: 700, mb: 0.5 }}>{name}</Typography>
                <Typography variant="h4" sx={{ fontWeight: 800, color: highlight ? 'primary.main' : 'text.primary' }}>
                  {price}
                </Typography>
                {sub && <Typography variant="body2" color="text.secondary">{sub}</Typography>}
                <Typography variant="caption" color="text.disabled" sx={{ mb: 2, display: 'block' }}>{note}</Typography>
                <Button
                  component={Link}
                  href={ctaHref}
                  variant={highlight ? 'contained' : 'outlined'}
                  fullWidth
                  sx={{ mb: 3 }}
                >
                  {cta}
                </Button>
                <Box component="ul" sx={{ pl: 2, m: 0, flexGrow: 1 }}>
                  {features.map((f) => (
                    <Box component="li" key={f} sx={{ mb: 0.75 }}>
                      <Typography variant="body2" sx={{ fontWeight: f.endsWith(':') ? 600 : 400 }}>{f}</Typography>
                    </Box>
                  ))}
                </Box>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      {/* Comparison table */}
      <Box sx={{ mb: 8 }}>
        <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 4 }}>
          Full comparison
        </Typography>
        <TableContainer component={Paper} elevation={1}>
          <Table size="small">
            <TableHead>
              <TableRow sx={{ bgcolor: 'rgba(255,255,255,0.08)' }}>
                <TableCell sx={{ fontWeight: 600, minWidth: 200 }}>Feature</TableCell>
                <TableCell align="center" sx={{ fontWeight: 600 }}>Trial</TableCell>
                <TableCell align="center" sx={{ fontWeight: 600, color: 'primary.main' }}>Professional</TableCell>
                <TableCell align="center" sx={{ fontWeight: 600 }}>Enterprise</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {comparisonRows.map(({ feature, trial, pro, enterprise }) => (
                <TableRow key={feature} sx={{ '&:nth-of-type(odd)': { bgcolor: 'rgba(255,255,255,0.05)' } }}>
                  <TableCell>{feature}</TableCell>
                  <TableCell align="center"><CellValue value={trial} /></TableCell>
                  <TableCell align="center"><CellValue value={pro} /></TableCell>
                  <TableCell align="center"><CellValue value={enterprise} /></TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Box>

      {/* FAQ */}
      <Box>
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
    </Container>
  );
}

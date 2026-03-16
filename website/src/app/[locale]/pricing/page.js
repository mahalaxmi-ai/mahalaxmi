import {
  Container, Box, Typography, Grid, Card, CardContent, Button,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow,
  Paper, Chip, Accordion, AccordionSummary, AccordionDetails, Breadcrumbs,
} from '@mui/material';
import { ExpandMore, CheckCircle, Remove, NavigateNext } from '@mui/icons-material';
import Link from 'next/link';
import { getProductOffering } from '@/lib/productApi';

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
    a: "No. All orchestration runs locally. Provider API calls go directly from your machine to your provider's endpoint. ThriveTech's licensing system only receives a machine fingerprint and license token for validation — no code, no prompts, no AI output.",
  },
];

function CellValue({ value }) {
  if (value === true) return <CheckCircle color="success" fontSize="small" />;
  if (value === false) return <Remove color="disabled" fontSize="small" />;
  return <Typography variant="body2">{value}</Typography>;
}

function renderCTA(tier) {
  const label = tier.cta_label ?? tier.cta ?? 'Get started';
  const variant = tier.highlight || tier.isRecommended ? 'contained' : 'outlined';

  if (tier.cta_action === 'download') {
    return (
      <Button component={Link} href="/download" variant={variant} fullWidth sx={{ mb: 3 }}>
        {label}
      </Button>
    );
  }
  if (tier.cta_action === 'contact') {
    return (
      <Button component={Link} href="/contact" variant={variant} fullWidth sx={{ mb: 3 }}>
        {label}
      </Button>
    );
  }
  // Default — product/checkout
  const href = tier.cta_href ?? tier.ctaHref ?? '/products/mahalaxmi-ai-terminal-orchestration';
  return (
    <Button component={Link} href={href} variant={variant} fullWidth sx={{ mb: 3 }}>
      {label}
    </Button>
  );
}

export default async function MahalaxmiPricingPage({ params }) {
  const { locale } = await params;

  let tiers = [];
  let comparisonRows = [];
  try {
    const offering = await getProductOffering();
    tiers = offering.pricing_tiers ?? [];
    comparisonRows = offering.comparison_rows ?? [];
  } catch {
    // Platform unavailable — page renders with empty tiers
  }

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
      {tiers.length > 0 && (
        <Grid container spacing={3} sx={{ mb: 8 }} justifyContent="center">
          {tiers.map((tier) => {
            const name = tier.name;
            const price = tier.price_display ?? (tier.price_monthly != null ? `$${tier.price_monthly}` : 'Contact us');
            const sub = tier.price_subtitle ?? '';
            const note = tier.price_note ?? '';
            const highlight = !!(tier.highlight || tier.isRecommended);
            const features = tier.features ?? [];
            return (
              <Grid item xs={12} sm={6} md={4} key={tier.slug ?? name}>
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
                    {renderCTA(tier)}
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
            );
          })}
        </Grid>
      )}

      {/* Comparison table — only when API provides comparison_rows */}
      {comparisonRows.length > 0 && (
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
      )}

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

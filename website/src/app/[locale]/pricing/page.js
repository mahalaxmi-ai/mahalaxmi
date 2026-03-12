import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  Container,
  Divider,
  Grid,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Typography,
} from '@mui/material';
import { ArrowForward, CheckCircle, Cloud, DesktopWindows } from '@mui/icons-material';
import Link from 'next/link';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Pricing — Mahalaxmi',
    description:
      'Mahalaxmi pricing: desktop license for local AI orchestration or cloud subscription for a dedicated server. Flat pricing, no hidden fees.',
    alternates: {
      canonical: getCanonical(locale, '/pricing'),
      languages: getAlternateLanguages('/pricing'),
    },
    openGraph: {
      title: 'Pricing — Mahalaxmi',
      description: 'Desktop license or cloud subscription. Flat pricing, cancel anytime.',
      url: '/pricing',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const desktopFeatures = [
  'Unlimited local AI workers',
  'All supported AI providers',
  'VS Code extension included',
  'Lifetime updates for major version',
  'Runs on your own hardware',
  'Community support',
];

const cloudFeatures = [
  'Everything in Desktop',
  'Dedicated cloud VM',
  '24/7 uptime — no laptop required',
  'One-click VS Code connection',
  'Automatic provisioning',
  'Email support',
];

export default async function PricingPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 8 } }}>
      <Box sx={{ textAlign: 'center', mb: 7 }}>
        <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 2 }}>
          Simple, transparent pricing
        </Typography>
        <Typography variant="h6" color="text.secondary" sx={{ maxWidth: 560, mx: 'auto' }}>
          Choose the plan that fits your workflow. No hidden fees. AI provider costs are always billed directly by your provider — not us.
        </Typography>
      </Box>

      <Grid container spacing={4} justifyContent="center" sx={{ mb: 8 }}>
        {/* Desktop */}
        <Grid item xs={12} md={5}>
          <Card
            elevation={1}
            sx={{ height: '100%', borderRadius: 3, border: '1px solid', borderColor: 'divider' }}
          >
            <CardContent sx={{ p: 4 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 2 }}>
                <DesktopWindows color="primary" sx={{ fontSize: 32 }} />
                <Typography variant="h5" sx={{ fontWeight: 700 }}>Desktop</Typography>
              </Box>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                Install Mahalaxmi on your own machine and run AI workers locally.
              </Typography>
              <Typography variant="h4" sx={{ fontWeight: 800, mb: 0.5 }}>
                From $29
                <Typography component="span" variant="body2" color="text.secondary"> /month</Typography>
              </Typography>
              <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 3 }}>
                Annual and lifetime options available
              </Typography>
              <List dense sx={{ mb: 3 }}>
                {desktopFeatures.map((f) => (
                  <ListItem key={f} disableGutters>
                    <ListItemIcon sx={{ minWidth: 32 }}>
                      <CheckCircle color="success" sx={{ fontSize: 18 }} />
                    </ListItemIcon>
                    <ListItemText primary={f} primaryTypographyProps={{ variant: 'body2' }} />
                  </ListItem>
                ))}
              </List>
              <Button
                component={Link}
                href="/login?redirect=/pricing"
                variant="outlined"
                color="primary"
                fullWidth
                size="large"
                endIcon={<ArrowForward />}
              >
                Get Desktop License
              </Button>
            </CardContent>
          </Card>
        </Grid>

        {/* Cloud */}
        <Grid item xs={12} md={5}>
          <Card
            elevation={6}
            sx={{
              height: '100%',
              borderRadius: 3,
              border: '2px solid',
              borderColor: 'primary.main',
              position: 'relative',
            }}
          >
            <Chip
              label="Most Popular"
              color="primary"
              size="small"
              sx={{ position: 'absolute', top: 16, right: 16, fontWeight: 700 }}
            />
            <CardContent sx={{ p: 4 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 2 }}>
                <Cloud color="primary" sx={{ fontSize: 32 }} />
                <Typography variant="h5" sx={{ fontWeight: 700 }}>Cloud</Typography>
              </Box>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                Your dedicated orchestration server — always on, always connected.
              </Typography>
              <Typography variant="h4" sx={{ fontWeight: 800, mb: 0.5 }}>
                From $49
                <Typography component="span" variant="body2" color="text.secondary"> /month</Typography>
              </Typography>
              <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 3 }}>
                Monthly or annual billing
              </Typography>
              <List dense sx={{ mb: 3 }}>
                {cloudFeatures.map((f) => (
                  <ListItem key={f} disableGutters>
                    <ListItemIcon sx={{ minWidth: 32 }}>
                      <CheckCircle color="success" sx={{ fontSize: 18 }} />
                    </ListItemIcon>
                    <ListItemText primary={f} primaryTypographyProps={{ variant: 'body2' }} />
                  </ListItem>
                ))}
              </List>
              <Button
                component={Link}
                href="/cloud/pricing"
                variant="contained"
                color="primary"
                fullWidth
                size="large"
                endIcon={<ArrowForward />}
              >
                See Cloud Plans
              </Button>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      <Divider sx={{ mb: 6 }} />

      {/* Enterprise */}
      <Box sx={{ textAlign: 'center', mb: 4 }}>
        <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>
          Need something larger?
        </Typography>
        <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
          For teams needing multiple cloud VMs, custom resource limits, or centralized billing — contact our sales team.
        </Typography>
        <Button
          component="a"
          href="mailto:sales@mahalaxmi.ai"
          variant="outlined"
          size="large"
          endIcon={<ArrowForward />}
        >
          Contact sales@mahalaxmi.ai
        </Button>
      </Box>
    </Container>
  );
}

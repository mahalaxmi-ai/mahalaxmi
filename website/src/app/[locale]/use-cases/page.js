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
import { ArrowForward } from '@mui/icons-material';
import Link from 'next/link';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Use Cases — Mahalaxmi',
    description:
      'How teams use Mahalaxmi: large refactors, parallel feature development, test generation, documentation, and more.',
    alternates: {
      canonical: getCanonical(locale, '/use-cases'),
      languages: getAlternateLanguages('/use-cases'),
    },
    openGraph: {
      title: 'Use Cases — Mahalaxmi',
      description: 'Large refactors, parallel features, test generation, and more.',
      url: '/use-cases',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const useCases = [
  {
    title: 'Large-scale refactoring',
    body: 'Rename a module, migrate to a new API, or enforce a new coding standard across thousands of files. Assign one worker per directory and merge results when done.',
    example: 'Migrated a 120k-line codebase from JavaScript to TypeScript in one session.',
  },
  {
    title: 'Parallel feature development',
    body: 'Develop several independent features simultaneously. Each worker operates in its own branch scope and proposes changes for your review before anything is committed.',
    example: 'Shipped 4 independent features across a weekend with a single reviewer in the loop.',
  },
  {
    title: 'Automated test generation',
    body: 'Point workers at untested modules and let them write unit, integration, and edge-case tests. Review the output in VS Code before committing.',
    example: 'Raised test coverage from 41% to 87% across a backend service in one day.',
  },
  {
    title: 'Documentation generation',
    body: 'Workers read your source code and generate API references, README files, and architecture diagrams in parallel. No more documentation debt.',
    example: 'Generated full API docs for a 30-endpoint REST service from source comments.',
  },
  {
    title: 'Dependency upgrades',
    body: 'Upgrade a major dependency across your entire project — workers handle the breaking changes in each affected file while you track progress in real time.',
    example: 'Migrated from React 17 to React 18 including concurrent-mode fixes across 200 components.',
  },
  {
    title: 'Bug triage at scale',
    body: 'Feed a backlog of bug reports to Mahalaxmi workers. Each worker investigates one issue, proposes a fix, and generates a test to prevent regression.',
    example: 'Closed 23 GitHub issues in a single afternoon with all fixes code-reviewed by one engineer.',
  },
];

export default async function UseCasesPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 8 } }}>
      <Box sx={{ textAlign: 'center', mb: 8 }}>
        <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 2 }}>
          What teams use Mahalaxmi for
        </Typography>
        <Typography variant="h6" color="text.secondary" sx={{ maxWidth: 600, mx: 'auto' }}>
          From solo developers tackling technical debt to engineering teams shipping features in parallel.
        </Typography>
      </Box>

      <Grid container spacing={4} sx={{ mb: 10 }}>
        {useCases.map((uc) => (
          <Grid item xs={12} md={6} key={uc.title}>
            <Card
              elevation={0}
              variant="outlined"
              sx={{ height: '100%', borderRadius: 3 }}
            >
              <CardContent sx={{ p: 3 }}>
                <Typography variant="h6" sx={{ fontWeight: 700, mb: 1 }}>
                  {uc.title}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                  {uc.body}
                </Typography>
                <Box
                  sx={{
                    bgcolor: 'primary.50',
                    borderLeft: '3px solid',
                    borderColor: 'primary.main',
                    borderRadius: '0 6px 6px 0',
                    px: 2,
                    py: 1,
                  }}
                >
                  <Typography variant="caption" color="primary.dark" sx={{ fontStyle: 'italic' }}>
                    {uc.example}
                  </Typography>
                </Box>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      <Box sx={{ textAlign: 'center', borderTop: '1px solid', borderColor: 'divider', pt: 6 }}>
        <Typography variant="h5" sx={{ fontWeight: 700, mb: 2 }}>
          Start your first orchestration session
        </Typography>
        <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
          <Button
            component={Link}
            href="/pricing"
            variant="contained"
            size="large"
            endIcon={<ArrowForward />}
          >
            View Pricing
          </Button>
          <Button
            component={Link}
            href="/docs/quickstart"
            variant="outlined"
            size="large"
          >
            Quickstart Guide
          </Button>
        </Box>
      </Box>
    </Container>
  );
}

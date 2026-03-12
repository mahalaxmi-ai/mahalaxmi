import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Container, Box, Typography, Grid, Card, CardContent, Chip, Breadcrumbs,
} from '@mui/material';
import { NavigateNext } from '@mui/icons-material';
import Link from 'next/link';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Mahalaxmi Use Cases — Real-World AI Development Workflows',
    description: 'How engineering teams use Mahalaxmi: feature development, test coverage campaigns, technical debt sprints, security audit remediation, CI/CD automation, and architecture migrations.',
    alternates: {
      canonical: getCanonical(locale, '/use-cases'),
      languages: getAlternateLanguages('/use-cases'),
    },
    openGraph: {
      title: 'Mahalaxmi Use Cases — Real-World AI Development Workflows',
      description: 'How teams go from 4-hour AI sessions to 20-minute parallel cycles.',
      url: '/use-cases',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const useCases = [
  {
    number: '01',
    title: 'Full Feature Development in a Single Cycle',
    who: 'Individual developers with Claude Code or similar subscriptions',
    timeSaved: '4–6 hours → 30–60 minutes',
    tier: 'Trial',
    scenario: 'You need to add user authentication to a Node.js API. Normally this means: schema changes, migration, middleware, route protection, token refresh, tests, docs — done sequentially over a full afternoon.',
    withMahalaxmi: 'Open a requirements template for "API Authentication", fill in your stack (Express, PostgreSQL, JWT), and start a cycle. Three managers analyze your existing codebase and produce a 9-task plan. You review the plan (90 seconds), approve it, and eight workers start simultaneously. An hour later, nine clean PRs are ready. Post-cycle validation confirms the auth tests pass. You merge.',
  },
  {
    number: '02',
    title: 'Technical Debt Sprint',
    who: 'Engineering teams with accumulated legacy code',
    timeSaved: 'Multi-week sprint → 2–3 days',
    tier: 'Professional',
    scenario: 'Your team has 200 endpoints with inconsistent error handling, mixed response formats, and no OpenAPI specs. A real modernization sprint would take two engineers three weeks.',
    withMahalaxmi: 'Write a requirements doc describing the target error handling standard and response format. Let managers analyze the endpoint inventory and produce a task per endpoint group. Run parallel cycles over several hours. Each cycle handles a batch of endpoints. The GraphRAG knowledge graph ensures workers understand which utilities are shared across endpoints so they update the right abstractions.',
  },
  {
    number: '03',
    title: 'Test Coverage Campaigns',
    who: 'Teams with coverage gaps before a release',
    timeSaved: '1–2 weeks → 4–8 hours',
    tier: 'Professional',
    scenario: "You're 3 weeks from a major release. Coverage is 45%. You need 80%. Writing 200 unit tests manually is soul-crushing work that nobody wants to own.",
    withMahalaxmi: 'Point Mahalaxmi at your codebase with a "test coverage campaign" template. Manager agents analyze which modules have low coverage and identify the most impactful test targets. Workers are assigned modules and write tests using your project\'s existing test patterns. The build gate verifies each worker\'s tests pass before the PR is created. Coverage goes from 45% to 82% while your engineers work on feature code.',
  },
  {
    number: '04',
    title: 'Multi-Provider Enterprise Teams',
    who: 'Enterprise engineering organizations with multiple AI provider contracts',
    timeSaved: 'Idle AI capacity → productive work',
    tier: 'Enterprise',
    scenario: 'Your enterprise has Claude Enterprise, AWS Bedrock with Titan, and Google Vertex AI contracts. Developers use whichever tool they personally prefer, leaving significant purchased capacity idle.',
    withMahalaxmi: 'Configure the team with three manager providers and route workers to providers based on task type. Documentation and test tasks go to the faster, cheaper models. Complex architectural work goes to the most capable model. Managers can run simultaneously — one per provider — and their proposals feed the consensus engine. Per-developer cost reports feed into your existing AI spend dashboards.',
  },
  {
    number: '05',
    title: 'Intake-Driven Automation',
    who: 'Teams using Jira, GitHub Issues, or Slack for work tracking',
    timeSaved: 'Manual triage → fully automated',
    tier: 'Enterprise',
    scenario: 'Your backlog has 30 "good first issue" bugs — each isolated, well-defined, and under 2 hours of work. Nobody is picking them up because they\'re boring.',
    withMahalaxmi: 'Configure a GitHub Issues adapter targeting your repository. Mahalaxmi polls every 5 minutes. When an issue with the automated label appears, it\'s ingested as a work item, you review and confirm it in the intake panel, and a cycle starts automatically. Workers fix the bug, run tests, create a PR, and post the PR link as a GitHub comment closing the issue.',
  },
  {
    number: '06',
    title: 'Post-PR-Review Fix Loops',
    who: 'Any team with code review processes',
    timeSaved: 'Manual re-work → automated',
    tier: 'Professional',
    scenario: 'A developer submits a PR. A reviewer comments with specific change requests. Normally the developer reads the comment, re-opens their IDE, makes the changes, re-pushes. This takes 15–30 minutes per round of review.',
    withMahalaxmi: 'The PR review response loop monitor detects the review comments. A fix worker is dispatched with the original task context + the review comments as additional instructions. The worker makes the targeted changes and pushes. The reviewer gets notified of the update in minutes, not hours.',
  },
  {
    number: '07',
    title: 'Security Audit Remediation',
    who: 'Teams facing compliance reviews or security audit findings',
    timeSaved: 'Manual remediation tracking → automated execution',
    tier: 'Enterprise',
    scenario: 'A security audit produces a report: 12 hardcoded secrets found, 8 dependencies with known CVEs, 3 endpoints missing input validation.',
    withMahalaxmi: 'Create a requirements document from the audit findings. Each finding becomes a worker task. Managers group the findings by type and scope. Workers remediate each group in parallel. The security pipeline re-scans each worker\'s diff before the PR is created, providing a clean audit trail showing that each finding was addressed.',
  },
  {
    number: '08',
    title: 'New Developer Onboarding Projects',
    who: 'New hires who need a meaningful project to learn the codebase',
    timeSaved: 'Weeks of context-building → productive contribution in days',
    tier: 'Trial',
    scenario: 'A new backend engineer joins. Their first project is "add rate limiting to the API." They spend two weeks learning the codebase before they can make a meaningful PR.',
    withMahalaxmi: 'The new engineer uses Mahalaxmi to bootstrap the rate limiting feature — watching managers analyze the codebase, reading the generated execution plan (an excellent codebase tour), reviewing the workers\' PRs to understand patterns. They contribute meaningfully in week one and learn the codebase through the orchestration output rather than from scratch.',
  },
  {
    number: '09',
    title: 'Architecture Migration',
    who: 'Teams migrating frameworks, languages, or architectural patterns',
    timeSaved: '6-month migration → 3–4 weeks',
    tier: 'Enterprise',
    scenario: 'Migrate a Django REST Framework API to FastAPI. 40 endpoints, 15 database models, 200 test cases. A full team migration would take months.',
    withMahalaxmi: 'Structure the migration as a chain of cycles. Cycle 1 migrates the data layer. Cycle 2 migrates the service layer. Cycle 3 migrates the endpoint handlers. Cycle 4 updates tests. Cycle 5 updates documentation. Auto-chain mode runs each cycle automatically when the previous completes.',
  },
  {
    number: '10',
    title: 'Documentation Campaigns',
    who: 'Engineering teams with perpetually out-of-date documentation',
    timeSaved: 'Never done → always current',
    tier: 'Professional',
    scenario: 'Your README is from 2023. Your API docs don\'t mention the last 6 endpoints. Your architectural overview doesn\'t reflect the microservices migration. No one has time to update them.',
    withMahalaxmi: 'A "documentation refresh" template points Mahalaxmi at the gap between codebase state and documentation state. Manager agents compare the codebase index against existing docs and identify outdated sections. Workers update each section in parallel — pulling real code examples from the codebase index rather than writing generic placeholders.',
  },
];

const tierColors = {
  Trial: 'success',
  Professional: 'primary',
  Enterprise: 'secondary',
};

export default async function MahalaxmiUseCasesPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 8 } }}>
      <Breadcrumbs separator={<NavigateNext fontSize="small" />} sx={{ mb: 3 }}>
        <Link href="/" style={{ textDecoration: 'none', color: 'inherit' }}>Mahalaxmi</Link>
        <Typography color="text.primary">Use Cases</Typography>
      </Breadcrumbs>

      <Box sx={{ mb: 6 }}>
        <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 2 }}>
          Use Cases
        </Typography>
        <Typography variant="h6" color="text.secondary">
          How engineering teams use Mahalaxmi to parallelize real development work.
        </Typography>
      </Box>

      <Grid container spacing={4}>
        {useCases.map(({ number, title, who, timeSaved, tier, scenario, withMahalaxmi }) => (
          <Grid item xs={12} key={number}>
            <Card elevation={1}>
              <CardContent sx={{ p: { xs: 3, md: 4 } }}>
                <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 2, mb: 2, flexWrap: 'wrap' }}>
                  <Typography variant="h5" color="primary" sx={{ fontWeight: 800, minWidth: 40 }}>{number}</Typography>
                  <Box sx={{ flex: 1 }}>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.5, flexWrap: 'wrap' }}>
                      <Typography variant="h5" sx={{ fontWeight: 700 }}>{title}</Typography>
                      <Chip label={tier} color={tierColors[tier] || 'default'} size="small" />
                    </Box>
                    <Typography variant="body2" color="text.secondary">
                      <strong>Who:</strong> {who}
                    </Typography>
                    <Typography variant="body2" color="success.main" sx={{ fontWeight: 600 }}>
                      Time saved: {timeSaved}
                    </Typography>
                  </Box>
                </Box>

                <Grid container spacing={3}>
                  <Grid item xs={12} md={6}>
                    <Box sx={{ bgcolor: 'grey.50', p: 2.5, borderRadius: 1, height: '100%' }}>
                      <Typography variant="subtitle2" sx={{ fontWeight: 700, mb: 1, color: 'text.secondary', textTransform: 'uppercase', fontSize: '0.75rem', letterSpacing: 1 }}>
                        The Scenario
                      </Typography>
                      <Typography variant="body2" color="text.secondary">{scenario}</Typography>
                    </Box>
                  </Grid>
                  <Grid item xs={12} md={6}>
                    <Box sx={{ bgcolor: 'primary.50', p: 2.5, borderRadius: 1, height: '100%', borderLeft: '3px solid', borderColor: 'primary.main' }}>
                      <Typography variant="subtitle2" sx={{ fontWeight: 700, mb: 1, color: 'primary.main', textTransform: 'uppercase', fontSize: '0.75rem', letterSpacing: 1 }}>
                        With Mahalaxmi
                      </Typography>
                      <Typography variant="body2">{withMahalaxmi}</Typography>
                    </Box>
                  </Grid>
                </Grid>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>
    </Container>
  );
}

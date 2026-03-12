import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';

export const metadata = {
  title: 'Mahalaxmi Roadmap',
  description: 'The Mahalaxmi open source roadmap — current phase, planned providers including AWS ECS and GCP Cloud Run, cycle metering, and Phase 28 advanced features.',
  alternates: {
    canonical: '/open-source/roadmap',
  },
};

const headingSx = {
  color: '#00C8C8',
  fontWeight: 700,
  mt: 5,
  mb: 1.5,
};

const statusColor = (status) => {
  if (status === 'completed') return '#22C55E';
  if (status === 'in-progress') return '#00C8C8';
  return '#6B7280';
};

const statusLabel = (status) => {
  if (status === 'completed') return 'Completed';
  if (status === 'in-progress') return 'In Progress';
  return 'Planned';
};

const phases = [
  {
    phase: 1,
    title: 'Foundation',
    status: 'completed',
    items: [
      'Manager-Worker consensus engine',
      'DAG task graph with parallel execution',
      'Git worktree isolation per Worker',
      'PTY control layer for Claude Code',
      'Branch-and-PR git strategy',
    ],
  },
  {
    phase: 2,
    title: 'Provider Expansion',
    status: 'completed',
    items: [
      'GitHub Copilot PTY adapter',
      'Grok (xAI) PTY adapter',
      'Ollama local provider support',
      'Google Gemini CLI adapter',
      'ProviderRouter with fallback chain',
    ],
  },
  {
    phase: 3,
    title: 'Cloud Infrastructure',
    status: 'in-progress',
    items: [
      'Managed cloud server provisioning',
      'VS Code Extension with deep-link launch',
      'httpOnly cookie auth with server-side PAK handling',
      'Stripe billing integration',
      'User dashboard with server lifecycle management',
    ],
  },
  {
    phase: 4,
    title: 'Container Provider Targets',
    status: 'planned',
    items: [
      'AWS ECS provider — run Mahalaxmi Workers as Fargate tasks',
      'GCP Cloud Run provider — serverless Worker execution on Google Cloud',
      'Docker Compose provider for local multi-container setups',
      'Kubernetes operator for self-hosted cluster deployments',
    ],
  },
  {
    phase: 5,
    title: 'Metering and Observability',
    status: 'planned',
    items: [
      'Cycle metering system — track compute cycles per Worker and session',
      'Per-project usage analytics dashboard',
      'Cost attribution by provider and task type',
      'Structured log aggregation and exporters',
      'OpenTelemetry trace integration',
    ],
  },
  {
    phase: 28,
    title: 'Advanced Features',
    status: 'planned',
    items: [
      'Cross-repository orchestration — Workers spanning multiple git remotes',
      'Adaptive consensus — dynamic strategy selection based on task confidence scores',
      'Provider performance profiling with automatic routing optimization',
      'Human-in-the-loop escalation for low-confidence task outcomes',
      'Multi-tenant workspace isolation for enterprise deployments',
    ],
  },
];

export default function RoadmapPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Mahalaxmi Roadmap
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        The Mahalaxmi roadmap tracks planned features, provider targets, and platform milestones.
        Completed phases are shipped in the open source CLI. Planned phases represent the team&apos;s
        current direction and are subject to change based on community feedback.
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 4, fontStyle: 'italic' }}>
        Have a feature request? Open an issue on GitHub or reach out at{' '}
        <Box
          component="a"
          href="mailto:support@mahalaxmi.ai"
          sx={{ color: '#00C8C8', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
        >
          support@mahalaxmi.ai
        </Box>
        .
      </Typography>

      <Divider sx={{ mb: 4 }} />

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2.5 }}>
        {phases.map((phase) => (
          <Card
            key={phase.phase}
            variant="outlined"
            sx={{
              backgroundColor: '#0D1117',
              borderColor: phase.status === 'in-progress' ? '#00C8C8' : 'divider',
              transition: 'border-color 0.2s',
            }}
          >
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 1.5, flexWrap: 'wrap' }}>
                <Typography
                  variant="caption"
                  sx={{
                    fontFamily: 'monospace',
                    fontWeight: 700,
                    color: '#6B7280',
                    letterSpacing: 1,
                  }}
                >
                  PHASE {phase.phase}
                </Typography>
                <Typography variant="h6" component="h2" fontWeight={700}>
                  {phase.title}
                </Typography>
                <Typography
                  variant="caption"
                  sx={{
                    color: statusColor(phase.status),
                    border: `1px solid ${statusColor(phase.status)}`,
                    borderRadius: '4px',
                    px: 1,
                    py: 0.25,
                    fontWeight: 600,
                    letterSpacing: 0.5,
                    ml: 'auto',
                  }}
                >
                  {statusLabel(phase.status)}
                </Typography>
              </Box>
              <Box component="ul" sx={{ pl: 3, m: 0 }}>
                {phase.items.map((item) => (
                  <Typography
                    key={item}
                    component="li"
                    variant="body2"
                    color="text.secondary"
                    sx={{ mb: 0.5 }}
                  >
                    {item}
                  </Typography>
                ))}
              </Box>
            </CardContent>
          </Card>
        ))}
      </Box>

      <Divider sx={{ my: 5 }} />

      <Typography variant="h5" component="h2" sx={headingSx}>
        Upcoming Highlights
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 2 }}>
        {[
          'AWS ECS provider — Fargate-based Worker execution in your own AWS account.',
          'GCP Cloud Run provider — serverless Worker dispatch on Google Cloud.',
          'Cycle metering system — granular compute tracking per session and project.',
        ].map((item) => (
          <Typography key={item} component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
            {item}
          </Typography>
        ))}
      </Box>

      <Box
        component="a"
        href="https://github.com/thrivetech2t/mahalaxmi"
        target="_blank"
        rel="noopener noreferrer"
        sx={{
          display: 'inline-block',
          color: '#00C8C8',
          border: '1px solid #00C8C8',
          borderRadius: 1,
          px: 2,
          py: 1,
          textDecoration: 'none',
          fontWeight: 600,
          mt: 2,
          '&:hover': { backgroundColor: 'rgba(0,200,200,0.08)' },
        }}
      >
        View Repository on GitHub →
      </Box>
    </Container>
  );
}

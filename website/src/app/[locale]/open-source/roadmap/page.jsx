'use client';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Link from 'next/link';


const TEAL = '#00C8C8';

const phases = [
  {
    status: 'completed',
    label: 'Completed',
    items: [
      { name: 'Initial release', description: 'Core orchestration engine, CLI, and worker task execution.' },
      { name: 'VS Code Extension', description: 'Launch and manage Mahalaxmi workers directly from VS Code.' },
      { name: 'Cloud provisioning', description: 'One-click cloud server provisioning from the dashboard.' },
      { name: 'Multi-provider routing', description: 'Route tasks to multiple AI providers (Claude, OpenAI, and more).' },
    ],
  },
  {
    status: 'in-progress',
    label: 'In Progress',
    items: [
      { name: 'Cycle metering', description: 'Per-cycle usage tracking and metered billing for orchestration runs.' },
    ],
  },
  {
    status: 'planned',
    label: 'Planned',
    items: [
      { name: 'AWS ECS provider', description: 'Native support for running workers on Amazon Elastic Container Service.' },
      { name: 'GCP Cloud Run provider', description: 'Deploy and scale workers on Google Cloud Run.' },
      { name: 'Multi-seat management', description: 'Team accounts with role-based access and seat-level controls.' },
      { name: 'Phase 28 advanced orchestration', description: 'Next-generation task graph scheduling and inter-worker dependencies.' },
    ],
  },
];

function StatusDot({ status }) {
  const color =
    status === 'completed'
      ? TEAL
      : status === 'in-progress'
      ? '#F5A623'
      : '#555E6B';

  return (
    <Box
      sx={{
        width: 14,
        height: 14,
        borderRadius: '50%',
        backgroundColor: color,
        flexShrink: 0,
        mt: '4px',
        border: status === 'in-progress' ? `2px solid ${color}` : 'none',
        boxShadow: status === 'in-progress' ? `0 0 6px ${color}` : 'none',
      }}
    />
  );
}

function CheckIcon() {
  return (
    <Box
      component="svg"
      viewBox="0 0 16 16"
      sx={{ width: 16, height: 16, flexShrink: 0, mt: '2px' }}
      aria-hidden="true"
    >
      <circle cx="8" cy="8" r="8" fill={TEAL} />
      <path
        d="M4.5 8.5l2.5 2.5 5-5"
        stroke="#0A0F14"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
    </Box>
  );
}

function PhaseSection({ phase }) {
  const isCompleted = phase.status === 'completed';
  const isInProgress = phase.status === 'in-progress';

  const labelColor = isCompleted ? TEAL : isInProgress ? '#F5A623' : '#555E6B';

  return (
    <Box sx={{ mb: 5 }}>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 2 }}>
        <Box
          sx={{
            width: 10,
            height: 10,
            borderRadius: '50%',
            backgroundColor: labelColor,
            flexShrink: 0,
          }}
        />
        <Typography
          variant="overline"
          sx={{ color: labelColor, fontWeight: 700, letterSpacing: '0.1em' }}
        >
          {phase.label}
        </Typography>
      </Box>

      <Box
        sx={{
          borderLeft: `2px solid`,
          borderColor: isCompleted ? TEAL : isInProgress ? '#F5A623' : '#2A3040',
          pl: 3,
          display: 'flex',
          flexDirection: 'column',
          gap: 2.5,
        }}
      >
        {phase.items.map((item) => (
          <Box key={item.name} sx={{ display: 'flex', gap: 1.5, alignItems: 'flex-start' }}>
            {isCompleted ? (
              <CheckIcon />
            ) : (
              <StatusDot status={phase.status} />
            )}
            <Box>
              <Typography
                variant="body1"
                fontWeight={600}
                sx={{ color: isCompleted ? 'text.primary' : isInProgress ? '#F5A623' : 'text.disabled' }}
              >
                {item.name}
              </Typography>
              <Typography
                variant="body2"
                sx={{ color: isCompleted ? 'text.secondary' : isInProgress ? 'text.secondary' : '#3A4455', mt: 0.25 }}
              >
                {item.description}
              </Typography>
            </Box>
          </Box>
        ))}
      </Box>
    </Box>
  );
}

export default function RoadmapPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Roadmap
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
        Our development roadmap — what we have shipped, what is actively being built, and what is next.
      </Typography>

      <Box
        sx={{
          display: 'flex',
          alignItems: 'flex-start',
          gap: 1.5,
          p: 2,
          mb: 4,
          borderRadius: 2,
          border: '1px solid',
          borderColor: 'divider',
          backgroundColor: 'rgba(0,200,200,0.04)',
        }}
      >
        <Box
          component="svg"
          viewBox="0 0 20 20"
          sx={{ width: 18, height: 18, flexShrink: 0, mt: '2px', color: TEAL }}
          fill="currentColor"
          aria-hidden="true"
        >
          <path
            fillRule="evenodd"
            d="M18 10A8 8 0 1 1 2 10a8 8 0 0 1 16 0zm-7-4a1 1 0 1 1-2 0 1 1 0 0 1 2 0zM9 9a1 1 0 0 0 0 2v3a1 1 0 0 0 2 0v-3a1 1 0 0 0-1-1H9z"
            clipRule="evenodd"
          />
        </Box>
        <Typography variant="body2" color="text.secondary">
          Roadmap subject to change.{' '}
          <Box
            component={Link}
            href="https://github.com/thrivetech2t/mahalaxmi"
            target="_blank"
            rel="noopener noreferrer"
            sx={{ color: TEAL, textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
          >
            Follow us on GitHub
          </Box>{' '}
          for the latest updates.
        </Typography>
      </Box>

      <Divider sx={{ mb: 5 }} />

      {phases.map((phase) => (
        <PhaseSection key={phase.status} phase={phase} />
      ))}

      <Divider sx={{ mb: 4 }} />

      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flexWrap: 'wrap' }}>
        <Typography variant="body2" color="text.secondary">
          Have a feature request or want to contribute?
        </Typography>
        <Box
          component={Link}
          href="https://github.com/thrivetech2t/mahalaxmi/issues"
          target="_blank"
          rel="noopener noreferrer"
          sx={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 0.75,
            px: 2,
            py: 0.75,
            borderRadius: 1,
            border: `1px solid ${TEAL}`,
            color: TEAL,
            textDecoration: 'none',
            fontSize: '0.875rem',
            fontWeight: 600,
            '&:hover': { backgroundColor: 'rgba(0,200,200,0.08)' },
          }}
        >
          Open an Issue on GitHub
        </Box>
      </Box>
    </Container>
  );
}

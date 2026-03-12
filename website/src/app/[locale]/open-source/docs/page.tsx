import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Link from 'next/link';

export const metadata = {
  title: 'Getting Started — Mahalaxmi Open Source',
  description: 'Quick-start guide for Mahalaxmi open source: install the CLI, initialise a project, and launch your first multi-agent orchestration session.',
  alternates: {
    canonical: '/open-source/docs',
  },
};

const codeBlockSx = {
  fontFamily: 'monospace',
  backgroundColor: '#0D1117',
  color: '#00C8C8',
  p: 3,
  borderRadius: 1,
  my: 2,
  overflowX: 'auto',
  whiteSpace: 'pre' as const,
  fontSize: '0.9rem',
  lineHeight: 1.8,
};

const headingSx = {
  color: '#00C8C8',
  fontWeight: 700,
  mt: 5,
  mb: 1.5,
};

const commands = [
  'npm install -g @thrivetech/mahalaxmi',
  'mahalaxmi init',
  'mahalaxmi start',
];

export default function OpenSourceDocsPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Getting Started with Mahalaxmi Open Source
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        Get Mahalaxmi running locally in under five minutes. All you need is Node.js and an AI
        provider.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      {/* Quick Start */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Quick Start
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 1 }}>
        Run the following three commands to install, initialise, and launch Mahalaxmi:
      </Typography>
      <Box sx={codeBlockSx}>
        {commands.join('\n')}
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* Prerequisites */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Prerequisites
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 3 }}>
        <Typography component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
          <Box component="span" sx={{ fontWeight: 600, color: 'text.primary' }}>Node.js 18+</Box>
          {' '}— the Mahalaxmi CLI requires Node.js 18 or higher. Verify with{' '}
          <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>
            node --version
          </Box>.
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
          <Box component="span" sx={{ fontWeight: 600, color: 'text.primary' }}>AI Provider</Box>
          {' '}— at least one of the following:
          <Box component="ul" sx={{ pl: 3, mt: 0.5 }}>
            {[
              { name: 'Claude API key', detail: 'Anthropic Claude Code (recommended)' },
              { name: 'Ollama', detail: 'run open-weight models locally — no API key required' },
              { name: 'GitHub Copilot', detail: 'requires an active GitHub Copilot subscription' },
            ].map(({ name, detail }) => (
              <Typography key={name} component="li" variant="body2" color="text.secondary" sx={{ mb: 0.5 }}>
                <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>{name}</Box>
                {' — '}{detail}
              </Typography>
            ))}
          </Box>
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary">
          <Box component="span" sx={{ fontWeight: 600, color: 'text.primary' }}>npm</Box>
          {' '}— included with Node.js; used to install the global CLI package.
        </Typography>
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* What happens when you run `mahalaxmi start` */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        What Happens When You Run{' '}
        <Box component="span" sx={{ fontFamily: 'monospace', fontSize: '1rem' }}>
          mahalaxmi start
        </Box>
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Running{' '}
        <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>
          mahalaxmi start
        </Box>{' '}
        boots the Manager-Worker consensus engine. Here is the sequence:
      </Typography>
      <Box component="ol" sx={{ pl: 3, mb: 3 }}>
        {[
          {
            title: 'Manager initialises',
            detail:
              'The Manager reads your project configuration, connects to the configured AI provider via PTY, and loads the task graph.',
          },
          {
            title: 'Tasks are distributed',
            detail:
              'The Manager analyses pending work and assigns discrete tasks to available Worker agents. Tasks with no unresolved dependencies are dispatched immediately.',
          },
          {
            title: 'Workers execute in parallel',
            detail:
              'Each Worker runs in an isolated git worktree branch, executing its assigned task using the AI provider. Workers cannot interfere with each other at the filesystem level.',
          },
          {
            title: 'Consensus validates output',
            detail:
              'When Workers report completion, the consensus engine reconciles any overlapping outputs using the configured merge strategy (Union, Intersection, or WeightedVoting).',
          },
          {
            title: 'Results are merged',
            detail:
              'Validated outputs are committed, pull requests are opened from Worker branches, and the Manager proceeds to the next wave of tasks in the DAG.',
          },
        ].map(({ title, detail }, index) => (
          <Typography key={index} component="li" variant="body1" color="text.secondary" sx={{ mb: 1.5 }}>
            <Box component="span" sx={{ fontWeight: 600, color: 'text.primary' }}>{title}:</Box>
            {' '}{detail}
          </Typography>
        ))}
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* Next steps */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Next Steps
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
        Ready to go deeper? The full walkthrough covers provider configuration, task graph
        customisation, and deploying to the Mahalaxmi Cloud.
      </Typography>
      <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
        <Box
          component={Link}
          href="/docs/quickstart"
          sx={{
            display: 'block',
            p: 3,
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 2,
            textDecoration: 'none',
            color: 'inherit',
            flex: '1 1 200px',
            transition: 'border-color 0.2s',
            '&:hover': { borderColor: '#00C8C8' },
          }}
        >
          <Typography variant="h6" fontWeight={600} gutterBottom>
            Full Quickstart Walkthrough
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Step-by-step guide covering provider setup, project init options, and your first
            orchestrated session.
          </Typography>
        </Box>
        <Box
          component={Link}
          href="/open-source/architecture"
          sx={{
            display: 'block',
            p: 3,
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 2,
            textDecoration: 'none',
            color: 'inherit',
            flex: '1 1 200px',
            transition: 'border-color 0.2s',
            '&:hover': { borderColor: '#00C8C8' },
          }}
        >
          <Typography variant="h6" fontWeight={600} gutterBottom>
            Architecture Overview
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Learn how the Manager-Worker consensus engine, DAG task graph, PTY control, and
            provider routing work under the hood.
          </Typography>
        </Box>
      </Box>
    </Container>
  );
}

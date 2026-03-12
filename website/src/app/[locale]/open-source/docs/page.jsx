import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Link from 'next/link';

export const metadata = {
  title: 'Getting Started — Mahalaxmi Open Source',
  description: 'Get up and running with the Mahalaxmi CLI in three commands. Prerequisites, quick start guide, and next steps.',
  alternates: {
    canonical: '/open-source/docs',
  },
};

const headingSx = {
  color: '#00C8C8',
  fontWeight: 700,
  mt: 5,
  mb: 1.5,
};

const codeBlockSx = {
  fontFamily: 'monospace',
  backgroundColor: '#0D1117',
  color: '#00C8C8',
  p: 2,
  borderRadius: 1,
};

export default function OpenSourceDocsPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      {/* Breadcrumb */}
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 4 }}>
        <Link href="/open-source" style={{ color: '#00C8C8', textDecoration: 'none' }}>
          Open Source
        </Link>
        <Typography color="text.secondary">/</Typography>
        <Typography color="text.secondary">Getting Started</Typography>
      </Box>

      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Getting Started
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        Install the Mahalaxmi CLI, connect your AI provider, and launch your first multi-agent run in under a minute.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      {/* Section 1: Prerequisites */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Prerequisites
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Before installing Mahalaxmi, make sure you have the following:
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 2 }}>
        {[
          { name: 'Node.js 18+', desc: 'The CLI requires Node.js 18 or later. Download from nodejs.org.' },
          { name: 'Git', desc: 'Mahalaxmi uses git worktrees for worker isolation. Git must be available in your PATH.' },
          { name: 'AI provider account', desc: 'You need an account with a supported provider such as Anthropic (Claude Code), GitHub Copilot, or Grok. An API key or CLI auth session is required during configuration.' },
        ].map(({ name, desc }) => (
          <Typography key={name} component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
            <Box component="span" sx={{ color: '#00C8C8', fontFamily: 'monospace', fontWeight: 600 }}>{name}</Box>
            {' — '}{desc}
          </Typography>
        ))}
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* Section 2: 3-Command Quick Start */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        3-Command Quick Start
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Install the CLI globally, configure your provider, then run Mahalaxmi against your project.
      </Typography>

      <Typography variant="subtitle2" color="text.secondary" sx={{ mb: 1, mt: 2 }}>
        1. Install the Mahalaxmi CLI
      </Typography>
      <Box sx={codeBlockSx}>
        npm install -g @mahalaxmi/cli
      </Box>

      <Typography variant="subtitle2" color="text.secondary" sx={{ mb: 1, mt: 3 }}>
        2. Configure your AI provider
      </Typography>
      <Box sx={codeBlockSx}>
        mahalaxmi configure --provider claude-code --key &lt;key&gt;
      </Box>

      <Typography variant="subtitle2" color="text.secondary" sx={{ mb: 1, mt: 3 }}>
        3. Start orchestrating
      </Typography>
      <Box sx={codeBlockSx}>
        mahalaxmi run
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* Section 3: What's Next */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        {"What's Next"}
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Now that you have Mahalaxmi running, explore these resources to go deeper:
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 2 }}>
        <Typography component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
          <Link href="/open-source/architecture" style={{ color: '#00C8C8', textDecoration: 'none' }}>
            Architecture Overview
          </Link>
          {' — '}
          Understand the consensus engine, DAG task graph, PTY control, and git worktree isolation that power Mahalaxmi.
        </Typography>
        <Typography component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
          <Link href="/docs/quickstart" style={{ color: '#00C8C8', textDecoration: 'none' }}>
            Cloud Quickstart
          </Link>
          {' — '}
          Skip the CLI setup and run Mahalaxmi on managed cloud infrastructure with a single click.
        </Typography>
      </Box>
    </Container>
  );
}

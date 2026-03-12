import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Chip from '@mui/material/Chip';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Link from 'next/link';

export const metadata = {
  title: 'Open Source — Mahalaxmi',
  description:
    'Mahalaxmi is open source under the MIT License. Explore the architecture, providers, docs, changelog, roadmap, community, and contributing guides.',
  alternates: {
    canonical: '/open-source',
  },
};

interface NavCard {
  title: string;
  description: string;
  href: string;
}

const navCards: NavCard[] = [
  {
    title: 'Getting Started',
    description:
      'Install the CLI, run your first orchestration session, and understand what happens under the hood when you run mahalaxmi start.',
    href: '/open-source/docs',
  },
  {
    title: 'Architecture',
    description:
      'Deep-dive into the Manager-Worker consensus engine, DAG task graph, PTY control, worktree isolation, and provider routing.',
    href: '/open-source/architecture',
  },
  {
    title: 'Providers',
    description:
      'Supported AI providers: Claude Code, GitHub Copilot, Grok, Ollama, and Gemini. Learn how to add a custom provider via the Plugin SDK.',
    href: '/open-source/providers',
  },
  {
    title: 'Contributing',
    description:
      'Fork the repo, pick a good-first-issue, and submit a pull request. Contribution guidelines, code style, and review process explained.',
    href: '/open-source/contributing',
  },
  {
    title: 'Changelog',
    description:
      'Version history and release notes. See what changed in each release and how to migrate between major versions.',
    href: '/open-source/changelog',
  },
  {
    title: 'Roadmap',
    description:
      'Planned features and improvements. Vote on items, track progress, and see what the community is building next.',
    href: '/open-source/roadmap',
  },
  {
    title: 'Community',
    description:
      'Join the Mahalaxmi community on GitHub Discussions. Ask questions, share projects, and help shape the platform.',
    href: '/open-source/community',
  },
];

export default function OpenSourcePage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      {/* Hero */}
      <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 2, flexWrap: 'wrap', mb: 1 }}>
        <Typography variant="h3" component="h1" fontWeight={700}>
          Mahalaxmi Open Source
        </Typography>
        <Chip
          label="MIT License"
          size="small"
          sx={{
            mt: 1.2,
            backgroundColor: 'rgba(0,200,200,0.12)',
            color: '#00C8C8',
            border: '1px solid #00C8C8',
            fontWeight: 700,
            letterSpacing: 0.5,
          }}
        />
      </Box>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
        Mahalaxmi is free and open source. The core orchestration engine, provider adapters, and
        CLI are all available under the MIT License. Read the code, run it locally, or contribute
        back.
      </Typography>

      <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap', mb: 5 }}>
        <Box
          component="a"
          href="https://github.com/thrivetech2t/mahalaxmi"
          target="_blank"
          rel="noopener noreferrer"
          sx={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 1,
            color: '#00C8C8',
            border: '1px solid #00C8C8',
            borderRadius: 1,
            px: 2.5,
            py: 1,
            textDecoration: 'none',
            fontWeight: 600,
            fontSize: '0.9rem',
            transition: 'background-color 0.2s',
            '&:hover': { backgroundColor: 'rgba(0,200,200,0.08)' },
          }}
        >
          View on GitHub →
        </Box>
        <Box
          component="a"
          href="#"
          aria-label="VS Code Marketplace — Coming soon"
          sx={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 1,
            color: 'text.secondary',
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
            px: 2.5,
            py: 1,
            textDecoration: 'none',
            fontWeight: 600,
            fontSize: '0.9rem',
            cursor: 'default',
          }}
        >
          VS Code Marketplace
          <Chip
            label="Coming soon"
            size="small"
            sx={{
              height: 20,
              fontSize: '0.7rem',
              backgroundColor: 'rgba(255,255,255,0.06)',
              color: 'text.secondary',
            }}
          />
        </Box>
      </Box>

      <Divider sx={{ mb: 5 }} />

      {/* Navigation cards */}
      <Typography variant="h5" component="h2" fontWeight={700} sx={{ mb: 3 }}>
        Explore the Open Source Project
      </Typography>
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: { xs: '1fr', sm: '1fr 1fr' },
          gap: 2,
          mb: 6,
        }}
      >
        {navCards.map((card) => (
          <Card
            key={card.href}
            component={Link}
            href={card.href}
            variant="outlined"
            sx={{
              backgroundColor: 'transparent',
              borderColor: 'divider',
              textDecoration: 'none',
              color: 'inherit',
              transition: 'border-color 0.2s',
              '&:hover': { borderColor: '#00C8C8' },
            }}
          >
            <CardContent>
              <Typography variant="h6" fontWeight={600} gutterBottom>
                {card.title}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                {card.description}
              </Typography>
            </CardContent>
          </Card>
        ))}
      </Box>

      <Divider sx={{ mb: 5 }} />

      {/* Provider Plugin SDK callout */}
      <Box
        sx={{
          border: '1px solid #00C8C8',
          borderRadius: 2,
          p: 4,
          backgroundColor: 'rgba(0,200,200,0.04)',
        }}
      >
        <Typography variant="h5" component="h2" fontWeight={700} sx={{ mb: 1.5 }}>
          Provider Plugin SDK
        </Typography>
        <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
          Mahalaxmi works with any AI CLI tool that runs in a terminal. The Provider Plugin SDK
          lets you integrate a new provider in three steps: implement a PTY adapter, a prompt
          formatter, and a response parser. Once registered, your provider is immediately available
          for task routing and fallback chains — no changes to core orchestration logic required.
        </Typography>
        <Box
          component="a"
          href="https://github.com/thrivetech2t/mahalaxmi"
          target="_blank"
          rel="noopener noreferrer"
          sx={{
            display: 'inline-block',
            color: '#00C8C8',
            textDecoration: 'none',
            fontWeight: 600,
            '&:hover': { textDecoration: 'underline' },
          }}
        >
          View the Provider Plugin SDK on GitHub →
        </Box>
      </Box>

      <Divider sx={{ my: 5 }} />

      {/* License */}
      <Typography variant="body2" color="text.secondary" sx={{ textAlign: 'center' }}>
        Mahalaxmi is released under the{' '}
        <Box
          component="a"
          href="https://github.com/thrivetech2t/mahalaxmi/blob/main/LICENSE"
          target="_blank"
          rel="noopener noreferrer"
          sx={{ color: '#00C8C8', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
        >
          MIT License
        </Box>
        . Copyright © ThriveTech Services LLC.
      </Typography>
    </Container>
  );
}

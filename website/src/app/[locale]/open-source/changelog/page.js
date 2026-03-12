import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';

export const metadata = {
  title: 'Changelog — Mahalaxmi CLI',
  description: 'Version history for the Mahalaxmi open source CLI. Release notes, bug fixes, and feature additions.',
  alternates: {
    canonical: '/open-source/changelog',
  },
};

const headingSx = {
  color: '#00C8C8',
  fontWeight: 700,
  mt: 5,
  mb: 1.5,
};

const releases = [
  {
    version: 'v1.4.0',
    date: '2026-03-01',
    tag: 'Latest',
    changes: [
      { type: 'feat', text: 'Add Google Gemini CLI provider via PTY adapter.' },
      { type: 'feat', text: 'ProviderRouter: configurable fallback chain with priority ordering.' },
      { type: 'fix', text: 'Resolve race condition in DAG edge resolution when multiple Workers complete simultaneously.' },
      { type: 'fix', text: 'Correct worktree cleanup order when Manager receives SIGTERM during active cycle.' },
    ],
  },
  {
    version: 'v1.3.0',
    date: '2026-02-14',
    tag: null,
    changes: [
      { type: 'feat', text: 'Add Ollama provider for local self-hosted model execution.' },
      { type: 'feat', text: 'Add Grok (xAI) PTY adapter.' },
      { type: 'improvement', text: 'Semantic deduplication using Jaccard similarity now applied before merge strategy.' },
      { type: 'fix', text: 'Fix GitHub Copilot session teardown leaving orphaned PTY processes.' },
    ],
  },
  {
    version: 'v1.2.0',
    date: '2026-01-20',
    tag: null,
    changes: [
      { type: 'feat', text: 'Add GitHub Copilot CLI PTY adapter.' },
      { type: 'feat', text: 'WeightedVoting and ComplexityWeighted consensus strategies.' },
      { type: 'improvement', text: 'LLM arbitration now invoked automatically when no consensus strategy reaches threshold.' },
      { type: 'fix', text: 'Fix branch naming collision when two Workers target the same output path.' },
    ],
  },
  {
    version: 'v1.1.0',
    date: '2025-12-10',
    tag: null,
    changes: [
      { type: 'feat', text: 'Git worktree isolation — each Worker operates in a dedicated branch checkout.' },
      { type: 'feat', text: 'Branch-and-PR git strategy with automatic Pull Request creation on task completion.' },
      { type: 'feat', text: 'DAG task graph with dependency-aware parallel scheduling.' },
      { type: 'improvement', text: 'Improved PTY output parsing for Claude Code multi-turn sessions.' },
    ],
  },
  {
    version: 'v1.0.0',
    date: '2025-11-01',
    tag: 'Initial Release',
    changes: [
      { type: 'feat', text: 'Manager-Worker consensus engine with Union and Intersection merge strategies.' },
      { type: 'feat', text: 'Claude Code PTY control adapter — first supported provider.' },
      { type: 'feat', text: 'YAML-based orchestration configuration.' },
      { type: 'feat', text: 'CLI entrypoint with --config and --dry-run flags.' },
    ],
  },
];

const typeColor = (type) => {
  if (type === 'feat') return '#00C8C8';
  if (type === 'fix') return '#F87171';
  return '#A3A3A3';
};

const typeLabel = (type) => {
  if (type === 'feat') return 'feat';
  if (type === 'fix') return 'fix';
  return 'improvement';
};

export default function ChangelogPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Changelog
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 1 }}>
        Version history for the{' '}
        <Box component="span" sx={{ color: '#00C8C8', fontWeight: 600 }}>
          Mahalaxmi open source CLI
        </Box>
        . This changelog covers the CLI orchestration tool only — for cloud platform updates,
        see the dashboard release notes.
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 4, fontStyle: 'italic' }}>
        Versions follow{' '}
        <Box
          component="a"
          href="https://semver.org"
          target="_blank"
          rel="noopener noreferrer"
          sx={{ color: '#00C8C8', textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
        >
          Semantic Versioning
        </Box>
        . Breaking changes are noted in the affected release.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
        {releases.map((release) => (
          <Card
            key={release.version}
            variant="outlined"
            sx={{
              backgroundColor: '#0D1117',
              borderColor: release.tag === 'Latest' ? '#00C8C8' : 'divider',
            }}
          >
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2, flexWrap: 'wrap' }}>
                <Typography variant="h6" component="h2" fontWeight={700} sx={{ fontFamily: 'monospace' }}>
                  {release.version}
                </Typography>
                {release.tag && (
                  <Typography
                    variant="caption"
                    sx={{
                      color: release.tag === 'Latest' ? '#00C8C8' : '#22C55E',
                      border: `1px solid ${release.tag === 'Latest' ? '#00C8C8' : '#22C55E'}`,
                      borderRadius: '4px',
                      px: 1,
                      py: 0.25,
                      fontWeight: 600,
                      letterSpacing: 0.5,
                    }}
                  >
                    {release.tag}
                  </Typography>
                )}
                <Typography variant="body2" color="text.secondary" sx={{ ml: 'auto' }}>
                  {release.date}
                </Typography>
              </Box>

              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 0.75 }}>
                {release.changes.map((change, index) => (
                  <Box key={index} sx={{ display: 'flex', alignItems: 'flex-start', gap: 1.5 }}>
                    <Typography
                      variant="caption"
                      sx={{
                        color: typeColor(change.type),
                        fontFamily: 'monospace',
                        fontWeight: 700,
                        minWidth: '80px',
                        pt: 0.15,
                        letterSpacing: 0.5,
                      }}
                    >
                      {typeLabel(change.type)}
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      {change.text}
                    </Typography>
                  </Box>
                ))}
              </Box>
            </CardContent>
          </Card>
        ))}
      </Box>

      <Divider sx={{ my: 5 }} />

      <Typography variant="h5" component="h2" sx={headingSx}>
        Full Release History
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
        Every tagged release is available on GitHub. You can compare any two versions using
        the GitHub compare view.
      </Typography>
      <Box
        component="a"
        href="https://github.com/thrivetech2t/mahalaxmi/releases"
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
          '&:hover': { backgroundColor: 'rgba(0,200,200,0.08)' },
        }}
      >
        View All Releases on GitHub →
      </Box>
    </Container>
  );
}

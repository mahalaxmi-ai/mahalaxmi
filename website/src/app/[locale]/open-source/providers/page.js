import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';

export const metadata = {
  title: 'Providers — Mahalaxmi Open Source',
  description: 'AI providers supported by Mahalaxmi: Claude Code, GitHub Copilot, Grok, Ollama, and Gemini. Learn how to add a custom provider via the Plugin SDK.',
  alternates: {
    canonical: '/open-source/providers',
  },
};

const headingSx = {
  color: '#00C8C8',
  fontWeight: 700,
  mt: 5,
  mb: 1.5,
};

const providers = [
  {
    name: 'Claude Code',
    vendor: 'Anthropic',
    badge: 'Primary',
    description:
      'The primary supported provider. Claude Code is a full-featured AI coding CLI that Mahalaxmi controls via PTY for orchestrated, multi-agent software development workflows.',
  },
  {
    name: 'GitHub Copilot',
    vendor: 'GitHub / Microsoft',
    badge: 'Supported',
    description:
      'GitHub Copilot CLI integration allows Mahalaxmi Workers to leverage Copilot completions and chat within the orchestration pipeline using native PTY control.',
  },
  {
    name: 'Grok',
    vendor: 'xAI',
    badge: 'Supported',
    description:
      'Grok is xAI\'s conversational AI model. Mahalaxmi supports Grok via its CLI interface, enabling Workers to route tasks to Grok when configured as a provider.',
  },
  {
    name: 'Ollama',
    vendor: 'Ollama',
    badge: 'Local / Self-Hosted',
    description:
      'Run open-weight models locally. Ollama support lets teams keep sensitive workloads entirely on-premise without any external API calls, using Mahalaxmi\'s PTY adapter.',
  },
  {
    name: 'Gemini',
    vendor: 'Google',
    badge: 'Supported',
    description:
      'Google Gemini is available as a provider through the Gemini CLI. Mahalaxmi routes tasks to Gemini using the same PTY control layer shared by all providers.',
  },
];

const badgeColor = (badge) => {
  if (badge === 'Primary') return '#00C8C8';
  if (badge === 'Local / Self-Hosted') return '#8B5CF6';
  return '#22C55E';
};

export default function ProvidersPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Supported AI Providers
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        Mahalaxmi orchestrates any AI CLI tool through native PTY control. The providers listed
        below ship with built-in adapters. Additional providers can be added via the{' '}
        <Box component="span" sx={{ color: '#00C8C8' }}>Provider Plugin SDK</Box>.
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 4, fontStyle: 'italic' }}>
        The VS Code Extension connects to any provider via the Provider Plugin SDK.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        {providers.map((provider) => (
          <Card
            key={provider.name}
            variant="outlined"
            sx={{
              backgroundColor: '#0D1117',
              borderColor: 'divider',
              '&:hover': { borderColor: '#00C8C8' },
              transition: 'border-color 0.2s',
            }}
          >
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 1, flexWrap: 'wrap' }}>
                <Typography variant="h6" component="h2" fontWeight={700}>
                  {provider.name}
                </Typography>
                <Typography
                  variant="caption"
                  sx={{
                    color: badgeColor(provider.badge),
                    border: `1px solid ${badgeColor(provider.badge)}`,
                    borderRadius: '4px',
                    px: 1,
                    py: 0.25,
                    fontWeight: 600,
                    letterSpacing: 0.5,
                  }}
                >
                  {provider.badge}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {provider.vendor}
                </Typography>
              </Box>
              <Typography variant="body2" color="text.secondary">
                {provider.description}
              </Typography>
            </CardContent>
          </Card>
        ))}
      </Box>

      <Divider sx={{ my: 5 }} />

      {/* Provider Plugin SDK */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Provider Plugin SDK
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Any AI CLI tool can be integrated with Mahalaxmi by implementing the Provider Plugin SDK
        interface. A custom provider needs to supply three things:
      </Typography>
      <Box component="ol" sx={{ pl: 3, mb: 3 }}>
        {[
          'A PTY adapter that starts the CLI process and manages its stdio streams.',
          'A prompt formatter that translates Mahalaxmi task descriptors into the provider\'s expected input format.',
          'A response parser that extracts structured output from the CLI\'s raw terminal output.',
        ].map((item, index) => (
          <Typography key={index} component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
            {item}
          </Typography>
        ))}
      </Box>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Once implemented, the provider registers itself in the Mahalaxmi configuration file and
        becomes immediately available for task routing — including as a fallback in the{' '}
        <Box component="span" sx={{ color: '#00C8C8', fontFamily: 'monospace' }}>ProviderRouter</Box> chain.
      </Typography>
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
          '&:hover': { backgroundColor: 'rgba(0,200,200,0.08)' },
        }}
      >
        View Provider Plugin SDK on GitHub →
      </Box>
    </Container>
  );
}

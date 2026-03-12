import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';

export const metadata = {
  title: 'Providers — Mahalaxmi Open Source',
  description:
    'AI providers supported by Mahalaxmi: Claude Code, GitHub Copilot, Grok, Ollama, and Gemini. Learn how to add a custom provider via the Plugin SDK.',
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

const codeBlockSx = {
  fontFamily: 'monospace',
  backgroundColor: '#0D1117',
  color: '#E6EDF3',
  p: 3,
  borderRadius: 1,
  my: 2,
  overflowX: 'auto',
  whiteSpace: 'pre',
  fontSize: '0.85rem',
  border: '1px solid rgba(0,200,200,0.2)',
};

const providers = [
  {
    name: 'Claude Code',
    vendor: 'Anthropic',
    badge: 'Primary',
    useCase: 'Multi-agent software development, code generation, and complex reasoning tasks.',
    description:
      'The primary supported provider. Claude Code is a full-featured AI coding CLI that Mahalaxmi controls via PTY for orchestrated, multi-agent software development workflows. Its deep reasoning and long-context capabilities make it the default choice for complex engineering tasks.',
  },
  {
    name: 'GitHub Copilot',
    vendor: 'GitHub / Microsoft',
    badge: 'Supported',
    useCase: 'IDE-style completions and inline code suggestions within the orchestration pipeline.',
    description:
      'GitHub Copilot CLI integration allows Mahalaxmi Workers to leverage Copilot completions and chat within the orchestration pipeline using native PTY control. Ideal for teams already on the GitHub ecosystem who want orchestrated multi-agent capabilities.',
  },
  {
    name: 'Grok',
    vendor: 'xAI',
    badge: 'Supported',
    useCase: 'Real-time knowledge tasks and conversational reasoning with xAI models.',
    description:
      "Grok is xAI's conversational AI model. Mahalaxmi supports Grok via its CLI interface, enabling Workers to route tasks to Grok when configured as a provider. Grok's up-to-date training data makes it well-suited for time-sensitive knowledge retrieval tasks.",
  },
  {
    name: 'Ollama',
    vendor: 'Ollama',
    badge: 'Local / Self-Hosted',
    useCase: 'Fully offline, air-gapped, or on-premise workloads where no external API calls are permitted.',
    description:
      "Run open-weight models locally. Ollama support lets teams keep sensitive workloads entirely on-premise without any external API calls, using Mahalaxmi's PTY adapter. Supports any model available in the Ollama library including Llama, Mistral, and Phi.",
  },
  {
    name: 'Gemini',
    vendor: 'Google',
    badge: 'Supported',
    useCase: 'Long-context document processing and multimodal tasks via Google Gemini CLI.',
    description:
      'Google Gemini is available as a provider through the Gemini CLI. Mahalaxmi routes tasks to Gemini using the same PTY control layer shared by all providers, giving access to Gemini\'s strong multimodal and long-context capabilities within orchestrated pipelines.',
  },
];

const badgeColor = (badge) => {
  if (badge === 'Primary') return '#00C8C8';
  if (badge === 'Local / Self-Hosted') return '#8B5CF6';
  return '#22C55E';
};

const pluginSkeletonCode = `// provider-plugin-skeleton.js
// Minimal Mahalaxmi Provider Plugin SDK implementation

export default class MyProviderPlugin {
  /**
   * Unique identifier used in mahalaxmi.config.json
   */
  static id = 'my-provider';

  /**
   * Human-readable display name shown in logs and the dashboard.
   */
  static displayName = 'My Provider';

  /**
   * Execute a task and return a structured result.
   *
   * @param {object} task     - Task descriptor from the Manager
   * @param {string} task.id  - Unique task ID
   * @param {string} task.prompt - The prompt or instruction to execute
   * @param {object} context  - Runtime context supplied by Mahalaxmi
   * @param {string} context.workdir  - Absolute path to the Worker's git worktree
   * @param {object} context.env      - Sanitised environment variables
   * @param {object} context.logger   - Structured logger (info, warn, error)
   *
   * @returns {Promise<{output: string, exitCode: number, artifacts: string[]}>}
   */
  async execute(task, context) {
    const { prompt, id } = task;
    const { workdir, logger } = context;

    // 1. Start the provider CLI inside the Worker's git worktree.
    const session = await this.startSession({ cwd: workdir });

    try {
      // 2. Send the prompt through the PTY session.
      await session.send(prompt);

      // 3. Wait for the provider to signal completion.
      const rawOutput = await session.waitForCompletion();

      // 4. Parse and normalise the raw terminal output.
      const output = this.parseOutput(rawOutput);

      logger.info({ taskId: id, provider: MyProviderPlugin.id }, 'Task complete');

      return {
        output,
        exitCode: 0,
        artifacts: [],
      };
    } catch (err) {
      logger.error({ taskId: id, err }, 'Provider execution failed');
      return {
        output: '',
        exitCode: 1,
        artifacts: [],
      };
    } finally {
      await session.close();
    }
  }

  // ── Internal helpers ────────────────────────────────────────────────────────

  async startSession(options) {
    // Implement PTY session initialisation for your CLI tool.
    throw new Error('startSession() not implemented');
  }

  parseOutput(raw) {
    // Strip ANSI escape codes and extract the relevant response text.
    return raw.replace(/\\x1B\\[[0-9;]*m/g, '').trim();
  }
}`;

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

      <Divider sx={{ mb: 4 }} />

      {/* Section 1: Supported Providers */}
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
              <Typography
                variant="body2"
                sx={{ color: '#00C8C8', mb: 1, fontStyle: 'italic' }}
              >
                Use case: {provider.useCase}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                {provider.description}
              </Typography>
            </CardContent>
          </Card>
        ))}
      </Box>

      <Divider sx={{ my: 5 }} />

      {/* Section 2: Provider Plugin SDK */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Provider Plugin SDK
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Any AI CLI tool can be integrated with Mahalaxmi by implementing the Provider Plugin SDK
        interface. The core contract is the{' '}
        <Box component="span" sx={{ color: '#00C8C8', fontFamily: 'monospace' }}>execute(task, context)</Box>{' '}
        method, which receives a task descriptor and a runtime context, and returns a structured
        result object.
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        A custom provider needs to supply three things:
      </Typography>
      <Box component="ol" sx={{ pl: 3, mb: 3 }}>
        {[
          'A PTY adapter that starts the CLI process and manages its stdio streams.',
          "A prompt formatter that translates Mahalaxmi task descriptors into the provider's expected input format.",
          "A response parser that extracts structured output from the CLI's raw terminal output.",
        ].map((item, index) => (
          <Typography
            key={index}
            component="li"
            variant="body1"
            color="text.secondary"
            sx={{ mb: 1 }}
          >
            {item}
          </Typography>
        ))}
      </Box>

      <Typography variant="subtitle2" sx={{ color: '#00C8C8', mb: 1, fontWeight: 600 }}>
        Minimal plugin skeleton
      </Typography>
      <Box sx={codeBlockSx}>{pluginSkeletonCode}</Box>

      <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
        Once implemented, register the provider in{' '}
        <Box component="span" sx={{ color: '#00C8C8', fontFamily: 'monospace' }}>mahalaxmi.config.json</Box>{' '}
        and it becomes immediately available for task routing — including as a fallback in the{' '}
        <Box component="span" sx={{ color: '#00C8C8', fontFamily: 'monospace' }}>ProviderRouter</Box> chain.
        Full SDK documentation, TypeScript types, and example implementations are on GitHub.
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

      <Divider sx={{ my: 5 }} />

      {/* Section 3: Submit plugin via PR CTA */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Submit Your Provider Plugin
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Built a plugin for a provider not listed above? Contribute it to the Mahalaxmi open-source
        repository and help the community expand the ecosystem. Accepted plugins are bundled with
        the next Mahalaxmi release and listed on this page.
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
        To submit:
      </Typography>
      <Box component="ol" sx={{ pl: 3, mb: 4 }}>
        {[
          'Fork the mahalaxmi repository on GitHub.',
          'Add your plugin under packages/providers/<your-provider-name>/.',
          'Include a README with setup instructions and at least one example.',
          'Open a Pull Request — a maintainer will review within a few business days.',
        ].map((step, index) => (
          <Typography
            key={index}
            component="li"
            variant="body1"
            color="text.secondary"
            sx={{ mb: 1 }}
          >
            {step}
          </Typography>
        ))}
      </Box>
      <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
        <Box
          component="a"
          href="https://github.com/thrivetech2t/mahalaxmi"
          target="_blank"
          rel="noopener noreferrer"
          sx={{
            display: 'inline-block',
            backgroundColor: '#00C8C8',
            color: '#000',
            borderRadius: 1,
            px: 3,
            py: 1.25,
            textDecoration: 'none',
            fontWeight: 700,
            '&:hover': { backgroundColor: '#00a8a8' },
          }}
        >
          Submit a Plugin via PR
        </Box>
        <Box
          component="a"
          href="https://github.com/thrivetech2t/mahalaxmi/issues"
          target="_blank"
          rel="noopener noreferrer"
          sx={{
            display: 'inline-block',
            color: '#00C8C8',
            border: '1px solid #00C8C8',
            borderRadius: 1,
            px: 3,
            py: 1.25,
            textDecoration: 'none',
            fontWeight: 600,
            '&:hover': { backgroundColor: 'rgba(0,200,200,0.08)' },
          }}
        >
          Open an Issue
        </Box>
      </Box>
    </Container>
  );
}

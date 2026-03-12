import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Box,
  Button,
  Container,
  Divider,
  Paper,
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
    title: 'Whitepaper — Mahalaxmi',
    description:
      'Technical whitepaper: the architecture and design principles behind the Mahalaxmi AI terminal orchestration engine.',
    alternates: {
      canonical: getCanonical(locale, '/whitepaper'),
      languages: getAlternateLanguages('/whitepaper'),
    },
    openGraph: {
      title: 'Mahalaxmi Technical Whitepaper',
      description: 'Architecture and design principles of the Mahalaxmi orchestration engine.',
      url: '/whitepaper',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const sections = [
  {
    heading: '1. Introduction',
    body: `Modern software teams increasingly rely on AI coding assistants, but today's tools are inherently sequential — one agent, one conversation, one stream of changes. Mahalaxmi is built on the observation that software tasks are naturally decomposable and that the bottleneck is not the AI model but the orchestration layer that coordinates agents, manages context, and presents results for human review.

Mahalaxmi provides a lightweight orchestration engine that runs multiple AI coding agents in parallel, assigns tasks from a shared queue, and routes proposed file changes through a structured review workflow. The system is model-agnostic and works with any CLI-based AI coding tool.`,
  },
  {
    heading: '2. Architecture Overview',
    body: `The Mahalaxmi orchestration engine is a local process that manages a pool of worker subprocesses. Each worker runs an AI coding CLI (Claude Code, Codex CLI, Gemini CLI, etc.) in a sandboxed working directory. The engine assigns tasks from a queue, captures proposed changes as structured diffs, and surfaces them to the developer through the VS Code extension.

Key components:
- Task Queue — accepts natural-language task descriptions and assigns them to available workers.
- Worker Pool — a configurable number of concurrent AI CLI subprocesses, each with isolated context.
- Diff Aggregator — collects proposed file changes from workers and deduplicates conflicting edits.
- Review Gateway — presents changes to the developer in VS Code; nothing is committed without explicit approval.
- Cloud Bridge — when running on a Mahalaxmi Cloud server, connects the local VS Code extension to the remote engine over an encrypted WebSocket tunnel.`,
  },
  {
    heading: '3. Worker Isolation Model',
    body: `Each worker is assigned a task and a working directory snapshot. Workers propose changes as diffs — they do not write directly to the shared working tree. This isolation means:
- Workers cannot interfere with each other's changes.
- A failed or misbehaving worker does not corrupt shared state.
- Changes are always reviewable before being applied.

The engine merges non-conflicting diffs automatically. Conflicting edits are flagged for manual resolution in the VS Code extension.`,
  },
  {
    heading: '4. AI Provider Abstraction',
    body: `Mahalaxmi does not prescribe a specific AI model. It communicates with AI coding tools through their standard CLI interfaces — stdin/stdout streams and file-system conventions. This means:
- Switching providers requires only a configuration change, not a code change.
- Different workers in the same session can use different providers.
- New providers are supported as soon as they release a CLI tool.

Supported providers include Anthropic Claude Code, OpenAI Codex CLI, Google Gemini CLI, xAI Grok CLI, GitHub Copilot CLI, and any Ollama-compatible local model.`,
  },
  {
    heading: '5. Security Design',
    body: `AI provider API keys are stored in the operating system's native credential store (macOS Keychain, Windows Credential Manager, Linux Secret Service) when running on desktop. On Mahalaxmi Cloud servers, keys are encrypted with AES-256-GCM before being persisted to disk and are decrypted only during the VM boot sequence. Keys are never transmitted to ThriveTech, never logged, and never returned by any API endpoint.

The Cloud Bridge connection between the VS Code extension and a remote Mahalaxmi server uses a mutually-authenticated TLS tunnel. Connection credentials are delivered via the VS Code deep link mechanism and are single-use.`,
  },
  {
    heading: '6. Open Source',
    body: `The Mahalaxmi orchestration core is open source under the MIT License. The source code is available on GitHub at github.com/thrivetech2t/mahalaxmi. Commercial add-ons (the cloud management layer, VS Code extension, and license server) are proprietary.

We believe the orchestration primitives — the worker pool, task queue, diff aggregator, and review gateway — should be auditable and forkable by the community. Contributions, bug reports, and feature proposals are welcome.`,
  },
];

export default async function WhitepaperPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="md" sx={{ py: { xs: 4, md: 8 } }}>
      <Paper elevation={1} sx={{ p: { xs: 3, md: 6 }, borderRadius: 3 }}>
        <Box sx={{ borderBottom: '3px solid', borderColor: 'primary.main', pb: 3, mb: 5 }}>
          <Typography variant="overline" color="primary" sx={{ fontWeight: 700, letterSpacing: 2 }}>
            Technical Whitepaper
          </Typography>
          <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mt: 1, mb: 1 }}>
            Mahalaxmi: Parallel AI Orchestration for Software Development
          </Typography>
          <Typography variant="body2" color="text.secondary">
            ThriveTech Services LLC &nbsp;|&nbsp; Version 1.0 &nbsp;|&nbsp; 2026
          </Typography>
        </Box>

        {sections.map(({ heading, body }, i) => (
          <Box key={heading} sx={{ mb: 5 }}>
            <Typography variant="h5" component="h2" sx={{ fontWeight: 700, mb: 2 }}>
              {heading}
            </Typography>
            {body.split('\n\n').map((para, j) => (
              <Typography
                key={j}
                variant="body1"
                paragraph
                sx={{ whiteSpace: 'pre-line', color: j === 0 ? 'text.primary' : 'text.secondary' }}
              >
                {para}
              </Typography>
            ))}
            {i < sections.length - 1 && <Divider sx={{ mt: 4 }} />}
          </Box>
        ))}

        <Box sx={{ mt: 6, pt: 4, borderTop: '1px solid', borderColor: 'divider', textAlign: 'center' }}>
          <Typography variant="h6" sx={{ fontWeight: 700, mb: 2 }}>
            Ready to try it yourself?
          </Typography>
          <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
            <Button
              component={Link}
              href="/pricing"
              variant="contained"
              size="large"
              endIcon={<ArrowForward />}
            >
              Get Started
            </Button>
            <Button
              component={Link}
              href="/open-source"
              variant="outlined"
              size="large"
            >
              View Open Source
            </Button>
          </Box>
        </Box>
      </Paper>
    </Container>
  );
}

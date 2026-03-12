import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Container, Box, Typography, Paper, Grid, Card, CardContent,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow,
  Chip, Breadcrumbs, Button, Divider,
} from '@mui/material';
import { NavigateNext, Download, ArrowForward } from '@mui/icons-material';
import Link from 'next/link';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  return {
    title: 'Mahalaxmi Technical Whitepaper — Multi-Agent AI Development at Scale',
    description: 'Technical whitepaper covering Mahalaxmi\'s Manager-Worker DAG architecture, consensus mechanisms, PTY-native terminal control, intelligent context routing, GraphRAG knowledge graph, and enterprise governance.',
    alternates: {
      canonical: getCanonical(locale, '/whitepaper'),
      languages: getAlternateLanguages('/whitepaper'),
    },
    openGraph: {
      title: 'Mahalaxmi Technical Whitepaper — Multi-Agent AI Development at Scale',
      description: 'Architectural principles, consensus mechanisms, context routing strategies, and enterprise governance features for large-scale multi-agent development.',
      url: '/whitepaper',
      images: [{ url: '/mahalaxmi_logo.png' }],
      locale: getOpenGraphLocale(locale),
    },
  };
}

const sections = [
  {
    id: '1',
    title: '1. Introduction',
    content: `The AI coding assistant market has matured rapidly. Tools like Claude Code, GitHub Copilot, OpenAI Codex, and their open-source counterparts have demonstrated that LLMs can produce high-quality code given appropriate context and direction. Individual developers report 2–5× productivity gains from these tools.

The bottleneck is no longer whether AI can write code — it is whether a single AI agent can handle the full scope of a complex software project without becoming a single point of serialization.

A real codebase has hundreds of independent modules, services, and components that could be developed in parallel. Yet every AI coding assistant available today works in exactly one conversation at a time, with one developer watching one terminal.

Mahalaxmi is the orchestration layer that changes this. Rather than one AI agent working sequentially through a task list, Mahalaxmi deploys a team of AI agents organized into a Manager-Worker hierarchy. Manager agents analyze the codebase, deliberate on an execution plan, and produce a dependency-ordered task graph. Worker agents execute tasks in parallel, each isolated in its own Git worktree, with context pre-filtered to only the code they need. The results are automatically integrated via pull requests, tested through a post-cycle validation pipeline, and surfaced through a real-time desktop UI.`,
  },
  {
    id: '2',
    title: '2. Problem Statement',
    subsections: [
      {
        title: '2.1 The Sequential AI Development Bottleneck',
        content: `Current AI coding tools suffer from three fundamental constraints:

Single-agent serialization. One conversation produces one stream of work. If a task has 20 independent subtasks, they execute one at a time. The parallelism inherent in the problem structure is discarded.

Context window saturation. Coding agents degrade in quality as conversation context grows. Large codebases exhaust context windows, causing agents to "forget" earlier constraints or produce code that conflicts with work already done in the same session.

Tool lock-in. Most enterprise environments already have contracts with multiple AI providers (Anthropic, OpenAI, AWS Bedrock, Google Vertex). Existing orchestration tools typically target a single provider, leaving significant purchased capacity idle.`,
      },
      {
        title: '2.2 The Human-in-the-Loop Gap',
        content: `Teams using AI coding tools today operate them manually: a developer types a prompt, reviews the output, applies a change, and repeats. This keeps development fully serialized at the human level. Automating the orchestration layer — while preserving human oversight at key decision points — is the unsolved problem.`,
      },
      {
        title: '2.3 Enterprise Governance Challenges',
        content: `As AI coding adoption grows, organizations face new governance requirements: tracking AI-generated code for compliance and audit purposes, controlling per-developer and per-project AI spend, enforcing security policies (no secrets in output, dependency audits), and managing multi-seat licensing for team deployments. No existing tool addresses all of these at the infrastructure level.`,
      },
    ],
  },
  {
    id: '3',
    title: '3. Architecture Overview',
    content: `Mahalaxmi is a cross-platform desktop application built on Tauri 2.x (Rust + WebView), providing a native application experience on Windows, macOS, and Linux without requiring a cloud backend.`,
    crateTable: [
      { crate: 'mahalaxmi-core', responsibility: 'Shared domain types, i18n (10 locales), config, error types, logging' },
      { crate: 'mahalaxmi-pty', responsibility: 'PTY spawning, stream I/O, VT100/ANSI parsing, event emission' },
      { crate: 'mahalaxmi-providers', responsibility: 'AiProvider trait + implementations for 8+ AI coding tools' },
      { crate: 'mahalaxmi-orchestration', responsibility: 'Cycle state machine, consensus engine, worker queue, DAG scheduling, Git worktree management' },
      { crate: 'mahalaxmi-detection', responsibility: 'State detection rules, pattern matching, auto-response for interactive prompts' },
    ],
  },
  {
    id: '4',
    title: '4. The Consensus Engine',
    content: `The Consensus Engine is the component responsible for merging multiple manager agents' execution plans into a single, coherent, deduplicated task graph.

When the Manager Phase begins, N manager agents (configurable, 1–8) independently analyze the codebase and requirements document. Each manager produces a proposed execution plan — a set of tasks with names, descriptions, target files, complexity estimates, and dependencies. The Consensus Engine then merges these proposals using one of four strategies:

Union: includes all unique tasks from all managers. Semantic deduplication identifies tasks that describe the same work with different words and merges them. Best for maximizing coverage.

Intersection: includes only tasks that appeared in every manager's plan. Produces a minimal, high-confidence plan. Best for conservative or safety-critical work.

WeightedVoting: tasks weighted by the historical performance reputation of the provider that proposed them. Best when provider quality varies significantly.

ComplexityWeighted: tasks weighted by their complexity scores. Ensures high-complexity tasks from any manager are included. Best for complex projects where missing a hard task is costly.

The deduplication pipeline is CamelCase-aware — it tokenizes task names into constituent words before computing similarity. Multi-field Jaccard similarity is computed across the task name, description, and file scope. Tasks in the ambiguous zone are sent to LLM arbitration: a prompt asks the model to determine whether two candidate tasks should be merged, kept separate, or synthesized into a new task that captures the intent of both.`,
  },
  {
    id: '5',
    title: '5. PTY-Native Terminal Control',
    content: `Mahalaxmi controls AI coding tools by taking ownership of their terminal (PTY — pseudoterminal). This is architecturally distinct from screen capture or clipboard injection approaches.

When Mahalaxmi spawns an AI coding tool, it spawns the process with a PTY attached. Mahalaxmi's PTY engine (mahalaxmi-pty) reads raw bytes from the PTY output stream and writes commands to the PTY input stream. The byte stream is parsed using a VT100/ANSI state machine to extract semantic content — text, cursor positions, colors — without rendering the content to a screen.

This architecture provides several properties not achievable with screen capture:

Reliability. The byte stream is deterministic for a given tool version. OCR errors do not occur because there is no image processing.

Universality. Any terminal-based AI CLI tool can be controlled — the approach is not specific to a font, color scheme, or terminal emulator.

Latency. Reading raw bytes from a PTY is faster than screen capture pipelines, which typically require frame capture, image decoding, and OCR.

Completeness. The full output, including color codes and terminal control sequences, is available for analysis.`,
  },
  {
    id: '6',
    title: '6. Intelligent Context Routing',
    content: `Workers do not receive the full codebase as context. The full codebase context problem is one of the fundamental failure modes of single-agent AI coding: as context grows, quality degrades.

Mahalaxmi's context routing system selects a minimal, maximally relevant subset of files for each worker task. Files are scored using three signals:

Lexical Jaccard (α): the Jaccard similarity between the vocabulary of the task description and the vocabulary of the file's content.

Import-graph proximity (β): the BFS distance in the codebase's import dependency graph from the task's explicitly named target files to each candidate file. Files closer in the import graph to the task's targets score higher.

Historical co-occurrence (γ): files that have been modified together in previous cycles involving similar tasks score higher.

The combined score is: score(f) = α · lexical(f) + β · proximity(f) + γ · cooccurrence(f)

Files are ranked by this score and added to the worker's context window until the configured token budget is exhausted. Workers receive exactly the files they need — not the whole repository.`,
  },
];

const perfData = [
  { task: '6-task REST API feature', sequential: '~18 min', parallel: '~4 min', speedup: '4.5×' },
  { task: '15-task test coverage', sequential: '~55 min', parallel: '~12 min', speedup: '4.6×' },
  { task: '22-task microservice', sequential: '~90 min', parallel: '~21 min', speedup: '4.3×' },
];

export default async function MahalaxmiWhitepaperPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="md" sx={{ py: { xs: 4, md: 8 } }}>
      <Breadcrumbs separator={<NavigateNext fontSize="small" />} sx={{ mb: 3 }}>
        <Link href="/" style={{ textDecoration: 'none', color: 'inherit' }}>Mahalaxmi</Link>
        <Typography color="text.primary">Whitepaper</Typography>
      </Breadcrumbs>

      <Paper elevation={1} sx={{ p: { xs: 3, md: 6 } }}>
        {/* Header */}
        <Box sx={{ borderBottom: '3px solid', borderColor: 'primary.main', pb: 3, mb: 5 }}>
          <Chip label="Technical Whitepaper" color="primary" size="small" sx={{ mb: 2 }} />
          <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 1 }}>
            Mahalaxmi Terminal Orchestration
          </Typography>
          <Typography variant="h5" color="text.secondary" sx={{ mb: 2 }}>
            Multi-Agent AI Development at Scale
          </Typography>
          <Typography variant="body2" color="text.disabled">
            Version 1.0 &nbsp;|&nbsp; March 2026 &nbsp;|&nbsp; ThriveTech Services LLC
          </Typography>
        </Box>

        {/* Abstract */}
        <Box sx={{ bgcolor: 'grey.50', p: 3, borderRadius: 1, mb: 5 }}>
          <Typography variant="overline" color="text.secondary" sx={{ fontWeight: 700 }}>Abstract</Typography>
          <Typography variant="body1" sx={{ mt: 1 }}>
            Modern software development increasingly relies on AI coding assistants, yet these tools are almost universally designed for one-agent, one-developer interaction. Mahalaxmi Terminal Orchestration breaks this constraint by running dozens of AI coding agents in parallel — each with isolated workspaces, intelligent task assignment, and full PTY-native terminal control — coordinated by a Manager-Worker directed acyclic graph (DAG) architecture. This paper describes the architectural principles, consensus mechanisms, context routing strategies, and enterprise governance features that make large-scale multi-agent development practical on a standard developer workstation.
          </Typography>
        </Box>

        {/* Observed performance */}
        <Box sx={{ mb: 5 }}>
          <Typography variant="h5" component="h2" sx={{ fontWeight: 700, mb: 2 }}>
            Observed Performance
          </Typography>
          <TableContainer component={Paper} elevation={0} variant="outlined">
            <Table size="small">
              <TableHead>
                <TableRow sx={{ bgcolor: 'grey.100' }}>
                  <TableCell sx={{ fontWeight: 600 }}>Task Type</TableCell>
                  <TableCell align="center" sx={{ fontWeight: 600 }}>Sequential</TableCell>
                  <TableCell align="center" sx={{ fontWeight: 600 }}>Parallel (8 workers)</TableCell>
                  <TableCell align="center" sx={{ fontWeight: 600, color: 'success.main' }}>Speedup</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {perfData.map(({ task, sequential, parallel, speedup }) => (
                  <TableRow key={task}>
                    <TableCell>{task}</TableCell>
                    <TableCell align="center">{sequential}</TableCell>
                    <TableCell align="center">{parallel}</TableCell>
                    <TableCell align="center">
                      <Typography variant="body2" sx={{ fontWeight: 700, color: 'success.main' }}>{speedup}</Typography>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
          <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block' }}>
            Speedup is sub-linear due to dependency-ordered phases and merge overhead. Consistently 4–5× across project types.
          </Typography>
        </Box>

        <Divider sx={{ mb: 5 }} />

        {/* Sections */}
        {sections.map((section) => (
          <Box key={section.id} sx={{ mb: 5 }}>
            <Typography variant="h5" component="h2" sx={{ fontWeight: 700, mb: 2, pb: 1, borderBottom: '1px solid', borderColor: 'divider' }}>
              {section.title}
            </Typography>

            {section.content && (
              <Typography variant="body1" sx={{ whiteSpace: 'pre-line', lineHeight: 1.8 }}>
                {section.content}
              </Typography>
            )}

            {section.crateTable && (
              <TableContainer component={Paper} elevation={0} variant="outlined" sx={{ mt: 2 }}>
                <Table size="small">
                  <TableHead>
                    <TableRow sx={{ bgcolor: 'grey.100' }}>
                      <TableCell sx={{ fontWeight: 600 }}>Crate</TableCell>
                      <TableCell sx={{ fontWeight: 600 }}>Responsibility</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {section.crateTable.map(({ crate, responsibility }) => (
                      <TableRow key={crate}>
                        <TableCell><code>{crate}</code></TableCell>
                        <TableCell>{responsibility}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            )}

            {section.subsections && section.subsections.map((sub) => (
              <Box key={sub.title} sx={{ mt: 3 }}>
                <Typography variant="h6" sx={{ fontWeight: 600, mb: 1 }}>{sub.title}</Typography>
                <Typography variant="body1" sx={{ whiteSpace: 'pre-line', lineHeight: 1.8 }}>{sub.content}</Typography>
              </Box>
            ))}
          </Box>
        ))}

        {/* Technology stack */}
        <Box sx={{ mb: 5 }}>
          <Typography variant="h5" component="h2" sx={{ fontWeight: 700, mb: 2, pb: 1, borderBottom: '1px solid', borderColor: 'divider' }}>
            7. Technology Stack
          </Typography>
          <Grid container spacing={2}>
            {[
              { label: 'Core', value: 'Rust (portable-pty, tokio, rusqlite, tree-sitter, axum)' },
              { label: 'Desktop shell', value: 'Tauri 2.x (cross-platform, native webview, system keyring)' },
              { label: 'Frontend', value: 'Next.js + TypeScript' },
              { label: 'AI providers', value: 'Claude Code, OpenAI Foundry, AWS Bedrock, Google Gemini, Kiro, Goose, DeepSeek, Qwen Coder' },
              { label: 'IDE extensions', value: 'VS Code, JetBrains, Neovim, Visual Studio' },
            ].map(({ label, value }) => (
              <Grid item xs={12} key={label}>
                <Card elevation={0} variant="outlined">
                  <CardContent sx={{ py: 1.5, px: 2, '&:last-child': { pb: 1.5 } }}>
                    <Typography variant="caption" color="text.secondary" sx={{ fontWeight: 700, textTransform: 'uppercase' }}>{label}</Typography>
                    <Typography variant="body2">{value}</Typography>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        </Box>

        {/* Privacy */}
        <Box sx={{ bgcolor: 'primary.50', borderLeft: '4px solid', borderColor: 'primary.main', p: 3, borderRadius: '0 8px 8px 0', mb: 5 }}>
          <Typography variant="h6" sx={{ fontWeight: 700, mb: 1 }}>Security & Privacy Architecture</Typography>
          <Typography variant="body1">
            All orchestration runs locally on the developer&apos;s machine. AI provider calls go directly from the developer&apos;s machine to the provider&apos;s endpoint — Mahalaxmi never proxies, relays, or receives AI prompts, completions, or code. License validation transmits only a machine fingerprint and license token.
          </Typography>
        </Box>

        {/* CTA */}
        <Box sx={{ textAlign: 'center', pt: 3, borderTop: '1px solid', borderColor: 'divider' }}>
          <Typography variant="body1" sx={{ mb: 3 }}>
            Ready to run a team of AI agents on your codebase?
          </Typography>
          <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
            <Button component={Link} href="/products/mahalaxmi-ai-terminal-orchestration" variant="contained" startIcon={<Download />}>
              Download Free
            </Button>
            <Button component={Link} href="/features" variant="outlined" endIcon={<ArrowForward />}>
              View all features
            </Button>
          </Box>
        </Box>
      </Paper>
    </Container>
  );
}

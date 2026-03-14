'use client';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Container, Box, Typography, Grid, Card, CardContent,
  Table, TableBody, TableCell, TableContainer, TableHead, TableRow,
  Paper, Chip, Breadcrumbs,
} from '@mui/material';
import Link from 'next/link';
import { NavigateNext } from '@mui/icons-material';

const categories = [
  {
    name: 'Orchestration Engine',
    features: [
      {
        title: 'Manager-Worker DAG Architecture',
        tier: null,
        body: 'Mahalaxmi organizes AI agents into a two-tier hierarchy. Manager agents analyze requirements and produce execution plans. Worker agents execute tasks in dependency order. Tasks within a phase run in parallel; tasks with dependencies wait for prerequisites.',
        details: ['Configurable number of managers (1–8)', 'Automatic DAG validation — rejects circular dependencies before execution starts', 'Phase-based scheduling — all tasks in a phase run concurrently', 'Per-task file scope, provider assignment, and complexity estimate'],
      },
      {
        title: 'Consensus Engine — Four Strategies',
        tier: null,
        body: 'When multiple manager agents propose execution plans, the consensus engine merges them into a single coherent plan.',
        details: [
          'Union (default) — combines all unique tasks; best for maximizing coverage',
          'Intersection — only tasks all managers agreed on; best for conservative or safety-critical work',
          'WeightedVoting — tasks weighted by manager provider historical performance',
          'ComplexityWeighted — tasks weighted by complexity score; best for complex projects',
        ],
      },
      {
        title: 'Semantic Deduplication + LLM Arbitration',
        tier: null,
        body: 'CamelCase-aware tokenization, stop-word filtering, multi-field Jaccard similarity (name + description + file scope), three-zone classification, and LLM arbitration for ambiguous pairs. No duplicate worker tasks, no missing coverage.',
        details: [],
      },
      {
        title: 'Adversarial Manager Deliberation',
        tier: 'Professional+',
        body: 'Structured 3-turn deliberation per domain: Proposer presents a task plan, Challenger critiques and identifies gaps, Synthesizer produces a refined plan. Mirrors how good engineering teams do design review.',
        details: [],
      },
    ],
  },
  {
    name: 'Worker Execution',
    features: [
      {
        title: 'Git Worktree Isolation',
        tier: null,
        body: 'Every worker task runs in a dedicated Git worktree — a separate working directory on the same repository. Workers cannot interfere with each other\'s file changes. Each worker\'s output is a clean branch ready for PR.',
        details: ['Branch names encode cycle label and a unique 8-character cycle ID', 'Re-running the same requirements file never produces colliding branch names'],
      },
      {
        title: 'Intelligent Context Routing',
        tier: null,
        body: 'Workers receive a curated file selection scored by three signals: Lexical Jaccard (keyword overlap), Import-graph proximity (BFS distance in dependency graph), Historical co-occurrence (files modified together in similar past cycles).',
        details: ['Configurable token budget per worker', 'Workers get what they need — nothing they don\'t'],
      },
      {
        title: 'PTY-Native Terminal Control',
        tier: null,
        body: 'Mahalaxmi controls AI coding tools by owning their terminal (PTY), not by scraping screens. Works with any terminal-based AI CLI tool, reads raw bytes with no rendering artifacts, detects interactive prompts via pattern matching and auto-responds.',
        details: ['Reliable regardless of font, color scheme, or terminal emulator', 'Captures raw terminal bytes for faithful replay in the UI'],
      },
      {
        title: 'Self-Verification Pipeline',
        tier: null,
        body: 'Before a worker pushes its branch, it runs your project\'s own verification tools. Workers that fail verification receive the failure output as additional context and retry. Workers that fail twice are flagged for human review.',
        details: ['Test runner: cargo test, pytest, jest, go test', 'Linter: clippy, eslint, pylint, golangci-lint', 'Build gate: verifies the project compiles after changes', 'Security gate: runs the full security pipeline on the diff'],
      },
      {
        title: 'Auto-Chain Cycle Continuation',
        tier: null,
        body: 'For large requirements files organized as wave groups, Mahalaxmi detects when a cycle completes a phase and automatically starts the next cycle for the next incomplete wave group.',
        details: [],
      },
    ],
  },
  {
    name: 'Intelligence',
    features: [
      {
        title: 'Codebase Indexing',
        tier: null,
        body: 'When a project is opened, Mahalaxmi indexes it using Tree-sitter parsers: function/class/struct definitions with line ranges, import/export dependency mapping, cross-file call graph edges, and file relevance scores.',
        details: ['Supported: Rust, TypeScript/JavaScript, Python, Go, Java, C/C++, and more via Tree-sitter grammar ecosystem'],
      },
      {
        title: 'GraphRAG Knowledge Graph',
        tier: 'Professional+',
        body: 'A queryable code knowledge graph answering questions like "What calls authenticate()?" or "What is the blast radius of changing auth_middleware.rs?" The impact scorer computes a 0–10 risk score based on downstream dependency count and centrality.',
        details: [],
      },
      {
        title: 'Cross-Agent Shared Memory',
        tier: null,
        body: 'Workers read from and write to a shared memory store. Discoveries propagate across the cycle. Memory entries have scope (Session / Project / Global), configurable decay, and — for Enterprise — team sync.',
        details: [],
      },
      {
        title: 'Codebase Q&A and Wiki Generation',
        tier: 'Professional+',
        body: 'Ask questions about your codebase in natural language. The same capability powers automatic wiki generation — structured technical documentation generated from codebase analysis.',
        details: [],
      },
    ],
  },
  {
    name: 'Human-in-the-Loop',
    features: [
      {
        title: 'Interactive Plan Review',
        tier: null,
        body: 'Before workers start, the full execution plan is displayed. Approve immediately, or add/remove tasks, adjust file scope, and add free-text instructions for individual workers. Every modification is stored in the plan audit log.',
        details: [],
      },
      {
        title: 'Budget Gate',
        tier: null,
        body: 'Configure a cost ceiling per cycle. When estimated spend approaches the limit, execution pauses for confirmation. Shows tokens consumed, estimated remaining, cost at current provider rates, and which workers are running.',
        details: [],
      },
      {
        title: 'Post-Cycle Validation Dashboard',
        tier: 'Professional+',
        body: 'After workers complete, a validation run checks that acceptance criteria are met. Shows per-criterion pass/fail, which files changed for each criterion, per-file accept/reject controls, and gap task generation for unmet criteria.',
        details: [],
      },
      {
        title: 'PR Review Response Loop',
        tier: 'Professional+',
        body: 'After a worker\'s PR receives human code review comments, a fix worker is automatically dispatched — pre-loaded with the original task context, the diff, and the reviewer\'s comments.',
        details: [],
      },
    ],
  },
  {
    name: 'Security & Compliance',
    features: [
      {
        title: 'Security Pipeline',
        tier: null,
        body: 'Every worker diff is scanned by four parallel scanners before the branch is pushed.',
        details: [
          'Secrets detection: 40+ patterns — API keys, tokens, private keys, connection strings',
          'Dependency audit: known CVEs via OSV.dev for npm, cargo, pip, go.sum',
          'SAST: cargo-audit, semgrep common vulnerability patterns',
          'License compliance: SPDX identification, blocks problematic licenses in commercial contexts',
        ],
      },
      {
        title: 'HIPAA and FedRAMP Compliance Profiles',
        tier: 'Enterprise',
        body: 'Pre-configured compliance overlays enforcing relevant controls.',
        details: [
          'HIPAA: disables plaintext logging of PHI patterns, enforces secrets scanner, requires audit log retention, enforces TLS 1.2+',
          'FedRAMP: restricts provider list to US-hosted services, enforces FIPS-compliant cryptographic operations, requires continuous vulnerability scanning',
        ],
      },
    ],
  },
  {
    name: 'Enterprise',
    features: [
      {
        title: 'Team Collaboration & Seat Management',
        tier: 'Enterprise',
        body: 'Configure a roster of named AI developer agents, each with assigned AI provider, priority weight, and maximum concurrent manager assignments. Tracks active sessions, enforces license limits, and generates per-developer cost reports.',
        details: [],
      },
      {
        title: 'Cost Analytics & Reporting',
        tier: null,
        body: 'Pre-cycle cost estimates, actual token spend per provider per cycle, per-project cost history with bar chart visualization, CSV and JSON export for finance integration.',
        details: [],
      },
      {
        title: 'IDE Extensions',
        tier: 'Professional+',
        body: 'Native integrations for VS Code, JetBrains, and Neovim.',
        details: [
          'VS Code: live sidebar, plan approval UI, per-file accept/reject, worker terminal panels, cost status bar',
          'JetBrains: tool window, plan approval dialog, file status decorations, cycle status bar widget',
          'Neovim: Lua plugin with floating panel, Telescope picker, statusline integration, full command palette',
        ],
      },
      {
        title: 'Headless Service Mode',
        tier: 'Enterprise',
        body: 'mahalaxmi-service is a standalone REST+SSE API server for CI/CD integration.',
        details: [
          'POST /v1/cycles — start a cycle from CI/CD',
          'GET /v1/cycles/:id — poll cycle status',
          'GET /v1/events — SSE stream of cycle events',
          'Intake adapters: Jira, Slack, GitHub Issues, custom REST',
          'Output adapters: Jira comments, Slack threads, GitHub issue auto-close, HMAC-signed webhooks',
        ],
      },
    ],
  },
];

const platformRows = [
  ['macOS (Apple Silicon + Intel)', 'Supported'],
  ['Windows 10/11', 'Supported'],
  ['Linux (x86_64)', 'Supported'],
  ['Linux (ARM64)', 'Beta'],
];

const tierColors = {
  'Professional+': 'primary',
  'Enterprise': 'secondary',
};

export default async function MahalaxmiFeaturesPage({ params }) {
  const { locale } = await params;

  return (
    <Container maxWidth="lg" sx={{ py: { xs: 4, md: 8 } }}>
      <Breadcrumbs separator={<NavigateNext fontSize="small" />} sx={{ mb: 3 }}>
        <Link href="/" style={{ textDecoration: 'none', color: 'inherit' }}>Mahalaxmi</Link>
        <Typography color="text.primary">Features</Typography>
      </Breadcrumbs>

      <Box sx={{ mb: 6 }}>
        <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 2 }}>
          Features
        </Typography>
        <Typography variant="h6" color="text.secondary">
          Every feature exists to answer one question: how do we ship faster without breaking things?
        </Typography>
      </Box>

      {categories.map(({ name, features }) => (
        <Box key={name} sx={{ mb: 8 }}>
          <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 4, pb: 1, borderBottom: '3px solid', borderColor: 'primary.main' }}>
            {name}
          </Typography>
          <Grid container spacing={3}>
            {features.map(({ title, tier, body, details }) => (
              <Grid item xs={12} md={6} key={title}>
                <Card elevation={1} sx={{ height: '100%' }}>
                  <CardContent sx={{ p: 3 }}>
                    <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 1, mb: 1.5, flexWrap: 'wrap' }}>
                      <Typography variant="h6" sx={{ fontWeight: 600, flex: 1 }}>{title}</Typography>
                      {tier && (
                        <Chip label={tier} color={tierColors[tier] || 'default'} size="small" sx={{ flexShrink: 0 }} />
                      )}
                    </Box>
                    <Typography variant="body2" color="text.secondary" sx={{ mb: details.length ? 2 : 0 }}>
                      {body}
                    </Typography>
                    {details.length > 0 && (
                      <Box component="ul" sx={{ pl: 2, m: 0 }}>
                        {details.map((d) => (
                          <Box component="li" key={d} sx={{ mb: 0.5 }}>
                            <Typography variant="body2" color="text.secondary">{d}</Typography>
                          </Box>
                        ))}
                      </Box>
                    )}
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        </Box>
      ))}

      {/* Platform support */}
      <Box sx={{ mb: 8 }}>
        <Typography variant="h4" component="h2" sx={{ fontWeight: 700, mb: 4, pb: 1, borderBottom: '3px solid', borderColor: 'primary.main' }}>
          Platform Support
        </Typography>
        <TableContainer component={Paper} elevation={1} sx={{ maxWidth: 480 }}>
          <Table size="small">
            <TableHead>
              <TableRow sx={{ bgcolor: 'rgba(255,255,255,0.08)' }}>
                <TableCell sx={{ fontWeight: 600 }}>Platform</TableCell>
                <TableCell sx={{ fontWeight: 600 }}>Status</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {platformRows.map(([platform, status]) => (
                <TableRow key={platform}>
                  <TableCell>{platform}</TableCell>
                  <TableCell>
                    <Chip label={status} color={status === 'Supported' ? 'success' : 'warning'} size="small" variant="outlined" />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
          Distribution: GitHub Releases, WinGet, Chocolatey, Scoop, Homebrew, Flathub, AUR, MSIX
        </Typography>
      </Box>
    </Container>
  );
}

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';

export const metadata = {
  title: 'Architecture — Mahalaxmi Open Source',
  description: 'Deep-dive into the Mahalaxmi architecture: consensus engine, DAG task graph, PTY control, git worktree isolation, and provider routing.',
  alternates: {
    canonical: '/open-source/architecture',
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
  p: 2,
  borderRadius: 1,
  my: 2,
  overflowX: 'auto',
  whiteSpace: 'pre',
  fontSize: '0.85rem',
};

const dagDiagram = `
  Task A ──┐
           ├──► Task C ──► Task E (final)
  Task B ──┘         ▲
                     │
           Task D ───┘

  Independent tasks (A, B, D) execute in parallel.
  Task C waits for A and B to complete.
  Task E waits for C and D to complete.
`;

export default function ArchitecturePage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Mahalaxmi Architecture
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        An overview of the core systems that power Mahalaxmi&apos;s multi-agent orchestration platform.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      {/* Section 1: Manager-Worker Consensus Engine */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Manager-Worker Consensus Engine
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        The <Box component="span" sx={{ color: '#00C8C8', fontFamily: 'monospace' }}>Manager</Box> is
        the orchestration brain. It analyzes incoming requirements, decomposes them into discrete
        tasks, and distributes those tasks to autonomous <Box component="span" sx={{ color: '#00C8C8', fontFamily: 'monospace' }}>Worker</Box> agents.
        Each Worker executes independently inside an isolated environment and reports its results
        back to the Manager.
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        When multiple Workers produce overlapping or conflicting outputs, the consensus engine
        reconciles them using one of four configurable merge strategies:
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 2 }}>
        {[
          { name: 'Union', desc: 'Combines all non-duplicate results from every Worker. Best when you want maximum coverage.' },
          { name: 'Intersection', desc: 'Retains only results agreed upon by all Workers. Best when high confidence is required.' },
          { name: 'WeightedVoting', desc: 'Each Worker casts a weighted vote; the result with the highest aggregate weight wins.' },
          { name: 'ComplexityWeighted', desc: 'Tasks with higher complexity scores carry more influence in the final merge decision.' },
        ].map(({ name, desc }) => (
          <Typography key={name} component="li" variant="body1" color="text.secondary" sx={{ mb: 1 }}>
            <Box component="span" sx={{ color: '#00C8C8', fontFamily: 'monospace', fontWeight: 600 }}>{name}</Box>
            {' — '}{desc}
          </Typography>
        ))}
      </Box>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        For conflicts that no voting strategy can resolve cleanly, the Manager invokes
        LLM arbitration — sending the conflicting outputs to the configured language model and
        asking it to reason about which result is authoritative. Semantic deduplication using
        Jaccard similarity prevents near-duplicate outputs from inflating result sets before the
        merge strategy is applied.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* Section 2: DAG Task Graph */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        DAG Task Graph
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Every Mahalaxmi run is represented internally as a{' '}
        <Box component="span" sx={{ color: '#00C8C8' }}>Directed Acyclic Graph (DAG)</Box>.
        Each node in the graph is a discrete task; each directed edge encodes a dependency
        relationship — the target node may not begin execution until all of its source nodes
        have completed successfully.
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 1 }}>
        Workers observe the graph and immediately claim any node whose dependencies are fully
        resolved, enabling maximal parallelism without manual coordination:
      </Typography>
      <Box sx={codeBlockSx}>{dagDiagram}</Box>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        The Manager constructs the DAG from the requirement analysis phase and updates edge
        weights dynamically as Workers report completion or failure, allowing downstream tasks
        to be re-queued or retried automatically.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* Section 3: PTY Control */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        PTY Control
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Mahalaxmi controls AI CLI tools through native{' '}
        <Box component="span" sx={{ color: '#00C8C8' }}>pseudo-terminal (PTY)</Box> sessions —
        not screen capture or OCR. This means Mahalaxmi reads and writes directly to the
        terminal byte stream, giving it deterministic, low-latency control over any AI CLI tool
        that runs in a standard terminal.
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Because PTY control is protocol-agnostic, Mahalaxmi works with any provider whose CLI
        speaks to a terminal. Adding support for a new provider requires only a PTY adapter —
        no proprietary integration or API access is needed beyond what the CLI already provides.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* Section 4: Git Worktree Isolation */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Git Worktree Isolation
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Every Worker operates inside a dedicated{' '}
        <Box component="span" sx={{ color: '#00C8C8' }}>git worktree</Box> — a lightweight
        checkout of the repository at a unique branch. Workers can read, modify, and commit code
        without interfering with one another because each worktree is fully isolated at the
        filesystem level.
      </Typography>
      <Box sx={codeBlockSx}>{`  main branch
  │
  ├── worktree-worker-1  (branch: task/worker-1)
  ├── worktree-worker-2  (branch: task/worker-2)
  └── worktree-worker-N  (branch: task/worker-N)

  Completed worktrees → Pull Request → merge to main`}</Box>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        When a Worker finishes its task, the Manager opens a Pull Request from the Worker&apos;s
        branch. The PR is reviewed, merged, and the worktree is cleaned up. This workflow keeps
        the main branch stable while giving Workers the freedom to experiment and fail safely.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* Section 5: Provider Routing */}
      <Typography variant="h5" component="h2" sx={headingSx}>
        Provider Routing
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        The <Box component="span" sx={{ color: '#00C8C8', fontFamily: 'monospace' }}>ProviderRouter</Box> sits
        between the Manager and the underlying AI CLI tools. When a task is dispatched, the
        router selects the appropriate provider based on the session configuration. If the
        primary provider becomes unavailable, the router applies{' '}
        <Box component="span" sx={{ color: '#00C8C8' }}>fallback logic</Box> — automatically
        retrying on the next configured provider in priority order until the request succeeds or
        all options are exhausted.
      </Typography>
      <Box sx={codeBlockSx}>{`  Manager
    └──► ProviderRouter
           ├── Claude Code  (priority 1)
           ├── GitHub Copilot  (priority 2, fallback)
           ├── Grok  (priority 3, fallback)
           └── Ollama  (priority 4, local fallback)`}</Box>
      <Typography variant="body1" color="text.secondary">
        Providers are pluggable via the Provider Plugin SDK. Custom providers slot into the
        routing chain without any changes to the core orchestration logic.
      </Typography>
    </Container>
  );
}

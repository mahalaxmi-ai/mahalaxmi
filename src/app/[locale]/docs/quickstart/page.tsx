import type { Metadata } from 'next';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

export const metadata: Metadata = {
  title: 'Quickstart Guide — Mahalaxmi AI',
  description: 'Install and run your first Mahalaxmi AI orchestration cycle in minutes',
};

interface Step {
  number: number;
  title: string;
  description: string;
  command?: string;
  notes?: string[];
}

const steps: Step[] = [
  {
    number: 1,
    title: 'Install',
    description: 'Install the Mahalaxmi CLI globally via npm.',
    command: 'npm install -g @thrivetech/mahalaxmi',
    notes: ['Requires Node.js 18 or higher'],
  },
  {
    number: 2,
    title: 'Configure',
    description:
      'Run the init command in your project directory. This creates mahalaxmi.config.json and prompts you for your Claude API key or Ollama URL.',
    command: 'mahalaxmi init',
    notes: [
      'Provide your Claude API key — or point to Ollama running on localhost:11434',
      'Ensure your GitHub repo is configured so Workers can open PRs',
    ],
  },
  {
    number: 3,
    title: 'Run your first cycle',
    description:
      'Start the orchestration engine. The Manager reads your task queue and assigns tasks to Workers.',
    command: 'mahalaxmi start',
  },
  {
    number: 4,
    title: 'Review PRs',
    description:
      'Workers open GitHub pull requests with their proposed changes. Review each PR in GitHub and approve or reject it — nothing is merged without your sign-off.',
  },
];

function CodeBlock({ children }: { children: string }) {
  return (
    <Box
      component="pre"
      sx={{
        bgcolor: '#0d1117',
        color: '#e6edf3',
        fontFamily: 'monospace',
        fontSize: '0.9rem',
        borderRadius: 1,
        p: 2,
        mt: 1.5,
        mb: 1.5,
        overflowX: 'auto',
        border: '1px solid',
        borderColor: 'divider',
      }}
    >
      <code>{children}</code>
    </Box>
  );
}

export default function QuickstartPage() {
  return (
    <Container maxWidth="md" sx={{ py: 8 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Quickstart Guide
      </Typography>
      <Typography variant="h6" color="text.secondary" sx={{ mb: 4 }}>
        Install and run your first AI orchestration cycle in minutes.
      </Typography>

      <Box
        sx={{
          bgcolor: 'background.paper',
          border: '1px solid',
          borderColor: 'divider',
          borderRadius: 2,
          p: 3,
          mb: 6,
        }}
      >
        <Typography variant="subtitle1" fontWeight={600} gutterBottom>
          Prerequisites
        </Typography>
        <Box component="ul" sx={{ m: 0, pl: 3 }}>
          <Typography component="li" variant="body2" color="text.secondary" sx={{ mb: 0.5 }}>
            Node.js 18 or higher
          </Typography>
          <Typography component="li" variant="body2" color="text.secondary" sx={{ mb: 0.5 }}>
            Claude API key <em>or</em> Ollama running on <code>localhost:11434</code>
          </Typography>
          <Typography component="li" variant="body2" color="text.secondary">
            GitHub repository configured with push access
          </Typography>
        </Box>
      </Box>

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 0 }}>
        {steps.map((step, index) => (
          <Box key={step.number}>
            <Box sx={{ display: 'flex', gap: 3, py: 4 }}>
              <Box
                sx={{
                  flexShrink: 0,
                  width: 36,
                  height: 36,
                  borderRadius: '50%',
                  bgcolor: 'primary.main',
                  color: 'primary.contrastText',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  fontWeight: 700,
                  fontSize: '0.95rem',
                }}
              >
                {step.number}
              </Box>
              <Box sx={{ flex: 1 }}>
                <Typography variant="h6" fontWeight={600} gutterBottom>
                  {step.title}
                </Typography>
                <Typography variant="body1" color="text.secondary" sx={{ mb: step.command ? 0 : 1 }}>
                  {step.description}
                </Typography>
                {step.command && <CodeBlock>{step.command}</CodeBlock>}
                {step.notes && step.notes.length > 0 && (
                  <Box component="ul" sx={{ m: 0, pl: 3, mt: 1 }}>
                    {step.notes.map((note) => (
                      <Typography
                        key={note}
                        component="li"
                        variant="body2"
                        color="text.secondary"
                        sx={{ mb: 0.5 }}
                      >
                        {note}
                      </Typography>
                    ))}
                  </Box>
                )}
              </Box>
            </Box>
            {index < steps.length - 1 && <Divider />}
          </Box>
        ))}
      </Box>
    </Container>
  );
}

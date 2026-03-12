'use client';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Divider from '@mui/material/Divider';
import Link from 'next/link';

const codeBlockSx = {
  fontFamily: 'monospace',
  backgroundColor: '#0D1117',
  color: '#00C8C8',
  p: 2,
  borderRadius: 1,
  my: 1,
};


export default function QuickstartPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Quickstart
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        Get up and running with Mahalaxmi in minutes.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        Prerequisites
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 4 }}>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          Node.js 18 or higher
        </Typography>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          npm (included with Node.js)
        </Typography>
        <Typography component="li" variant="body1">
          An AI provider account (e.g., Anthropic Claude, OpenAI)
        </Typography>
      </Box>

      <Divider sx={{ mb: 4 }} />

      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        Installation
      </Typography>

      <Typography variant="h6" fontWeight={600} sx={{ mt: 3, mb: 0.5 }}>
        1. Install the Mahalaxmi CLI
      </Typography>
      <Box sx={codeBlockSx}>
        npm install -g @mahalaxmi/cli
      </Box>

      <Typography variant="h6" fontWeight={600} sx={{ mt: 3, mb: 0.5 }}>
        2. Configure your AI provider
      </Typography>
      <Box sx={codeBlockSx}>
        mahalaxmi configure --provider claude-code --key &lt;api-key&gt;
      </Box>

      <Typography variant="h6" fontWeight={600} sx={{ mt: 3, mb: 0.5 }}>
        3. Run Mahalaxmi
      </Typography>
      <Box sx={codeBlockSx}>
        mahalaxmi run
      </Box>

      <Divider sx={{ my: 4 }} />

      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        What Happens Next
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        After running <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>mahalaxmi run</Box>,
        the orchestration engine starts and connects to your configured AI provider.
        Workers are assigned tasks and begin executing in parallel. You can monitor
        progress, review outputs, and manage your terminal sessions from the dashboard.
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        Explore the next steps below to integrate Mahalaxmi into your existing workflow.
      </Typography>

      <Box sx={{ display: 'flex', gap: 3, flexWrap: 'wrap' }}>
        <Box
          component={Link}
          href="/docs/vscode"
          sx={{
            display: 'block',
            p: 3,
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 2,
            textDecoration: 'none',
            color: 'inherit',
            flex: '1 1 200px',
            '&:hover': { borderColor: '#00C8C8' },
          }}
        >
          <Typography variant="h6" fontWeight={600} gutterBottom>
            VS Code Extension
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Install and configure the Mahalaxmi VS Code extension to orchestrate AI workers directly from your editor.
          </Typography>
        </Box>

        <Box
          component={Link}
          href="/docs/cloud"
          sx={{
            display: 'block',
            p: 3,
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 2,
            textDecoration: 'none',
            color: 'inherit',
            flex: '1 1 200px',
            '&:hover': { borderColor: '#00C8C8' },
          }}
        >
          <Typography variant="h6" fontWeight={600} gutterBottom>
            Cloud Deployment
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Deploy Mahalaxmi to the cloud and scale your AI orchestration across distributed infrastructure.
          </Typography>
        </Box>
      </Box>
    </Container>
  );
}

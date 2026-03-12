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
  overflowX: 'auto' as const,
};


export default function VsCodePage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        VS Code Extension
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        The Mahalaxmi VS Code extension lets you review and accept AI-proposed file changes directly
        in your editor, connected to your cloud server.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      {/* Install */}
      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        Install from VS Code Marketplace
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        The extension is not yet published to the VS Code Marketplace. Once published, you will be
        able to install it by searching for <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>Mahalaxmi</Box> in
        the Extensions panel (<Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>Ctrl+Shift+X</Box>).
        In the meantime, follow the{' '}
        <Box
          component={Link}
          href="#"
          sx={{ color: '#00C8C8', textDecoration: 'underline' }}
        >
          manual install instructions
        </Box>{' '}
        provided by your account team.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* Connect to cloud server */}
      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        Connect to a Cloud Server
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        After subscribing on{' '}
        <Box component={Link} href="/cloud/pricing" sx={{ color: '#00C8C8' }}>
          /cloud/pricing
        </Box>
        , follow these steps to open your server in VS Code:
      </Typography>
      <Box component="ol" sx={{ pl: 3, mb: 3 }}>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          Go to{' '}
          <Box component={Link} href="/dashboard/servers" sx={{ color: '#00C8C8' }}>
            /dashboard/servers
          </Box>
          .
        </Typography>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          Find your server and click <strong>Open in VS Code</strong>.
        </Typography>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          VS Code launches and opens pre-configured — no manual connection setup required.
        </Typography>
      </Box>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        The <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>vscode://</Box> deep
        link passes your <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>api_key</Box> and
        server address to the extension, which establishes the connection automatically.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* Plan approval */}
      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        Reviewing AI-Proposed Changes
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        When Mahalaxmi proposes a plan — a set of file changes to accomplish a task — a summary
        appears in the <strong>Mahalaxmi</strong> panel in the VS Code sidebar (Activity Bar).
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 3 }}>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          Open the <strong>Mahalaxmi</strong> sidebar panel to see pending proposals.
        </Typography>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          Each proposal lists the files it intends to create, modify, or delete.
        </Typography>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          Click a file name to open a diff view showing exactly what will change.
        </Typography>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          Approve the entire plan with <strong>Accept All</strong> or review files individually.
        </Typography>
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* File acceptance */}
      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        Accepting or Rejecting Individual File Changes
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        You can accept or reject each file change independently before the pull request is opened:
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 3 }}>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          <strong>Accept</strong> — applies the change and marks the file as approved.
        </Typography>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          <strong>Reject</strong> — discards the proposed change for that file; the remaining accepted
          files are still included in the PR.
        </Typography>
        <Typography component="li" variant="body1" sx={{ mb: 1 }}>
          Once you have reviewed all files, click <strong>Open PR</strong> to create the pull request
          with only the accepted changes.
        </Typography>
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* Troubleshooting */}
      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        Troubleshooting
      </Typography>
      <Box sx={{ mb: 2 }}>
        <Typography variant="h6" fontWeight={600} gutterBottom>
          &ldquo;Open in VS Code&rdquo; does nothing
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Ensure the Mahalaxmi VS Code extension is installed. Without it, your browser cannot hand
          off the <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>vscode://</Box> deep
          link to VS Code. Install the extension, then click <strong>Open in VS Code</strong> again.
        </Typography>
      </Box>
      <Box sx={{ mb: 2 }}>
        <Typography variant="h6" fontWeight={600} gutterBottom>
          VS Code opens but does not connect
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Check that your server status is <strong>active</strong> on{' '}
          <Box component={Link} href="/dashboard/servers" sx={{ color: '#00C8C8' }}>
            /dashboard/servers
          </Box>
          . If the status shows <strong>Degraded</strong> or <strong>Failed</strong>, see the{' '}
          <Box component={Link} href="/docs/cloud#troubleshooting" sx={{ color: '#00C8C8' }}>
            Cloud Server Setup troubleshooting
          </Box>{' '}
          section.
        </Typography>
      </Box>
    </Container>
  );
}

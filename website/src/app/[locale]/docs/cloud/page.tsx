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

export const metadata = {
  title: 'Cloud Server Setup | Mahalaxmi Docs',
  description: 'Provision a Mahalaxmi cloud server, configure your project, and connect VS Code in four steps.',
};

export default function CloudDocsPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Cloud Server Setup
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        Provision a dedicated cloud server, configure your project, and open it in VS Code in
        four steps.
      </Typography>

      <Divider sx={{ mb: 4 }} />

      {/* Setup steps */}
      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        Setup Steps
      </Typography>

      <Typography variant="h6" fontWeight={600} sx={{ mt: 3, mb: 0.5 }}>
        1. Subscribe
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Go to{' '}
        <Box component={Link} href="/cloud/pricing" sx={{ color: '#00C8C8' }}>
          /cloud/pricing
        </Box>
        , select a tier, and complete checkout. Stripe processes the payment and activates your
        subscription immediately.
      </Typography>

      <Typography variant="h6" fontWeight={600} sx={{ mt: 3, mb: 0.5 }}>
        2. Wait for provisioning
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        After checkout, your new server appears on{' '}
        <Box component={Link} href="/dashboard/servers" sx={{ color: '#00C8C8' }}>
          /dashboard/servers
        </Box>{' '}
        with status <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>Provisioning</Box>.
        Provisioning typically takes 2–5 minutes. The page updates automatically when provisioning
        completes and the server reaches <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>active</Box> status.
      </Typography>

      <Typography variant="h6" fontWeight={600} sx={{ mt: 3, mb: 0.5 }}>
        3. Configure project name
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 1 }}>
        Click the <strong>Configure</strong> button on your server card. Enter a URL-safe project
        name that:
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 2 }}>
        <Typography component="li" variant="body1" sx={{ mb: 0.5 }}>
          Is between 3 and 40 characters long
        </Typography>
        <Typography component="li" variant="body1" sx={{ mb: 0.5 }}>
          Contains only lowercase letters, numbers, and hyphens (no spaces or special characters)
        </Typography>
      </Box>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        The project name becomes part of the server&rsquo;s URL and cannot be changed after it is set.
      </Typography>

      <Typography variant="h6" fontWeight={600} sx={{ mt: 3, mb: 0.5 }}>
        4. Open in VS Code
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        Click <strong>Open in VS Code</strong> on the server card. VS Code opens with the server
        pre-configured — no manual connection setup required. See{' '}
        <Box component={Link} href="/docs/vscode" sx={{ color: '#00C8C8' }}>
          VS Code Extension
        </Box>{' '}
        for details on reviewing AI-proposed changes once connected.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* Deep link explanation */}
      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        How the Deep Link Works
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        When you click <strong>Open in VS Code</strong>, the dashboard fetches a pre-built deep link
        from the server and redirects your browser to a{' '}
        <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>vscode://</Box>{' '}
        URL. That URL carries your{' '}
        <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>api_key</Box> and
        server address. VS Code intercepts the URL, passes the parameters to the Mahalaxmi
        extension, and the extension establishes the connection automatically — no copy-pasting of
        keys or host names needed.
      </Typography>
      <Box sx={codeBlockSx}>
        vscode://mahalaxmi.mahalaxmi/connect?api_key=&lt;key&gt;&amp;host=&lt;server-host&gt;
      </Box>
      <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
        The deep link is generated server-side and is single-use. Do not share it.
      </Typography>

      <Divider sx={{ my: 4 }} />

      {/* Server lifecycle */}
      <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
        Server Lifecycle
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
        A server passes through the following statuses during its lifetime:
      </Typography>
      <Box component="ul" sx={{ pl: 3, mb: 3 }}>
        {[
          { status: 'pending_payment', desc: 'Waiting for payment confirmation from Stripe.' },
          { status: 'provisioning', desc: 'Infrastructure is being allocated (2–5 minutes).' },
          { status: 'active', desc: 'Server is running and ready to use.' },
          { status: 'degraded', desc: 'Server is running but health checks are failing.' },
          { status: 'stopping', desc: 'Server is shutting down.' },
          { status: 'stopped', desc: 'Server is stopped; no charges accrue.' },
          { status: 'deleting', desc: 'Deletion is in progress.' },
          { status: 'deleted', desc: 'Server has been permanently removed.' },
          { status: 'failed', desc: 'Provisioning or recovery failed unrecoverably.' },
        ].map(({ status, desc }) => (
          <Typography key={status} component="li" variant="body1" sx={{ mb: 1 }}>
            <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>{status}</Box>{' '}
            — {desc}
          </Typography>
        ))}
      </Box>

      <Divider sx={{ my: 4 }} />

      {/* Troubleshooting */}
      <Typography variant="h5" component="h2" fontWeight={600} id="troubleshooting" gutterBottom>
        Troubleshooting
      </Typography>

      <Box sx={{ mb: 3 }}>
        <Typography variant="h6" fontWeight={600} gutterBottom>
          Status shows &ldquo;Degraded&rdquo;
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Your server is running but health checks are failing — it is operational but unhealthy.
          Try reloading the dashboard after a minute. If the status does not return to{' '}
          <Box component="span" sx={{ fontFamily: 'monospace', color: '#00C8C8' }}>active</Box>,
          contact support at{' '}
          <Box
            component="a"
            href="mailto:support@mahalaxmi.ai"
            sx={{ color: '#00C8C8', textDecoration: 'underline' }}
          >
            support@mahalaxmi.ai
          </Box>{' '}
          and include your server ID.
        </Typography>
      </Box>

      <Box sx={{ mb: 3 }}>
        <Typography variant="h6" fontWeight={600} gutterBottom>
          Status shows &ldquo;Failed&rdquo;
        </Typography>
        <Typography variant="body1" color="text.secondary">
          The server encountered an unrecoverable error and cannot be restarted automatically.
          Contact{' '}
          <Box
            component="a"
            href="mailto:support@mahalaxmi.ai"
            sx={{ color: '#00C8C8', textDecoration: 'underline' }}
          >
            support@mahalaxmi.ai
          </Box>{' '}
          with your server ID. The support team will diagnose the failure and, if possible, provision
          a replacement server on the same subscription.
        </Typography>
      </Box>

      <Box sx={{ mb: 3 }}>
        <Typography variant="h6" fontWeight={600} gutterBottom>
          &ldquo;Open in VS Code&rdquo; does nothing
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Ensure the Mahalaxmi VS Code extension is installed. See{' '}
          <Box component={Link} href="/docs/vscode" sx={{ color: '#00C8C8' }}>
            VS Code Extension
          </Box>{' '}
          for installation instructions.
        </Typography>
      </Box>
    </Container>
  );
}

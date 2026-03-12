import type { Metadata } from 'next';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Divider from '@mui/material/Divider';
import Alert from '@mui/material/Alert';

export const metadata: Metadata = {
  title: 'API Reference | Mahalaxmi Docs',
  description: 'REST API reference for the Mahalaxmi AI terminal orchestration platform.',
  alternates: {
    canonical: 'https://mahalaxmi.ai/docs/api',
  },
};

const codeStyle = {
  display: 'block',
  backgroundColor: '#0d0d0d',
  border: '1px solid #2a2a2a',
  borderRadius: 1,
  p: 2,
  overflowX: 'auto',
  fontFamily: 'monospace',
  fontSize: '0.85rem',
  color: '#e0e0e0',
  whiteSpace: 'pre' as const,
};

const endpoints = [
  {
    method: 'GET',
    path: '/api/mahalaxmi/servers',
    description: 'List all cloud servers belonging to the authenticated user.',
    auth: true,
    responseExample: `[
  {
    "id": "srv_01hxyz",
    "name": "my-dev-box",
    "status": "active",
    "region": "us-east-1",
    "created_at": "2025-10-01T12:00:00Z"
  }
]`,
    notes: 'Returns a bare array. The api_key field is not included in list responses.',
  },
  {
    method: 'GET',
    path: '/api/mahalaxmi/servers/:id',
    description: 'Get full details for a single server, including the api_key used by VS Code.',
    auth: true,
    responseExample: `{
  "id": "srv_01hxyz",
  "name": "my-dev-box",
  "status": "active",
  "region": "us-east-1",
  "api_key": "mhx_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "ip_address": "1.2.3.4",
  "created_at": "2025-10-01T12:00:00Z"
}`,
    notes: 'The api_key has the prefix mhx_ and is used by the VS Code extension to authenticate to the cloud agent.',
  },
  {
    method: 'GET',
    path: '/api/mahalaxmi/servers/:id/vscode-config',
    description: 'Get a pre-built VS Code deep link and config JSON for connecting directly to the server.',
    auth: true,
    responseExample: `{
  "deep_link": "vscode://mahalaxmi.mahalaxmi-vscode/connect?api_key=mhx_xxx&host=1.2.3.4",
  "config_json": {
    "api_key": "mhx_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "host": "1.2.3.4",
    "port": 4025
  }
}`,
    notes: 'The deep link is assembled server-side. Never construct or expose it client-side.',
  },
  {
    method: 'GET',
    path: '/api/products',
    description: 'Retrieve the full product catalog. This endpoint is public and does not require authentication.',
    auth: false,
    responseExample: `[
  {
    "id": "prod_01",
    "name": "Starter",
    "slug": "starter",
    "price_monthly": 1900,
    "price_yearly": 19000
  }
]`,
    notes: 'Prices are in cents (USD). No authentication cookie required.',
  },
  {
    method: 'GET',
    path: '/api/products/:slug',
    description: 'Retrieve details for a single product by its slug.',
    auth: false,
    responseExample: `{
  "id": "prod_01",
  "name": "Starter",
  "slug": "starter",
  "description": "Entry-level plan for individual developers.",
  "price_monthly": 1900,
  "price_yearly": 19000,
  "features": ["1 cloud server", "Claude Code included", "Community support"]
}`,
    notes: 'Returns 404 if the slug does not match any product.',
  },
  {
    method: 'GET',
    path: '/api/categories',
    description: 'Retrieve all product categories. Public endpoint, no authentication required.',
    auth: false,
    responseExample: `[
  {
    "id": "cat_01",
    "name": "Individual",
    "slug": "individual"
  },
  {
    "id": "cat_02",
    "name": "Team",
    "slug": "team"
  }
]`,
    notes: null,
  },
];

export default function ApiReferencePage() {
  return (
    <Box sx={{ maxWidth: 900, mx: 'auto', px: { xs: 2, md: 4 }, py: 6 }}>
      <Typography variant="h3" component="h1" fontWeight={700} gutterBottom>
        API Reference
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        This reference documents the REST API routes proxied by the Mahalaxmi website. All
        server-management endpoints forward requests to the Mahalaxmi backend using
        server-side credentials — PAK keys and JWTs are never exposed to the browser.
      </Typography>

      <Box sx={{ mb: 5 }}>
        <Typography variant="h5" component="h2" fontWeight={600} gutterBottom>
          Authentication
        </Typography>
        <Alert severity="info" sx={{ mb: 2 }}>
          <strong>Server-side only:</strong> PAK keys and JWTs are handled entirely on the
          server. They are never sent to or accessible from the browser. Do not attempt to
          read or relay these credentials from client-side code.
        </Alert>
        <Typography variant="body1" sx={{ mb: 2 }}>
          All authenticated API calls require the <code>mahalaxmi_token</code> httpOnly cookie
          to be present in the request. This cookie is set automatically when you log in via{' '}
          <code>/api/auth/login</code> and is sent by the browser on every same-origin request.
        </Typography>
        <Typography variant="body2" color="text.secondary">
          If the cookie is missing or expired the API returns <code>401 Unauthorized</code>.
          Redirect the user to <code>/login</code> to re-authenticate.
        </Typography>
      </Box>

      <Divider sx={{ mb: 5 }} />

      <Typography variant="h5" component="h2" fontWeight={600} sx={{ mb: 3 }}>
        Endpoints
      </Typography>

      {endpoints.map((ep) => (
        <Box
          key={ep.path}
          sx={{
            mb: 5,
            pb: 4,
            borderBottom: '1px solid',
            borderColor: 'divider',
            '&:last-child': { borderBottom: 'none' },
          }}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 1 }}>
            <Typography
              component="span"
              sx={{
                backgroundColor: 'primary.main',
                color: 'primary.contrastText',
                px: 1.5,
                py: 0.25,
                borderRadius: 1,
                fontFamily: 'monospace',
                fontWeight: 700,
                fontSize: '0.8rem',
              }}
            >
              {ep.method}
            </Typography>
            <Typography
              component="span"
              sx={{ fontFamily: 'monospace', fontWeight: 600, fontSize: '1rem' }}
            >
              {ep.path}
            </Typography>
            {ep.auth ? (
              <Typography
                component="span"
                sx={{
                  ml: 'auto',
                  fontSize: '0.75rem',
                  color: 'warning.main',
                  border: '1px solid',
                  borderColor: 'warning.main',
                  px: 1,
                  py: 0.25,
                  borderRadius: 1,
                }}
              >
                Auth required
              </Typography>
            ) : (
              <Typography
                component="span"
                sx={{
                  ml: 'auto',
                  fontSize: '0.75rem',
                  color: 'success.main',
                  border: '1px solid',
                  borderColor: 'success.main',
                  px: 1,
                  py: 0.25,
                  borderRadius: 1,
                }}
              >
                Public
              </Typography>
            )}
          </Box>

          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            {ep.description}
          </Typography>

          {ep.notes && (
            <Typography
              variant="body2"
              sx={{ mb: 2, color: 'text.secondary', fontStyle: 'italic' }}
            >
              {ep.notes}
            </Typography>
          )}

          <Typography variant="overline" display="block" sx={{ mb: 0.5 }}>
            Example Response
          </Typography>
          <Box component="pre" sx={codeStyle}>
            {ep.responseExample}
          </Box>
        </Box>
      ))}

      <Divider sx={{ mb: 4 }} />

      <Alert severity="warning">
        <strong>Security note:</strong> The <code>api_key</code> field (prefix{' '}
        <code>mhx_</code>) and all JWT tokens are handled exclusively on the server. They
        are never included in client-facing responses or accessible via browser developer
        tools. If you believe a key has been compromised, contact{' '}
        <strong>support@mahalaxmi.ai</strong> immediately.
      </Alert>
    </Box>
  );
}

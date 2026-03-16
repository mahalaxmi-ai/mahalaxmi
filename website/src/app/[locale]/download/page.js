export const dynamic = 'force-dynamic';

import {
  Box, Button, Container, Divider, Typography,
} from '@mui/material';
import { Download } from '@mui/icons-material';
import Link from 'next/link';
import { getDownloads } from '@/lib/productApi';

export async function generateMetadata() {
  return { title: 'Download Mahalaxmi — Terminal Orchestration' };
}

export default async function DownloadPage() {
  let downloads = {};
  try {
    downloads = await getDownloads();
  } catch {
    // Platform unavailable — show contact fallback
  }

  const { desktop_app, vscode_extension, latest_version } = downloads;
  const hasDesktop = desktop_app && Object.keys(desktop_app).length > 0;
  const hasVSCode = !!vscode_extension?.url;
  const hasAny = hasDesktop || hasVSCode;

  return (
    <Container maxWidth="md" sx={{ py: { xs: 4, md: 8 } }}>
      <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 1 }}>
        Download Mahalaxmi{latest_version ? ` ${latest_version}` : ''}
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 6 }}>
        Cross-platform AI terminal orchestration. BYOK — works with Claude Code, OpenAI, Bedrock, Gemini, and more.
      </Typography>

      {!hasAny && (
        <Box sx={{ py: 6, textAlign: 'center', border: '1px dashed', borderColor: 'divider', borderRadius: 2 }}>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
            Downloads temporarily unavailable. Please try again shortly or contact support.
          </Typography>
          <Button component={Link} href="mailto:support@mahalaxmi.ai" variant="outlined">
            Contact support
          </Button>
        </Box>
      )}

      {hasDesktop && (
        <Box sx={{ mb: 6 }}>
          <Typography variant="h5" sx={{ fontWeight: 700, mb: 3 }}>Desktop App</Typography>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {desktop_app.windows_exe && (
              <Button
                component="a"
                href={desktop_app.windows_exe.url}
                download
                variant="contained"
                size="large"
                startIcon={<Download />}
                sx={{ justifyContent: 'flex-start', maxWidth: 420 }}
              >
                Windows — {desktop_app.windows_exe.filename}
              </Button>
            )}
            {desktop_app.macos_dmg && (
              <Button
                component="a"
                href={desktop_app.macos_dmg.url}
                download
                variant="outlined"
                size="large"
                startIcon={<Download />}
                sx={{ justifyContent: 'flex-start', maxWidth: 420 }}
              >
                macOS — {desktop_app.macos_dmg.filename}
              </Button>
            )}
            {desktop_app.linux_deb && (
              <Button
                component="a"
                href={desktop_app.linux_deb.url}
                download
                variant="outlined"
                size="large"
                startIcon={<Download />}
                sx={{ justifyContent: 'flex-start', maxWidth: 420 }}
              >
                Linux (Debian/Ubuntu) — {desktop_app.linux_deb.filename}
              </Button>
            )}
            {desktop_app.linux_appimage && (
              <Button
                component="a"
                href={desktop_app.linux_appimage.url}
                download
                variant="outlined"
                size="large"
                startIcon={<Download />}
                sx={{ justifyContent: 'flex-start', maxWidth: 420 }}
              >
                Linux (AppImage) — {desktop_app.linux_appimage.filename}
              </Button>
            )}
          </Box>
        </Box>
      )}

      {hasDesktop && hasVSCode && <Divider sx={{ mb: 6 }} />}

      {hasVSCode && (
        <Box sx={{ mb: 6 }}>
          <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>VS Code Extension</Typography>
          {vscode_extension.version && (
            <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
              v{vscode_extension.version}
            </Typography>
          )}
          <Button
            component="a"
            href={vscode_extension.url}
            download
            variant="outlined"
            size="large"
            startIcon={<Download />}
            sx={{ justifyContent: 'flex-start', maxWidth: 420, mb: 2 }}
          >
            {vscode_extension.filename}
          </Button>
          {vscode_extension.install_instructions && (
            <Typography variant="body2" color="text.secondary">
              {vscode_extension.install_instructions}
            </Typography>
          )}
        </Box>
      )}
    </Container>
  );
}

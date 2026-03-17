export const dynamic = 'force-dynamic';

import {
  Box, Button, Container, Divider, Typography,
} from '@mui/material';
import { Download } from '@mui/icons-material';
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
  const da = desktop_app ?? {};

  return (
    <Container maxWidth="md" sx={{ py: { xs: 4, md: 8 } }}>
      <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 1 }}>
        Download Mahalaxmi{latest_version ? ` ${latest_version}` : ''}
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 6 }}>
        Cross-platform AI terminal orchestration. BYOK — works with Claude Code, OpenAI, Bedrock, Gemini, and more.
      </Typography>

      <Box sx={{ mb: 6 }}>
        <Typography variant="h5" sx={{ fontWeight: 700, mb: 3 }}>Desktop App</Typography>
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          {da.windows_exe ? (
            <Button
              component="a"
              href={da.windows_exe.url}
              download
              variant="contained"
              size="large"
              startIcon={<Download />}
              sx={{ justifyContent: 'flex-start', maxWidth: 420 }}
            >
              Windows — {da.windows_exe.filename}
            </Button>
          ) : (
            <Typography variant="body2" color="text.secondary" sx={{ fontStyle: 'italic' }}>
              Windows — Coming soon
            </Typography>
          )}
          {da.macos_dmg ? (
            <Button
              component="a"
              href={da.macos_dmg.url}
              download
              variant="outlined"
              size="large"
              startIcon={<Download />}
              sx={{ justifyContent: 'flex-start', maxWidth: 420 }}
            >
              macOS — {da.macos_dmg.filename}
            </Button>
          ) : (
            <Typography variant="body2" color="text.secondary" sx={{ fontStyle: 'italic' }}>
              macOS — Coming soon
            </Typography>
          )}
          {da.linux_deb ? (
            <Button
              component="a"
              href={da.linux_deb.url}
              download
              variant="outlined"
              size="large"
              startIcon={<Download />}
              sx={{ justifyContent: 'flex-start', maxWidth: 420 }}
            >
              Linux (Debian/Ubuntu) — {da.linux_deb.filename}
            </Button>
          ) : (
            <Typography variant="body2" color="text.secondary" sx={{ fontStyle: 'italic' }}>
              Linux — Coming soon
            </Typography>
          )}
          {da.linux_appimage && (
            <Button
              component="a"
              href={da.linux_appimage.url}
              download
              variant="outlined"
              size="large"
              startIcon={<Download />}
              sx={{ justifyContent: 'flex-start', maxWidth: 420 }}
            >
              Linux (AppImage) — {da.linux_appimage.filename}
            </Button>
          )}
        </Box>
      </Box>

      <Divider sx={{ mb: 6 }} />

      <Box sx={{ mb: 6 }}>
        <Typography variant="h5" sx={{ fontWeight: 700, mb: 1 }}>VS Code Extension</Typography>
        {vscode_extension?.url ? (
          <>
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
          </>
        ) : (
          <Typography variant="body2" color="text.secondary" sx={{ fontStyle: 'italic' }}>
            VS Code Extension — Coming soon
          </Typography>
        )}
      </Box>
    </Container>
  );
}

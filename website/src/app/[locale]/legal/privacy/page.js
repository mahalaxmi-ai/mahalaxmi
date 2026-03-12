'use client';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Container,
  Typography,
  Box,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Link as MuiLink,
} from '@mui/material';


const sectionHeadingSx = {
  mt: 5,
  mb: 1.5,
  pb: 0.5,
  borderBottom: '1px solid',
  borderColor: 'divider',
};

const highlightBoxSx = {
  bgcolor: '#ebf8ff',
  borderInlineStart: '4px solid',
  borderColor: 'primary.main',
  p: 2.5,
  my: 2.5,
  borderRadius: '0 6px 6px 0',
};

function DataTable({ columns, rows }) {
  return (
    <TableContainer component={Paper} variant="outlined" sx={{ my: 2 }}>
      <Table size="small">
        <TableHead>
          <TableRow>
            {columns.map((col) => (
              <TableCell key={col} sx={{ fontWeight: 600, bgcolor: 'grey.50' }}>
                {col}
              </TableCell>
            ))}
          </TableRow>
        </TableHead>
        <TableBody>
          {rows.map((row, i) => (
            <TableRow key={i}>
              {row.map((cell, j) => (
                <TableCell key={j} sx={{ verticalAlign: 'top' }}>
                  {cell}
                </TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

export default async function MahalaxmiPrivacyPage({ params }) {
  const { locale } = await params;

  return (
    <Container maxWidth="md" sx={{ py: { xs: 4, md: 6 } }}>
      <Paper elevation={1} sx={{ p: { xs: 3, md: 6 } }}>
        {/* Header */}
        <Box sx={{ borderBottom: '3px solid', borderColor: 'primary.main', pb: 3, mb: 5 }}>
          <Typography variant="h3" component="h1" sx={{ fontWeight: 700, mb: 1 }}>
            Privacy Policy
          </Typography>
          <Typography variant="subtitle1" color="text.secondary">
            Mahalaxmi Terminal Automation by ThriveTech Services LLC
          </Typography>
          <Typography variant="body2" color="text.disabled" sx={{ mt: 0.5 }}>
            Effective Date: February 22, 2026 &nbsp;|&nbsp; Last Updated: February 22, 2026
          </Typography>
        </Box>

        <Typography variant="body1" paragraph>
          ThriveTech Services LLC (&ldquo;ThriveTech,&rdquo; &ldquo;we,&rdquo; &ldquo;us,&rdquo; or &ldquo;our&rdquo;) operates Mahalaxmi, an AI terminal orchestration application for software developers. This Privacy Policy describes how we collect, use, store, and protect your information when you use the Mahalaxmi desktop application (the &ldquo;App&rdquo;) and related services.
        </Typography>

        <Typography variant="body1" paragraph>
          We are committed to protecting your privacy. Mahalaxmi is designed as a privacy-first developer tool &mdash; your source code never leaves your machine through our systems, we collect the minimum data necessary to operate the service, and we do not sell your information to anyone.
        </Typography>

        <Box sx={highlightBoxSx}>
          <Typography variant="body1">
            <strong>Key Privacy Commitments:</strong><br />
            Your source code is processed locally on your machine. ThriveTech servers never receive your code, project files, or AI conversation content. We do not collect analytics, telemetry, or usage tracking data. We do not display advertisements or share data with advertisers.
          </Typography>
        </Box>

        {/* Section 1 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          1. Information We Collect
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          1.1 Information You Provide Directly
        </Typography>

        <DataTable
          columns={['Data Type', 'What We Collect', 'Why']}
          rows={[
            ['Account Information', 'Email address, name', 'License activation and customer support'],
            ['License Key', 'Product license key', 'Verifying your subscription is valid'],
            [
              'Payment Information',
              'Processed by third-party payment providers (Microsoft Store, Apple App Store, or Stripe). We do not store credit card numbers or payment details.',
              'Processing your subscription',
            ],
          ]}
        />

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          1.2 Information Collected Automatically
        </Typography>

        <DataTable
          columns={['Data Type', 'What We Collect', 'Why']}
          rows={[
            [
              'Machine Fingerprint',
              'A one-way hash derived from your hardware identifiers (CPU, OS, hostname). The raw identifiers are never transmitted \u2014 only the irreversible hash.',
              'Binding your license to your device to prevent unauthorized sharing',
            ],
            ['App Version', 'The version of Mahalaxmi you are running', 'Ensuring compatibility during license validation'],
            ['Platform', 'Operating system (Windows, macOS, Linux) and architecture', 'Delivering correct updates and troubleshooting'],
          ]}
        />

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          1.3 Information We Do NOT Collect
        </Typography>
        <Typography variant="body1" paragraph>
          Mahalaxmi does <strong>not</strong> collect, transmit, or store any of the following:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li>Your source code, project files, or repository contents</li>
          <li>AI conversation content, prompts, or responses</li>
          <li>AI provider API keys or credentials (stored locally in your operating system&rsquo;s secure keychain)</li>
          <li>File contents from your codebase</li>
          <li>Usage analytics, feature tracking, or behavioral data</li>
          <li>Location data (GPS, IP-based geolocation)</li>
          <li>Contacts, photos, or other personal files</li>
          <li>Browsing history or activity on other applications</li>
          <li>Advertising identifiers</li>
        </Box>

        {/* Section 2 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          2. How We Use Your Information
        </Typography>

        <Typography variant="body1" paragraph>
          We use the limited information we collect for the following purposes only:
        </Typography>

        <DataTable
          columns={['Purpose', 'Data Used', 'Legal Basis']}
          rows={[
            ['License Validation', 'License key, machine fingerprint, app version, platform', 'Performance of contract'],
            ['Customer Support', 'Email address, name, app version, platform', 'Performance of contract / Legitimate interest'],
            ['Software Updates', 'App version, platform', 'Performance of contract'],
            ['Service Communications', 'Email address', 'Performance of contract'],
          ]}
        />

        <Typography variant="body1" paragraph>
          We do not use your information for advertising, profiling, automated decision-making, or any purpose other than those listed above.
        </Typography>

        {/* Section 3 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          3. How Your AI Providers Work
        </Typography>

        <Typography variant="body1" paragraph>
          Mahalaxmi orchestrates AI coding tools (such as Claude Code, Codex CLI, Gemini CLI, and others) that run locally on your machine. It is important to understand how data flows:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li><strong>Your code and prompts are sent directly from your machine to the AI provider.</strong> ThriveTech does not proxy, intercept, or store this communication.</li>
          <li>Each AI provider has its own privacy policy governing how they handle your data. We encourage you to review them.</li>
          <li>When you configure AI provider API keys in Mahalaxmi, those keys are stored in your operating system&rsquo;s secure credential storage (macOS Keychain, Windows Credential Manager, or Linux Secret Service). They are never transmitted to ThriveTech.</li>
          <li>For users who configure local AI providers (such as Ollama), all processing occurs entirely on your machine with no external communication.</li>
        </Box>

        {/* Section 4 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          4. Network Communications
        </Typography>

        <Typography variant="body1" paragraph>
          Mahalaxmi makes the following outbound network connections:
        </Typography>

        <DataTable
          columns={['Destination', 'Purpose', 'Frequency', 'Data Sent']}
          rows={[
            [
              'ThriveTech License Server (license.thrivetechservice.com)',
              'License validation',
              'On app launch, then every 5 minutes',
              'License key, machine fingerprint hash, app version, platform',
            ],
            [
              'GitHub Releases (github.com/thrivetech2t)',
              'Update checks (MS Store and Mac App Store handle updates via their own channels)',
              'On app launch',
              'Current app version',
            ],
            [
              'AI Provider APIs (varies by provider)',
              'AI coding assistance',
              'When you initiate tasks',
              'Your prompts and code (sent directly to provider, not through ThriveTech)',
            ],
          ]}
        />

        <Typography variant="body1" paragraph>
          All ThriveTech network communications are encrypted using HTTPS (TLS 1.2 or later). HTTP connections are automatically upgraded to HTTPS. Localhost connections for local development are exempted.
        </Typography>

        {/* Section 5 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          5. Data Storage and Security
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          5.1 Local Storage
        </Typography>
        <Typography variant="body1" paragraph>
          Mahalaxmi stores the following data locally on your machine:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li><strong>Application settings</strong> &mdash; stored in the Mahalaxmi configuration directory (~/.mahalaxmi/ or platform equivalent)</li>
          <li><strong>AI provider credentials</strong> &mdash; stored in your operating system&rsquo;s secure keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service), not in plaintext files</li>
          <li><strong>Orchestration history</strong> &mdash; task completion metrics and performance data, stored locally for your reference</li>
          <li><strong>Codebase index</strong> &mdash; a structural index of your project for context-aware AI assistance, stored in memory during sessions and not persisted to disk</li>
        </Box>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          5.2 Server-Side Storage
        </Typography>
        <Typography variant="body1" paragraph>
          ThriveTech stores the following on our servers:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li>Your account information (email, name) for license management</li>
          <li>License records (key, activation date, machine fingerprint hash, subscription status)</li>
          <li>Customer support correspondence</li>
        </Box>
        <Typography variant="body1" paragraph>
          We do not store your source code, AI conversations, provider credentials, or project data on our servers.
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          5.3 Security Measures
        </Typography>
        <Typography variant="body1" paragraph>
          We protect your information using:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li>HTTPS-only communication with our servers</li>
          <li>HMAC-SHA256 signature verification for license tokens</li>
          <li>OS-native secure credential storage for API keys</li>
          <li>Machine fingerprint hashing (one-way, irreversible)</li>
          <li>No logging of credentials or API keys in application logs</li>
        </Box>

        {/* Section 6 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          6. Data Sharing
        </Typography>

        <Typography variant="body1" paragraph>
          We do not sell, rent, or trade your personal information. We share your data only in these limited circumstances:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li><strong>Payment processors</strong> &mdash; Microsoft Store, Apple App Store, or Stripe process your payments. We do not have access to your full payment details.</li>
          <li><strong>Legal requirements</strong> &mdash; We may disclose information if required by law, court order, or governmental regulation.</li>
          <li><strong>Business transfer</strong> &mdash; In the event of a merger, acquisition, or sale of assets, your information may be transferred. We will notify you of any such change.</li>
        </Box>
        <Typography variant="body1" paragraph>
          We do not share data with advertising networks, analytics services, data brokers, or any third party for marketing purposes.
        </Typography>

        {/* Section 7 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          7. Data Retention
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li><strong>Account and license data</strong> &mdash; retained for the duration of your subscription plus 90 days after cancellation, then deleted</li>
          <li><strong>Support correspondence</strong> &mdash; retained for 2 years after the last communication</li>
          <li><strong>Local application data</strong> &mdash; remains on your machine until you uninstall the app or manually delete it</li>
        </Box>

        {/* Section 8 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          8. Your Rights
        </Typography>

        <Typography variant="body1" paragraph>
          Depending on your location, you may have the following rights regarding your personal data:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li><strong>Access</strong> &mdash; Request a copy of the personal data we hold about you</li>
          <li><strong>Correction</strong> &mdash; Request correction of inaccurate data</li>
          <li><strong>Deletion</strong> &mdash; Request deletion of your personal data</li>
          <li><strong>Portability</strong> &mdash; Request your data in a machine-readable format</li>
          <li><strong>Objection</strong> &mdash; Object to processing of your data</li>
          <li><strong>Restriction</strong> &mdash; Request restriction of processing</li>
        </Box>
        <Typography variant="body1" paragraph>
          To exercise any of these rights, contact us at{' '}
          <MuiLink href="mailto:legal@mahalaxmi.ai">legal@mahalaxmi.ai</MuiLink>.
          We will respond within 30 days.
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          8.1 California Residents (CCPA/CPRA)
        </Typography>
        <Typography variant="body1" paragraph>
          If you are a California resident, you have the right to know what personal information we collect and how we use it, request deletion of your personal information, and opt out of the sale of personal information. We do not sell personal information. To make a request, contact{' '}
          <MuiLink href="mailto:legal@mahalaxmi.ai">legal@mahalaxmi.ai</MuiLink>.
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          8.2 European Economic Area Residents (GDPR)
        </Typography>
        <Typography variant="body1" paragraph>
          If you are in the EEA, our legal bases for processing your data are: performance of our contract with you (license validation, service delivery) and legitimate interest (security, fraud prevention). You may lodge a complaint with your local data protection authority.
        </Typography>

        {/* Section 9 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          9. Children&rsquo;s Privacy
        </Typography>
        <Typography variant="body1" paragraph>
          Mahalaxmi is a professional developer tool not directed at children under the age of 13 (or 16 in the EEA). We do not knowingly collect personal information from children. If we learn that we have collected information from a child, we will delete it promptly. If you believe a child has provided us with personal data, contact us at{' '}
          <MuiLink href="mailto:legal@mahalaxmi.ai">legal@mahalaxmi.ai</MuiLink>.
        </Typography>

        {/* Section 10 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          10. Cookies and Tracking
        </Typography>
        <Typography variant="body1" paragraph>
          The Mahalaxmi desktop application does not use cookies, web beacons, or tracking pixels. We do not use any analytics services (such as Google Analytics, Mixpanel, or Amplitude) within the application. The ThriveTech website (thrivetechservice.com) may use essential cookies for site functionality; these are covered by the website&rsquo;s separate cookie notice.
        </Typography>

        {/* Section 11 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          11. Third-Party Services
        </Typography>
        <Typography variant="body1" paragraph>
          Mahalaxmi integrates with third-party AI providers that you configure. Each provider&rsquo;s handling of your data is governed by their own privacy policies:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li>Anthropic (Claude Code): <MuiLink href="https://www.anthropic.com/privacy" target="_blank" rel="noopener noreferrer">anthropic.com/privacy</MuiLink></li>
          <li>OpenAI (Codex CLI): <MuiLink href="https://openai.com/policies/privacy-policy" target="_blank" rel="noopener noreferrer">openai.com/policies/privacy-policy</MuiLink></li>
          <li>Google (Gemini CLI): <MuiLink href="https://policies.google.com/privacy" target="_blank" rel="noopener noreferrer">policies.google.com/privacy</MuiLink></li>
          <li>xAI (Grok CLI): <MuiLink href="https://x.ai/legal/privacy-policy" target="_blank" rel="noopener noreferrer">x.ai/legal/privacy-policy</MuiLink></li>
          <li>GitHub (Copilot CLI): <MuiLink href="https://docs.github.com/en/site-policy/privacy-policies" target="_blank" rel="noopener noreferrer">docs.github.com/en/site-policy/privacy-policies</MuiLink></li>
        </Box>
        <Typography variant="body1" paragraph>
          ThriveTech is not responsible for the privacy practices of third-party AI providers. We encourage you to review their policies before configuring them in Mahalaxmi.
        </Typography>

        {/* Section 12 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          12. App Store Privacy Labels
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          Apple App Store &mdash; App Privacy &ldquo;Nutrition Label&rdquo;
        </Typography>
        <Typography variant="body1" paragraph>
          For Apple&rsquo;s App Privacy disclosure, Mahalaxmi&rsquo;s data practices are:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li><strong>Data Used to Track You:</strong> None. Mahalaxmi does not track you across other companies&rsquo; apps or websites.</li>
          <li><strong>Data Linked to You:</strong> Email address and name (for license management), device identifiers (machine fingerprint hash for license binding).</li>
          <li><strong>Data Not Linked to You:</strong> Diagnostics (crash logs, if any, are anonymized).</li>
          <li><strong>Data Not Collected:</strong> Location, contacts, browsing history, search history, usage data, financial information, health data, fitness data, photos, videos, audio, gameplay content, sensitive information.</li>
        </Box>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          Microsoft Store &mdash; Privacy Statement
        </Typography>
        <Typography variant="body1" paragraph>
          Mahalaxmi collects the minimum data necessary for license validation (email, license key, device identifier hash). No analytics, advertising, or tracking data is collected.
        </Typography>

        {/* Section 13 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          13. Data Deletion
        </Typography>
        <Typography variant="body1" paragraph>
          You can delete your data as follows:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li><strong>Local data:</strong> Uninstall Mahalaxmi and delete the ~/.mahalaxmi/ directory. Keychain entries can be removed through your operating system&rsquo;s credential manager.</li>
          <li><strong>Server data:</strong> Email <MuiLink href="mailto:legal@mahalaxmi.ai">legal@mahalaxmi.ai</MuiLink> to request deletion of your account and all associated data. We will process your request within 30 days.</li>
        </Box>

        {/* Section 14 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          14. Changes to This Policy
        </Typography>
        <Typography variant="body1" paragraph>
          We may update this Privacy Policy from time to time. When we make material changes, we will update the &ldquo;Last Updated&rdquo; date at the top of this page and, where appropriate, notify you via email or an in-app notification. We encourage you to review this policy periodically.
        </Typography>

        {/* Section 15 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          15. Contact Us
        </Typography>

        <Paper variant="outlined" sx={{ p: 2.5, mt: 2 }}>
          <Typography variant="body1"><strong>ThriveTech Services LLC</strong></Typography>
          <Typography variant="body1">West Palm Beach, Florida, United States</Typography>
          <Typography variant="body1">
            Email: <MuiLink href="mailto:legal@mahalaxmi.ai">legal@mahalaxmi.ai</MuiLink>
          </Typography>
          <Typography variant="body1">
            Website: <MuiLink href="https://thrivetechservice.com" target="_blank" rel="noopener noreferrer">thrivetechservice.com</MuiLink>
          </Typography>
          <Typography variant="body1">
            For general support: <MuiLink href="mailto:support@mahalaxmi.ai">support@mahalaxmi.ai</MuiLink>
          </Typography>
        </Paper>

        {/* Footer */}
        <Box sx={{ mt: 6, pt: 2, borderTop: '1px solid', borderColor: 'divider', textAlign: 'center' }}>
          <Typography variant="body2" color="text.disabled">
            &copy; 2026 ThriveTech Services LLC. All rights reserved.
          </Typography>
        </Box>
      </Paper>
    </Container>
  );
}

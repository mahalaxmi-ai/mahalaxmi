import { setRequestLocale } from 'next-intl/server';
import { getAlternateLanguages, getCanonical, getOpenGraphLocale } from '@/utils/i18nMetadata';
import { locales } from '@/i18n/routing';
import {
  Container,
  Typography,
  Box,
  Paper,
  Link as MuiLink,
} from '@mui/material';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;

  return {
    title: 'Terms of Service — Mahalaxmi',
    description:
      'Terms of Service for Mahalaxmi Terminal Automation by ThriveTech Services LLC.',
    alternates: {
      canonical: getCanonical(locale, '/legal/terms'),
      languages: getAlternateLanguages('/legal/terms'),
    },
    openGraph: {
      title: 'Terms of Service — Mahalaxmi',
      description:
        'Terms of Service for Mahalaxmi Terminal Automation by ThriveTech Services LLC.',
      url: '/legal/terms',
      locale: getOpenGraphLocale(locale),
    },
  };
}

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

const warningBoxSx = {
  bgcolor: '#fffbeb',
  borderInlineStart: '4px solid',
  borderColor: 'warning.main',
  p: 2.5,
  my: 2.5,
  borderRadius: '0 6px 6px 0',
};

export default async function MahalaxmiTermsPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="md" sx={{ py: { xs: 4, md: 6 } }}>
      <Paper elevation={1} sx={{ p: { xs: 3, md: 6 } }}>
        {/* Header */}
        <Box sx={{ borderBottom: '3px solid', borderColor: 'primary.main', pb: 3, mb: 5 }}>
          <Typography variant="h3" component="h1" sx={{ fontWeight: 700, mb: 1 }}>
            Terms of Service
          </Typography>
          <Typography variant="subtitle1" color="text.secondary">
            Mahalaxmi Terminal Automation by ThriveTech Services LLC
          </Typography>
          <Typography variant="body2" color="text.disabled" sx={{ mt: 0.5 }}>
            Effective Date: February 22, 2026 &nbsp;|&nbsp; Last Updated: February 22, 2026
          </Typography>
        </Box>

        <Typography variant="body1" paragraph>
          These Terms of Service (&ldquo;Terms&rdquo;) govern your use of Mahalaxmi, an AI terminal orchestration application (&ldquo;the App&rdquo;), developed and operated by ThriveTech Services LLC (&ldquo;ThriveTech,&rdquo; &ldquo;we,&rdquo; &ldquo;us,&rdquo; or &ldquo;our&rdquo;), a company registered in West Palm Beach, Florida, United States.
        </Typography>

        <Typography variant="body1" paragraph>
          By installing, accessing, or using Mahalaxmi, you agree to be bound by these Terms. If you do not agree, do not use the App.
        </Typography>

        {/* Section 1 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          1. License Grant
        </Typography>
        <Typography variant="body1" paragraph>
          Subject to your compliance with these Terms and payment of applicable fees, ThriveTech grants you a limited, non-exclusive, non-transferable, revocable license to install and use Mahalaxmi on devices you own or control, solely for your personal or internal business purposes.
        </Typography>
        <Typography variant="body1" paragraph>
          This license does not include the right to:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li>Sublicense, resell, or distribute the App to third parties</li>
          <li>Reverse engineer, decompile, or disassemble the App, except as permitted by applicable law</li>
          <li>Remove or modify any proprietary notices, labels, or marks in the App</li>
          <li>Use the App to develop a competing product</li>
          <li>Share your license key with others or use it on more devices than permitted by your subscription</li>
        </Box>

        {/* Section 2 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          2. Subscription and Payment
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          2.1 Subscription Plans
        </Typography>
        <Typography variant="body1" paragraph>
          Mahalaxmi is offered as a subscription service with various tiers. Current plans and pricing are available at{' '}
          <MuiLink href="https://mahalaxmi.ai/pricing" target="_blank" rel="noopener noreferrer">mahalaxmi.ai/pricing</MuiLink>.
          Features available to you depend on your subscription tier.
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          2.2 Payment Processing
        </Typography>
        <Typography variant="body1" paragraph>
          Payments are processed through the Microsoft Store, Apple App Store, or directly via Stripe, depending on your platform. All payments are subject to the terms of the respective payment processor. ThriveTech does not store your credit card or payment details.
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          2.3 Renewals and Cancellation
        </Typography>
        <Typography variant="body1" paragraph>
          Subscriptions automatically renew at the end of each billing period unless cancelled. You may cancel at any time through your app store account or by contacting{' '}
          <MuiLink href="mailto:support@mahalaxmi.ai">support@mahalaxmi.ai</MuiLink>.
          Cancellation takes effect at the end of the current billing period. No refunds are provided for partial billing periods, except as required by applicable law or app store policies.
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          2.4 License Validation
        </Typography>
        <Typography variant="body1" paragraph>
          The App validates your license periodically by communicating with ThriveTech servers. An internet connection is required for initial activation. After activation, the App provides a grace period for offline use. If your license expires, is revoked, or cannot be validated beyond the grace period, certain features will be restricted.
        </Typography>

        {/* Section 3 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          3. Your Responsibilities
        </Typography>
        <Typography variant="body1" paragraph>
          You are responsible for:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li>Maintaining the confidentiality of your license key and account credentials</li>
          <li>All activity that occurs under your account</li>
          <li>Configuring and managing your own AI provider accounts and API keys</li>
          <li>Ensuring your use of third-party AI providers complies with their respective terms of service</li>
          <li>The content you process through the App, including source code and prompts sent to AI providers</li>
          <li>Complying with all applicable laws and regulations in your use of the App</li>
        </Box>

        {/* Section 4 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          4. Third-Party AI Providers
        </Typography>

        <Box sx={highlightBoxSx}>
          <Typography variant="body1">
            <strong>Important:</strong> Mahalaxmi orchestrates third-party AI coding tools that you independently configure. ThriveTech is not a party to your relationship with these AI providers and is not responsible for their services, outputs, or handling of your data.
          </Typography>
        </Box>

        <Typography variant="body1" paragraph>
          When you use Mahalaxmi to interact with AI providers (such as Anthropic, OpenAI, Google, xAI, GitHub, and others):
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li>Your code and prompts are sent directly from your machine to the AI provider. ThriveTech does not proxy, intercept, or store this communication.</li>
          <li>You are solely responsible for maintaining valid accounts and API keys with each provider.</li>
          <li>Your use of each provider is governed by their terms of service and privacy policies.</li>
          <li>ThriveTech does not guarantee the availability, accuracy, or quality of any third-party AI provider&rsquo;s output.</li>
          <li>AI-generated code and suggestions should be reviewed before use in production systems.</li>
        </Box>

        {/* Section 5 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          5. Intellectual Property
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          5.1 ThriveTech&rsquo;s IP
        </Typography>
        <Typography variant="body1" paragraph>
          Mahalaxmi, including its source code, design, documentation, trademarks, and all related intellectual property, is owned by ThriveTech Services LLC. Nothing in these Terms transfers any IP rights to you beyond the license granted in Section 1.
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          5.2 Your IP
        </Typography>
        <Typography variant="body1" paragraph>
          You retain all rights to your source code, projects, and content that you process through the App. ThriveTech claims no ownership of your code or AI-generated output. The App processes your content locally on your machine; ThriveTech does not access, store, or have any rights to your project content.
        </Typography>

        {/* Section 6 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          6. Disclaimers
        </Typography>

        <Box sx={warningBoxSx}>
          <Typography variant="body1">
            <strong>THE APP IS PROVIDED &ldquo;AS IS&rdquo; AND &ldquo;AS AVAILABLE.&rdquo;</strong> To the maximum extent permitted by applicable law, ThriveTech disclaims all warranties, express or implied, including warranties of merchantability, fitness for a particular purpose, and non-infringement.
          </Typography>
        </Box>

        <Typography variant="body1" paragraph>
          Without limiting the foregoing:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li>ThriveTech does not warrant that the App will be error-free or uninterrupted.</li>
          <li>ThriveTech does not warrant the accuracy, reliability, or completeness of any AI-generated output.</li>
          <li>ThriveTech is not responsible for any damage to your code, systems, or data resulting from the use of the App or third-party AI providers.</li>
          <li>You are solely responsible for maintaining backups of your code and data.</li>
        </Box>

        {/* Section 7 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          7. Limitation of Liability
        </Typography>
        <Typography variant="body1" paragraph>
          To the maximum extent permitted by applicable law, in no event shall ThriveTech, its officers, directors, employees, or agents be liable for any indirect, incidental, special, consequential, or punitive damages, including loss of profits, data, or goodwill, arising out of or in connection with your use of the App, regardless of the theory of liability.
        </Typography>
        <Typography variant="body1" paragraph>
          ThriveTech&rsquo;s total aggregate liability for any claims arising under these Terms shall not exceed the amount you paid to ThriveTech for the App in the twelve (12) months preceding the claim.
        </Typography>

        {/* Section 8 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          8. Acceptable Use
        </Typography>
        <Typography variant="body1" paragraph>
          You agree not to use Mahalaxmi to:
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li>Violate any applicable law, regulation, or third-party rights</li>
          <li>Generate or distribute malware, viruses, or malicious code</li>
          <li>Attempt to circumvent license validation, copy protection, or security measures</li>
          <li>Interfere with or disrupt ThriveTech&rsquo;s services or infrastructure</li>
          <li>Access or attempt to access other users&rsquo; accounts or data</li>
          <li>Use the App in any manner that could damage, disable, or overburden ThriveTech&rsquo;s servers</li>
        </Box>
        <Typography variant="body1" paragraph>
          Violation of these acceptable use terms may result in immediate license revocation without refund.
        </Typography>

        {/* Section 9 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          9. Termination
        </Typography>
        <Typography variant="body1" paragraph>
          ThriveTech may terminate or suspend your license immediately if you breach these Terms. Upon termination, your right to use the App ceases. You may terminate by cancelling your subscription and uninstalling the App. Sections 5 (Intellectual Property), 6 (Disclaimers), 7 (Limitation of Liability), and 11 (Governing Law) survive termination.
        </Typography>

        {/* Section 10 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          10. Updates and Changes
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          10.1 App Updates
        </Typography>
        <Typography variant="body1" paragraph>
          ThriveTech may release updates to the App from time to time. Updates distributed through the Microsoft Store or Apple App Store are governed by the respective store&rsquo;s update mechanisms. We recommend keeping the App up to date for the best experience and security.
        </Typography>

        <Typography variant="h6" component="h3" sx={{ mt: 2.5, mb: 1 }}>
          10.2 Changes to Terms
        </Typography>
        <Typography variant="body1" paragraph>
          We may modify these Terms at any time. Material changes will be communicated via email or in-app notification at least 30 days before taking effect. Continued use of the App after changes take effect constitutes acceptance. If you disagree with changes, you may cancel your subscription and stop using the App.
        </Typography>

        {/* Section 11 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          11. Governing Law and Disputes
        </Typography>
        <Typography variant="body1" paragraph>
          These Terms are governed by the laws of the State of Florida, United States, without regard to conflict of law principles. Any dispute arising from these Terms shall be resolved in the state or federal courts located in Palm Beach County, Florida. You consent to the personal jurisdiction of these courts.
        </Typography>

        {/* Section 12 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          12. General Provisions
        </Typography>
        <Box component="ul" sx={{ pl: 3, mb: 2 }}>
          <li><strong>Entire Agreement:</strong> These Terms, together with the Privacy Policy, constitute the entire agreement between you and ThriveTech regarding the App.</li>
          <li><strong>Severability:</strong> If any provision is found unenforceable, the remaining provisions continue in effect.</li>
          <li><strong>Waiver:</strong> Failure to enforce any provision does not constitute a waiver of that provision.</li>
          <li><strong>Assignment:</strong> You may not assign these Terms without ThriveTech&rsquo;s written consent. ThriveTech may assign these Terms freely.</li>
          <li><strong>Force Majeure:</strong> ThriveTech is not liable for delays or failures due to circumstances beyond our reasonable control.</li>
        </Box>

        {/* Section 13 */}
        <Typography variant="h5" component="h2" sx={sectionHeadingSx}>
          13. Contact Us
        </Typography>

        <Paper variant="outlined" sx={{ p: 2.5, mt: 2 }}>
          <Typography variant="body1"><strong>ThriveTech Services LLC</strong></Typography>
          <Typography variant="body1">West Palm Beach, Florida, United States</Typography>
          <Typography variant="body1">
            Legal inquiries: <MuiLink href="mailto:legal@mahalaxmi.ai">legal@mahalaxmi.ai</MuiLink>
          </Typography>
          <Typography variant="body1">
            General support: <MuiLink href="mailto:support@mahalaxmi.ai">support@mahalaxmi.ai</MuiLink>
          </Typography>
          <Typography variant="body1">
            Website: <MuiLink href="https://mahalaxmi.ai" target="_blank" rel="noopener noreferrer">mahalaxmi.ai</MuiLink>
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

'use client';

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Accordion from '@mui/material/Accordion';
import AccordionSummary from '@mui/material/AccordionSummary';
import AccordionDetails from '@mui/material/AccordionDetails';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import Link from '@mui/material/Link';

const faqs: { question: string; answer: React.ReactNode }[] = [
  {
    question: 'What is Mahalaxmi AI?',
    answer:
      'Mahalaxmi AI is an AI terminal orchestration platform. It manages Manager-Worker agent cycles, allowing you to run coordinated AI coding agents across cloud servers from a single dashboard. The platform handles scheduling, session lifecycle, and inter-agent communication so you can focus on building.',
  },
  {
    question: 'What AI providers are supported?',
    answer: (
      <>
        Mahalaxmi supports multiple AI coding providers out of the box:
        <ul>
          <li>Claude Code (Anthropic)</li>
          <li>GitHub Copilot</li>
          <li>Grok (xAI)</li>
          <li>Ollama (local models)</li>
          <li>Google Gemini</li>
        </ul>
        You can configure which provider each worker uses from the server settings in your
        dashboard.
      </>
    ),
  },
  {
    question: 'How do I connect VS Code to a cloud server?',
    answer: (
      <>
        Full step-by-step instructions are available on the{' '}
        <Link href="/docs/cloud" underline="hover">
          Cloud Servers guide
        </Link>
        . In short: provision a server from your dashboard, then click{' '}
        <strong>Open in VS Code</strong> to launch the pre-built deep link. The VS Code
        extension handles authentication automatically.
      </>
    ),
  },
  {
    question: 'What does a "Degraded" server status mean?',
    answer: (
      <>
        A <strong>Degraded</strong> status means the server process is running but is in an
        unhealthy state — for example, the agent runtime is unresponsive or a critical service
        failed to start. The server has not stopped, but it may not function correctly.
        <br />
        <br />
        If your server shows Degraded, try stopping and restarting it from the dashboard. If
        the problem persists, contact{' '}
        <Link href="mailto:support@mahalaxmi.ai" underline="hover">
          support@mahalaxmi.ai
        </Link>{' '}
        with your server ID.
      </>
    ),
  },
  {
    question: 'Is my code stored on cloud servers?',
    answer:
      'Work files are written to the cloud server only during an active session. When you stop the server, all work files are deleted from the cloud instance. No source code is retained between sessions. Persistent data such as configuration is stored encrypted at rest and is separate from session work files.',
  },
  {
    question: 'How do I cancel my subscription?',
    answer: (
      <>
        To cancel your subscription:
        <ol>
          <li>
            Go to{' '}
            <Link href="/dashboard/billing" underline="hover">
              /dashboard/billing
            </Link>
            .
          </li>
          <li>
            Click the <strong>Manage Billing</strong> button.
          </li>
          <li>You will be redirected to the Stripe customer portal.</li>
          <li>In the portal, select your subscription and choose Cancel.</li>
        </ol>
        Cancellation takes effect at the end of your current billing period. If you need
        assistance, email{' '}
        <Link href="mailto:support@mahalaxmi.ai" underline="hover">
          support@mahalaxmi.ai
        </Link>
        .
      </>
    ),
  },
  {
    question: 'How do I get support?',
    answer: (
      <>
        For technical issues, billing questions, or general inquiries, email{' '}
        <Link href="mailto:support@mahalaxmi.ai" underline="hover">
          support@mahalaxmi.ai
        </Link>
        . Include your account email and a description of the issue. For server-specific
        problems, include the server ID visible in your dashboard.
      </>
    ),
  },
];

export default function FaqPage() {
  return (
    <>
      <title>FAQ | Mahalaxmi Docs</title>
      <meta
        name="description"
        content="Frequently asked questions about Mahalaxmi AI terminal orchestration platform."
      />
      <link rel="canonical" href="https://mahalaxmi.ai/docs/faq" />

      <Box sx={{ maxWidth: 800, mx: 'auto', px: { xs: 2, md: 4 }, py: 6 }}>
        <Typography variant="h3" component="h1" fontWeight={700} gutterBottom>
          Frequently Asked Questions
        </Typography>
        <Typography variant="body1" color="text.secondary" sx={{ mb: 5 }}>
          Can&apos;t find what you&apos;re looking for? Email{' '}
          <Link href="mailto:support@mahalaxmi.ai" underline="hover">
            support@mahalaxmi.ai
          </Link>{' '}
          and we&apos;ll get back to you.
        </Typography>

        {faqs.map((faq) => (
          <Accordion
            key={faq.question}
            disableGutters
            elevation={0}
            sx={{
              border: '1px solid',
              borderColor: 'divider',
              mb: 1.5,
              borderRadius: 1,
              '&:before': { display: 'none' },
            }}
          >
            <AccordionSummary expandIcon={<ExpandMoreIcon />}>
              <Typography fontWeight={600}>{faq.question}</Typography>
            </AccordionSummary>
            <AccordionDetails>
              <Typography variant="body2" color="text.secondary" component="div">
                {faq.answer}
              </Typography>
            </AccordionDetails>
          </Accordion>
        ))}
      </Box>
    </>
  );
}

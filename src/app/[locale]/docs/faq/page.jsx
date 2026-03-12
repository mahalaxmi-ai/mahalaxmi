'use client';

import { useState } from 'react';
import {
  Box,
  Container,
  Typography,
  Accordion,
  AccordionSummary,
  AccordionDetails,
} from '@mui/material';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';

const faqs = [
  {
    id: 'what-is-mahalaxmi',
    question: 'What is Mahalaxmi?',
    answer:
      'Mahalaxmi is an AI terminal orchestration platform by ThriveTech Services LLC. It lets you spin up cloud-hosted AI coding environments and connect directly from VS Code — giving you a powerful, always-on server that your editor can reach from anywhere.',
  },
  {
    id: 'connect-vscode',
    question: 'How do I connect VS Code to my cloud server?',
    answer:
      'Once your server is active, open VS Code and install the Mahalaxmi extension from the Marketplace. Click the Mahalaxmi icon in the activity bar, then select "Connect to Server". The extension will use your API key to authenticate and open a remote session on your cloud server automatically.',
  },
  {
    id: 'stopped-status',
    question: 'What does the "Stopped" status mean?',
    answer:
      'A "Stopped" server has been gracefully shut down and is no longer running. Your data and project files are preserved on disk. You can restart the server at any time from your dashboard to resume work.',
  },
  {
    id: 'failed-status',
    question: 'What does the "Failed" status mean?',
    answer:
      'A "Failed" status means the server encountered an unrecoverable error during provisioning or operation. Please contact our support team at support@mahalaxmi.ai with your server ID and we will investigate and resolve the issue as quickly as possible.',
  },
  {
    id: 'restart-stopped-server',
    question: 'How do I restart a stopped server?',
    answer:
      'From your dashboard, locate the stopped server in your server list. Click the "Restart" button on the server card. The server will transition through a provisioning state and become active again within a few minutes.',
  },
  {
    id: 'delete-server',
    question: 'How do I delete a server?',
    answer:
      'Open your dashboard and find the server you want to remove. Click the menu icon on the server card and select "Delete". You will be asked to confirm the action. Deletion is asynchronous — the server will move to a "deleting" state and be fully removed shortly after confirmation.',
  },
  {
    id: 'project-name',
    question: 'What is a project name?',
    answer:
      'A project name is a human-readable label you assign to a server when you create it. It helps you identify the server in your dashboard and in VS Code. You can use any descriptive name — for example, the name of the application or feature you are building on that server.',
  },
  {
    id: 'change-ai-provider',
    question: 'How do I change my AI provider?',
    answer:
      'AI provider settings are configured per server at creation time. To use a different provider, create a new server and select your preferred AI provider from the dropdown during setup. If you need to migrate an existing project, reach out to support@mahalaxmi.ai and we can assist with the transition.',
  },
];

export default function FaqPage() {
  const [expanded, setExpanded] = useState(false);

  const handleChange = (panel) => (_event, isExpanded) => {
    setExpanded(isExpanded ? panel : false);
  };

  return (
    <Container maxWidth="md" sx={{ py: 8 }}>
      <Typography variant="h3" component="h1" fontWeight={700} gutterBottom>
        Frequently Asked Questions
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 6 }}>
        Find answers to common questions about Mahalaxmi.
      </Typography>

      <Box>
        {faqs.map((faq) => (
          <Accordion
            key={faq.id}
            expanded={expanded === faq.id}
            onChange={handleChange(faq.id)}
            disableGutters
            elevation={0}
            sx={{
              border: '1px solid',
              borderColor: 'divider',
              mb: 1,
              borderRadius: 1,
              '&:before': { display: 'none' },
            }}
          >
            <AccordionSummary
              expandIcon={<ExpandMoreIcon />}
              aria-controls={`${faq.id}-content`}
              id={`${faq.id}-header`}
            >
              <Typography fontWeight={600}>{faq.question}</Typography>
            </AccordionSummary>
            <AccordionDetails>
              <Typography variant="body2" color="text.secondary">
                {faq.answer}
              </Typography>
            </AccordionDetails>
          </Accordion>
        ))}
      </Box>

      <Box
        sx={{
          mt: 8,
          p: 4,
          borderRadius: 2,
          bgcolor: 'background.paper',
          border: '1px solid',
          borderColor: 'divider',
          textAlign: 'center',
        }}
      >
        <Typography variant="h6" fontWeight={600} gutterBottom>
          Still have questions?
        </Typography>
        <Typography variant="body2" color="text.secondary">
          Email{' '}
          <Box
            component="a"
            href="mailto:support@mahalaxmi.ai"
            sx={{ color: 'primary.main', textDecoration: 'none' }}
          >
            support@mahalaxmi.ai
          </Box>
        </Typography>
      </Box>
    </Container>
  );
}

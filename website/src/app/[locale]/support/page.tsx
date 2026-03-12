'use client';

import React, { useState } from 'react';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Accordion from '@mui/material/Accordion';
import AccordionSummary from '@mui/material/AccordionSummary';
import AccordionDetails from '@mui/material/AccordionDetails';
import Link from '@mui/material/Link';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';

const SUPPORT_EMAIL = 'support@mahalaxmi.ai';

const faqs = [
  {
    question: 'How do I get started with Mahalaxmi?',
    answer:
      'Sign up at mahalaxmi.ai, create a project, and spin up your first cloud server in minutes. See our Quickstart guide for step-by-step instructions.',
  },
  {
    question: 'How do I reset my API key?',
    answer:
      'Go to your dashboard settings and click "Regenerate API Key". Your old key will be invalidated immediately.',
  },
  {
    question: 'What happens if my server enters a degraded state?',
    answer:
      'A degraded server is still running but may have reduced performance. Check the server logs in your dashboard for details, or contact support if the issue persists.',
  },
  {
    question: 'Can I run multiple agents on the same project?',
    answer:
      'Yes. Mahalaxmi is designed for multi-agent orchestration. You can assign multiple workers to a single project and they will coordinate automatically.',
  },
  {
    question: 'How is billing calculated?',
    answer:
      'Billing is based on active server-hours. Stopped or deleted servers are not billed. See the pricing page for current rates.',
  },
];

export default function SupportPage() {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [subject, setSubject] = useState('');
  const [message, setMessage] = useState('');

  function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const body = `Name: ${name}\nEmail: ${email}\n\n${message}`;
    const mailto = `mailto:${SUPPORT_EMAIL}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
    window.location.href = mailto;
  }

  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Support
      </Typography>

      <Box sx={{ mb: 4, p: 2, bgcolor: 'action.hover', borderRadius: 2 }}>
        <Typography variant="body1">
          Email us directly:{' '}
          <Link href={`mailto:${SUPPORT_EMAIL}`} fontWeight={700} fontSize="1.1rem">
            {SUPPORT_EMAIL}
          </Link>
        </Typography>
      </Box>

      <Typography variant="h5" fontWeight={600} gutterBottom>
        Contact Form
      </Typography>
      <Box component="form" onSubmit={handleSubmit} sx={{ mb: 6 }}>
        <TextField
          label="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          fullWidth
          required
          margin="normal"
        />
        <TextField
          label="Email"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          fullWidth
          required
          margin="normal"
        />
        <TextField
          label="Subject"
          value={subject}
          onChange={(e) => setSubject(e.target.value)}
          fullWidth
          required
          margin="normal"
        />
        <TextField
          label="Message"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          fullWidth
          required
          multiline
          minRows={4}
          margin="normal"
        />
        <Button type="submit" variant="contained" size="large" sx={{ mt: 2 }}>
          Send via Email Client
        </Button>
      </Box>

      <Divider sx={{ my: 4 }} />

      <Typography variant="h5" fontWeight={600} gutterBottom>
        Frequently Asked Questions
      </Typography>

      {faqs.map((faq) => (
        <Accordion key={faq.question} disableGutters>
          <AccordionSummary expandIcon={<ExpandMoreIcon />}>
            <Typography fontWeight={500}>{faq.question}</Typography>
          </AccordionSummary>
          <AccordionDetails>
            <Typography color="text.secondary">{faq.answer}</Typography>
          </AccordionDetails>
        </Accordion>
      ))}

      <Box sx={{ mt: 3 }}>
        <Typography variant="body2" color="text.secondary">
          More answers in the{' '}
          <Link href="/docs/faq" underline="hover">
            full FAQ
          </Link>
          .
        </Typography>
      </Box>
    </Container>
  );
}

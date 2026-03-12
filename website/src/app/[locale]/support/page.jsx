'use client';

import { useState } from 'react';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import TextField from '@mui/material/TextField';
import MenuItem from '@mui/material/MenuItem';
import Button from '@mui/material/Button';
import Alert from '@mui/material/Alert';
import Accordion from '@mui/material/Accordion';
import AccordionSummary from '@mui/material/AccordionSummary';
import AccordionDetails from '@mui/material/AccordionDetails';
import Divider from '@mui/material/Divider';
import MuiLink from '@mui/material/Link';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';

const TEAL = '#00C8C8';

const SUBJECTS = [
  { value: 'technical', label: 'Technical Issue' },
  { value: 'billing', label: 'Billing' },
  { value: 'feature', label: 'Feature Request' },
  { value: 'other', label: 'Other' },
];

const FAQ_ITEMS = [
  {
    question: 'How do I reset my password?',
    answer: (
      <>
        Go to{' '}
        <MuiLink href="/forgot-password" sx={{ color: TEAL }}>
          /forgot-password
        </MuiLink>{' '}
        and enter your email address. You will receive a reset link within a few minutes.
      </>
    ),
  },
  {
    question: 'How do I cancel my subscription?',
    answer: (
      <>
        Use Manage Billing at{' '}
        <MuiLink href="/dashboard/billing" sx={{ color: TEAL }}>
          /dashboard/billing
        </MuiLink>{' '}
        to cancel or modify your subscription plan.
      </>
    ),
  },
  {
    question: 'Where do I find my api_key?',
    answer:
      'Click "Open in VS Code" on your server card in the Dashboard. The deep link passes your api_key directly to the VS Code extension.',
  },
  {
    question: 'How do I report a security issue?',
    answer: (
      <>
        Email{' '}
        <MuiLink href="mailto:security@mahalaxmi.ai" sx={{ color: TEAL }}>
          security@mahalaxmi.ai
        </MuiLink>{' '}
        — do NOT open a public GitHub issue for security vulnerabilities.
      </>
    ),
  },
  {
    question: 'How do I get started with Mahalaxmi?',
    answer: (
      <>
        Visit the{' '}
        <MuiLink href="/docs/quickstart" sx={{ color: TEAL }}>
          Quickstart guide
        </MuiLink>{' '}
        to provision your first server and connect via the VS Code extension in minutes.
      </>
    ),
  },
];

const INITIAL_FORM = { name: '', email: '', subject: '', message: '' };

export default function SupportPage() {
  const [form, setForm] = useState(INITIAL_FORM);
  const [submitted, setSubmitted] = useState(false);
  const [errors, setErrors] = useState({});

  function validate() {
    const next = {};
    if (!form.name.trim()) next.name = 'Name is required.';
    if (!form.email.trim()) {
      next.email = 'Email is required.';
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.email)) {
      next.email = 'Enter a valid email address.';
    }
    if (!form.subject) next.subject = 'Please select a subject.';
    if (!form.message.trim()) next.message = 'Message is required.';
    return next;
  }

  function handleChange(e) {
    const { name, value } = e.target;
    setForm((prev) => ({ ...prev, [name]: value }));
    if (errors[name]) {
      setErrors((prev) => ({ ...prev, [name]: undefined }));
    }
  }

  function handleSubmit(e) {
    e.preventDefault();
    const validation = validate();
    if (Object.keys(validation).length > 0) {
      setErrors(validation);
      return;
    }
    const subjectLabel =
      SUBJECTS.find((s) => s.value === form.subject)?.label ?? form.subject;
    const body = encodeURIComponent(
      `Name: ${form.name}\nEmail: ${form.email}\nSubject: ${subjectLabel}\n\n${form.message}`
    );
    const mailtoHref = `mailto:support@mahalaxmi.ai?subject=${encodeURIComponent(
      `[Support] ${subjectLabel}`
    )}&body=${body}`;
    window.location.href = mailtoHref;
    setSubmitted(true);
    setForm(INITIAL_FORM);
  }

  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" fontWeight={700} gutterBottom>
        Support
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        We are here to help. Reach out via the contact form below or use one of the direct channels.
      </Typography>

      {/* Contact Information */}
      <Box
        sx={{
          mb: 5,
          p: 3,
          borderRadius: 2,
          border: `1px solid ${TEAL}`,
          backgroundColor: 'rgba(0,200,200,0.05)',
        }}
      >
        <Typography variant="h5" fontWeight={700} gutterBottom>
          Contact Information
        </Typography>
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5, mt: 1 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Typography variant="body1" fontWeight={600}>
              Primary Support:
            </Typography>
            <MuiLink
              href="mailto:support@mahalaxmi.ai"
              sx={{
                color: TEAL,
                fontWeight: 700,
                fontSize: '1.05rem',
                textDecoration: 'underline',
              }}
            >
              support@mahalaxmi.ai
            </MuiLink>
          </Box>
          <Typography variant="body2" color="text.secondary">
            We typically respond within 1 business day.
          </Typography>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Typography variant="body1" fontWeight={600}>
              Bug Reports (open-source only):
            </Typography>
            <MuiLink
              href="https://github.com/thrivetech2t/mahalaxmi/issues"
              target="_blank"
              rel="noopener noreferrer"
              sx={{ color: TEAL }}
            >
              GitHub Issues
            </MuiLink>
          </Box>
        </Box>
      </Box>

      <Divider sx={{ mb: 5 }} />

      {/* Contact Form */}
      <Box sx={{ mb: 6 }}>
        <Typography variant="h5" fontWeight={700} gutterBottom>
          Send a Message
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
          Fill in the form and click &quot;Send Message&quot; to open your email client pre-filled with your details.
        </Typography>

        {submitted && (
          <Alert severity="success" sx={{ mb: 3 }}>
            Your message has been sent. We will reply to support@mahalaxmi.ai within 1 business day.
          </Alert>
        )}

        <Box component="form" onSubmit={handleSubmit} noValidate>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2.5 }}>
            <TextField
              label="Name"
              name="name"
              value={form.name}
              onChange={handleChange}
              error={Boolean(errors.name)}
              helperText={errors.name ?? ''}
              fullWidth
              required
            />
            <TextField
              label="Email"
              name="email"
              type="email"
              value={form.email}
              onChange={handleChange}
              error={Boolean(errors.email)}
              helperText={errors.email ?? ''}
              fullWidth
              required
            />
            <TextField
              select
              label="Subject"
              name="subject"
              value={form.subject}
              onChange={handleChange}
              error={Boolean(errors.subject)}
              helperText={errors.subject ?? ''}
              fullWidth
              required
            >
              {SUBJECTS.map((opt) => (
                <MenuItem key={opt.value} value={opt.value}>
                  {opt.label}
                </MenuItem>
              ))}
            </TextField>
            <TextField
              label="Message"
              name="message"
              value={form.message}
              onChange={handleChange}
              error={Boolean(errors.message)}
              helperText={errors.message ?? ''}
              fullWidth
              required
              multiline
              minRows={5}
            />
            <Box>
              <Button
                type="submit"
                variant="contained"
                size="large"
                sx={{
                  backgroundColor: TEAL,
                  color: '#000',
                  fontWeight: 700,
                  px: 4,
                  '&:hover': { backgroundColor: '#00A0A0' },
                }}
              >
                Send Message
              </Button>
            </Box>
          </Box>
        </Box>
      </Box>

      <Divider sx={{ mb: 5 }} />

      {/* FAQ Accordion */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="h5" fontWeight={700} gutterBottom sx={{ mb: 3 }}>
          Frequently Asked Questions
        </Typography>
        {FAQ_ITEMS.map((item, index) => (
          <Accordion
            key={index}
            disableGutters
            sx={{
              backgroundColor: 'rgba(255,255,255,0.03)',
              border: '1px solid',
              borderColor: 'divider',
              mb: 1,
              borderRadius: '8px !important',
              '&:before': { display: 'none' },
            }}
          >
            <AccordionSummary
              expandIcon={<ExpandMoreIcon sx={{ color: TEAL }} />}
              sx={{ fontWeight: 600 }}
            >
              <Typography variant="body1" fontWeight={600}>
                {item.question}
              </Typography>
            </AccordionSummary>
            <AccordionDetails>
              <Typography variant="body2" color="text.secondary">
                {item.answer}
              </Typography>
            </AccordionDetails>
          </Accordion>
        ))}
      </Box>
    </Container>
  );
}

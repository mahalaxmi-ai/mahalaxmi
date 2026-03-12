import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Container from '@mui/material/Container';
import Alert from '@mui/material/Alert';
import Divider from '@mui/material/Divider';
import MuiLink from '@mui/material/Link';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Security | Mahalaxmi AI',
  description: 'Report security vulnerabilities responsibly to the Mahalaxmi AI security team.',
  alternates: {
    canonical: '/open-source/security',
  },
};

const TEAL = '#00C8C8';

const reportSteps = [
  'Describe the vulnerability in detail — include the affected component, version, and observed behavior.',
  'Provide reproduction steps — a minimal proof-of-concept or step-by-step instructions to trigger the issue.',
  'Include an impact assessment — what data or systems could be affected and under what conditions.',
  'Email security@mahalaxmi.ai with the subject line "Security Vulnerability Report".',
];

const doNotItems = [
  'Do NOT open a public GitHub issue for security vulnerabilities.',
  'Do NOT post details on social media before a patch is available.',
  'Do NOT share vulnerability details in public Slack or Discord channels before the patch is released.',
];

export default function SecurityPage() {
  return (
    <Container maxWidth="md" sx={{ py: 6 }}>
      <Typography variant="h3" component="h1" gutterBottom fontWeight={700}>
        Security Vulnerability Disclosure
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 4 }}>
        We take security seriously. If you discover a vulnerability, please follow the responsible disclosure process below.
      </Typography>

      {/* Prominent email warning */}
      <Alert
        severity="warning"
        sx={{
          mb: 4,
          border: '2px solid #F5A623',
          backgroundColor: 'rgba(245,166,35,0.08)',
          '& .MuiAlert-message': { width: '100%' },
        }}
      >
        <Typography variant="subtitle1" fontWeight={700} sx={{ mb: 1 }}>
          Report security vulnerabilities to{' '}
          <MuiLink
            href="mailto:security@mahalaxmi.ai"
            sx={{
              color: '#F5A623',
              fontWeight: 800,
              fontSize: '1.1em',
              textDecoration: 'underline',
            }}
          >
            security@mahalaxmi.ai
          </MuiLink>
        </Typography>
        <Typography variant="body2" fontWeight={600}>
          DO NOT open a public GitHub issue for security vulnerabilities.
        </Typography>
      </Alert>

      {/* Prominent mailto button */}
      <Box sx={{ mb: 5, display: 'flex', justifyContent: 'center' }}>
        <MuiLink
          href="mailto:security@mahalaxmi.ai"
          sx={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 1,
            px: 4,
            py: 1.5,
            borderRadius: 2,
            border: `2px solid #F5A623`,
            backgroundColor: 'rgba(245,166,35,0.1)',
            color: '#F5A623',
            textDecoration: 'none',
            fontSize: '1.1rem',
            fontWeight: 700,
            letterSpacing: '0.02em',
            '&:hover': { backgroundColor: 'rgba(245,166,35,0.18)' },
          }}
        >
          security@mahalaxmi.ai
        </MuiLink>
      </Box>

      <Divider sx={{ mb: 4 }} />

      {/* SLA commitments */}
      <Box sx={{ mb: 5 }}>
        <Typography variant="h5" fontWeight={700} gutterBottom>
          Response SLA
        </Typography>
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
            mt: 2,
          }}
        >
          <Box
            sx={{
              p: 2.5,
              borderRadius: 2,
              border: '1px solid',
              borderColor: 'divider',
              borderLeft: `4px solid ${TEAL}`,
              backgroundColor: 'rgba(0,200,200,0.04)',
            }}
          >
            <Typography variant="body1" fontWeight={700} sx={{ color: TEAL }}>
              48-hour initial response
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              We respond to all security reports within 48 hours.
            </Typography>
          </Box>
          <Box
            sx={{
              p: 2.5,
              borderRadius: 2,
              border: '1px solid',
              borderColor: 'divider',
              borderLeft: `4px solid ${TEAL}`,
              backgroundColor: 'rgba(0,200,200,0.04)',
            }}
          >
            <Typography variant="body1" fontWeight={700} sx={{ color: TEAL }}>
              14-day patch for critical vulnerabilities
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              Critical vulnerabilities receive a patch within 14 days of confirmation.
            </Typography>
          </Box>
        </Box>
      </Box>

      <Divider sx={{ mb: 4 }} />

      {/* How to report */}
      <Box sx={{ mb: 5 }}>
        <Typography variant="h5" fontWeight={700} gutterBottom>
          How to Report
        </Typography>
        <Box
          component="ol"
          sx={{
            pl: 3,
            mt: 2,
            display: 'flex',
            flexDirection: 'column',
            gap: 1.5,
          }}
        >
          {reportSteps.map((step, index) => (
            <Box component="li" key={index}>
              <Typography variant="body1" color="text.secondary">
                {step}
              </Typography>
            </Box>
          ))}
        </Box>
      </Box>

      <Divider sx={{ mb: 4 }} />

      {/* Do NOT section */}
      <Box sx={{ mb: 5 }}>
        <Typography variant="h5" fontWeight={700} gutterBottom>
          Please Do NOT
        </Typography>
        <Box
          sx={{
            mt: 2,
            p: 3,
            borderRadius: 2,
            border: '1px solid #CF6679',
            backgroundColor: 'rgba(207,102,121,0.06)',
          }}
        >
          <Box
            component="ul"
            sx={{
              pl: 3,
              m: 0,
              display: 'flex',
              flexDirection: 'column',
              gap: 1.5,
            }}
          >
            {doNotItems.map((item, index) => (
              <Box component="li" key={index}>
                <Typography variant="body1" fontWeight={500} sx={{ color: '#CF6679' }}>
                  {item}
                </Typography>
              </Box>
            ))}
          </Box>
        </Box>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
          Public disclosure before a patch is available puts all Mahalaxmi users at risk. We appreciate your cooperation.
        </Typography>
      </Box>

      <Divider sx={{ mb: 4 }} />

      {/* PGP */}
      <Box sx={{ mb: 5 }}>
        <Typography variant="h5" fontWeight={700} gutterBottom>
          PGP Encryption
        </Typography>
        <Typography variant="body1" color="text.secondary" sx={{ mt: 1 }}>
          PGP key available on request. Email{' '}
          <MuiLink
            href="mailto:security@mahalaxmi.ai"
            sx={{ color: TEAL, textDecoration: 'none', '&:hover': { textDecoration: 'underline' } }}
          >
            security@mahalaxmi.ai
          </MuiLink>{' '}
          and we will provide our public key so you can encrypt sensitive vulnerability details.
        </Typography>
      </Box>

      <Divider sx={{ mb: 4 }} />

      {/* Hall of Fame */}
      <Box sx={{ mb: 2 }}>
        <Typography variant="h5" fontWeight={700} gutterBottom>
          Hall of Fame
        </Typography>
        <Typography variant="body1" color="text.secondary" sx={{ mt: 1 }}>
          Responsible disclosure acknowledgements
        </Typography>
        <Box
          sx={{
            mt: 2,
            p: 3,
            borderRadius: 2,
            border: '1px solid',
            borderColor: 'divider',
            backgroundColor: 'rgba(255,255,255,0.02)',
            textAlign: 'center',
          }}
        >
          <Typography variant="body2" color="text.disabled">
            No acknowledgements yet. Be the first responsible security researcher.
          </Typography>
        </Box>
      </Box>
    </Container>
  );
}

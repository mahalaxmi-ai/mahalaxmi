'use client';
import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import MuiLink from '@mui/material/Link';
import Divider from '@mui/material/Divider';
import { Link } from '@/i18n/navigation';

const footerLinks = [
  { label: 'GitHub', href: 'https://github.com/thrivetech2t/mahalaxmi', external: true },
  { label: 'VS Code Marketplace', href: 'https://marketplace.visualstudio.com', external: true },
  { label: 'Docs', href: '/docs' },
  { label: 'Support', href: '/support' },
  { label: 'legal@mahalaxmi.ai', href: 'mailto:legal@mahalaxmi.ai', external: true },
];

export default function Footer() {
  return (
    <Box
      component="footer"
      sx={{
        borderTop: '1px solid rgba(0,200,200,0.1)',
        backgroundColor: '#0A2A2A',
        mt: 'auto',
        py: 4,
      }}
    >
      <Container maxWidth="lg">
        <Box
          sx={{
            display: 'flex',
            flexWrap: 'wrap',
            justifyContent: 'center',
            gap: 3,
            mb: 3,
          }}
        >
          {footerLinks.map((link) =>
            link.external ? (
              <MuiLink
                key={link.label}
                href={link.href}
                target={link.href.startsWith('http') ? '_blank' : undefined}
                rel={link.href.startsWith('http') ? 'noopener noreferrer' : undefined}
                sx={{ color: 'text.secondary', textDecoration: 'none', fontSize: '0.875rem', '&:hover': { color: '#00C8C8' } }}
              >
                {link.label}
              </MuiLink>
            ) : (
              <MuiLink
                key={link.label}
                component={Link}
                href={link.href}
                sx={{ color: 'text.secondary', textDecoration: 'none', fontSize: '0.875rem', '&:hover': { color: '#00C8C8' } }}
              >
                {link.label}
              </MuiLink>
            )
          )}
        </Box>
        <Divider sx={{ borderColor: 'rgba(0,200,200,0.1)', mb: 3 }} />
        <Typography variant="body2" color="text.secondary" align="center" sx={{ fontSize: '0.8rem' }}>
          © {new Date().getFullYear()} ThriveTech Services LLC. All rights reserved.
        </Typography>
      </Container>
    </Box>
  );
}

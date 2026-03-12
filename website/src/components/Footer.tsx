import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import MuiLink from '@mui/material/Link';
import Divider from '@mui/material/Divider';

const FOOTER_LINKS = [
  { label: 'GitHub', href: 'https://github.com/thrivetech2t/mahalaxmi' },
  { label: 'VS Code Marketplace', href: '#' },
  { label: 'Docs', href: '/docs' },
  { label: 'Support', href: '/support' },
  { label: 'Terms of Service', href: '/terms' },
  { label: 'Privacy Policy', href: '/privacy' },
];

export default function Footer() {
  return (
    <Box
      component="footer"
      sx={{
        backgroundColor: '#0A2A2A',
        borderTop: '1px solid rgba(255,255,255,0.1)',
        py: 4,
        mt: 'auto',
      }}
    >
      <Container maxWidth="lg">
        <Box
          sx={{
            display: 'flex',
            flexWrap: 'wrap',
            justifyContent: 'center',
            gap: { xs: 2, md: 3 },
            mb: 3,
          }}
        >
          {FOOTER_LINKS.map((link, index) => (
            <MuiLink
              key={index}
              href={link.href}
              underline="hover"
              sx={{
                color: 'rgba(255,255,255,0.6)',
                fontSize: '0.875rem',
                '&:hover': { color: '#00C8C8' },
              }}
            >
              {link.label}
            </MuiLink>
          ))}
        </Box>

        <Divider sx={{ borderColor: 'rgba(255,255,255,0.08)', mb: 3 }} />

        <Box sx={{ display: 'flex', flexDirection: { xs: 'column', md: 'row' }, justifyContent: 'space-between', alignItems: 'center', gap: 1 }}>
          <Typography variant="body2" sx={{ color: 'rgba(255,255,255,0.4)', fontSize: '0.8rem' }}>
            Contact:{' '}
            <MuiLink
              href="mailto:legal@mahalaxmi.ai"
              underline="hover"
              sx={{ color: 'rgba(255,255,255,0.6)', '&:hover': { color: '#00C8C8' } }}
            >
              legal@mahalaxmi.ai
            </MuiLink>
          </Typography>
          <Typography variant="body2" sx={{ color: 'rgba(255,255,255,0.4)', fontSize: '0.8rem' }}>
            © 2026 ThriveTech Services LLC
          </Typography>
        </Box>
      </Container>
    </Box>
  );
}

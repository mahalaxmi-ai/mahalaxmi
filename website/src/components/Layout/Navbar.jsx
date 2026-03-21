'use client';

import { useState } from 'react';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import Drawer from '@mui/material/Drawer';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemText from '@mui/material/ListItemText';
import Typography from '@mui/material/Typography';
import useMediaQuery from '@mui/material/useMediaQuery';
import { useTheme } from '@mui/material/styles';
import MenuIcon from '@mui/icons-material/Menu';
import CloseIcon from '@mui/icons-material/Close';
import { Link } from '@/i18n/navigation';
import Image from 'next/image';
import { useAuth } from '@/contexts/AuthContext';

const navLinks = [
  { label: 'Features', href: '/features' },
  { label: 'Pricing', href: '/pricing' },
  { label: 'Products', href: '/products' },
  { label: 'Open Source', href: '/open-source' },
  { label: 'Docs', href: '/docs' },
  { label: 'Protocol', href: '/mfop' },
];

export default function Navbar() {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const [drawerOpen, setDrawerOpen] = useState(false);
  const { user, logout } = useAuth();

  return (
    <AppBar
      position="fixed"
      sx={{
        backgroundColor: 'rgba(10, 42, 42, 0.95)',
        backdropFilter: 'blur(8px)',
        borderBottom: '1px solid rgba(0,200,200,0.15)',
        boxShadow: 'none',
      }}
    >
      <Toolbar sx={{ maxWidth: 1200, width: '100%', mx: 'auto', px: { xs: 2, md: 3 } }}>
        {/* Logo */}
        <Box component={Link} href="/" sx={{ display: 'flex', alignItems: 'center', gap: 1, textDecoration: 'none', flexShrink: 0 }}>
          <Image
            src="/mahalaxmi_logo.png"
            alt="Mahalaxmi"
            width={36}
            height={36}
            style={{ borderRadius: '50%', objectFit: 'cover' }}
            priority
          />
          <Typography
            variant="h6"
            sx={{ color: '#00C8C8', fontWeight: 700, letterSpacing: '-0.02em', fontSize: '1.1rem' }}
          >
            Mahalaxmi
          </Typography>
        </Box>

        {/* Desktop nav links */}
        {!isMobile && (
          <Box sx={{ display: 'flex', gap: 0.5, mx: 'auto' }}>
            {navLinks.map((link) => (
              <Button
                key={link.href}
                component={Link}
                href={link.href}
                sx={{
                  color: 'text.primary',
                  fontWeight: 500,
                  fontSize: '0.875rem',
                  '&:hover': { color: '#00C8C8', backgroundColor: 'rgba(0,200,200,0.08)' },
                }}
              >
                {link.label}
              </Button>
            ))}
          </Box>
        )}

        {/* CTA buttons */}
        {!isMobile && (
          <Box sx={{ display: 'flex', gap: 1, flexShrink: 0 }}>
            {user ? (
              <>
                <Button
                  component={Link}
                  href="/dashboard/servers"
                  variant="outlined"
                  size="small"
                  sx={{
                    borderColor: 'rgba(0,200,200,0.4)',
                    color: '#00C8C8',
                    '&:hover': { borderColor: '#00C8C8', backgroundColor: 'rgba(0,200,200,0.08)' },
                  }}
                >
                  Dashboard
                </Button>
                <Button
                  onClick={logout}
                  variant="contained"
                  size="small"
                  sx={{
                    backgroundColor: '#00C8C8',
                    color: '#000',
                    fontWeight: 700,
                    '&:hover': { backgroundColor: '#00AAAA' },
                  }}
                >
                  Logout
                </Button>
              </>
            ) : (
              <>
                <Button
                  component={Link}
                  href="/login"
                  variant="outlined"
                  size="small"
                  sx={{
                    borderColor: 'rgba(0,200,200,0.4)',
                    color: '#00C8C8',
                    '&:hover': { borderColor: '#00C8C8', backgroundColor: 'rgba(0,200,200,0.08)' },
                  }}
                >
                  Login
                </Button>
                <Button
                  component={Link}
                  href="/register"
                  variant="contained"
                  size="small"
                  sx={{
                    backgroundColor: '#00C8C8',
                    color: '#000',
                    fontWeight: 700,
                    '&:hover': { backgroundColor: '#00AAAA' },
                  }}
                >
                  Get Started
                </Button>
              </>
            )}
          </Box>
        )}

        {/* Mobile menu */}
        {isMobile && (
          <>
            <Box sx={{ ml: 'auto' }}>
              <IconButton onClick={() => setDrawerOpen(true)} sx={{ color: 'text.primary' }}>
                <MenuIcon />
              </IconButton>
            </Box>
            <Drawer
              anchor="right"
              open={drawerOpen}
              onClose={() => setDrawerOpen(false)}
              PaperProps={{ sx: { backgroundColor: '#0A2A2A', width: 260 } }}
            >
              <Box sx={{ p: 2, display: 'flex', justifyContent: 'flex-end' }}>
                <IconButton onClick={() => setDrawerOpen(false)} sx={{ color: 'text.primary' }}>
                  <CloseIcon />
                </IconButton>
              </Box>
              <List>
                {navLinks.map((link) => (
                  <ListItem key={link.href} component={Link} href={link.href} onClick={() => setDrawerOpen(false)} sx={{ color: 'text.primary' }}>
                    <ListItemText primary={link.label} />
                  </ListItem>
                ))}
                {user ? (
                  <>
                    <ListItem component={Link} href="/dashboard/servers" onClick={() => setDrawerOpen(false)} sx={{ color: '#00C8C8' }}>
                      <ListItemText primary="Dashboard" />
                    </ListItem>
                    <ListItem onClick={() => { setDrawerOpen(false); logout(); }} sx={{ color: '#00C8C8', cursor: 'pointer' }}>
                      <ListItemText primary="Logout" />
                    </ListItem>
                  </>
                ) : (
                  <>
                    <ListItem component={Link} href="/login" onClick={() => setDrawerOpen(false)} sx={{ color: '#00C8C8' }}>
                      <ListItemText primary="Login" />
                    </ListItem>
                    <ListItem component={Link} href="/register" onClick={() => setDrawerOpen(false)} sx={{ color: '#00C8C8' }}>
                      <ListItemText primary="Get Started" />
                    </ListItem>
                  </>
                )}
              </List>
            </Drawer>
          </>
        )}
      </Toolbar>
    </AppBar>
  );
}

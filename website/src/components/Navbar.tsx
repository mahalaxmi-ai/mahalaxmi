'use client';

import React, { useState } from 'react';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import Drawer from '@mui/material/Drawer';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemText from '@mui/material/ListItemText';
import Box from '@mui/material/Box';
import useMediaQuery from '@mui/material/useMediaQuery';
import MenuIcon from '@mui/icons-material/Menu';
import CloseIcon from '@mui/icons-material/Close';
import Image from 'next/image';
import Link from 'next/link';
import NextLink from 'next/link';

const NAV_LINKS = [
  { label: 'Features', href: '/features' },
  { label: 'Pricing', href: '/pricing' },
  { label: 'Products', href: '/products' },
  { label: 'Open Source', href: '/open-source' },
  { label: 'Docs', href: '/docs' },
];

export default function Navbar() {
  const [drawerOpen, setDrawerOpen] = useState(false);
  const isMobile = useMediaQuery('(max-width:768px)');

  const handleDrawerToggle = () => {
    setDrawerOpen((prev) => !prev);
  };

  const handleDrawerClose = () => {
    setDrawerOpen(false);
  };

  return (
    <>
      <AppBar
        position="sticky"
        elevation={0}
        sx={{
          backgroundColor: '#0A2A2A',
          borderBottom: '1px solid rgba(255,255,255,0.08)',
        }}
      >
        <Toolbar sx={{ justifyContent: 'space-between', px: { xs: 2, md: 4 } }}>
          {/* Left: Logo */}
          <Box component={NextLink} href="/" sx={{ display: 'flex', alignItems: 'center', textDecoration: 'none' }}>
            <Image
              src="/mahalaxmi_logo.png"
              alt="Mahalaxmi"
              height={36}
              width={120}
              style={{ objectFit: 'contain' }}
              priority
            />
          </Box>

          {/* Center: Nav links (desktop) */}
          {!isMobile && (
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              {NAV_LINKS.map((link) => (
                <Button
                  key={link.href}
                  component={NextLink}
                  href={link.href}
                  sx={{
                    color: '#ffffff',
                    textTransform: 'none',
                    fontWeight: 400,
                    fontSize: '0.95rem',
                    '&:hover': { backgroundColor: 'rgba(255,255,255,0.06)' },
                  }}
                >
                  {link.label}
                </Button>
              ))}
            </Box>
          )}

          {/* Right: Auth buttons (desktop) or hamburger (mobile) */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {isMobile ? (
              <IconButton
                color="inherit"
                edge="end"
                onClick={handleDrawerToggle}
                aria-label="Open navigation menu"
              >
                <MenuIcon />
              </IconButton>
            ) : (
              <>
                <Button
                  component={NextLink}
                  href="/login"
                  variant="text"
                  color="inherit"
                  sx={{ textTransform: 'none', fontWeight: 400 }}
                >
                  Login
                </Button>
                <Button
                  component={NextLink}
                  href="/register"
                  variant="contained"
                  sx={{
                    backgroundColor: '#00C8C8',
                    color: '#0A2A2A',
                    fontWeight: 600,
                    textTransform: 'none',
                    '&:hover': { backgroundColor: '#00A8A8' },
                  }}
                >
                  Get Started
                </Button>
              </>
            )}
          </Box>
        </Toolbar>
      </AppBar>

      {/* Mobile Drawer */}
      <Drawer
        anchor="right"
        open={drawerOpen}
        onClose={handleDrawerClose}
        PaperProps={{
          sx: {
            backgroundColor: '#0A2A2A',
            width: 260,
            borderLeft: '1px solid rgba(255,255,255,0.08)',
          },
        }}
      >
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', p: 1 }}>
          <IconButton color="inherit" onClick={handleDrawerClose} aria-label="Close navigation menu">
            <CloseIcon sx={{ color: '#ffffff' }} />
          </IconButton>
        </Box>
        <List>
          {NAV_LINKS.map((link) => (
            <ListItem key={link.href} disablePadding>
              <ListItemButton
                component={NextLink}
                href={link.href}
                onClick={handleDrawerClose}
                sx={{ '&:hover': { backgroundColor: 'rgba(255,255,255,0.06)' } }}
              >
                <ListItemText
                  primary={link.label}
                  primaryTypographyProps={{ sx: { color: '#ffffff', fontWeight: 400 } }}
                />
              </ListItemButton>
            </ListItem>
          ))}
          <ListItem disablePadding sx={{ mt: 2, px: 2 }}>
            <Button
              component={NextLink}
              href="/login"
              variant="text"
              color="inherit"
              fullWidth
              sx={{ textTransform: 'none', color: '#ffffff', justifyContent: 'flex-start' }}
              onClick={handleDrawerClose}
            >
              Login
            </Button>
          </ListItem>
          <ListItem disablePadding sx={{ px: 2, mt: 1 }}>
            <Button
              component={NextLink}
              href="/register"
              variant="contained"
              fullWidth
              sx={{
                backgroundColor: '#00C8C8',
                color: '#0A2A2A',
                fontWeight: 600,
                textTransform: 'none',
                '&:hover': { backgroundColor: '#00A8A8' },
              }}
              onClick={handleDrawerClose}
            >
              Get Started
            </Button>
          </ListItem>
        </List>
      </Drawer>
    </>
  );
}

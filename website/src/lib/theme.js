'use client';

import { createTheme } from '@mui/material/styles';
import { rtlLocales } from '@/i18n/routing';

export default function createAppTheme(locale = 'en-US') {
  const direction = rtlLocales.includes(locale) ? 'rtl' : 'ltr';

  return createTheme({
    direction,
    palette: {
      mode: 'dark',
      primary: {
        main: '#00C8C8',
        light: '#33D4D4',
        dark: '#008C8C',
        contrastText: '#000000',
      },
      secondary: {
        main: '#C8A040',
        light: '#D4B060',
        dark: '#8C7030',
        contrastText: '#000000',
      },
      background: {
        default: '#0A2A2A',
        paper: '#0F3535',
      },
      text: {
        primary: '#E0F0F0',
        secondary: '#99CCCC',
      },
    },
    typography: {
      fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
    },
    components: {
      MuiCssBaseline: {
        styleOverrides: {
          body: {
            backgroundColor: '#0A2A2A',
            color: '#E0F0F0',
          },
        },
      },
    },
  });
}

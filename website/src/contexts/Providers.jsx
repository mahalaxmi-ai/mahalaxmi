'use client';

import { useState, useMemo } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ThemeProvider } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import { CacheProvider } from '@emotion/react';
import createCache from '@emotion/cache';
import createAppTheme from '@/lib/theme';
import { AuthProvider } from '@/contexts/AuthContext';

function createEmotionCache(dir) {
  return createCache({ key: dir === 'rtl' ? 'mui-rtl' : 'mui' });
}

export default function Providers({ children, locale = 'en-US', dir = 'ltr' }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            retry: 2,
            staleTime: 5 * 60 * 1000,
            gcTime: 10 * 60 * 1000,
          },
        },
      })
  );

  const theme = useMemo(() => createAppTheme(locale), [locale]);
  const emotionCache = useMemo(() => createEmotionCache(dir), [dir]);

  return (
    <QueryClientProvider client={queryClient}>
      <CacheProvider value={emotionCache}>
        <ThemeProvider theme={theme}>
          <CssBaseline />
          <AuthProvider>{children}</AuthProvider>
        </ThemeProvider>
      </CacheProvider>
    </QueryClientProvider>
  );
}

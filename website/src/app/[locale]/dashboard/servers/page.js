import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import { Suspense } from 'react';
import { Box, CircularProgress } from '@mui/material';
import ServersContent from './ServersContent';
import { getProviderLabels, getTierLabels } from '@/lib/cloudConstants';

export const dynamic = 'force-dynamic';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'My Cloud Servers — Mahalaxmi',
    robots: { index: false },
  };
}

export default async function DashboardServersPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  let providerLabels = {};
  let tierLabels = {};
  try {
    [providerLabels, tierLabels] = await Promise.all([
      getProviderLabels(),
      getTierLabels(),
    ]);
  } catch {
    // Platform unavailable — ServerCard falls back to raw slug/status strings
  }

  return (
    <Suspense fallback={
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 10 }}>
        <CircularProgress />
      </Box>
    }>
      <ServersContent providerLabels={providerLabels} tierLabels={tierLabels} />
    </Suspense>
  );
}

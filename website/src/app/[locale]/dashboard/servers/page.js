import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import { Suspense } from 'react';
import { Box, CircularProgress } from '@mui/material';
import ServersContent from './ServersContent';

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

  return (
    <Suspense fallback={
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 10 }}>
        <CircularProgress />
      </Box>
    }>
      <ServersContent />
    </Suspense>
  );
}

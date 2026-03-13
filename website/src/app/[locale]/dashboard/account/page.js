import { Suspense } from 'react';
import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import { CircularProgress, Box } from '@mui/material';
import AccountContent from './AccountContent';

export const dynamic = 'force-dynamic';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'Account — Mahalaxmi',
    robots: { index: false },
  };
}

export default async function AccountPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);
  return (
    <Suspense fallback={<Box sx={{ display: 'flex', justifyContent: 'center', py: 10 }}><CircularProgress /></Box>}>
      <AccountContent />
    </Suspense>
  );
}

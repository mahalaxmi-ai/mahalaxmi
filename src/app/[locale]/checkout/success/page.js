import { Suspense } from 'react';
import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import { Box, CircularProgress } from '@mui/material';
import MahalaxmiCheckoutSuccessContent from './MahalaxmiCheckoutSuccessContent';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'Server Provisioning — Mahalaxmi Cloud',
    description: 'Your Mahalaxmi Cloud server is being provisioned.',
    alternates: { canonical: '/checkout/success' },
    robots: { index: false },
  };
}

export default async function CheckoutSuccessPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Suspense
      fallback={
        <Box display="flex" justifyContent="center" p={8}>
          <CircularProgress />
        </Box>
      }
    >
      <MahalaxmiCheckoutSuccessContent />
    </Suspense>
  );
}

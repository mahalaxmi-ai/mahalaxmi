import { Suspense } from 'react';
import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import MahalaxmiCheckoutSuccessContent from './MahalaxmiCheckoutSuccessContent';

export const dynamic = 'force-dynamic';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'Server Provisioning — Mahalaxmi Cloud',
    description: 'Your Mahalaxmi Cloud server is being provisioned.',
    robots: { index: false },
  };
}

export default async function MahalaxmiCheckoutSuccessPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Suspense
      fallback={
        <div style={{ display: 'flex', justifyContent: 'center', padding: '64px', color: '#00C8C8' }}>
          Loading...
        </div>
      }
    >
      <MahalaxmiCheckoutSuccessContent />
    </Suspense>
  );
}

import { Suspense } from 'react';
import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import VerifyEmailContent from './VerifyEmailContent';

export const dynamic = 'force-dynamic';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'Verify Email — Mahalaxmi',
    robots: { index: false },
  };
}

export default async function VerifyEmailPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Suspense fallback={<div style={{ display: 'flex', justifyContent: 'center', padding: '64px', color: '#00C8C8' }}>Loading…</div>}>
      <VerifyEmailContent />
    </Suspense>
  );
}

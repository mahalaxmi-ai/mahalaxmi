import { Suspense } from 'react';
import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import ResetPasswordContent from './ResetPasswordContent';

export const dynamic = 'force-dynamic';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'Reset Password — Mahalaxmi',
    robots: { index: false },
  };
}

export default async function ResetPasswordPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Suspense fallback={<div style={{ display: 'flex', justifyContent: 'center', padding: '64px', color: '#00C8C8' }}>Loading…</div>}>
      <ResetPasswordContent />
    </Suspense>
  );
}

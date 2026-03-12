import { Suspense } from 'react';
import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import ResendVerificationContent from './ResendVerificationContent';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'Resend Verification Email — Mahalaxmi',
    robots: { index: false },
  };
}

export default async function ResendVerificationPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Suspense>
      <ResendVerificationContent />
    </Suspense>
  );
}

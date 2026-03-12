import { Suspense } from 'react';
import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import ForgotPasswordContent from './ForgotPasswordContent';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'Forgot Password — Mahalaxmi',
    robots: { index: false },
  };
}

export default async function ForgotPasswordPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Suspense>
      <ForgotPasswordContent />
    </Suspense>
  );
}

import { Suspense } from 'react';
import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import LoginContent from './LoginContent';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'Sign In — Mahalaxmi',
    description: 'Sign in to your Mahalaxmi account.',
    robots: { index: false },
  };
}

export default async function LoginPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Suspense>
      <LoginContent />
    </Suspense>
  );
}

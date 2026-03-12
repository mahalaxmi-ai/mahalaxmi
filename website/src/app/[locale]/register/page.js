import { Suspense } from 'react';
import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import RegisterContent from './RegisterContent';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata() {
  return {
    title: 'Create Account — Mahalaxmi',
    description: 'Create your Mahalaxmi account and start running AI agents in minutes.',
    robots: { index: false },
  };
}

export default async function RegisterPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Suspense>
      <RegisterContent />
    </Suspense>
  );
}
